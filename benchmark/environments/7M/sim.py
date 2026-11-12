"""Document co-authoring simulation config for task 7M."""
import json
from pathlib import Path

from benchmark.tools.sim_document import (
    CombineStep,
    DocumentSim,
    FactCheckStep,
    InsertFigureStep,
    ReferencesStep,
    ReviewStep,
    WriteSectionStep,
)

_WRITE_STEPS = [
    WriteSectionStep(agent_id="WRITER_A", section="CHAPTER_1"),
    WriteSectionStep(agent_id="WRITER_A", section="CHAPTER_2"),
    WriteSectionStep(agent_id="WRITER_B", section="CHAPTER_3"),
]

_REFERENCES_STEPS = [
    ReferencesStep(agent_id="WRITER_A", scope="CHAPTER_1"),
    ReferencesStep(agent_id="WRITER_A", scope="CHAPTER_2"),
    ReferencesStep(agent_id="WRITER_B", scope="CHAPTER_3"),
]

# FACT_CHECKER handles 2 requests (one per writer, not per chapter)
_FACT_CHECK_STEPS = [
    FactCheckStep(agent_id="FACT_CHECKER", section="CHAPTERS_1_2"),
    FactCheckStep(agent_id="FACT_CHECKER", section="CHAPTER_3"),
]

_INSERT_FIGURE_STEPS = [
    InsertFigureStep(agent_id="FIGURE_CREATOR", section="CHAPTER_1"),
    InsertFigureStep(agent_id="FIGURE_CREATOR", section="CHAPTER_3"),
]

_REVIEW_STEPS = [
    ReviewStep(agent_id="EDITOR", scope=chapter,
               sections=("CHAPTER_1", "CHAPTER_2", "CHAPTER_3"))
    for chapter in ("CHAPTER_1", "CHAPTER_2", "CHAPTER_3")
]

_COMBINE_STEPS = [
    CombineStep(agent_id="EDITOR", sections=["CHAPTER_1", "CHAPTER_2", "CHAPTER_3"]),
]


class MultiAuthorFigureDocumentSim(DocumentSim):
    """7M: Two writers, figure creator, fact checker, and editor
    co-authoring a three-chapter document with figures.

    Decision tools: review_sections (EDITOR may request revision → retry) and
    fact_check (FACT_CHECKER may flag issues → writer revises → re-check).
    make_editorial_decision is not used in this task.
    """

    _DECISION_TOOLS: dict = {
        "review_sections": 0.3,
        "fact_check": 0.2,
    }

    def __init__(self) -> None:
        super().__init__(
            write_steps=_WRITE_STEPS,
            references_steps=_REFERENCES_STEPS,
            fact_check_steps=_FACT_CHECK_STEPS,
            insert_figure_steps=_INSERT_FIGURE_STEPS,
            review_steps=_REVIEW_STEPS,
            combine_steps=_COMBINE_STEPS,
            metadata=json.loads((Path(__file__).resolve().parents[2]
                                 / "descriptions/7M/metadata.json").read_text()),
        )
