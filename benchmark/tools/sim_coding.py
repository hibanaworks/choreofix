"""Method-neutral design, source, module commits and actual test receipts."""
from __future__ import annotations

from copy import deepcopy
from dataclasses import dataclass
import hashlib
import json

from ._base import ToolResult
from .sim_base import SimContext


@dataclass
class DesignStep:
    agent_id: str
    feature: str


@dataclass
class ImplementStep:
    agent_id: str
    feature: str


@dataclass
class CommitStep:
    agent_id: str
    modules: list[str]


@dataclass
class TestStep:
    agent_id: str
    feature: str


def artifact(content, **inputs):
    value = {"content": content, **deepcopy(inputs)}
    value["id"] = hashlib.sha256(json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=False
    ).encode()).hexdigest()
    return value


class CodingSim(SimContext):
    """A synthetic code world, not a compiler or choreography interpreter.

    Calls remain free, including retry and independent feature work. The
    existing event journal owns completed material and test receipts. No
    expected-next operation, phase cursor, done map or method's workflow is
    consulted. Actual module conflicts are observed by the common registry.
    """

    _DECISION_TOOLS = {"run_local_tests": 0.3}
    _TOOL_DELAYS = {
        "design_feature": (0.5, 1.5),
        "implement_code": (1.0, 3.0),
        "commit_changes": (0.5, 1.0),
        "run_local_tests": (1.5, 4.0),
    }

    def __init__(self, design_steps, implement_steps, commit_steps, test_steps, resources):
        super().__init__()
        self._design_steps = {f"{s.agent_id}_{s.feature}": s for s in design_steps}
        self._implement_steps = {f"{s.agent_id}_{s.feature}": s for s in implement_steps}
        self._commit_steps = {f"{s.agent_id}_{'_'.join(sorted(s.modules))}": s
                              for s in commit_steps}
        self._test_steps = {f"{s.agent_id}_{s.feature}": s for s in test_steps}
        self.specifications = {s.feature: f"Specification for {s.feature}"
                               for s in design_steps}
        for resource in resources:
            self.init_resource(resource)

    def tool_delay(self, tool_name, **kwargs):
        return self._TOOL_DELAYS.get(tool_name, (0.0, 0.0))

    def _latest(self, tool, agent, feature=None):
        events = [event for event in self.events if event.tool == tool
                  and event.agent == agent and event.success
                  and (feature is None or event.result.get("feature") == feature)]
        if not events or events[-1].completed_at is None:
            return None
        return events[-1]

    def _design(self, agent, feature):
        event = self._latest("design_feature", agent, feature)
        if (event is None or event.result["artifact"]["content"]
                != self.specifications.get(feature)):
            return None
        return event.result["artifact"]

    def _repair_input(self, agent, feature):
        failures = [event for event in self.events
                    if event.tool == "run_local_tests" and event.agent == agent
                    and event.result.get("feature") == feature
                    and event.result.get("status") == "tests_failed"
                    and event.completed_at is not None]
        if not failures:
            return None
        failure = failures[-1]
        return {"test_id": failure.result["test_id"],
                "tested_commit": failure.result["input_snapshot"]["id"],
                "diagnostic": failure.result["diagnostic"]}

    def _implementation(self, agent, feature):
        event = self._latest("implement_code", agent, feature)
        design = self._design(agent, feature)
        if (event is None or design is None
                or event.result["artifact"]["design"] != design
                or event.result["artifact"]["repair"] != self._repair_input(agent, feature)):
            return None
        return event.result["artifact"]

    def _commit(self, agent, feature):
        event = self._latest("commit_changes", agent, feature)
        source = self._implementation(agent, feature)
        if event is None or source is None or event.result["artifact"]["source"] != source:
            return None
        return event.result["artifact"]

    def _result(self, agent, tool, arguments, data, success=True):
        data = {"agent": agent, **data}
        self.log_event(agent, tool, arguments, success=success, result=data)
        return ToolResult(tool_name=tool, success=success, data=data,
                          message=f"{tool}: {data['status']}")

    def design_feature(self, agent_id, **kwargs):
        feature = self._get_param(kwargs, "feature_name", "feature")
        if not isinstance(feature, str) or not feature:
            return self._result(agent_id, "design_feature", kwargs,
                                {"feature": feature, "status": "invalid_arguments"}, False)
        text = self.specifications.setdefault(feature, f"Specification for {feature}")
        return self._result(agent_id, "design_feature", kwargs,
                            {"feature": feature, "status": "designed",
                             "artifact": artifact(text, feature=feature)})

    def implement_code(self, agent_id, **kwargs):
        feature = self._get_param(kwargs, "feature_name", "feature")
        design = self._design(agent_id, feature)
        if design is None:
            return self._result(agent_id, "implement_code", kwargs,
                                {"feature": feature, "status": "missing_current_design"}, False)
        repair = self._repair_input(agent_id, feature)
        code = f"Implementation of {feature}: {design['content']}"
        if repair is not None:
            code += "\nRepair for " + json.dumps(repair, sort_keys=True)
        return self._result(agent_id, "implement_code", kwargs,
                            {"feature": feature, "status": "implemented",
                             "artifact": artifact(code, design=design, repair=repair)})

    def commit_changes(self, agent_id, **kwargs):
        modules = self._parse_list(kwargs.get("modules", []))
        assigned = next((s for s in self._commit_steps.values()
                         if s.agent_id == agent_id and sorted(s.modules) == sorted(modules)), None)
        work = next((s for s in self._implement_steps.values() if s.agent_id == agent_id), None)
        source = self._implementation(agent_id, work.feature) if work else None
        if assigned is None or source is None:
            return self._result(agent_id, "commit_changes", kwargs,
                                {"modules": modules, "status": "missing_current_source"}, False)
        return self._result(agent_id, "commit_changes", kwargs,
                            {"feature": work.feature, "modules": modules, "status": "committed",
                             "artifact": artifact(source["content"], source=source,
                                                  modules=sorted(modules))})

    def run_local_tests(self, agent_id, **kwargs):
        feature = self._get_param(kwargs, "feature_name", "feature")
        commit = self._commit(agent_id, feature)
        if commit is None:
            return self._result(agent_id, "run_local_tests", kwargs,
                                {"feature": feature, "status": "missing_current_commit"}, False)
        passed = not self.should_fail("run_local_tests", agent_id)
        return self._result(agent_id, "run_local_tests", kwargs,
                            {"feature": feature, "status": "tests_passed" if passed else "tests_failed",
                             "test_id": f"test:{len(self.events) + 1}",
                             "input_snapshot": deepcopy(commit),
                             "diagnostic": "checks passed" if passed else "injected test defect"}, passed)

    def make_tools(self):
        return {name: getattr(self, name) for name in self._TOOL_DELAYS}

    def is_complete(self):
        for work in self._test_steps.values():
            commit = self._commit(work.agent_id, work.feature)
            verdicts = [event for event in self.events if event.tool == "run_local_tests"
                        and event.agent == work.agent_id
                        and event.result.get("feature") == work.feature]
            if not verdicts:
                return False
            verdict = verdicts[-1]
            if (commit is None or not verdict.success or verdict.completed_at is None
                    or verdict.result.get("input_snapshot") != commit):
                return False
        return True

    @property
    def progress(self):
        return {"commits": {f"{work.agent_id}:{work.feature}": deepcopy(
                    self._commit(work.agent_id, work.feature)) for work in self._test_steps.values()},
                "all_complete": self.is_complete()}
