"""Document effects and content-bound evidence shared by 3M and 7E/7M/7H.

The journal owns material history. Task configuration names actual required
inputs and outputs, never a preferred communication or resource-acquisition order.
"""
from __future__ import annotations

import copy
import time
from dataclasses import dataclass

from ._base import ToolResult
from .sim_base import SimContext, Violation


@dataclass(frozen=True)
class ResearchStep:
    agent_id: str
    topic: str
    section: str


@dataclass(frozen=True)
class WriteSectionStep:
    agent_id: str
    section: str


@dataclass(frozen=True)
class ReferencesStep:
    agent_id: str
    scope: str


@dataclass(frozen=True)
class ReviewStep:
    agent_id: str
    scope: str
    sections: tuple[str, ...] = ()


@dataclass(frozen=True)
class FactCheckStep:
    agent_id: str
    section: str


@dataclass(frozen=True)
class InsertFigureStep:
    agent_id: str
    section: str


@dataclass(frozen=True)
class CombineStep:
    agent_id: str
    sections: list[str]


@dataclass(frozen=True)
class EditorialStep:
    agent_id: str
    decision_type: str


class DocumentSim(SimContext):
    _DECISION_TOOLS = {
        "review_sections": 0.3,
        "fact_check": 0.2,
        "make_editorial_decision": 0.3,
    }
    _TOOL_DELAYS = {
        "research_topic": (1.0, 3.0),
        "write_section": (1.5, 4.0), "revise_section": (1.0, 3.0),
        "read_section": (0.1, 0.3),
        "update_references": (0.5, 1.5), "review_sections": (1.0, 2.0),
        "fact_check": (1.0, 2.5), "create_figure": (1.0, 2.0),
        "insert_figure": (0.5, 1.0), "combine_sections": (1.0, 2.0),
        "make_editorial_decision": (0.5, 1.5),
    }

    def __init__(self, *, write_steps, references_steps, review_steps,
                 combine_steps, metadata, fact_check_steps=(),
                 insert_figure_steps=(), editorial_steps=(),
                 section_inputs=None, reference_inputs=None, research_steps=()):
        super().__init__()
        self._write_steps = tuple(write_steps)
        self._references_steps = tuple(references_steps)
        self._review_steps = tuple(review_steps)
        self._combine_steps = tuple(combine_steps)
        self._fact_check_steps = tuple(fact_check_steps)
        self._insert_figure_steps = tuple(insert_figure_steps)
        self._editorial_steps = tuple(editorial_steps)
        self._research_steps = tuple(research_steps)
        self._section_inputs = dict(section_inputs or {})
        self._reference_inputs = dict(reference_inputs or {})
        for resource in metadata["resources"]:
            self.init_resource(resource)
        self.load_from_metadata(metadata)

    def tool_delay(self, tool_name, **kwargs):
        return self._TOOL_DELAYS.get(tool_name, (0.0, 0.0))

    def make_tools(self):
        names = ["write_section", "revise_section", "read_section", "update_references",
                 "review_sections", "combine_sections"]
        if self._research_steps:
            names.append("research_topic")
        if self._fact_check_steps:
            names.append("fact_check")
        if self._insert_figure_steps:
            names += ["create_figure", "insert_figure"]
        if self._editorial_steps:
            names.append("make_editorial_decision")
        return {name: getattr(self, name) for name in names}

    def _sections(self):
        return tuple(dict.fromkeys(step.section for step in self._write_steps))

    def _scope(self, value):
        if value in self._sections():
            return (value,)
        if isinstance(value, str) and value.startswith("CHAPTERS_"):
            sections = tuple(f"CHAPTER_{number}" for number in value[9:].split("_"))
            if len(set(sections)) == len(sections) and all(
                section in self._sections() for section in sections
            ):
                return sections
        return ()

    def _material(self):
        latest = {}
        for event in self.events:
            if event.success and event.tool in {
                "write_section", "revise_section", "update_references"
            }:
                category = "references" if event.tool == "update_references" else "sections"
                latest[(category, event.result["section"])] = event
        snapshot = {"sections": {}, "references": {}}
        for (category, section), event in latest.items():
            snapshot[category][section] = copy.deepcopy(event.result["artifact"])
        if self._insert_figure_steps:
            snapshot["figures"] = {}
            for event in self.events:
                if event.tool == "insert_figure" and event.success:
                    snapshot["figures"][event.result["section"]] = copy.deepcopy(
                        event.result["artifact"]
                    )
                    latest[("figures", event.result["section"])] = event
        return snapshot, latest

    def _snapshot(self, sections):
        material, _ = self._material()
        return {category: {section: value for section, value in values.items()
                           if section in sections}
                for category, values in material.items()}

    def _inputs(self, names):
        _, latest = self._material()
        return {name: copy.deepcopy(latest[("sections", name)].result["artifact"])
                for name in names if ("sections", name) in latest
                and latest[("sections", name)].completed_at is not None}

    def _research(self, section):
        inputs = {}
        for step in self._research_steps:
            if step.section != section:
                continue
            events = [event for event in self.events
                      if event.tool == "research_topic" and event.agent == step.agent_id
                      and event.result["topic"] == step.topic]
            if not events or not events[-1].success or events[-1].completed_at is None:
                return None
            inputs[step.topic] = copy.deepcopy(events[-1].result["artifact"])
        return inputs

    def _prepared(self, sections=None):
        sections = self._sections() if sections is None else sections
        _, latest = self._material()
        requirements = [("write_section", self._write_steps, "section"),
                        ("update_references", self._references_steps, "scope")]
        for tool, steps, field in requirements:
            for step in steps:
                section = getattr(step, field)
                if section in sections and not any(
                    event.tool == tool and event.agent == step.agent_id and event.success
                    and event.completed_at is not None and event.result["section"] == section
                    for event in self.events
                ):
                    return False
        for category in ("sections", "references"):
            for section in sections:
                event = latest.get((category, section))
                if event is None or event.completed_at is None:
                    return False
                if self._research_steps:
                    research = self._research(section)
                    if research is None or event.result["artifact"]["research_inputs"] != research:
                        return False
                dependencies = (self._section_inputs if category == "sections"
                                else self._reference_inputs).get(section, ())
                if dependencies and (set(self._inputs(dependencies)) != set(dependencies)
                                     or event.result["artifact"]["input_sections"]
                                     != self._inputs(dependencies)):
                    return False
        inserted = []
        for step in self._insert_figure_steps:
            if step.section not in sections:
                continue
            event = latest.get(("figures", step.section))
            if event is None or event.completed_at is None or event.agent != step.agent_id:
                return False
            inserted.append(event.result["artifact"]["id"])
        return len(inserted) == len(set(inserted))

    def _latest(self, tool, agent, key, value):
        matching = [event for event in self.events if event.tool == tool
                    and event.agent == agent and event.result[key] == value
                    and event.completed_at is not None]
        return max(matching, key=lambda event: event.completed_at) if matching else None

    def _facts(self, sections):
        receipts = {}
        for step in self._fact_check_steps:
            inputs = self._scope(step.section)
            if not set(inputs).intersection(sections):
                continue
            event = self._latest("fact_check", step.agent_id, "section", step.section)
            if event is None or not event.success or event.result["input_snapshot"] != self._snapshot(inputs):
                return None
            receipts[step.section] = event.result["check_id"]
        return receipts

    def _reviews(self, snapshot):
        receipts = {}
        for step in self._review_steps:
            event = self._latest("review_sections", step.agent_id, "scope", step.scope)
            inputs = step.sections or self._scope(step.scope)
            expected = {category: {section: value for section, value in values.items()
                                  if section in inputs}
                        for category, values in snapshot.items()}
            if event is None or not event.success or event.result["input_snapshot"] != expected:
                return None
            receipts[step.scope] = event.result["review_id"]
        return receipts

    def _editorial(self, snapshot):
        receipts = {}
        for step in self._editorial_steps:
            event = self._latest("make_editorial_decision", step.agent_id,
                                 "decision_type", step.decision_type)
            if event is None or not event.success or event.result["input_snapshot"] != snapshot:
                return None
            receipts[step.decision_type] = event.result["decision_id"]
        return receipts

    def _record(self, tool, agent, arguments, success, result, violations=None):
        self.log_event(agent, tool, copy.deepcopy(arguments), success,
                       copy.deepcopy(result), violations)
        return ToolResult(tool_name=tool, success=success, data=result,
                          message=f"{tool}: {result.get('status')}")

    def research_topic(self, agent_id, **kwargs):
        topic = kwargs.get("topic")
        valid = (agent_id in {step.agent_id for step in self._research_steps}
                 and isinstance(topic, str) and bool(topic.strip()))
        result = {"topic": topic, "agent": agent_id,
                  "status": "researched" if valid else "invalid_target",
                  "artifact": {"id": f"research:{len(self.events) + 1}",
                               "content": f"Research on {topic} by {agent_id}"}}
        return self._record("research_topic", agent_id, kwargs, valid, result)

    def write_section(self, agent_id, **kwargs):
        section = self._get_param(kwargs, "section_name", "section")
        content = kwargs.get("content_notes", f"{section} written by {agent_id}")
        research = self._research(section)
        valid = (agent_id in {step.agent_id for step in self._write_steps}
                 and section in self._sections() and isinstance(content, str)
                 and research is not None)
        artifact = {"id": f"document:{len(self.events) + 1}", "content": content}
        if self._research_steps:
            artifact["research_inputs"] = research
        if self._section_inputs.get(section):
            artifact["input_sections"] = self._inputs(self._section_inputs[section])
        result = {"section": section, "status": "written" if valid else "invalid_target",
                  "agent": agent_id, "artifact": artifact}
        return self._record("write_section", agent_id, kwargs, valid, result)

    def read_section(self, agent_id, **kwargs):
        section = self._get_param(kwargs, "section_name", "section")
        _, latest = self._material()
        entry = latest.get(("sections", section)) if isinstance(section, str) else None
        allowed = (agent_id in {step.agent_id for step in self._write_steps}
                   and section in self._sections())
        valid = allowed and entry is not None and entry.completed_at is not None
        result = {"section": section,
                  "status": "read" if valid else "missing_material" if allowed else "invalid_target",
                  "artifact": copy.deepcopy(entry.result["artifact"]) if valid else None}
        return self._record("read_section", agent_id, kwargs, valid, result)

    def revise_section(self, agent_id, **kwargs):
        section = self._get_param(kwargs, "section_name", "section")
        snapshot, latest = self._material()
        prior = latest.get(("sections", section))
        feedback = kwargs.get("feedback", "Cross-chapter consistency revision")
        research = self._research(section)
        valid = (agent_id in {step.agent_id for step in self._write_steps}
                 and prior is not None and prior.completed_at is not None
                 and isinstance(feedback, str) and research is not None)
        previous = snapshot["sections"].get(section, {}).get("content", section)
        artifact = {"id": f"document:{len(self.events) + 1}",
                    "content": f"{previous}\nRevised by {agent_id}: {feedback}",
                    "feedback": feedback}
        if self._research_steps:
            artifact["research_inputs"] = research
        if self._section_inputs.get(section):
            artifact["input_sections"] = self._inputs(self._section_inputs[section])
        result = {"section": section, "status": "revised" if valid else "missing_material",
                  "agent": agent_id, "artifact": artifact}
        return self._record("revise_section", agent_id, kwargs, valid, result)

    def update_references(self, agent_id, **kwargs):
        section = self._get_param(kwargs, "section_name", "scope")
        research = self._research(section)
        valid = (agent_id in {step.agent_id for step in self._write_steps}
                 and section in self._sections() and research is not None)
        artifact = {"id": f"references:{len(self.events) + 1}",
                    "content": f"References for {section} by {agent_id}"}
        if self._research_steps:
            artifact["research_inputs"] = research
        if self._reference_inputs.get(section):
            artifact["input_sections"] = self._inputs(self._reference_inputs[section])
        result = {"section": section, "scope": section, "agent": agent_id,
                  "status": "references updated" if valid else "invalid_target",
                  "artifact": artifact}
        return self._record("update_references", agent_id, kwargs, valid, result)

    def _decision(self, tool, agent, target, inputs, valid):
        snapshot = self._snapshot(inputs)
        key, value = target
        previous = self._latest(tool, agent, key, value)
        unchanged = (previous is not None and not previous.success
                     and previous.result["status"] == "needs_revision"
                     and previous.result["input_snapshot"] == snapshot)
        failed = self.should_fail(tool, agent)
        success = bool(valid and not failed and not unchanged)
        identifier = {"review_sections": "review_id", "fact_check": "check_id",
                      "make_editorial_decision": "decision_id"}[tool]
        prefix = {"review_sections": "review", "fact_check": "fact",
                  "make_editorial_decision": "editorial"}[tool]
        result = {key: value, "agent": agent,
                  "status": "reviewed" if success else (
                      "needs_revision" if valid else "missing_material"),
                  identifier: f"{prefix}:{len(self.events) + 1}",
                  "input_snapshot": snapshot}
        return success, result

    def review_sections(self, agent_id, **kwargs):
        scope = self._get_param(kwargs, "section_name", "scope")
        step = next((step for step in self._review_steps
                     if step.agent_id == agent_id and step.scope == scope), None)
        inputs = (step.sections or self._scope(scope)) if step else ()
        valid = step is not None and self._prepared(inputs) and self._facts(inputs) is not None
        success, result = self._decision("review_sections", agent_id, ("scope", scope), inputs, valid)
        if self._fact_check_steps:
            result["fact_receipts"] = self._facts(inputs) if valid else None
        return self._record("review_sections", agent_id, kwargs, success, result)

    def fact_check(self, agent_id, **kwargs):
        section = self._get_param(kwargs, "section", "section_name")
        inputs = self._scope(section)
        valid = (any(step.agent_id == agent_id and step.section == section
                     for step in self._fact_check_steps)
                 and self._prepared(inputs))
        success, result = self._decision("fact_check", agent_id, ("section", section),
                                         inputs, valid)
        return self._record("fact_check", agent_id, kwargs, success, result)

    def create_figure(self, agent_id, **kwargs):
        description = kwargs.get("description")
        valid = (any(step.agent_id == agent_id for step in self._insert_figure_steps)
                 and isinstance(description, str) and bool(description.strip()))
        artifact = {"id": f"figure:{len(self.events) + 1}", "content": description}
        result = {"figure_id": artifact["id"], "agent": agent_id,
                  "status": "created" if valid else "invalid_figure", "artifact": artifact}
        return self._record("create_figure", agent_id, kwargs, valid, result)

    def insert_figure(self, agent_id, **kwargs):
        figure_id = kwargs.get("figure_id")
        section = self._get_param(kwargs, "section", "section_name")
        figure = self._latest("create_figure", agent_id, "figure_id", figure_id)
        _, latest = self._material()
        written = latest.get(("sections", section))
        valid = (any(step.agent_id == agent_id and step.section == section
                     for step in self._insert_figure_steps)
                 and written is not None and written.completed_at is not None
                 and figure is not None and figure.success)
        result = {"figure_id": figure_id, "section": section, "agent": agent_id,
                  "status": "inserted" if valid else "missing_material",
                  "artifact": copy.deepcopy(figure.result["artifact"]) if valid else None}
        return self._record("insert_figure", agent_id, kwargs, valid, result)

    def make_editorial_decision(self, agent_id, **kwargs):
        step = next((step for step in self._editorial_steps
                     if step.agent_id == agent_id), None)
        kind = step.decision_type if step else "unknown"
        summary = kwargs.get("review_summary")
        snapshot, _ = self._material()
        valid = (step is not None and isinstance(summary, str) and bool(summary.strip())
                 and self._prepared() and self._facts(self._sections()) is not None
                 and self._reviews(snapshot) is not None)
        success, result = self._decision("make_editorial_decision", agent_id,
                                         ("decision_type", kind), self._sections(), valid)
        result["review_receipts"] = self._reviews(snapshot) if valid else None
        result["review_summary"] = summary
        return self._record("make_editorial_decision", agent_id, kwargs, success, result)

    def combine_sections(self, agent_id, **kwargs):
        sections = self._parse_list(kwargs.get("sections", []))
        snapshot, _ = self._material()
        reviews = self._reviews(snapshot)
        facts = self._facts(self._sections())
        editorial = self._editorial(snapshot)
        complete = any(step.agent_id == agent_id and sorted(step.sections) == sorted(sections)
                       for step in self._combine_steps)
        violations = []
        if (not complete or not self._prepared() or reviews is None
                or facts is None or editorial is None):
            violations.append(Violation(
                time.monotonic(), agent_id, "combine_sections", "unreviewed_document",
                "Combination lacks completed content-bound inputs and validation",
            ))
            self._violations.extend(violations)
        result = {"sections": sections, "status": "combined", "agent": agent_id,
                  "artifact_id": f"combined:{len(self.events) + 1}",
                  "input_snapshot": snapshot, "review_receipts": reviews}
        if self._fact_check_steps:
            result["fact_receipts"] = facts
        if self._editorial_steps:
            result["editorial_receipts"] = editorial
        return self._record("combine_sections", agent_id, kwargs, True, result, violations)

    def is_complete(self):
        combined = [event for event in self.events if event.tool == "combine_sections"]
        if not combined or combined[-1].completed_at is None or not self._prepared():
            return False
        event = combined[-1]
        snapshot, _ = self._material()
        return (event.success and not event.violations
                and event.result["input_snapshot"] == snapshot
                and event.result["review_receipts"] is not None
                and self._reviews(snapshot) is not None
                and self._facts(self._sections()) is not None
                and self._editorial(snapshot) is not None)

    @property
    def progress(self):
        snapshot, _ = self._material()
        return {"material": snapshot, "reviews": self._reviews(snapshot),
                "all_complete": self.is_complete()}
