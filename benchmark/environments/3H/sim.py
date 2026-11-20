"""Common document world for the four independently authored survey sections."""

import json
from pathlib import Path

from benchmark.tools.sim_document import (
    CombineStep,
    DocumentSim,
    EditorialStep,
    InsertFigureStep,
    ReferencesStep,
    ResearchStep,
    ReviewStep,
    WriteSectionStep,
)

_METADATA = json.loads(
    (Path(__file__).resolve().parents[2]
     / "descriptions" / "3H" / "metadata.json").read_text()
)

_SECTIONS = tuple(f"section_{chapter}" for chapter in "abcd")
_AUTHORS = tuple(f"RESEARCHER_{chapter}" for chapter in "ABCD")

_RESEARCH_STEPS = [
    ResearchStep(agent, f"subtopic_{chapter}", section)
    for agent, chapter, section in zip(_AUTHORS, "abcd", _SECTIONS, strict=True)
]
_WRITE_STEPS = [
    WriteSectionStep(agent, section)
    for agent, section in zip(_AUTHORS, _SECTIONS, strict=True)
]
_REFERENCES_STEPS = [
    ReferencesStep(agent, section)
    for agent, section in zip(_AUTHORS, _SECTIONS, strict=True)
]
_INSERT_FIGURE_STEPS = [
    InsertFigureStep("DATA_ANALYST", section) for section in _SECTIONS
]
_REVIEW_STEPS = [ReviewStep("REVIEWER", section) for section in _SECTIONS]
_EDITORIAL_STEPS = [EditorialStep("EDITOR_IN_CHIEF", "finalize")]
_COMBINE_STEPS = [CombineStep("EDITOR_IN_CHIEF", list(_SECTIONS))]


class LargeSurveyPaperSim(DocumentSim):
    _REVIEW_TOOL = "peer_review"
    _DECISION_TOOLS = {"peer_review": 0.2}
    _TOOL_DELAYS = {**DocumentSim._TOOL_DELAYS, "peer_review": (1.5, 3.0)}

    def __init__(self) -> None:
        super().__init__(
            research_steps=_RESEARCH_STEPS,
            write_steps=_WRITE_STEPS,
            references_steps=_REFERENCES_STEPS,
            insert_figure_steps=_INSERT_FIGURE_STEPS,
            review_steps=_REVIEW_STEPS,
            editorial_steps=_EDITORIAL_STEPS,
            combine_steps=_COMBINE_STEPS,
            metadata=_METADATA,
        )

    def _can_edit(self, agent, section):
        return any(step.agent_id == agent and step.section == section
                   for step in self._write_steps)
