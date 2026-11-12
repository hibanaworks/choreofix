"""Shared document data causality, independent of either agent runtime."""
import asyncio
import importlib
import itertools
import unittest

from benchmark.tools import load_tools
from benchmark.tools.sim_base import FailureSpec


def world(task):
    name = {"7M": "MultiAuthorFigureDocumentSim", "7H": "LargeCollaborativeDocumentSim"}[task]
    sim = getattr(importlib.import_module(f"benchmark.environments.{task}.sim"), name)()
    sim._delay_multiplier = 0
    return sim, load_tools(task, sim=sim)


class FigureDocumentCausality(unittest.IsolatedAsyncioTestCase):
    async def test_chapter_reads_return_actual_completed_material_without_aliases(self):
        sim, registry = world("7M")
        early = await registry.call("read_section", agent_id="WRITER_A",
                                    section_name="CHAPTER_3")
        self.assertFalse(early.success)
        self.assertIsNone(early.data["artifact"])
        written = await registry.call("write_section", agent_id="WRITER_B",
                                      section_name="CHAPTER_3", content_notes="実際の独立した章")
        read = await registry.call("read_section", agent_id="WRITER_A",
                                   section_name="CHAPTER_3")
        self.assertTrue(read.success)
        self.assertEqual(read.data["artifact"], written.data["artifact"])
        read.data["artifact"]["content"] = "forged outside the world"
        again = await registry.call("read_section", agent_id="WRITER_A",
                                    section_name="CHAPTER_3")
        self.assertEqual(again.data["artifact"]["content"], "実際の独立した章")
        revised = await registry.call("revise_section", agent_id="WRITER_B",
                                      section_name="CHAPTER_3", feedback="A real change")
        current = await registry.call("read_section", agent_id="WRITER_A",
                                      section_name="CHAPTER_3")
        self.assertEqual(current.data["artifact"], revised.data["artifact"])
        self.assertNotEqual(current.data["artifact"]["id"], again.data["artifact"]["id"])
        self.assertEqual(again.data["artifact"]["content"], "実際の独立した章")
        self.assertEqual(sim.violations, [])

    async def test_a_pending_chapter_write_is_not_readable_before_its_result(self):
        sim, registry = world("7M")
        sim._delay_multiplier = 1
        sim.tool_delay = lambda tool, **kwargs: (0.02, 0.02) if tool == "write_section" else (0, 0)
        pending = asyncio.create_task(registry.call(
            "write_section", agent_id="WRITER_B", section_name="CHAPTER_3",
            content_notes="Not completed yet"))
        await asyncio.sleep(0)
        early = await registry.call("read_section", agent_id="WRITER_A",
                                    section_name="CHAPTER_3")
        self.assertFalse(early.success)
        self.assertIsNone(early.data["artifact"])
        self.assertFalse(pending.done())
        written = await pending
        completed = await registry.call("read_section", agent_id="WRITER_A",
                                        section_name="CHAPTER_3")
        self.assertEqual(completed.data["artifact"], written.data["artifact"])

    async def test_cross_chapter_reads_do_not_require_a_fixed_writer_order(self):
        for order in [("CHAPTER_1", "CHAPTER_3"), ("CHAPTER_3", "CHAPTER_1")]:
            sim, registry = world("7M")
            for chapter in order:
                author = "WRITER_A" if chapter == "CHAPTER_1" else "WRITER_B"
                reader = "WRITER_B" if author == "WRITER_A" else "WRITER_A"
                written = await registry.call("write_section", agent_id=author,
                                              section_name=chapter, content_notes=chapter)
                read = await registry.call("read_section", agent_id=reader,
                                           section_name=chapter)
                self.assertTrue(read.success)
                self.assertEqual(read.data["artifact"], written.data["artifact"])
            invalid = await registry.call("read_section", agent_id="WRITER_A",
                                          section_name="OUTSIDE")
            self.assertFalse(invalid.success)
            self.assertIsNone(invalid.data["artifact"])
            unauthorized = await registry.call("read_section", agent_id="FACT_CHECKER",
                                               section_name=order[0])
            self.assertFalse(unauthorized.success)
            self.assertIsNone(unauthorized.data["artifact"])
            self.assertEqual(sim.violations, [])

    async def test_all_720_preparation_orders_keep_writes_and_references_independent(self):
        operations = [(tool, agent, chapter)
                      for tool in ("write_section", "update_references")
                      for agent, chapter in (("WRITER_A", "CHAPTER_1"),
                                             ("WRITER_A", "CHAPTER_2"),
                                             ("WRITER_B", "CHAPTER_3"))]
        review_orders = list(itertools.permutations(("CHAPTER_1", "CHAPTER_2", "CHAPTER_3")))
        completed = 0
        for index, order in enumerate(itertools.permutations(range(6))):
            sim, registry = world("7M")
            for position in order:
                tool, agent, chapter = operations[position]
                result = await registry.call(tool, agent_id=agent, section_name=chapter)
                self.assertTrue(result.success, (order, tool, chapter))
            figures = ("CHAPTER_1", "CHAPTER_3")
            for chapter in figures if index % 2 == 0 else reversed(figures):
                figure = await registry.call("create_figure", agent_id="FIGURE_CREATOR",
                                             description=f"A freely described plot for {chapter}")
                self.assertTrue((await registry.call(
                    "insert_figure", agent_id="FIGURE_CREATOR",
                    figure_id=figure.data["figure_id"], section_name=chapter,
                )).success)
            checks = ("CHAPTERS_1_2", "CHAPTER_3")
            for scope in checks if index % 2 == 0 else reversed(checks):
                self.assertTrue((await registry.call(
                    "fact_check", agent_id="FACT_CHECKER", section_name=scope,
                )).success)
            for scope in review_orders[index % len(review_orders)]:
                self.assertTrue((await registry.call(
                    "review_sections", agent_id="EDITOR", section_name=scope,
                )).success)
            await registry.call("combine_sections", agent_id="EDITOR",
                                sections="CHAPTER_3, CHAPTER_1, CHAPTER_2")
            self.assertTrue(sim.is_complete(), order)
            self.assertEqual(sim.violations, [], order)
            completed += 1
        self.assertEqual(completed, 720)

    async def test_a_figure_requires_its_actual_created_result_and_existing_chapter(self):
        sim, registry = world("7M")
        self.assertFalse((await registry.call(
            "insert_figure", agent_id="FIGURE_CREATOR", figure_id="invented",
            section_name="CHAPTER_1",
        )).success)
        figure = await registry.call("create_figure", agent_id="FIGURE_CREATOR",
                                     description="Arbitrary, non-checklist figure title")
        identifier = figure.data["figure_id"]
        self.assertFalse((await registry.call(
            "insert_figure", agent_id="FIGURE_CREATOR", figure_id=identifier,
            section_name="CHAPTER_1",
        )).success)
        await registry.call("write_section", agent_id="WRITER_A", section_name="CHAPTER_1")
        self.assertFalse((await registry.call(
            "insert_figure", agent_id="FIGURE_CREATOR", figure_id=identifier,
            section_name="CHAPTER_2",
        )).success, "a different target cannot satisfy a required insertion")
        figure.data["artifact"]["content"] = "forged outside the world"
        inserted = await registry.call(
            "insert_figure", agent_id="FIGURE_CREATOR", figure_id=identifier,
            section_name="CHAPTER_1",
        )
        self.assertTrue(inserted.success)
        self.assertEqual(inserted.data["artifact"]["content"],
                         "Arbitrary, non-checklist figure title")
        self.assertFalse(sim.is_complete())

    async def test_figure_creation_can_overlap_writing_with_independent_resources(self):
        sim, registry = world("7M")
        await registry.call("write_section", agent_id="WRITER_A", section_name="CHAPTER_1")
        sim._delay_multiplier = 1
        sim.tool_delay = lambda tool, **kwargs: (0.02, 0.02)
        figure = asyncio.create_task(registry.call(
            "create_figure", agent_id="FIGURE_CREATOR", description="Independent plot"))
        await asyncio.sleep(0)
        writing = asyncio.create_task(registry.call(
            "write_section", agent_id="WRITER_B", section_name="CHAPTER_3"))
        await asyncio.sleep(0)
        self.assertFalse(figure.done())
        self.assertFalse(writing.done())
        actual, _ = await asyncio.gather(figure, writing)
        self.assertTrue((await registry.call(
            "insert_figure", agent_id="FIGURE_CREATOR", figure_id=actual.data["figure_id"],
            section_name="CHAPTER_1",
        )).success)
        self.assertEqual(sim.violations, [], "DOCUMENT and FIGURE_STORE are independent")

    async def test_a_pending_figure_is_not_a_completed_insertion_input(self):
        sim, registry = world("7M")
        await registry.call("write_section", agent_id="WRITER_A", section_name="CHAPTER_1")
        sim._delay_multiplier = 1
        sim.tool_delay = lambda tool, **kwargs: (0.02, 0.02) if tool == "create_figure" else (0, 0)
        pending = asyncio.create_task(registry.call(
            "create_figure", agent_id="FIGURE_CREATOR", description="Still rendering"))
        await asyncio.sleep(0)
        identifier = sim.events[-1].result["figure_id"]
        self.assertFalse((await registry.call(
            "insert_figure", agent_id="FIGURE_CREATOR", figure_id=identifier,
            section_name="CHAPTER_1",
        )).success)
        self.assertFalse(pending.done())
        actual = await pending
        self.assertTrue((await registry.call(
            "insert_figure", agent_id="FIGURE_CREATOR", figure_id=actual.data["figure_id"],
            section_name="CHAPTER_1",
        )).success)
        self.assertTrue(any(v.violation_type == "concurrent_resource_use" for v in sim.violations))

    async def test_future_material_or_checks_cannot_retroactively_approve_a_review(self):
        sim, registry = world("7M")
        premature = await registry.call("fact_check", agent_id="FACT_CHECKER",
                                        section_name="CHAPTERS_1_2")
        self.assertFalse(premature.success)
        for agent, chapter in (("WRITER_A", "CHAPTER_1"), ("WRITER_A", "CHAPTER_2"),
                               ("WRITER_B", "CHAPTER_3")):
            await registry.call("write_section", agent_id=agent, section_name=chapter)
            await registry.call("update_references", agent_id=agent, section_name=chapter)
        for chapter in ("CHAPTER_1", "CHAPTER_3"):
            figure = await registry.call("create_figure", agent_id="FIGURE_CREATOR",
                                         description=f"Actual {chapter} plot")
            await registry.call("insert_figure", agent_id="FIGURE_CREATOR",
                                figure_id=figure.data["figure_id"], section_name=chapter)
        early_review = await registry.call("review_sections", agent_id="EDITOR",
                                           section_name="CHAPTER_1")
        self.assertFalse(early_review.success)
        for scope in ("CHAPTERS_1_2", "CHAPTER_3"):
            await registry.call("fact_check", agent_id="FACT_CHECKER", section_name=scope)
        for scope in ("CHAPTER_2", "CHAPTER_3"):
            await registry.call("review_sections", agent_id="EDITOR", section_name=scope)
        await registry.call("combine_sections", agent_id="EDITOR",
                            sections="CHAPTER_1,CHAPTER_2,CHAPTER_3")
        self.assertFalse(sim.is_complete(), "the missing completed review is still missing")
        self.assertEqual(len(sim.violations), 1)
        await registry.call("review_sections", agent_id="EDITOR", section_name="CHAPTER_1")
        self.assertFalse(sim.is_complete(), "a late review cannot rewrite the publication")
        await registry.call("combine_sections", agent_id="EDITOR",
                            sections="CHAPTER_3,CHAPTER_2,CHAPTER_1")
        self.assertTrue(sim.is_complete())
        self.assertEqual(len(sim.violations), 1, "recovery preserves the unsafe effect")

    async def test_changed_figure_and_negative_fact_results_require_fresh_evidence(self):
        sim, registry = world("7M")
        for agent, chapter in (("WRITER_A", "CHAPTER_1"), ("WRITER_A", "CHAPTER_2"),
                               ("WRITER_B", "CHAPTER_3")):
            await registry.call("write_section", agent_id=agent, section_name=chapter)
            await registry.call("update_references", agent_id=agent, section_name=chapter)
        for chapter in ("CHAPTER_1", "CHAPTER_3"):
            figure = await registry.call("create_figure", agent_id="FIGURE_CREATOR",
                                         description=f"Original {chapter} figure")
            await registry.call("insert_figure", agent_id="FIGURE_CREATOR",
                                figure_id=figure.data["figure_id"], section_name=chapter)
        sim._failure_specs = [FailureSpec("fact_check", "FACT_CHECKER", 0)]
        for _ in range(2):
            self.assertFalse((await registry.call(
                "fact_check", agent_id="FACT_CHECKER", section_name="CHAPTERS_1_2",
            )).success)
        await registry.call("revise_section", agent_id="WRITER_B", section_name="CHAPTER_1",
                            feedback="Correct the factual inconsistency")
        for scope in ("CHAPTERS_1_2", "CHAPTER_3"):
            self.assertTrue((await registry.call(
                "fact_check", agent_id="FACT_CHECKER", section_name=scope,
            )).success)
        for chapter in ("CHAPTER_1", "CHAPTER_2", "CHAPTER_3"):
            await registry.call("review_sections", agent_id="EDITOR", section_name=chapter)
        await registry.call("combine_sections", agent_id="EDITOR",
                            sections="CHAPTER_1,CHAPTER_2,CHAPTER_3")
        self.assertTrue(sim.is_complete())
        replacement = await registry.call("create_figure", agent_id="FIGURE_CREATOR",
                                          description="Updated actual measurements")
        self.assertTrue(sim.is_complete(), "an unused figure does not change the document")
        await registry.call("insert_figure", agent_id="FIGURE_CREATOR",
                            figure_id=replacement.data["figure_id"], section_name="CHAPTER_3")
        self.assertFalse(sim.is_complete())
        self.assertFalse((await registry.call("review_sections", agent_id="EDITOR",
                                              section_name="CHAPTER_3")).success)
        await registry.call("fact_check", agent_id="FACT_CHECKER", section_name="CHAPTER_3")
        for chapter in ("CHAPTER_1", "CHAPTER_2", "CHAPTER_3"):
            self.assertTrue((await registry.call(
                "review_sections", agent_id="EDITOR", section_name=chapter,
            )).success)
        await registry.call("combine_sections", agent_id="EDITOR",
                            sections="CHAPTER_1,CHAPTER_2,CHAPTER_3")
        self.assertTrue(sim.is_complete())
        self.assertEqual(sim.violations, [])

    async def test_part_reviews_can_proceed_while_independent_part_i_is_unwritten(self):
        sim, registry = world("7H")
        await registry.call("write_section", agent_id="WRITER_B", section_name="CHAPTER_3")
        await registry.call("update_references", agent_id="WRITER_B", section_name="CHAPTER_3")
        await registry.call("write_section", agent_id="WRITER_C", section_name="CHAPTER_4")
        figure = await registry.call("create_figure", agent_id="FIGURE_CREATOR",
                                     description="Chapter three actual results")
        await registry.call("insert_figure", agent_id="FIGURE_CREATOR",
                            figure_id=figure.data["figure_id"], section_name="CHAPTER_3")
        self.assertFalse((await registry.call("fact_check", agent_id="FACT_CHECKER",
                                              section_name="CHAPTER_3")).success,
                         "chapter three's final references predated evaluation data")
        await registry.call("update_references", agent_id="WRITER_B", section_name="CHAPTER_3")
        self.assertTrue((await registry.call("fact_check", agent_id="FACT_CHECKER",
                                             section_name="CHAPTER_3")).success)
        self.assertTrue((await registry.call("review_sections", agent_id="REVIEWER",
                                             section_name="CHAPTER_3")).success)
        self.assertFalse(sim.is_complete())
        self.assertNotIn("CHAPTER_1", sim.progress["material"]["sections"])

    async def test_part_i_captures_completed_contributions_and_editorial_is_content_bound(self):
        sim, registry = world("7H")
        assignments = (("WRITER_A", "CHAPTER_1"), ("WRITER_A", "CHAPTER_2"),
                       ("WRITER_B", "CHAPTER_3"), ("WRITER_C", "CHAPTER_4"))
        for agent, chapter in assignments:
            await registry.call("write_section", agent_id=agent, section_name=chapter)
            await registry.call("update_references", agent_id=agent, section_name=chapter)
        for chapter in ("CHAPTER_1", "CHAPTER_3", "CHAPTER_4"):
            figure = await registry.call("create_figure", agent_id="FIGURE_CREATOR",
                                         description=f"Actual figure for {chapter}")
            await registry.call("insert_figure", agent_id="FIGURE_CREATOR",
                                figure_id=figure.data["figure_id"], section_name=chapter)
        self.assertFalse((await registry.call("fact_check", agent_id="FACT_CHECKER",
                                              section_name="CHAPTERS_1_2")).success)
        for chapter in ("CHAPTER_2", "CHAPTER_1"):
            await registry.call("revise_section", agent_id="WRITER_A", section_name=chapter,
                                feedback="Frame the actual published contributions")
        await registry.call("update_references", agent_id="WRITER_B", section_name="CHAPTER_3")
        for scope in ("CHAPTER_4", "CHAPTERS_1_2", "CHAPTER_3"):
            self.assertTrue((await registry.call("fact_check", agent_id="FACT_CHECKER",
                                                 section_name=scope)).success)
            self.assertTrue((await registry.call("review_sections", agent_id="REVIEWER",
                                                 section_name=scope)).success)
        sim._failure_specs = [FailureSpec("make_editorial_decision", "EDITOR_IN_CHIEF", 0)]
        for _ in range(2):
            self.assertFalse((await registry.call(
                "make_editorial_decision", agent_id="EDITOR_IN_CHIEF",
                review_summary="Free prose summarizing the actual reviewer reports",
            )).success)
        await registry.call("revise_section", agent_id="WRITER_B", section_name="CHAPTER_2",
                            feedback="Resolve the editor's requested terminology change")
        await registry.call("fact_check", agent_id="FACT_CHECKER", section_name="CHAPTERS_1_2")
        await registry.call("review_sections", agent_id="REVIEWER", section_name="CHAPTERS_1_2")
        sim._delay_multiplier = 1
        sim.tool_delay = lambda tool, **kwargs: (0.02, 0.02) if tool == "make_editorial_decision" else (0, 0)
        pending = asyncio.create_task(registry.call(
            "make_editorial_decision", agent_id="EDITOR_IN_CHIEF",
            review_summary="The requested terminology is corrected in the actual revised text",
        ))
        await asyncio.sleep(0)
        self.assertFalse(pending.done())
        await registry.call("combine_sections", agent_id="EDITOR_IN_CHIEF",
                            sections="CHAPTER_1,CHAPTER_2,CHAPTER_3,CHAPTER_4")
        self.assertFalse(sim.is_complete())
        self.assertTrue((await pending).success)
        self.assertFalse(sim.is_complete(), "a late decision cannot authorize a past publication")
        await registry.call("combine_sections", agent_id="EDITOR_IN_CHIEF",
                            sections="CHAPTER_4,CHAPTER_3,CHAPTER_2,CHAPTER_1")
        self.assertTrue(sim.is_complete())
        self.assertEqual(len(sim.violations), 1)
