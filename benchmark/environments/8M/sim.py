"""Full-stack development scored from completed artifacts and validation receipts."""

import json
from functools import partial
from pathlib import Path

from benchmark.tools._base import ToolResult
from benchmark.tools.sim_api_system import APISystemSim
from benchmark.tools.sim_base import SimContext, SimEvent

_METADATA = json.loads(
    (Path(__file__).resolve().parents[2] / "descriptions" / "8M" / "metadata.json").read_text()
)
_OPERATIONS = {
    "migrate_schema": ("DB_ADMIN", "migration_name", ("feature_schema",), "migrated"),
    "implement_backend": ("BACKEND_DEV", "component", ("api_endpoints",), "implemented"),
    "define_api": ("BACKEND_DEV", "endpoint", ("feature_api_spec",), "defined"),
    "implement_frontend": ("FRONTEND_DEV", "component", ("user_interface",), "implemented"),
    "code_review": ("REVIEWER", "pr_id", ("backend_api", "frontend_ui"), "reviewed"),
    "run_integration_tests": ("TESTER", "suite", ("full_stack_integration",), "tests passed"),
}


class FullStackAPIDevelopmentSim(SimContext):
    """Callers choose when to act. Each effect records the inputs available then."""

    _DECISION_TOOLS = {"code_review": 0.3, "run_integration_tests": 0.3}

    def __init__(self) -> None:
        super().__init__()
        self.load_from_metadata(_METADATA)
        for resource in _METADATA["resources"]:
            self.init_resource(resource)

    def make_tools(self):
        return {tool: partial(self._execute, tool) for tool in _OPERATIONS}

    def tool_delay(self, tool_name, **kwargs):
        return APISystemSim._TOOL_DELAYS[tool_name]

    def _latest(self, tool, *, target=None):
        for event in reversed(self._events):
            if event.tool != tool or event.result["status"] in {
                "invalid_target", "unavailable_inputs", "rework_required",
            }:
                continue
            if target is not None and event.args["pr_id"] != target:
                continue
            return event
        return None

    def _completed(self, tool, *, target=None):
        """A pending replacement cannot supply evidence for downstream calls."""
        event = self._latest(tool, target=target)
        return event if event is not None and event.completed_at is not None else None

    @staticmethod
    def _revision(event: SimEvent | None) -> int | None:
        return event.result["revision"] if event is not None else None

    def _backend(self, schema, api):
        backend = self._completed("implement_backend")
        if schema is None or api is None or backend is None:
            return None
        if backend.result["inputs"]["schema"] != schema:
            return None
        return {"schema": schema, "api": api, "code": self._revision(backend)}

    def _approval(self, target, subject):
        review = self._completed("code_review", target=target)
        if review is None or not review.success or review.result["inputs"] != subject:
            return None
        return self._revision(review)

    def _candidates(self):
        schema = self._revision(self._completed("migrate_schema"))
        contract = self._completed("define_api")
        api = contract.result["contract"].copy() if contract is not None else None
        backend = self._backend(schema, api)
        backend_review = self._approval("backend_api", backend)

        frontend = self._completed("implement_frontend")
        frontend_subject = None
        if backend_review is not None and frontend is not None:
            inputs = frontend.result["inputs"]
            if inputs["schema"] == schema and inputs["api"] == api and inputs["backend_review"] is not None:
                frontend_subject = {"schema": schema, "api": api, "code": self._revision(frontend)}

        return schema, api, backend, backend_review, frontend_subject

    def _execute(self, tool, agent_id, **arguments):
        owner, key, targets, status = _OPERATIONS[tool]
        target = arguments.get(key)
        outcome = {"agent": agent_id, key: target, "revision": len(self._events) + 1}
        if agent_id != owner or target not in targets:
            status = "invalid_target"
        elif tool == "define_api":
            method = arguments.get("method", "GET")
            if not isinstance(method, str) or not method.strip():
                status = "invalid_target"
            else:
                outcome["contract"] = {"endpoint": target, "method": method.strip().upper()}
        elif tool == "implement_backend":
            outcome["inputs"] = {"schema": self._revision(self._completed("migrate_schema"))}
        elif tool in {"implement_frontend", "code_review", "run_integration_tests"}:
            schema, api, backend, backend_review, frontend = self._candidates()
            if tool == "implement_frontend":
                outcome["inputs"] = {
                    "schema": schema, "api": api, "backend_review": backend_review,
                }
            elif tool == "code_review":
                subject = backend if target == "backend_api" else frontend
                outcome["inputs"] = subject
                prior = self._latest(tool, target=target)
                if subject is None or (prior is not None and prior.completed_at is None):
                    status = "unavailable_inputs"
                elif prior is not None and not prior.success and prior.result["inputs"] == subject:
                    status = "rework_required"
                elif self.should_fail(tool, agent_id):
                    status = "changes_requested"
            else:
                frontend_review = self._approval("frontend_ui", frontend)
                candidate = {
                    "schema": schema, "api": api,
                    "backend": backend["code"] if backend else None,
                    "frontend": frontend["code"] if frontend else None,
                }
                outcome["inputs"] = {
                    "candidate": candidate,
                    "backend_review": backend_review, "frontend_review": frontend_review,
                }
                prior = self._latest(tool)
                if (backend_review is None or frontend_review is None
                        or (prior is not None and prior.completed_at is None)):
                    status = "unavailable_inputs"
                elif prior is not None and not prior.success and prior.result["inputs"]["candidate"] == candidate:
                    status = "rework_required"
                elif self.should_fail(tool, agent_id):
                    status = "tests_failed"
        outcome["status"] = status
        success = status == _OPERATIONS[tool][3]
        self.log_event(agent_id, tool, arguments, success, outcome)
        return ToolResult(tool_name=tool, success=success, data=outcome, message=status.replace("_", " "))

    def is_complete(self):
        test = self._completed("run_integration_tests")
        if test is None or not test.success:
            return False
        schema, api, backend, backend_review, frontend = self._candidates()
        frontend_review = self._approval("frontend_ui", frontend)
        if backend_review is None or frontend_review is None:
            return False
        candidate = {"schema": schema, "api": api, "backend": backend["code"], "frontend": frontend["code"]}
        # A later positive review of the same revision need not repeat a passing test.
        return test.result["inputs"]["candidate"] == candidate

    @property
    def progress(self):
        return {
            "effects": len(self._events),
            "all_complete": self.is_complete(),
        }
