"""Build objects, shared material and checked link inputs for 10E/10M/10H.

This is a synthetic compiler/data model, not execution of a real compiler.
The journal records actual inputs and results. It never imports an agent schedule.
"""
from __future__ import annotations

import copy
import hashlib
import json
from dataclasses import dataclass

from ._base import ToolResult
from .sim_base import SimContext


@dataclass(frozen=True)
class CompileStep:
    agent_id: str
    module: str
    outputs: tuple[str, ...] = ()


@dataclass(frozen=True)
class BuildTestStep:
    agent_id: str
    module: str


@dataclass(frozen=True)
class ValidateStep:
    agent_id: str
    artifact: str
    module: str


@dataclass(frozen=True)
class LinkStep:
    agent_id: str
    modules: list[str]


class BuildSim(SimContext):
    _DECISION_TOOLS = {"run_build_tests": 0.3, "validate_artifact": 0.2}
    _TOOL_DELAYS = {
        "compile_module": (2.0, 5.0), "run_build_tests": (1.5, 4.0),
        "validate_artifact": (0.5, 1.5), "link_modules": (1.0, 3.0),
    }

    def __init__(self, *, compile_steps, link_steps, metadata,
                 initial_artifacts, build_test_steps=(), validate_steps=(),
                 resource_capacities=None):
        super().__init__()
        self._compile_steps = tuple(compile_steps)
        self._build_test_steps = tuple(build_test_steps)
        self._validate_steps = tuple(validate_steps)
        self._link_steps = tuple(link_steps)
        self._initial_artifacts = copy.deepcopy(initial_artifacts)
        self.load_from_metadata(metadata)
        for resource in metadata["resources"]:
            self.init_resource(resource, (resource_capacities or {}).get(resource, 1))
        modules = {step.module for step in self._compile_steps}
        materials = set(initial_artifacts) | {
            name for step in self._compile_steps for name in step.outputs
        }
        physical = metadata["tool_resource_map"]["compile_module"]
        resources = set(metadata["resources"])
        if (len(modules) != len(self._compile_steps)
                or not materials <= resources
                or materials | set(resource_capacities or {}) != resources
                or physical.get("argument") != "module_name"
                or set(physical.get("values", {})) != modules
                or any(not isinstance(physical["values"][step.module], list)
                       or len(set(physical["values"][step.module])) != len(physical["values"][step.module])
                       or not set(physical["values"][step.module]) <= resources
                       or not set(step.outputs) <= set(physical["values"][step.module])
                       for step in self._compile_steps)
                or any(step.module not in modules for step in self._build_test_steps)
                or any(step.module not in modules for step in self._validate_steps)
                or any(not set(step.modules) <= modules for step in self._link_steps)):
            raise ValueError("build project has unknown, duplicate or physically unbound data")

    @staticmethod
    def _normalize_module(value):
        return value.strip().lower().replace(" ", "_") if isinstance(value, str) else None

    def resource_requirements(self, tool_name, **kwargs):
        arguments = dict(kwargs)
        if tool_name == "compile_module":
            arguments["module_name"] = self._normalize_module(kwargs.get("module_name"))
        return super().resource_requirements(tool_name, **arguments)

    def tool_delay(self, tool_name, **kwargs):
        return self._TOOL_DELAYS.get(tool_name, (0.0, 0.0))

    def make_tools(self):
        names = ["compile_module", "link_modules"]
        if self._build_test_steps:
            names.append("run_build_tests")
        if self._validate_steps:
            names.append("validate_artifact")
        return {name: getattr(self, name) for name in names}

    def _record(self, tool, agent, arguments, success, result, violations=()):
        self.log_event(agent, tool, copy.deepcopy(arguments), success,
                       copy.deepcopy(result), list(violations))
        return ToolResult(tool_name=tool, success=success, data=result,
                          message=f"{tool}: {result['status']}")

    def _materials(self):
        values = {name: {"base": content, "modules": {}}
                  for name, content in self._initial_artifacts.items()}
        latest = {}
        for event in self.events:
            if event.tool == "compile_module" and event.success:
                for name, value in event.result["shared_outputs"].items():
                    values[name] = copy.deepcopy(value)
                    latest[name] = event
        return values, latest

    def _object(self, module):
        entry = next((event for event in reversed(self.events)
                      if event.tool == "compile_module" and event.success
                      and event.result["module"] == module), None)
        if entry is None or entry.completed_at is None:
            return None, [f"object not completed: {module}"]
        obj = entry.result["object"]
        material, latest = self._materials()
        issues = [f"shared input is not current and complete: {name}"
                  for name, value in obj["inputs"].items()
                  if material.get(name) != value
                  or name in latest and latest[name].completed_at is None]
        return copy.deepcopy(obj), issues

    def _proofs(self, module, obj):
        receipts, issues = {}, []
        for tool, steps in (("run_build_tests", self._build_test_steps),
                            ("validate_artifact", self._validate_steps)):
            for step in steps:
                if step.module != module:
                    continue
                event = max((entry for entry in self.events
                             if entry.tool == tool and entry.agent == step.agent_id
                             and entry.result["module"] == module
                             and entry.completed_at is not None),
                            key=lambda entry: entry.completed_at, default=None)
                if (event is None or not event.success
                        or event.result["input_object"] != obj):
                    issues.append(f"missing current completed {tool}: {module}")
                else:
                    receipts[tool] = event.result["receipt"]
        return receipts, issues

    def compile_module(self, agent_id, **kwargs):
        module = self._normalize_module(kwargs.get("module_name"))
        step = next((step for step in self._compile_steps if step.module == module), None)
        if step is None:
            return self._record("compile_module", agent_id, kwargs, False,
                                {"module": module, "status": "unknown_module"})
        material, latest = self._materials()
        names = set(material) | {name for step in self._compile_steps for name in step.outputs}
        required = [name for name in self.resource_requirements(
            "compile_module", agent_id=agent_id, module_name=module) if name in names]
        unavailable = [name for name in required
                       if name not in material and name not in step.outputs
                       or name in latest and latest[name].completed_at is None]
        if unavailable:
            return self._record("compile_module", agent_id, kwargs, False, {
                "module": module, "status": "inputs_unavailable", "inputs": unavailable,
            })
        updates = {}
        for name in step.outputs:
            value = copy.deepcopy(material.get(name, {"base": "", "modules": {}}))
            value["modules"][module] = f"declarations from {module}"
            if value != material.get(name):
                updates[name] = value
            material[name] = value
        inputs = {name: copy.deepcopy(material[name]) for name in required}
        encoded = json.dumps({"module": module, "inputs": inputs},
                             sort_keys=True, ensure_ascii=False, separators=(",", ":"))
        obj = {"id": f"object:{len(self.events) + 1}",
               "content_sha256": hashlib.sha256(encoded.encode()).hexdigest(),
               "module": module, "builder": agent_id, "bytes": encoded, "inputs": inputs}
        return self._record("compile_module", agent_id, kwargs, True, {
            "module": module, "status": "compiled", "object": obj,
            "shared_outputs": updates, "artifacts_modified": bool(updates),
            "modified_artifacts": sorted(updates), "agent": agent_id,
        })

    def _check(self, tool, agent, arguments, module, target_valid):
        obj, issues = self._object(module) if target_valid else (None, ["unknown_target"])
        violations = []
        failure = "inputs_unavailable" if obj is None or issues else None
        if failure is None:
            if any(event.tool == tool
                   and event.completed_at is None and event.result.get("input_object") is not None
                   for event in self.events):
                violations.append(self.log_violation(
                    agent, tool, "concurrent_worker_use",
                    f"{tool} has one worker, but executions overlap"))
            # A negative assessment belongs to this actual build occurrence.
            # Retrying the same object cannot make it a repaired build. A new
            # compile creates another object even if its bytes are unchanged.
            unchanged_failure = any(
                event.tool == tool and event.agent == agent and not event.success
                and event.completed_at is not None and event.result["input_object"] == obj
                for event in self.events
            )
            if unchanged_failure:
                failure = "unchanged_failed_build"
            elif self.should_fail(tool, agent):
                failure = "injected_failure"
        success = failure is None
        return self._record(tool, agent, arguments, success, {
            "module": module, "artifact": arguments.get("artifact"),
            "status": failure or "passed",
            "test_result" if tool == "run_build_tests" else "validation_result":
                "pass" if success else "fail",
            "input_object": obj, "issues": issues,
            "receipt": f"{tool}:{len(self.events) + 1}", "agent": agent,
        }, violations)

    def run_build_tests(self, agent_id, **kwargs):
        module = self._normalize_module(kwargs.get("module_name"))
        return self._check("run_build_tests", agent_id, kwargs, module,
                           any(step.module == module for step in self._build_test_steps))

    def validate_artifact(self, agent_id, **kwargs):
        artifact = self._normalize_module(kwargs.get("artifact"))
        step = next((step for step in self._validate_steps if step.artifact == artifact), None)
        return self._check("validate_artifact", agent_id, kwargs,
                           step.module if step else None, step is not None)

    def link_modules(self, agent_id, **kwargs):
        raw = kwargs.get("modules")
        raw = raw.split(",") if isinstance(raw, str) else raw
        modules = [self._normalize_module(name) for name in raw] if isinstance(raw, list) else []
        known = {step.module for step in self._compile_steps}
        if not modules or len(set(modules)) != len(modules) or not set(modules) <= known:
            return self._record("link_modules", agent_id, kwargs, False, {
                "modules": modules, "status": "invalid_modules", "inputs": {},
            })
        objects, receipts, issues = {}, {}, []
        for module in modules:
            obj, errors = self._object(module)
            issues.extend(errors)
            if obj is not None:
                objects[module] = obj
                receipts[module], errors = self._proofs(module, obj)
                issues.extend(errors)
        if len(objects) != len(modules):
            return self._record("link_modules", agent_id, kwargs, False, {
                "modules": modules, "status": "missing_objects", "inputs": objects,
                "issues": issues,
            })
        # A linker can physically emit untested/stale objects. Preserve that
        # effect and its violation instead of turning the world into a gate.
        violations = []
        if issues:
            violations.append(self.log_violation(
                agent_id, "link_modules", "uncertified_link", "; ".join(issues)))
        encoded = json.dumps(objects, sort_keys=True, ensure_ascii=False, separators=(",", ":"))
        return self._record("link_modules", agent_id, kwargs, True, {
            "modules": modules, "status": "linked" if not issues else "linked_with_issues",
            "inputs": objects, "checks": receipts, "issues": issues,
            "binary": {"id": f"link:{hashlib.sha256(encoded.encode()).hexdigest()}", "bytes": encoded},
            "agent": agent_id,
        }, violations)

    def is_complete(self):
        objects = {}
        for step in self._compile_steps:
            obj, issues = self._object(step.module)
            if obj is None or issues or obj["builder"] != step.agent_id:
                return False
            _, issues = self._proofs(step.module, obj)
            if issues:
                return False
            objects[step.module] = obj
        for step in self._link_steps:
            expected = {module: objects[module] for module in step.modules}
            if not any(event.tool == "link_modules" and event.agent == step.agent_id
                       and event.success and event.completed_at is not None
                       and not event.result["issues"] and event.result["inputs"] == expected
                       for event in self.events):
                return False
        return True

    @property
    def progress(self):
        material, _ = self._materials()
        return {
            "objects": {step.module: self._object(step.module)[0] for step in self._compile_steps},
            "shared_artifacts": material,
            "checks": [copy.deepcopy(event.result) for event in self.events
                       if event.tool in {"run_build_tests", "validate_artifact"}
                       and event.completed_at is not None],
            "links": [copy.deepcopy(event.result) for event in self.events
                      if event.tool == "link_modules" and event.completed_at is not None],
            "all_complete": self.is_complete(),
        }
