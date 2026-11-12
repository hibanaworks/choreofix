//! One I/O thread owns the original environment and starts its asynchronous calls.
use crate::{
    evidence::{Result, Trace},
    wire::{self, Reply},
};
use hibana_runtime::host::notification;
use pyo3::{
    prelude::*,
    types::{PyCFunction, PyDict, PyModule, PyType},
};
use serde_json::{Value, json};
use std::{fs, path::Path, sync::mpsc, time::Instant};

type Request = (Value, Reply);

pub fn interpreter_identity() -> Result<Value> {
    if std::env::var("PYTHONHASHSEED").as_deref() != Ok("0") {
        return Err("PYTHONHASHSEED=0 is required before interpreter startup".into());
    }
    Python::initialize();
    Python::attach(|py| -> Result<Value> {
        let sys = PyModule::import(py, "sys")?;
        Ok(
            json!({"version":sys.getattr("version")?.extract::<String>()?,
            "implementation":sys.getattr("implementation")?.getattr("name")?.extract::<String>()?,
            "base_prefix":sys.getattr("base_prefix")?.extract::<String>()?,
            "hash_seed":0,"adapter":"Rust PyO3 API calls, no evaluated snippets"}),
        )
    })
}

struct EventLoop {
    object: Py<PyAny>,
    worker: Option<std::thread::JoinHandle<std::result::Result<(), String>>>,
}

impl EventLoop {
    fn new() -> Result<Self> {
        let object = Python::attach(|py| -> PyResult<_> {
            Ok(PyModule::import(py, "asyncio")?
                .call_method0("new_event_loop")?
                .unbind())
        })?;
        let worker_loop = Python::attach(|py| object.clone_ref(py));
        let worker = std::thread::Builder::new()
            .name("upstream-environment".into())
            .spawn(move || {
                Python::attach(|py| -> PyResult<()> {
                    PyModule::import(py, "asyncio")?
                        .call_method1("set_event_loop", (worker_loop.bind(py),))?;
                    worker_loop.bind(py).call_method0("run_forever")?;
                    worker_loop.bind(py).call_method0("close")?;
                    Ok(())
                })
                .map_err(|e| e.to_string())
            })?;
        Ok(Self {
            object,
            worker: Some(worker),
        })
    }
}

impl Drop for EventLoop {
    fn drop(&mut self) {
        let stopped = Python::attach(|py| -> PyResult<()> {
            let event_loop = self.object.bind(py);
            event_loop.call_method1("call_soon_threadsafe", (event_loop.getattr("stop")?,))?;
            Ok(())
        });
        if let Err(error) = stopped {
            eprintln!("upstream loop shutdown: {error}");
        }
        if let Some(worker) = self.worker.take() {
            match worker.join() {
                Ok(Ok(())) => {}
                Ok(Err(error)) => eprintln!("upstream loop: {error}"),
                Err(_) => eprintln!("upstream loop panicked"),
            }
        }
    }
}

fn json_value(value: &Bound<'_, PyAny>) -> Result<Value> {
    let encoded: String = PyModule::import(value.py(), "json")?
        .call_method1("dumps", (value,))?
        .extract()?;
    Ok(serde_json::from_str(&encoded)?)
}

struct Environment {
    simulator: Py<PyAny>,
    registry: Py<PyAny>,
    event_loop: EventLoop,
    trace: Trace,
    started: Instant,
}

#[derive(Clone)]
pub struct Simulator(mpsc::Sender<Request>);

impl Simulator {
    pub fn new(
        task: &str,
        depth: u8,
        delay: f64,
        trace: Trace,
        failures: &[(&str, u32)],
    ) -> Result<(Self, Value)> {
        if depth > 2
            || !delay.is_finite()
            || delay < 0.0
            || std::env::var("PYTHONHASHSEED").as_deref() != Ok("0")
        {
            return Err("expected depth 0..2, a finite delay and PYTHONHASHSEED=0 before interpreter startup".into());
        }
        choreofix_inputs::verify_benchmark(Path::new("."))?;
        let interpreter = interpreter_identity()?;
        let source = Path::new(".").canonicalize()?;
        let metadata: Value = serde_json::from_slice(&fs::read(
            source
                .join("benchmark/descriptions")
                .join(task)
                .join("metadata.json"),
        )?)?;
        let (simulator, registry, inputs) = Python::attach(|py| -> Result<_> {
            let sys = PyModule::import(py, "sys")?;
            let version = sys.getattr("version_info")?;
            let major: u8 = version.get_item(0)?.extract()?;
            let minor: u8 = version.get_item(1)?.extract()?;
            let packages = Path::new(".venv/lib")
                .join(format!("python{major}.{minor}/site-packages"))
                .canonicalize()?;
            PyModule::import(py, "site")?
                .call_method1("addsitedir", (packages.to_str().ok_or("site path UTF-8")?,))?;
            sys.getattr("path")?
                .call_method1("insert", (0, source.to_str().ok_or("upstream path UTF-8")?))?;
            let name = format!("benchmark.environments.{task}.sim");
            let module = PyModule::import(py, &name)?;
            let actual = std::path::PathBuf::from(module.getattr("__file__")?.extract::<String>()?)
                .canonicalize()?;
            if actual
                != source
                    .join("benchmark/environments")
                    .join(task)
                    .join("sim.py")
            {
                return Err("a different upstream environment was imported".into());
            }
            let base = PyModule::import(py, "benchmark.tools.sim_base")?
                .getattr("SimContext")?
                .cast_into::<PyType>()
                .map_err(PyErr::from)?;
            let mut classes = Vec::new();
            for (_, value) in module.dict().iter() {
                if let Ok(class) = value.cast::<PyType>()
                    && class.getattr("__module__")?.extract::<String>()? == name
                    && class.is_subclass(&base)?
                {
                    classes.push(class.clone());
                }
            }
            if classes.len() != 1 {
                return Err("expected one original environment class".into());
            }
            let sim = classes[0].call0()?;
            sim.setattr("_seed", 0)?;
            sim.setattr("_delay_multiplier", delay)?;
            sim.call_method1("set_scenario_depth", (depth,))?;
            // Unscored recovery trials inject failures through the upstream API.
            // Their evidence records every override separately from scored runs.
            for (tool, call_index) in failures {
                let arguments = PyDict::new(py);
                arguments.set_item("tool_name", tool)?;
                arguments.set_item("call_index", call_index)?;
                let failure = PyModule::import(py, "benchmark.tools.sim_base")?
                    .getattr("FailureSpec")?
                    .call((), Some(&arguments))?;
                sim.getattr("_failure_specs")?
                    .call_method1("append", (failure,))?;
            }
            let kwargs = PyDict::new(py);
            kwargs.set_item("sim", &sim)?;
            let registry = PyModule::import(py, "benchmark.tools")?.call_method(
                "load_tools",
                (task,),
                Some(&kwargs),
            )?;
            let dataclasses = PyModule::import(py, "dataclasses")?;
            let mut inputs = serde_json::Map::new();
            for (key, value) in module.dict().iter() {
                let key: String = key.extract()?;
                if key.ends_with("_STEPS")
                    || matches!(
                        key.as_str(),
                        "_ANALYSES" | "_SEPARATIONS" | "_TESTS" | "_DISHES" | "_INGREDIENTS"
                    )
                {
                    let entries: Result<Vec<_>> = value
                        .try_iter()?
                        .map(|entry| json_value(&dataclasses.call_method1("asdict", (entry?,))?))
                        .collect();
                    inputs.insert(key, json!(entries?));
                } else if key == "_FORKS" {
                    inputs.insert(key, json_value(&value)?);
                }
            }
            if task == "7E" {
                for (attribute, name) in [
                    ("_write_steps", "_WRITE_STEPS"),
                    ("_references_steps", "_REFERENCES_STEPS"),
                    ("_review_steps", "_REVIEW_STEPS"),
                    ("_combine_steps", "_COMBINE_STEPS"),
                ] {
                    let entries: Result<Vec<_>> = sim
                        .getattr(attribute)?
                        .try_iter()?
                        .map(|entry| json_value(&dataclasses.call_method1("asdict", (entry?,))?))
                        .collect();
                    if inputs.insert(name.into(), json!(entries?)).is_some() {
                        return Err("document assignment has two sources".into());
                    }
                }
            }
            Ok((sim.unbind(), registry.unbind(), inputs))
        })?;
        let mut inputs = json!(inputs);
        if task == "14H" {
            inputs["tool_schemas"] = serde_json::from_slice(&fs::read(
                source.join("benchmark/descriptions/14H/tools.json"),
            )?)?;
        }
        let mut fixture = fixture(task, &metadata, &inputs)?;
        if task == "14H" {
            fixture["tool_schemas"] = inputs["tool_schemas"].clone();
        } else if matches!(
            task,
            "3E" | "4E"
                | "4M"
                | "5E"
                | "5M"
                | "5H"
                | "6M"
                | "7E"
                | "10E"
                | "10M"
                | "11E"
                | "13E"
                | "13M"
                | "14E"
                | "14M"
        ) {
            fixture["tool_schemas"] = serde_json::from_slice(&fs::read(
                source.join(format!("benchmark/descriptions/{task}/tools.json")),
            )?)?;
        }
        fixture["simulator_contract"] = json!(choreofix_inputs::SIMULATOR_CONTRACT);
        fixture["simulator_manifest_sha256"] = json!(crate::evidence::digest(Path::new(
            "benchmark/simulator-manifest.json"
        ))?);
        fixture["delay_multiplier"] = json!(delay);
        if let Some(profile) = metadata.get("tool_delays") {
            fixture["tool_delay_profile"] = profile.clone();
        }
        fixture["depth"] = json!(depth);
        fixture["seed"] = json!(0);
        fixture["interpreter"] = interpreter;
        fixture["failure_overrides"] = json!(failures);
        fixture["terminal_test_failure"] =
            json!(task != "1H" && failures.contains(&("run_local_tests", 0)));
        fixture["terminal_review_failure"] =
            json!(task.starts_with('2') && failures.contains(&("review_own_work", 0)));
        trace.record("simulator", &fixture)?;
        let event_loop = EventLoop::new()?;
        let environment = Environment {
            simulator,
            registry,
            event_loop,
            trace,
            started: Instant::now(),
        };
        let (send, receive) = mpsc::channel::<Request>();
        std::thread::Builder::new()
            .name("original-environment".into())
            .spawn(move || {
                // Test-only I/O withholding: requests have already crossed the
                // real guest and Endpoint path. The gate cannot grant a step.
                #[cfg(test)]
                let mut commit_gate: Option<(
                    std::collections::BTreeSet<String>,
                    Vec<Request>,
                )> = None;
                while let Ok((message, reply)) = receive.recv() {
                    #[cfg(test)]
                    {
                        if message["type"] == "hold_disjoint_commits" {
                            let agents: Option<std::collections::BTreeSet<String>> =
                                message["agents"].as_array().and_then(|values| {
                                    values
                                        .iter()
                                        .map(|value| value.as_str().map(str::to_owned))
                                        .collect()
                                });
                            let configured = match agents {
                                Some(agents)
                                    if commit_gate.is_none()
                                        && agents.len() > 1
                                        && message["agents"].as_array().map(Vec::len)
                                            == Some(agents.len()) =>
                                {
                                    environment
                                        .trace
                                        .record("test_control", &message)
                                        .map(|()| {
                                            commit_gate = Some((agents, Vec::new()));
                                            json!({"status":"configured"})
                                        })
                                }
                                _ => Err("invalid or duplicate test commit gate".into()),
                            };
                            wire::respond(&reply, message["id"].clone(), configured)
                                .expect("notify configured test I/O");
                            continue;
                        }
                        if message["type"] == "execute"
                            && message["tool"] == "commit_changes"
                            && let Some((agents, held)) = &mut commit_gate
                            && message["agent"]
                                .as_str()
                                .is_some_and(|a| agents.contains(a))
                        {
                            if held
                                .iter()
                                .any(|(prior, _)| prior["agent"] == message["agent"])
                            {
                                wire::respond(
                                    &reply,
                                    message["id"].clone(),
                                    Err("duplicate held commit".into()),
                                )
                                .expect("notify duplicate test I/O");
                                continue;
                            }
                            if let Err(error) = environment.trace.record(
                                "test_control",
                                &json!({"type":"commit_held","request":message}),
                            ) {
                                wire::respond(&reply, message["id"].clone(), Err(error))
                                    .expect("notify failed test I/O");
                                continue;
                            }
                            held.push((message, reply));
                            if held.len() == agents.len() {
                                let (agents, held) = commit_gate.take().expect("live test gate");
                                if let Err(error) = environment.trace.record(
                                    "test_control",
                                    &json!({"type":"commits_released","agents":agents}),
                                ) {
                                    for (message, reply) in held {
                                        wire::respond(
                                            &reply,
                                            message["id"].clone(),
                                            Err(error.to_string().into()),
                                        )
                                        .expect("notify failed held I/O");
                                    }
                                    continue;
                                }
                                for (message, reply) in held {
                                    environment.serve(message, reply);
                                }
                            }
                            continue;
                        }
                    }
                    environment.serve(message, reply);
                }
            })?;
        Ok((Self(send), fixture))
    }

    pub async fn call(&self, message: Value) -> Result<Value> {
        let (send, receive) = mpsc::channel();
        let (notify, mut ready) = notification()?;
        let id = message["id"].clone();
        self.send(message, (send, notify))?;
        let (received_id, result) = wire::receive(&receive, &mut ready)
            .await?
            .ok_or("upstream reply closed without a result")?;
        if received_id != id {
            return Err("upstream reply id mismatch".into());
        }
        result
    }

    pub(crate) fn send(&self, message: Value, reply: Reply) -> Result<()> {
        self.0
            .send((message, reply))
            .map_err(|_| "upstream environment closed".into())
    }

    #[cfg(test)]
    pub async fn observed_events(&self) -> Result<Vec<Value>> {
        let result = self.call(json!({"type":"observed_events"})).await?;
        Ok(result.as_array().ok_or("upstream event list")?.clone())
    }
}

impl Environment {
    fn serve(&self, message: Value, reply: Reply) {
        let id = message["id"].clone();
        let result = match message["type"].as_str() {
            Some("execute") => match reply.1.try_clone() {
                Ok(notify) => match self.execute(message, (reply.0.clone(), notify)) {
                    Ok(()) => return,
                    Err(error) => Err(error),
                },
                Err(error) => Err(error.into()),
            },
            Some("score") => self.score(),
            #[cfg(test)]
            Some("observed_events") => self.observed_events().map(Value::Array),
            _ => Err("unknown upstream request".into()),
        };
        wire::respond(&reply, id, result).expect("notify upstream result");
    }

    fn execute(&self, message: Value, reply: Reply) -> Result<()> {
        let tool = message["tool"].as_str().ok_or("upstream tool name")?;
        let agent = message["agent"].as_str().ok_or("upstream agent")?;
        let arguments = serde_json::to_string(&message["arguments"])?;
        let started = u64::try_from(self.started.elapsed().as_nanos())?;
        self.trace.record("domain_start", &message)?;
        let trace = self.trace.clone();
        let clock = self.started;
        let id = message["id"].clone();
        let tool = tool.to_owned();
        let agent = agent.to_owned();
        Python::attach(|py| -> Result<()> {
            let arguments = PyModule::import(py, "json")?
                .call_method1("loads", (arguments,))?
                .cast_into::<PyDict>()
                .map_err(PyErr::from)?;
            arguments.set_item("agent_id", &agent)?;
            let coroutine =
                self.registry
                    .bind(py)
                    .call_method("call", (&tool,), Some(&arguments))?;
            let future = PyModule::import(py, "asyncio")?.call_method1(
                "run_coroutine_threadsafe",
                (coroutine, self.event_loop.object.bind(py)),
            )?;
            let callback = PyCFunction::new_closure(
                py,
                None,
                None,
                move |args, _kwargs| -> PyResult<()> {
                    let result = (|| -> Result<Value> {
                        let result = args
                            .get_item(0)?
                            .call_method0("result")?
                            .call_method0("to_dict")?;
                        let result = json_value(&result)?;
                        trace.record(
                            "domain_end",
                            &json!({"id":id,"agent":agent,"tool":tool,"result":result}),
                        )?;
                        let response = json!({"type":"simulator_result", "id":id,"result":result,
                            "started_ns":started,"finished_ns":u64::try_from(clock.elapsed().as_nanos())?,
                            "clock":"runner monotonic elapsed"});
                        trace.record("simulator", &response)?;
                        Ok(response)
                    })();
                    wire::respond(&reply, id.clone(), result).map_err(|error| {
                        pyo3::exceptions::PyRuntimeError::new_err(error.to_string())
                    })
                },
            )?;
            future.call_method1("add_done_callback", (callback,))?;
            Ok(())
        })?;
        Ok(())
    }

    fn score(&self) -> Result<Value> {
        let result = Python::attach(|py| -> Result<Value> {
            let sim = self.simulator.bind(py);
            let dataclasses = PyModule::import(py, "dataclasses")?;
            let events: Result<Vec<_>> = sim
                .getattr("events")?
                .try_iter()?
                .map(|event| json_value(&dataclasses.call_method1("asdict", (event?,))?))
                .collect();
            let violations: Result<Vec<_>> = sim
                .getattr("violations")?
                .try_iter()?
                .map(|event| json_value(&dataclasses.call_method1("asdict", (event?,))?))
                .collect();
            Ok(
                json!({"type":"score", "complete":sim.call_method0("is_complete")?.extract::<bool>()?,
                "events":events?, "violations":violations?, "progress":json_value(&sim.getattr("progress")?)?}),
            )
        })?;
        self.trace.record("simulator", &result)?;
        Ok(result)
    }

    #[cfg(test)]
    pub fn observed_events(&self) -> Result<Vec<Value>> {
        Python::attach(|py| {
            let dataclasses = PyModule::import(py, "dataclasses")?;
            self.simulator
                .bind(py)
                .getattr("events")?
                .try_iter()?
                .map(|event| json_value(&dataclasses.call_method1("asdict", (event?,))?))
                .collect()
        })
    }
}

/// Immutable physical input data, including module-selected resource sets.
/// This is fixture/auditor interpretation, never Endpoint authorization.
pub(crate) fn resources_for_call<'a>(
    metadata: &'a Value,
    agent: &str,
    call: &Value,
) -> Result<Vec<&'a str>> {
    let entry = &metadata["tool_resource_map"][call["tool"].as_str().ok_or("resource tool")?];
    let names = match entry {
        Value::Array(names) => names,
        Value::String(value) if value == "@agent_resources" => metadata["agent_resources"][agent]
            .as_array()
            .ok_or("participant resource assignment")?,
        Value::Object(_) => {
            let argument = entry["argument"]
                .as_str()
                .ok_or("physical resource argument")?;
            let value = call["arguments"][argument]
                .as_str()
                .ok_or("physical resource input")?
                .trim()
                .to_lowercase()
                .replace(' ', "_");
            entry["values"][value]
                .as_array()
                .ok_or("physical resource input binding")?
        }
        _ => return Err("unknown physical resource definition".into()),
    };
    names
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| "physical resource name".into())
        })
        .collect()
}

fn fixture(task: &str, metadata: &Value, inputs: &Value) -> Result<Value> {
    let family = task.trim_end_matches(['E', 'M', 'H']).parse::<u8>()?;
    let mut actors = Vec::new();
    for agent in metadata["agents"].as_array().ok_or("original agent list")? {
        if task == "14H" {
            let name = agent.as_str().ok_or("scientist name")?;
            let resources = metadata["agent_resources"][name]
                .as_array()
                .ok_or("scientist resource metadata")?;
            let operations = inputs["tool_schemas"]
                .as_array()
                .ok_or("scientific domain schemas")?
                .iter()
                .map(|tool| &tool["function"])
                .filter(|tool| {
                    tool["agent_ids"]
                        .as_array()
                        .is_some_and(|ids| ids.contains(agent))
                })
                .cloned()
                .collect::<Vec<_>>();
            if operations.is_empty() || resources.iter().any(|resource| !resource.is_string()) {
                return Err("scientist lacks its original operations or resources".into());
            }
            // This world has dynamic materials and free operation choices, not
            // a static _STEPS recipe. Record the original schemas unchanged.
            actors.push(json!({"agent":agent,"resources":resources,"operations":operations}));
            continue;
        }
        let step = |table: &str, key: &str| -> Result<Value> {
            inputs[table]
                .as_array()
                .ok_or_else(|| format!("missing original {table}"))?
                .iter()
                .find(|step| step["agent_id"] == *agent)
                .map(|step| step[key].clone())
                .filter(|v| !v.is_null())
                .ok_or_else(|| format!("missing original input {table} {key}").into())
        };
        let calls = if task == "12E" {
            let dish = inputs["_DISHES"]
                .as_array()
                .ok_or("original kitchen dishes")?
                .iter()
                .find(|dish| dish["chef_id"] == *agent)
                .ok_or("original chef dish")?;
            let ingredient = inputs["_INGREDIENTS"]
                .as_array()
                .ok_or("original kitchen ingredients")?
                .iter()
                .find(|ingredient| ingredient["producer"] == *agent)
                .ok_or("original chef ingredient")?;
            json!([
                {"tool":"prepare_base_dish","arguments":{"dish":dish["dish_name"]}},
                {"tool":"prepare_ingredient","arguments":{"ingredient":ingredient["name"],"for_chef":ingredient["consumer"]}},
                {"tool":"combine_dish","arguments":{"dish":dish["dish_name"],"ingredient":dish["receives_ingredient"]}}
            ])
        } else if task == "11E" {
            if agent == "SUPERVISOR" {
                if step("_COLLECT_REPORTS_STEPS", "scope")? != "all_jobs" {
                    return Err("original manufacturing report scope differs".into());
                }
                json!([{"tool":"collect_reports","arguments":{}}])
            } else {
                json!(
                    inputs["_JOB_STEPS"]
                        .as_array()
                        .ok_or("original manufacturing jobs")?
                        .iter()
                        .filter(|step| step["agent_id"] == *agent)
                        .map(|step| json!({"tool":"do_job","arguments":{"job_id":step["job_id"]}}))
                        .collect::<Vec<_>>()
                )
            }
        } else if matches!(task, "10E" | "10M") {
            if agent == "INTEGRATOR" {
                let modules = step("_LINK_STEPS", "modules")?;
                let names = modules
                    .as_array()
                    .ok_or("build link modules")?
                    .iter()
                    .map(|m| m.as_str().ok_or("build module name"))
                    .collect::<std::result::Result<Vec<_>, _>>()?
                    .join(",");
                json!([{"tool":"link_modules","arguments":{"modules":names}}])
            } else if agent == "TEST_RUNNER" {
                json!(inputs["_BUILD_TEST_STEPS"].as_array().ok_or("build test inputs")?
                    .iter().filter(|step| step["agent_id"] == *agent)
                    .map(|step| json!({"tool":"run_build_tests","arguments":{"module_name":step["module"]}}))
                    .collect::<Vec<_>>())
            } else {
                json!([{"tool":"compile_module","arguments":{"module_name":step("_COMPILE_STEPS", "module")?}}])
            }
        } else if family == 9 {
            json!([{"tool":"think","arguments":{"philosopher":agent}}, {"tool":"eat","arguments":{"philosopher":agent}}])
        } else if family == 1 {
            let feature = step("_DESIGN_STEPS", "feature")?;
            let resources = metadata["agent_resources"][agent.as_str().ok_or("agent name")?]
                .as_array()
                .ok_or("agent modules")?
                .iter()
                .map(|v| v.as_str().ok_or("module name"))
                .collect::<std::result::Result<Vec<_>, _>>()?
                .join(", ");
            json!([{"tool":"design_feature","arguments":{"feature_name":feature}},
                {"tool":"implement_code","arguments":{"feature_name":feature}},
                {"tool":"commit_changes","arguments":{"modules":resources}},
                {"tool":"run_local_tests","arguments":{"feature_name":feature}}])
        } else if family == 2 {
            let sections = metadata["agent_resources"][agent.as_str().ok_or("agent name")?]
                .as_array()
                .ok_or("agent sections")?
                .iter()
                .map(|v| v.as_str().ok_or("section name"))
                .collect::<std::result::Result<Vec<_>, _>>()?
                .join(", ");
            let section = step("_DRAFT_STEPS", "section")?;
            json!([{"tool":"read_literature","arguments":{"topic":step("_READ_STEPS","topic")?}},
                {"tool":"draft_content","arguments":{"section_name":section}},
                {"tool":"write_to_document","arguments":{"sections":sections}},
                {"tool":"review_own_work","arguments":{"section_name":section}}])
        } else if task == "7E" {
            if agent == "EDITOR" {
                let mut calls = inputs["_REVIEW_STEPS"].as_array().ok_or("document reviews")?
                    .iter().filter(|row| row["agent_id"] == *agent)
                    .map(|row| json!({"tool":"review_sections","arguments":{"section_name":row["scope"]}}))
                    .collect::<Vec<_>>();
                let sections = step("_COMBINE_STEPS", "sections")?;
                let sections = sections
                    .as_array()
                    .ok_or("document chapters")?
                    .iter()
                    .map(|s| s.as_str().ok_or("document chapter"))
                    .collect::<std::result::Result<Vec<_>, _>>()?
                    .join(",");
                calls.push(json!({"tool":"combine_sections","arguments":{"sections":sections}}));
                json!(calls)
            } else {
                json!([
                    {"tool":"write_section","arguments":{"section_name":step("_WRITE_STEPS","section")?}},
                    {"tool":"update_references","arguments":{"section_name":step("_REFERENCES_STEPS","scope")?}}
                ])
            }
        } else if task == "3E" {
            if agent == "EDITOR" {
                let mut calls = inputs["_REVIEW_STEPS"].as_array().ok_or("original report reviews")?.iter()
                    .filter(|row| row["agent_id"] == *agent)
                    .map(|row| json!({"tool":"review_sections","arguments":{"section_name":row["scope"]}}))
                    .collect::<Vec<_>>();
                let sections = step("_COMBINE_STEPS", "sections")?;
                let sections = sections
                    .as_array()
                    .ok_or("original report sections")?
                    .iter()
                    .map(|section| section.as_str().ok_or("report section name"))
                    .collect::<std::result::Result<Vec<_>, _>>()?
                    .join(",");
                calls.push(json!({"tool":"combine_sections","arguments":{"sections":sections}}));
                Value::Array(calls)
            } else {
                json!([
                    {"tool":"research_topic","arguments":{"topic":step("_RESEARCH_STEPS","topic")?}},
                    {"tool":"write_section","arguments":{"section_name":step("_WRITE_STEPS","section")?}},
                    {"tool":"update_references","arguments":{"section_name":step("_REFERENCES_STEPS","scope")?}},
                    {"tool":"revise_section","arguments":{"section_name":step("_REVISE_STEPS","section")?}}
                ])
            }
        } else if task == "4E" {
            match agent.as_str() {
                Some("BACKEND_DEV") => json!([
                    {"tool":"define_api","arguments":{"api_name":step("_DEFINE_API_STEPS","api_name")?}},
                    {"tool":"implement_feature","arguments":{"feature_name":step("_IMPLEMENT_STEPS","feature")?}}
                ]),
                Some("FRONTEND_DEV") => {
                    json!([{"tool":"implement_feature","arguments":{"feature_name":step("_IMPLEMENT_STEPS","feature")?}}])
                }
                Some("TESTER") => {
                    json!([{"tool":"run_tests","arguments":{"suite":step("_RUN_TESTS_STEPS","suite")?}}])
                }
                _ => return Err("unknown original API coordination participant".into()),
            }
        } else if matches!(task, "5E" | "5M" | "5H") {
            let mut calls = Vec::new();
            for (tool, table) in [
                ("assess_patient", "_ASSESS_STEPS"),
                ("order_tests", "_ORDER_TESTS_STEPS"),
                ("perform_imaging", "_IMAGING_STEPS"),
                ("interpret_imaging", "_INTERPRET_IMAGING_STEPS"),
                ("diagnose", "_DIAGNOSE_STEPS"),
                ("review_prescriptions", "_PRESCRIPTION_REVIEW_STEPS"),
                ("conduct_case_conference", "_CASE_CONFERENCE_STEPS"),
                ("prepare_anesthesia", "_ANESTHESIA_STEPS"),
                ("perform_surgery", "_SURGERY_STEPS"),
                ("manage_icu", "_ICU_STEPS"),
            ] {
                if inputs[table].is_null() {
                    if metadata["tool_resource_map"].get(tool).is_none() {
                        continue;
                    }
                    return Err(
                        format!("declared medical tool {tool} lacks its assigned inputs").into(),
                    );
                }
                for row in inputs[table]
                    .as_array()
                    .ok_or("medical task inputs")?
                    .iter()
                    .filter(|row| row["agent_id"] == *agent)
                {
                    let mut arguments = json!({"patient_id":if tool == "conduct_case_conference" {
                        &row["case_id"]
                    } else { &row["patient_id"] }});
                    if tool == "order_tests" {
                        arguments["test_type"] = json!(
                            row["tests"]
                                .as_array()
                                .ok_or("assigned tests")?
                                .iter()
                                .map(|test| test.as_str().ok_or("assigned test name"))
                                .collect::<std::result::Result<Vec<_>, _>>()?
                                .join(",")
                        );
                    }
                    if matches!(tool, "perform_imaging" | "interpret_imaging") {
                        arguments["modality"] = row["modality"].clone();
                    }
                    if tool == "perform_surgery" {
                        arguments["procedure"] = row["procedure"].clone();
                    }
                    calls.push(json!({"tool":tool,"arguments":arguments}));
                }
            }
            Value::Array(calls)
        } else if matches!(task, "13E" | "13M") {
            let mut calls = Vec::new();
            for (tool, table, key) in [
                ("run_analysis", "_ANALYSES", "sample"),
                ("run_separation", "_SEPARATIONS", "material"),
            ] {
                for row in inputs[table]
                    .as_array()
                    .ok_or("laboratory assignments")?
                    .iter()
                    .filter(|s| s["scientist_id"] == *agent)
                {
                    calls.push(json!({"tool":tool,"arguments":{key:row[key]}}));
                }
            }
            if agent == "BIOLOGIST" {
                let separations = inputs["_SEPARATIONS"]
                    .as_array()
                    .ok_or("actual specimen names")?;
                let compound = separations
                    .iter()
                    .find(|s| s["scientist_id"] == "CHEMIST")
                    .ok_or("purified compound")?;
                let culture = separations
                    .iter()
                    .find(|s| s["scientist_id"] == "BIOLOGIST")
                    .ok_or("prepared culture")?;
                let specimen = if task == "13M" {
                    json!("synthesized_compound")
                } else {
                    compound["material"].clone()
                };
                calls.push(json!({"tool":"run_bioassay","arguments":{"compound":specimen,"cell_type":culture["material"]}}));
            }
            if task == "13M" && agent == "CHEMIST" {
                calls.push(
                    json!({"tool":"run_synthesis","arguments":{"compound":"synthesized_compound"}}),
                );
            }
            if task == "13M" && agent == "QC_CONTROLLER" {
                calls.push(json!({"tool":"verify_result","arguments":{"scientist":"CHEMIST","data_type":"purification"}}));
                calls.push(json!({"tool":"verify_result","arguments":{"scientist":"BIOLOGIST","data_type":"bioassay"}}));
            }
            Value::Array(calls)
        } else if matches!(task, "14E" | "14M") {
            let mut calls = Vec::new();
            for (tool, table, key) in [
                ("run_hplc_analysis", "_HPLC_STEPS", "sample"),
                ("run_mass_spec_analysis", "_MASS_SPEC_STEPS", "sample"),
                ("run_cell_assay", "_CELL_ASSAY_STEPS", "assay_type"),
            ] {
                for step in inputs[table]
                    .as_array()
                    .ok_or("instrument task inputs")?
                    .iter()
                    .filter(|s| s["scientist_id"] == *agent)
                {
                    calls.push(json!({"tool":tool,"arguments":{key:step[key]}}));
                }
            }
            if agent == "LEAD_SCIENTIST" {
                calls.push(json!({"tool":"submit_decision","arguments":{"go":true}}));
            }
            if task == "14M" && agent == "REGULATORY_SPECIALIST" {
                calls.push(json!({"tool":"verify_regulatory_compliance","arguments":{"decision":"go_no_go"}}));
            }
            Value::Array(calls)
        } else if task == "4M" {
            let mut calls = Vec::new();
            for (tool, table, field, key) in [
                (
                    "design_proposal",
                    "_DESIGN_PROPOSAL_STEPS",
                    "proposal",
                    "title",
                ),
                ("refactor_code", "_REFACTOR_STEPS", "target", "target"),
                ("code_review", "_CODE_REVIEW_STEPS", "target", "pr_id"),
                ("run_tests", "_RUN_TESTS_STEPS", "suite", "suite"),
                ("merge_code", "_MERGE_STEPS", "branch", "branch"),
            ] {
                for step in inputs[table]
                    .as_array()
                    .ok_or("shared-refactor task data")?
                    .iter()
                    .filter(|step| step["agent_id"] == *agent)
                {
                    calls.push(json!({"tool":tool,"arguments":{key:step[field]}}));
                }
            }
            Value::Array(calls)
        } else if task == "6M" {
            let mut calls = Vec::new();
            for (tool, table, field, key) in [
                (
                    "implement_feature",
                    "_IMPLEMENT_STEPS",
                    "feature",
                    "feature_name",
                ),
                ("write_tests", "_WRITE_TESTS_STEPS", "component", "target"),
                ("code_review", "_CODE_REVIEW_STEPS", "target", "pr_id"),
                ("run_tests", "_RUN_TESTS_STEPS", "suite", "suite"),
                ("merge_code", "_MERGE_STEPS", "branch", "branch"),
            ] {
                for step in inputs[table]
                    .as_array()
                    .ok_or("feature-delivery assignments")?
                    .iter()
                    .filter(|step| step["agent_id"] == *agent)
                {
                    calls.push(json!({"tool":tool,"arguments":{key:step[field]}}));
                }
            }
            Value::Array(calls)
        } else if task == "6E" {
            if agent == "REVIEWER" {
                Value::Array(inputs["_CODE_REVIEW_STEPS"].as_array().ok_or("original review inputs")?.iter().filter(|step|step["agent_id"]==*agent).map(|step|json!({"tool":"code_review","arguments":{"pr_id":step["target"]}})).collect())
            } else {
                json!([
                    {"tool":"implement_feature","arguments":{"feature_name":step("_IMPLEMENT_STEPS","feature")?}},
                    {"tool":"merge_code","arguments":{"branch":step("_MERGE_STEPS","branch")?}}
                ])
            }
        } else if task == "16E" {
            let (tool, table, owner, key) = match agent.as_str() {
                Some("DEVELOPER") => (
                    "compile_and_test",
                    "_BUILD_STEPS",
                    "developer_id",
                    "component",
                ),
                Some("TESTER") => ("run_integration_tests", "_TEST_STEPS", "agent_id", "suite"),
                Some("DEPLOYER") => ("deploy_application", "_DEPLOY_STEPS", "agent_id", "target"),
                _ => return Err("unknown original CI/CD agent".into()),
            };
            let value = inputs[table]
                .as_array()
                .ok_or("original CI/CD inputs")?
                .iter()
                .find(|step| step[owner] == *agent)
                .ok_or("original CI/CD assignment")?[key]
                .clone();
            if !value.is_string() {
                return Err("original CI/CD argument".into());
            }
            json!([{"tool":tool,"arguments":{key:value}}])
        } else if task == "8E" {
            let (tool, table, field, argument) = match agent.as_str() {
                Some("DB_ADMIN") => (
                    "migrate_schema",
                    "_MIGRATE_SCHEMA_STEPS",
                    "migration_name",
                    "migration_name",
                ),
                Some("BACKEND_DEV") => (
                    "implement_backend",
                    "_IMPLEMENT_BACKEND_STEPS",
                    "component",
                    "component",
                ),
                Some("TESTER") => (
                    "run_integration_tests",
                    "_INTEGRATION_TEST_STEPS",
                    "suite",
                    "suite",
                ),
                _ => return Err("unknown original API agent".into()),
            };
            json!([{"tool":tool,"arguments":{argument:step(table,field)?}}])
        } else {
            json!([])
        };
        let resources = if matches!(task, "10E" | "10M")
            || matches!(
                task,
                "3E" | "4M"
                    | "5E"
                    | "5M"
                    | "5H"
                    | "6M"
                    | "7E"
                    | "11E"
                    | "12E"
                    | "13E"
                    | "13M"
                    | "14E"
                    | "14M"
            ) {
            let mut names = std::collections::BTreeSet::new();
            for call in calls.as_array().ok_or("report assignments")? {
                names.extend(resources_for_call(
                    metadata,
                    agent.as_str().ok_or("participant")?,
                    call,
                )?);
            }
            json!(names)
        } else {
            metadata["tool_resource_map"][calls[0]["tool"].as_str().unwrap_or("")].clone()
        };
        actors.push(json!({"agent":agent,"calls":calls,
            "resources":resources,
            "modules":metadata["agent_resources"][agent.as_str().ok_or("agent name")?],
            "sections":metadata["agent_resources"][agent.as_str().ok_or("agent name")?],
            "forks":inputs["_FORKS"][agent.as_str().ok_or("agent name")?]}));
    }
    Ok(
        // These describe the original simulator's normal delay/failure profile.
        // They are never task authority. Actual model calls are verified from
        // participant events, independently of this environment metadata.
        json!({"type":"fixtures","task":task,"fixtures":actors,"inputs":inputs,
        "adapter":"rust_upstream_api","test_delays":{},
        "terminal_review_failure":false,"terminal_test_failure":false}),
    )
}

#[cfg(test)]
mod tests;
