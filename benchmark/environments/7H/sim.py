"""Document co-authoring with scoped major and precise minor revisions."""
import copy
import json
from pathlib import Path

from benchmark.tools.sim_document import (
    CombineStep,
    DocumentSim,
    EditorialStep,
    FactCheckStep,
    InsertFigureStep,
    ReferencesStep,
    ReviewStep,
    WriteSectionStep,
)

# WRITER_A writes chapter_1 and chapter_2 (Part I) as separate sections
_WRITE_STEPS = [
    WriteSectionStep(agent_id="WRITER_A", section="CHAPTER_1"),
    WriteSectionStep(agent_id="WRITER_A", section="CHAPTER_2"),
    WriteSectionStep(agent_id="WRITER_B", section="CHAPTER_3"),
    WriteSectionStep(agent_id="WRITER_C", section="CHAPTER_4"),
]

# WRITER_A updates refs for chapter_1 and chapter_2 separately
_REFERENCES_STEPS = [
    ReferencesStep(agent_id="WRITER_A", scope="CHAPTER_1"),
    ReferencesStep(agent_id="WRITER_A", scope="CHAPTER_2"),
    ReferencesStep(agent_id="WRITER_B", scope="CHAPTER_3"),
    ReferencesStep(agent_id="WRITER_C", scope="CHAPTER_4"),
]

_FACT_CHECK_STEPS = [
    FactCheckStep(agent_id="FACT_CHECKER", section="CHAPTERS_1_2"),
    FactCheckStep(agent_id="FACT_CHECKER", section="CHAPTER_3"),
    FactCheckStep(agent_id="FACT_CHECKER", section="CHAPTER_4"),
]

# REVIEWER calls review_sections (not peer_review)
_REVIEW_STEPS = [
    ReviewStep(agent_id="REVIEWER", scope="CHAPTERS_1_2"),
    ReviewStep(agent_id="REVIEWER", scope="CHAPTER_3"),
    ReviewStep(agent_id="REVIEWER", scope="CHAPTER_4"),
]

_INSERT_FIGURE_STEPS = [
    InsertFigureStep(agent_id="FIGURE_CREATOR", section="CHAPTER_1"),
    InsertFigureStep(agent_id="FIGURE_CREATOR", section="CHAPTER_3"),
    InsertFigureStep(agent_id="FIGURE_CREATOR", section="CHAPTER_4"),
]

# EIC calls make_editorial_decision once and combine_sections once
_EDITORIAL_STEPS = [
    EditorialStep(agent_id="EDITOR_IN_CHIEF", decision_type="final"),
]

_COMBINE_STEPS = [
    CombineStep(agent_id="EDITOR_IN_CHIEF", sections=["CHAPTER_1", "CHAPTER_2", "CHAPTER_3", "CHAPTER_4"]),
]


class LargeCollaborativeDocumentSim(DocumentSim):
    """7H: Three writers, figure creator, fact checker, reviewer, and
    editor-in-chief co-authoring a four-chapter document in two parts.

    A minor review names exact edits to its captured material. Publication can
    use that review receipt after the owners apply the edits and facts are current.
    """

    _DECISION_TOOLS: dict = {
        "review_sections": 0.3,
        "fact_check": 0.2,
        "make_editorial_decision": 0.3,
    }

    def __init__(self) -> None:
        super().__init__(
            write_steps=_WRITE_STEPS,
            references_steps=_REFERENCES_STEPS,
            fact_check_steps=_FACT_CHECK_STEPS,
            insert_figure_steps=_INSERT_FIGURE_STEPS,
            review_steps=_REVIEW_STEPS,
            editorial_steps=_EDITORIAL_STEPS,
            combine_steps=_COMBINE_STEPS,
            metadata=json.loads((Path(__file__).resolve().parents[2]
                                 / "descriptions/7H/metadata.json").read_text()),
            section_inputs={"CHAPTER_1": ("CHAPTER_3", "CHAPTER_4"),
                            "CHAPTER_2": ("CHAPTER_3", "CHAPTER_4")},
            reference_inputs={"CHAPTER_3": ("CHAPTER_4",)},
        )

    def _can_edit(self, agent, section):
        return any(step.agent_id == agent and step.section == section
                   for step in self._write_steps)

    def _published_review(self, step):
        return max((event for event in self.events
                    if event.tool == "review_sections" and event.agent == step.agent_id
                    and event.result["scope"] == step.scope
                    and event.result["status"] in {"reviewed", "needs_revision", "minor_revision"}
                    and event.completed_at is not None),
                   key=lambda event: event.completed_at, default=None)

    def _review_receipt(self, step, expected):
        event = self._published_review(step)
        if event is None:
            return None
        reviewed = event.result["input_snapshot"]
        receipt = event.result["review_id"]
        if event.success:
            return receipt if reviewed == expected else None
        if event.result["status"] != "minor_revision":
            return None
        return receipt if self._minor_remaining(event, expected) == {} else None

    @staticmethod
    def _minor_remaining(review, current):
        """Read the edits still owed from the captured and current material."""
        before = review.result["input_snapshot"]
        amended = copy.deepcopy(before)
        pending = {}
        for section, replacement in review.result["minor_replacements"].items():
            prior = before["sections"][section]
            actual = current["sections"].get(section, {})
            if actual == prior:
                pending[section] = replacement
            else:
                amended["sections"][section] = {
                    **prior, "id": actual.get("id"), "content": replacement,
                    "previous_id": prior["id"], "minor_review_id": review.result["review_id"],
                }
        return pending if amended == current else None

    @staticmethod
    def _minor_replacements(snapshot, edits):
        if not isinstance(edits, list) or not edits:
            return None
        by_section = {}
        for edit in edits:
            if not isinstance(edit, dict) or set(edit) != {"section_name", "before", "after"}:
                return None
            section, before, after = edit["section_name"], edit["before"], edit["after"]
            if (not isinstance(section, str) or section not in snapshot["sections"]
                    or not isinstance(before, str) or not before or not isinstance(after, str)
                    or before == after):
                return None
            content = snapshot["sections"][section]["content"]
            at = content.find(before)
            if at < 0 or at != content.rfind(before):
                return None
            by_section.setdefault(section, []).append((at, at + len(before), after))
        replacements = {}
        for section, spans in by_section.items():
            original = snapshot["sections"][section]["content"]
            content = original
            end = len(content)
            for start, stop, replacement in sorted(spans, reverse=True):
                if stop > end:
                    return None
                content = content[:start] + replacement + content[stop:]
                end = start
            if content == original:
                return None
            replacements[section] = content
        return replacements

    def review_sections(self, agent_id, **kwargs):
        scope = self._get_param(kwargs, "section_name", "scope")
        step = next((step for step in self._review_steps
                     if step.agent_id == agent_id and step.scope == scope), None)
        sections = self._scope(scope) if step else ()
        snapshot = self._snapshot(sections)
        facts = self._facts(sections) if step and self._prepared(sections) else None
        decision = kwargs.get("decision", "accept")
        edits = kwargs.get("minor_edits")
        valid = (facts is not None and decision in ("accept", "minor", "major")
                 and ("feedback" not in kwargs or isinstance(kwargs["feedback"], str)))
        replacements = None
        if valid and decision == "minor":
            replacements = self._minor_replacements(snapshot, edits)
            valid = replacements is not None
        elif edits is not None:
            valid = False

        previous = self._published_review(step) if valid else None
        if (previous and not previous.success
                and (previous.result["input_snapshot"] == snapshot
                     or (previous.result["status"] == "minor_revision"
                         and self._minor_remaining(previous, snapshot)))):
            status = "awaiting_revision"
        elif not valid:
            status = "invalid_review"
        elif self.should_fail("review_sections", agent_id) or decision == "major":
            status = "needs_revision"
        else:
            status = "minor_revision" if decision == "minor" else "reviewed"
        result = {"scope": scope, "agent": agent_id, "status": status,
                  "input_snapshot": snapshot, "fact_receipts": facts}
        if status in {"reviewed", "needs_revision", "minor_revision"}:
            result["review_id"] = f"review:{len(self.events) + 1}"
            if "feedback" in kwargs:
                result["feedback"] = kwargs["feedback"]
        if status == "minor_revision":
            result["minor_replacements"] = replacements
        return self._record("review_sections", agent_id, kwargs, status == "reviewed", result)

    def revise_section(self, agent_id, **kwargs):
        if "review_id" not in kwargs:
            return super().revise_section(agent_id, **kwargs)
        section = self._get_param(kwargs, "section_name", "section")
        receipt = kwargs["review_id"]
        artifact = None
        if isinstance(receipt, str) and self._can_edit(agent_id, section):
            for step in self._review_steps:
                scope = self._scope(step.scope)
                if section not in scope:
                    continue
                review = self._published_review(step)
                if (review is None or review.result["status"] != "minor_revision"
                        or review.result["review_id"] != receipt or not self._prepared(scope)):
                    continue
                pending = self._minor_remaining(review, self._snapshot(scope))
                if pending is None or section not in pending:
                    continue
                prior = review.result["input_snapshot"]["sections"][section]
                artifact = {**prior, "id": f"document:{len(self.events) + 1}",
                            "content": pending[section],
                            "previous_id": prior["id"], "minor_review_id": receipt}
                break
        result = {"section": section, "agent": agent_id, "artifact": artifact,
                  "status": "revised" if artifact is not None else "invalid_review_receipt"}
        return self._record("revise_section", agent_id, kwargs, artifact is not None, result)
