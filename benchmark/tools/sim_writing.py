"""Method-neutral literature, drafts, contributions and reviews for 2E/M/H."""

from __future__ import annotations

from copy import deepcopy
from dataclasses import dataclass
import hashlib
import json
from typing import Any

from ._base import ToolResult
from .sim_base import SimContext


@dataclass
class ReadStep:
    agent_id: str
    topic: str


@dataclass
class DraftStep:
    agent_id: str
    section: str


@dataclass
class WriteStep:
    agent_id: str
    sections: list[str]


@dataclass
class ReviewStep:
    agent_id: str
    section: str


def artifact(content, **inputs):
    value = {"content": content, **deepcopy(inputs)}
    value["id"] = hashlib.sha256(json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=False
    ).encode()).hexdigest()
    return value


class PaperWritingSim(SimContext):
    """Calls are free; completion needs actual content-bound results.

    There is no expected-next-call or done table. Immutable assignments describe
    the task. The ordinary event journal owns the actual material and results;
    unfinished replacements cannot supply another operation's inputs.
    """

    _DECISION_TOOLS = {"review_own_work": 0.3}
    _TOOL_DELAYS = {
        "read_literature": (0.5, 1.5),
        "draft_content": (1.0, 2.0),
        "write_to_document": (0.5, 1.0),
        "review_own_work": (0.5, 1.5),
    }

    def __init__(self, read_steps, draft_steps, write_steps, review_steps, resources):
        super().__init__()
        self._read_steps = {f"{s.agent_id}_{s.topic}": s for s in read_steps}
        self._draft_steps = {f"{s.agent_id}_{s.section}": s for s in draft_steps}
        self._write_steps = {f"{s.agent_id}_{'_'.join(sorted(s.sections))}": s
                             for s in write_steps}
        self._review_steps = {f"{s.agent_id}_{s.section}": s for s in review_steps}
        # Source text is world data, not an operation permission or phase.
        self.library = {s.topic: f"Literature on {s.topic}" for s in read_steps}
        for resource in resources:
            self.init_resource(resource)

    def tool_delay(self, tool_name, **kwargs):
        return self._TOOL_DELAYS.get(tool_name, (0.0, 0.0))

    def _latest(self, tool, agent, **fields):
        events = [event for event in self.events if event.tool == tool
                  and event.agent == agent and event.success
                  and all(event.result.get(key) == value for key, value in fields.items())]
        if not events or events[-1].completed_at is None:
            return None
        return events[-1]

    def _literature(self, agent):
        inputs = {}
        for step in self._read_steps.values():
            if step.agent_id == agent:
                event = self._latest("read_literature", agent, topic=step.topic)
                if event is None:
                    return None
                inputs[step.topic] = deepcopy(event.result["artifact"])
        return inputs or None

    def _draft(self, agent, section):
        event = self._latest("draft_content", agent, section=section)
        inputs = self._literature(agent)
        if event is None or inputs is None or event.result["artifact"]["literature"] != inputs:
            return None
        return event.result["artifact"]

    def _contribution(self, agent, section):
        event = self._latest("write_to_document", agent, section=section)
        draft = self._draft(agent, section)
        if event is None or draft is None or event.result["artifact"]["draft"] != draft:
            return None
        return event.result["artifact"]

    def _result(self, agent, tool, arguments, data, success=True):
        data = {"agent": agent, **data}
        self.log_event(agent, tool, arguments, success=success, result=data)
        return ToolResult(tool_name=tool, success=success, data=data,
                          message=f"{tool}: {data['status']}")

    def read_literature(self, agent_id, **kwargs):
        topic = kwargs.get("topic")
        if not isinstance(topic, str) or not topic:
            return self._result(agent_id, "read_literature", kwargs,
                                {"topic": topic, "status": "invalid_arguments"}, False)
        content = self.library.setdefault(topic, f"Literature on {topic}")
        return self._result(agent_id, "read_literature", kwargs,
                            {"topic": topic, "status": "read",
                             "artifact": artifact(content, topic=topic)})

    def draft_content(self, agent_id, **kwargs):
        section = self._get_param(kwargs, "section_name", "section")
        inputs = self._literature(agent_id)
        if not isinstance(section, str) or not section or inputs is None:
            return self._result(agent_id, "draft_content", kwargs,
                                {"section": section, "status": "missing_literature"}, False)
        content = f"Draft {section}: " + "\n".join(
            value["content"] for _, value in sorted(inputs.items()))
        return self._result(agent_id, "draft_content", kwargs,
                            {"section": section, "status": "drafted",
                             "artifact": artifact(content, literature=inputs)})

    def write_to_document(self, agent_id, **kwargs):
        sections = self._parse_list(kwargs.get("sections", []))
        assigned = next((s for s in self._write_steps.values()
                         if s.agent_id == agent_id and sorted(s.sections) == sorted(sections)), None)
        draft_step = next((s for s in self._draft_steps.values()
                           if s.agent_id == agent_id), None)
        draft = self._draft(agent_id, draft_step.section) if draft_step else None
        if assigned is None or draft is None:
            return self._result(agent_id, "write_to_document", kwargs,
                                {"sections": sections, "status": "missing_current_draft"}, False)
        return self._result(agent_id, "write_to_document", kwargs,
                            {"section": draft_step.section, "sections": sections,
                             "status": "written", "artifact": artifact(
                                 draft["content"], draft=draft, sections=sorted(sections))})

    def review_own_work(self, agent_id, **kwargs):
        section = self._get_param(kwargs, "section_name", "section")
        contribution = self._contribution(agent_id, section)
        if contribution is None:
            return self._result(agent_id, "review_own_work", kwargs,
                                {"section": section, "status": "missing_current_contribution"}, False)
        passed = not self.should_fail("review_own_work", agent_id)
        return self._result(agent_id, "review_own_work", kwargs,
                            {"section": section, "status": "reviewed" if passed else "needs_revision",
                             "review_id": f"review:{len(self.events) + 1}",
                             "input_snapshot": deepcopy(contribution)}, passed)

    def make_tools(self):
        return {name: getattr(self, name) for name in self._TOOL_DELAYS}

    def is_complete(self):
        for step in self._review_steps.values():
            contribution = self._contribution(step.agent_id, step.section)
            review = self._latest("review_own_work", step.agent_id, section=step.section)
            # The latest completed failed verdict cannot be erased by a prior
            # passing review. Failure is terminal for the protocol, not a pass.
            verdicts = [event for event in self.events if event.tool == "review_own_work"
                        and event.agent == step.agent_id and event.result.get("section") == step.section]
            if (contribution is None or review is None or not verdicts
                    or verdicts[-1] is not review
                    or review.result["input_snapshot"] != contribution):
                return False
        return True

    @property
    def progress(self):
        return {"contributions": {step.section: deepcopy(self._contribution(
                    step.agent_id, step.section)) for step in self._review_steps.values()},
                "all_complete": self.is_complete()}
