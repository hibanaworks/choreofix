"""Two authors, actual document material and completed full-document reviews."""
import json
from pathlib import Path

from benchmark.tools.sim_document import (
    CombineStep, DocumentSim, ReferencesStep, ReviewStep, WriteSectionStep,
)


class TwoAuthorDocumentSim(DocumentSim):
    _DECISION_TOOLS = {"review_sections": 0.3}

    def __init__(self):
        super().__init__(
            write_steps=[WriteSectionStep("WRITER_A", "CHAPTER_1"),
                         WriteSectionStep("WRITER_B", "CHAPTER_2")],
            references_steps=[ReferencesStep("WRITER_A", "CHAPTER_1"),
                              ReferencesStep("WRITER_B", "CHAPTER_2")],
            review_steps=[ReviewStep("EDITOR", chapter, ("CHAPTER_1", "CHAPTER_2"))
                          for chapter in ("CHAPTER_1", "CHAPTER_2")],
            combine_steps=[CombineStep("EDITOR", ["CHAPTER_1", "CHAPTER_2"])],
            metadata=json.loads((Path(__file__).resolve().parents[2]
                                 / "descriptions/7E/metadata.json").read_text()),
        )
