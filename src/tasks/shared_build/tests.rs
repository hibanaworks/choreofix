use super::{global::*, imp, localside, wire::Arguments};
use crate::shell::imp::service::ResourceReceipt;
use crate::shell::{
    ShellError, ShellResult,
    imp::{
        bus::Bus,
        service::{PairedEffects, ReceivedEffects, SimulatorOperation},
    },
};
use hibana::runtime::wire::ReceivedPayload;
use hibana::runtime::{SessionKitStorage, ids::SessionId};
use hibana_wasip1_runtime::protocol;
use serde_json::{Value, json};
use std::cell::Cell;
// Native material fixture. Actual Wasm and model runs use the neutral world.
struct BuildData(Cell<Vec<Value>>);
impl BuildData {
    fn material_result(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        let mut journal = self.0.take();
        let data = if tool == "compile_module" {
            let module = arguments["module_name"]
                .as_str()
                .ok_or("module name")?
                .trim()
                .to_lowercase()
                .replace(' ', "_");
            let changed = agent == "BUILDER_A"
                || journal
                    .iter()
                    .any(|entry| entry["data"]["agent"] == "BUILDER_A");
            let inputs = json!({"CORE_LIB":{"base":"core","modules":{}},"SHARED_TYPES":{"base":"provided types","modules":if changed {json!({"module_a":"declarations from module_a"})} else {json!({})}}});
            let bytes = String::from_utf8(
                imp::encode(&json!({"module":module,"inputs":inputs}))
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
            json!({"agent":agent,"module":module,"status":"compiled","object":{"id":format!("object:{}",journal.len()),"bytes":bytes,"content_sha256":"native-data-fixture","module":module,"builder":agent,"inputs":inputs}})
        } else {
            assert_eq!(tool, "link_modules");
            let a = &journal
                .iter()
                .rev()
                .find(|entry| entry["data"]["agent"] == "BUILDER_A")
                .ok_or("missing A object")?["data"]["object"];
            let b = &journal
                .iter()
                .rev()
                .find(|entry| entry["data"]["agent"] == "BUILDER_B")
                .ok_or("missing B object")?["data"]["object"];
            let inputs = json!({"module_a":a,"module_b":b});
            json!({"agent":agent,"status":"linked","inputs":inputs,"issues":[],"binary":{"id":"native-link","bytes":String::from_utf8(imp::encode(&inputs).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?}})
        };
        let result = json!({"tool_name":tool,"success":true,"data":data});
        journal.push(result.clone());
        self.0.set(journal);
        Ok(result)
    }
}
impl ReceivedEffects for BuildData {
    fn apply<'io, P: SimulatorOperation>(
        &'io self,
        received: ReceivedPayload<'_, P>,
    ) -> impl Future<Output = ShellResult<Value>> + use<'io, P> {
        let value = imp::decode(received.as_bytes());
        async move {
            self.material_result(P::RPC.0, P::RPC.1, value?)
                .map_err(ShellError::message)
        }
    }
}
impl PairedEffects for BuildData {
    async fn apply_pair<'a, 'b, P: SimulatorOperation<2>>(
        &self,
        first: impl Into<ResourceReceipt<'a, P>>,
        second: impl Into<ResourceReceipt<'b, P>>,
    ) -> ShellResult<Value> {
        let first = first.into();
        let second = second.into();

        let value = imp::decode(first.as_bytes())?;
        if value != imp::decode(second.as_bytes())? {
            return Err(ShellError::message("compile inputs disagree"));
        }
        self.material_result(P::RPC.0, P::RPC.1, value)
            .map_err(ShellError::message)
    }
}
#[test]
fn the_direct_files_and_resource_handoffs_form_one_valid_global() {
    assert_eq!(hibana::g::diagnose(&source()), None);
    assert_eq!(
        programs().observation()["programs"]
            .as_array()
            .unwrap()
            .len(),
        18
    );
}
#[test]
fn file_payload_validation_preserves_values_and_rejects_wrong_modules_or_extra_fields() {
    for (role, bytes, valid) in [
        (BUILDER_A, br#"{"module_name":"Module A"}"#.as_slice(), true),
        (
            BUILDER_A,
            br#"{"module_name":"module_b"}"#.as_slice(),
            false,
        ),
        (BUILDER_B, br#"{"module_name":"module_b"}"#.as_slice(), true),
        (
            BUILDER_B,
            br#"{"module_name":"module_b","extra":true}"#.as_slice(),
            false,
        ),
        (
            INTEGRATOR,
            br#"{"modules":"Module B, Module A"}"#.as_slice(),
            true,
        ),
        (
            INTEGRATOR,
            br#"{"modules":"module_a,module_a,module_b"}"#.as_slice(),
            false,
        ),
    ] {
        let result = match role {
            BUILDER_A => Arguments::<BUILDER_A>::from_bytes(bytes).map(|v| v.into_value()),
            BUILDER_B => Arguments::<BUILDER_B>::from_bytes(bytes).map(|v| v.into_value()),
            INTEGRATOR => Arguments::<INTEGRATOR>::from_bytes(bytes).map(|v| v.into_value()),
            _ => unreachable!(),
        };
        assert_eq!(result.is_ok(), valid);
        if let Ok(value) = result {
            assert_eq!(value, serde_json::from_slice::<Value>(bytes).unwrap());
        }
    }
}
#[hibana_runtime::main]
#[test]
async fn both_first_builders_use_real_file_and_resource_receives_before_their_effects()
-> ShellResult<()> {
    let p = programs();
    for first in [BUILDER_A, BUILDER_B] {
        let session = SessionId::new(SESSION);
        let (bus, _io, [a_terminal, b_terminal, i_terminal]) = Bus::open(
            ["BUILDER_A", "BUILDER_B", "INTEGRATOR"],
            [
                (session, CORE_LIB),
                (session, SHARED_TYPES),
                (session, A_FILE),
                (session, B_FILE),
                (session, INTEGRATOR_FILE),
            ],
        );
        let mut owner_slab = vec![0; 1024 * 1024];
        let mut caller_slab = vec![0; 1024 * 1024];
        let mut owner_storage = SessionKitStorage::uninit();
        let mut caller_storage = SessionKitStorage::uninit();
        let owner = owner_storage
            .init()
            .rendezvous(&mut owner_slab, &bus.transport)?;
        let caller = caller_storage
            .init()
            .rendezvous(&mut caller_slab, &bus.transport)?;
        let mut service = owner.enter(session, &p.service)?;
        let core = owner.enter(session, &p.core)?;
        let types = owner.enter(session, &p.types)?;
        let mut a = owner.enter(session, &p.a)?;
        let mut b = owner.enter(session, &p.b)?;
        let mut integrator = owner.enter(session, &p.integrator)?;
        let mut adata = owner.enter(session, &p.a_data)?;
        let mut bdata = owner.enter(session, &p.b_data)?;
        let mut idata = owner.enter(session, &p.integrator_data)?;
        let mut aio = owner.enter(session, &p.a_io)?;
        let mut bio = owner.enter(session, &p.b_io)?;
        let mut iio = owner.enter(session, &p.integrator_io)?;
        let mut avm = caller.enter(session, &p.a_vm)?;
        let mut bvm = caller.enter(session, &p.b_vm)?;
        let mut ivm = caller.enter(session, &p.integrator_vm)?;
        let afile = owner.enter(session, &p.a_file)?;
        let bfile = owner.enter(session, &p.b_file)?;
        let ifile = owner.enter(session, &p.integrator_file)?;
        let effects = BuildData(Cell::new(Vec::new()));
        hibana_runtime::race(
            async {
                async {
                    crate::shell::localside::file_metadata::run(&mut adata, afile, [FILES[0]])
                        .await?;
                };
                async {
                    crate::shell::localside::file_metadata::run(
                        &mut bdata,
                        bfile,
                        [FILES[0], FILES[2]],
                    )
                    .await?;
                };
                async {
                    crate::shell::localside::file_metadata::run(&mut idata, ifile, [FILES[1]])
                        .await?;
                };
                let (mut afile, mut bfile, ifile) = async {
                    let [mut a, mut b, mut integrator] = service.par()?;
                    async { a.recv::<Loan<A_FILE>>().await };
                    async { b.recv::<Loan<B_FILE>>().await };
                    async { integrator.recv::<Loan<INTEGRATOR_FILE>>().await };
                }
                .await?;
                async {
                    let io = crate::shell::imp::private_fs::PrivateFs::new(
                        a_terminal,
                        &bus,
                        &FILES[..1],
                    );
                    let request = aio
                        .offer()
                        .await?
                        .recv::<protocol::FdFdstatGetReqMsg>()
                        .await?;
                    let reply = io.objects().choreofs().fd_fdstat_get(request);
                    aio.send::<protocol::FdFdstatGetRetMsg>(&reply).await?;
                    assert_eq!(
                        aio.offer()
                            .await?
                            .recv::<crate::shell::global::Exit>()
                            .await?,
                        0
                    );
                };
                async {
                    let io = crate::shell::imp::private_fs::PrivateFs::new(
                        b_terminal,
                        &bus,
                        &[FILES[0], FILES[2]],
                    );
                    let request = bio
                        .offer()
                        .await?
                        .recv::<protocol::FdFdstatGetReqMsg>()
                        .await?;
                    let reply = io.objects().choreofs().fd_fdstat_get(request);
                    bio.send::<protocol::FdFdstatGetRetMsg>(&reply).await?;
                    assert_eq!(
                        bio.offer()
                            .await?
                            .recv::<crate::shell::global::Exit>()
                            .await?,
                        0
                    );
                };
                async {
                    let io = crate::shell::imp::private_fs::PrivateFs::new(
                        i_terminal,
                        &bus,
                        &FILES[1..2],
                    );
                    let request = iio
                        .offer()
                        .await?
                        .recv::<protocol::FdFdstatGetReqMsg>()
                        .await?;
                    let reply = io.objects().choreofs().fd_fdstat_get(request);
                    iio.send::<protocol::FdFdstatGetRetMsg>(&reply).await?;
                    assert_eq!(
                        iio.offer()
                            .await?
                            .recv::<crate::shell::global::Exit>()
                            .await?,
                        0
                    );
                };
                async {
                    localside::resources::run(&mut service, (core, types))
                        .await
                        .map_err(|e| ShellError::message(format!("resources: {e:?}")))?;
                };
                async {
                    localside::builder::run(&mut a, &effects)
                        .await
                        .map_err(|e| ShellError::message(format!("builder A: {e:?}")))?;
                };
                async {
                    localside::builder::run(&mut b, &effects)
                        .await
                        .map_err(|e| ShellError::message(format!("builder B: {e:?}")))?;
                };
                async {
                    localside::integrator::run(&mut integrator, &effects)
                        .await
                        .map_err(|e| ShellError::message(format!("integrator: {e:?}")))?;
                };
                match first {
                    BUILDER_A => {
                        afile
                            .send::<CompileA>(&Arguments::from_bytes(
                                br#"{"module_name":"module_a"}"#,
                            )?)
                            .await?
                    }
                    BUILDER_B => {
                        bfile
                            .send::<CompileB>(&Arguments::from_bytes(
                                br#"{"module_name":"module_b"}"#,
                            )?)
                            .await?
                    }
                    _ => unreachable!("two declared first-request fixtures"),
                }
                async {
                    let mut afile = afile;
                    afile
                        .send::<protocol::FdFdstatGetReqMsg>(&protocol::FdFdstatGetReq(
                            protocol::FdRequest::new(4),
                        ))
                        .await?;
                    afile.recv::<protocol::FdFdstatGetRetMsg>().await?;
                    avm.send::<protocol::FdFdstatGetReqMsg>(&protocol::FdFdstatGetReq(
                        protocol::FdRequest::new(0),
                    ))
                    .await?;
                    avm.recv::<protocol::FdFdstatGetRetMsg>().await?;
                    let arguments =
                        Arguments::<BUILDER_A>::from_bytes(br#"{"module_name":"module_a"}"#)
                            .unwrap();
                    if first != BUILDER_A {
                        afile
                            .send::<CompileA>(&arguments)
                            .await
                            .map_err(|e| ShellError::message(format!("A file request: {e:?}")))?;
                    }
                    imp::compiled(
                        &imp::decode(
                            afile.recv::<Compiled>().await.map_err(|e| {
                                ShellError::message(format!("A file result: {e:?}"))
                            })?,
                        )?,
                        "BUILDER_A",
                        "module_a",
                    )?;
                    afile
                        .send::<FileExit>(&0)
                        .await
                        .map_err(|e| ShellError::message(format!("A file close: {e:?}")))?;
                    avm.send::<crate::shell::global::Exit>(&0)
                        .await
                        .map_err(|e| ShellError::message(format!("A root close: {e:?}")))?;
                    Ok(())
                };
                async {
                    let mut bfile = bfile;
                    bfile
                        .send::<protocol::FdFdstatGetReqMsg>(&protocol::FdFdstatGetReq(
                            protocol::FdRequest::new(4),
                        ))
                        .await?;
                    bfile.recv::<protocol::FdFdstatGetRetMsg>().await?;
                    bvm.send::<protocol::FdFdstatGetReqMsg>(&protocol::FdFdstatGetReq(
                        protocol::FdRequest::new(0),
                    ))
                    .await?;
                    bvm.recv::<protocol::FdFdstatGetRetMsg>().await?;
                    let arguments =
                        Arguments::<BUILDER_B>::from_bytes(br#"{"module_name":"module_b"}"#)?;
                    if first != BUILDER_B {
                        imp::compiled(
                            &imp::decode(bfile.recv::<TypesUpdated>().await?)?,
                            "BUILDER_A",
                            "module_a",
                        )?;
                        bfile.send::<CompileB>(&arguments).await?;
                    }
                    let mut result = imp::decode(bfile.recv::<Compiled>().await?)?;
                    loop {
                        let object = imp::compiled(&result, "BUILDER_B", "module_b")?;
                        if object["inputs"]["SHARED_TYPES"]["modules"]["module_a"].is_string() {
                            break;
                        }
                        imp::compiled(
                            &imp::decode(bfile.recv::<TypesUpdated>().await?)?,
                            "BUILDER_A",
                            "module_a",
                        )?;
                        bfile.send::<CompileB>(&arguments).await?;
                        result = imp::decode(bfile.recv::<Compiled>().await?)?;
                    }
                    bfile
                        .send::<FileExit>(&0)
                        .await
                        .map_err(|e| ShellError::message(format!("B file close: {e:?}")))?;
                    bvm.send::<crate::shell::global::Exit>(&0)
                        .await
                        .map_err(|e| ShellError::message(format!("B root close: {e:?}")))?;
                    Ok(())
                };
                async {
                    let mut ifile = ifile;
                    ifile
                        .send::<protocol::FdFdstatGetReqMsg>(&protocol::FdFdstatGetReq(
                            protocol::FdRequest::new(4),
                        ))
                        .await?;
                    ifile.recv::<protocol::FdFdstatGetRetMsg>().await?;
                    ivm.send::<protocol::FdFdstatGetReqMsg>(&protocol::FdFdstatGetReq(
                        protocol::FdRequest::new(0),
                    ))
                    .await?;
                    ivm.recv::<protocol::FdFdstatGetRetMsg>().await?;
                    let arguments =
                        Arguments::<INTEGRATOR>::from_bytes(br#"{"modules":"module_a,module_b"}"#)
                            .unwrap();
                    ifile
                        .send::<Link>(&arguments)
                        .await
                        .map_err(|e| ShellError::message(format!("I file request: {e:?}")))?;
                    assert_eq!(
                        imp::decode(
                            ifile
                                .recv::<Linked>()
                                .await
                                .map_err(|e| ShellError::message(format!(
                                    "I file result: {e:?}"
                                )))?
                        )?["data"]["status"],
                        "linked"
                    );
                    ifile
                        .send::<FileExit>(&0)
                        .await
                        .map_err(|e| ShellError::message(format!("I file close: {e:?}")))?;
                    ivm.send::<crate::shell::global::Exit>(&0)
                        .await
                        .map_err(|e| ShellError::message(format!("I root close: {e:?}")))?;
                    Ok(())
                };
                Ok::<(), ShellError>(())
            },
            hibana_runtime::host::deadline(std::time::Duration::from_secs(10)),
        )
        .await?;
        let journal = effects.0.take();
        let agents: Vec<_> = journal
            .iter()
            .map(|value| value["data"]["agent"].as_str().unwrap())
            .collect();
        assert_eq!(
            agents,
            match first {
                BUILDER_A => vec!["BUILDER_A", "BUILDER_B", "INTEGRATOR"],
                BUILDER_B => vec!["BUILDER_B", "BUILDER_A", "BUILDER_B", "INTEGRATOR"],
                _ => unreachable!("two declared first-request fixtures"),
            }
        );
        if first == BUILDER_B {
            assert_ne!(
                journal[0]["data"]["object"]["inputs"]["SHARED_TYPES"],
                journal[2]["data"]["object"]["inputs"]["SHARED_TYPES"]
            );
        }
    }
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn a_stale_object_is_rejected_by_the_actual_link_receiver_before_any_link_effect()
-> ShellResult<()> {
    use hibana::{g, runtime::program::project};
    let effects = BuildData(Cell::new(Vec::new()));
    let b = effects
        .material_result(
            "BUILDER_B",
            "compile_module",
            json!({"module_name":"module_b"}),
        )
        .map_err(ShellError::message)?;
    let a = effects
        .material_result(
            "BUILDER_A",
            "compile_module",
            json!({"module_name":"module_a"}),
        )
        .map_err(ShellError::message)?;
    let global = g::seq(
        g::par(
            g::send::<BUILDER_A, INTEGRATOR, BuiltA>(),
            g::send::<BUILDER_B, INTEGRATOR, BuiltB>(),
        ),
        g::seq(
            g::send::<INTEGRATOR_FILE, INTEGRATOR, Link>(),
            g::send::<INTEGRATOR, INTEGRATOR_FILE, Linked>(),
        ),
    );
    let pa = project::<BUILDER_A>(&global);
    let pb = project::<BUILDER_B>(&global);
    let pi = project::<INTEGRATOR>(&global);
    let pf = project::<INTEGRATOR_FILE>(&global);
    let (bus, _io, _terminals) = Bus::open(["BUILDER_A", "BUILDER_B", "INTEGRATOR"], []);
    let mut slab = vec![0; 64 * 1024];
    let mut storage = SessionKitStorage::uninit();
    let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
    let session = SessionId::new(SESSION);
    let mut sender_a = rv.enter(session, &pa)?;
    let mut sender_b = rv.enter(session, &pb)?;
    let mut receiver = rv.enter(session, &pi)?;
    let mut file = rv.enter(session, &pf)?;
    let a = imp::encode(&a)?;
    let b = imp::encode(&b)?;
    let request =
        Arguments::<INTEGRATOR>::from_bytes(br#"{"modules":"module_a,module_b"}"#).unwrap();
    let result = hibana_runtime::race(
        async {
            async {
                localside::integrator::run(&mut receiver, &effects).await?;
            };
            async {
                sender_a.send::<BuiltA>(&a.as_slice()).await?;
            };
            async {
                sender_b.send::<BuiltB>(&b.as_slice()).await?;
            };
            async {
                file.send::<Link>(&request).await?;
            };
            Ok::<(), ShellError>(())
        },
        hibana_runtime::host::deadline(std::time::Duration::from_secs(5)),
    )
    .await;
    assert!(result.is_err_and(
        |error| error.to_string() == "current objects were built against different types"
    ));
    let journal = effects.0.take();
    assert_eq!(
        journal.len(),
        2,
        "only the two material-producing fixture calls may exist"
    );
    assert!(
        journal
            .iter()
            .all(|entry| entry["tool_name"] == "compile_module")
    );
    Ok(())
}
