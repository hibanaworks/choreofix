"""3H uses author material and completed evidence through the common tool registry."""
import asyncio
import importlib
import itertools
import unittest

from benchmark.tools import load_tools


def world():
    sim = importlib.import_module("benchmark.environments.3H.sim").LargeSurveyPaperSim()
    sim._delay_multiplier = 0
    return sim, load_tools("3H", sim=sim)


async def prepare(registry, chapter, *, references_first=False):
    author = f"RESEARCHER_{chapter.upper()}"
    section = f"section_{chapter}"
    research = await registry.call("research_topic", agent_id=author, topic=f"subtopic_{chapter}")
    figure = await registry.call("create_figure", agent_id="DATA_ANALYST",
                                 description=f"図 {chapter}: sources and coverage")
    tools = ("update_references", "write_section") if references_first else (
        "write_section", "update_references")
    for tool in tools:
        result = await registry.call(tool, agent_id=author, section_name=section)
        assert result.success, result
    inserted = await registry.call("insert_figure", agent_id="DATA_ANALYST",
                                   section_name=section, figure_id=figure.data["figure_id"])
    assert inserted.success, inserted
    return research, figure


class SurveyPaperCausality(unittest.IsolatedAsyncioTestCase):
    async def test_each_complete_section_can_advance_without_the_other_three(self):
        for order in itertools.permutations("abcd"):
            sim, registry = world()
            receipts = {}
            for index, chapter in enumerate(order):
                research, figure = await prepare(registry, chapter, references_first=index % 2 == 0)
                section = f"section_{chapter}"
                review = await registry.call("peer_review", agent_id="REVIEWER", manuscript=section)
                self.assertTrue(review.success, (order, review))
                snapshot = review.data["input_snapshot"]
                self.assertEqual(set(snapshot["sections"]), {section})
                self.assertEqual(snapshot["figures"][section], figure.data["artifact"])
                self.assertEqual(snapshot["sections"][section]["research_inputs"],
                                 {f"subtopic_{chapter}": research.data["artifact"]})
                receipts[section] = review.data["review_id"]
            decision = await registry.call("make_editorial_decision", agent_id="EDITOR_IN_CHIEF",
                                           review_summary="Integrate the four independent reviews.")
            self.assertTrue(decision.success)
            self.assertEqual(decision.data["review_receipts"], receipts)
            combined = await registry.call("combine_sections", agent_id="EDITOR_IN_CHIEF",
                                           sections="section_d, section_a, section_c, section_b")
            self.assertEqual(combined.data["editorial_receipts"],
                             {"finalize": decision.data["decision_id"]})
            self.assertTrue(sim.is_complete())
            self.assertEqual(sim.violations, [])

    async def test_major_rejection_requires_changed_content_and_new_completed_review(self):
        sim, registry = world()
        sim.set_scenario_depth(1)
        early = await registry.call("peer_review", agent_id="REVIEWER", manuscript="section_a")
        self.assertFalse(early.success)
        await prepare(registry, "a")
        rejected = await registry.call("peer_review", agent_id="REVIEWER", manuscript="section_a")
        self.assertFalse(rejected.success)
        self.assertEqual(rejected.data["status"], "needs_revision")
        # Updating only references must not launder the rejected draft.
        await registry.call("update_references", agent_id="RESEARCHER_A", section_name="section_a")
        still_rejected = await registry.call("peer_review", agent_id="REVIEWER", manuscript="section_a")
        self.assertFalse(still_rejected.success)
        foreign_revision = await registry.call("revise_section", agent_id="RESEARCHER_B",
                                              section_name="section_a", feedback="wrong writer")
        self.assertFalse(foreign_revision.success)
        revised = await registry.call("revise_section", agent_id="RESEARCHER_A",
                                      section_name="section_a", feedback="Compare both definitions.")
        self.assertTrue(revised.success)
        reviewed = await registry.call("peer_review", agent_id="REVIEWER", manuscript="section_a")
        self.assertTrue(reviewed.success)
        self.assertEqual(reviewed.data["input_snapshot"]["sections"]["section_a"],
                         revised.data["artifact"])

    async def test_figure_requires_current_research_and_a_real_target_draft(self):
        sim, registry = world()
        _, old_figure = await prepare(registry, "a")
        await registry.call("research_topic", agent_id="RESEARCHER_A", topic="subtopic_a")
        old_insert = await registry.call("insert_figure", agent_id="DATA_ANALYST",
                                         section_name="section_a", figure_id=old_figure.data["figure_id"])
        self.assertFalse(old_insert.success)
        for tool in ("update_references", "write_section"):
            self.assertTrue((await registry.call(tool, agent_id="RESEARCHER_A",
                                                 section_name="section_a")).success)
        stale = await registry.call("peer_review", agent_id="REVIEWER", manuscript="section_a")
        self.assertFalse(stale.success)
        new_figure = await registry.call("create_figure", agent_id="DATA_ANALYST",
                                         description="Based on the updated research")
        missing = await registry.call("insert_figure", agent_id="DATA_ANALYST",
                                      section_name="section_b", figure_id=new_figure.data["figure_id"])
        self.assertFalse(missing.success)
        self.assertTrue((await registry.call("insert_figure", agent_id="DATA_ANALYST",
                                             section_name="section_a",
                                             figure_id=new_figure.data["figure_id"])).success)
        self.assertTrue((await registry.call("peer_review", agent_id="REVIEWER",
                                             manuscript="section_a")).success)
        self.assertEqual(sim.violations, [])

    async def test_a_later_editorial_decision_cannot_validate_an_earlier_publication(self):
        sim, registry = world()
        sections = "section_a, section_b, section_c, section_d"
        for chapter in "abcd":
            await prepare(registry, chapter)
            self.assertTrue((await registry.call("peer_review", agent_id="REVIEWER",
                                                 manuscript=f"section_{chapter}")).success)
        await registry.call("combine_sections", agent_id="EDITOR_IN_CHIEF", sections=sections)
        self.assertFalse(sim.is_complete())
        self.assertEqual(len(sim.violations), 1)
        decision = await registry.call("make_editorial_decision", agent_id="EDITOR_IN_CHIEF",
                                       review_summary="All current drafts approved.")
        self.assertTrue(decision.success)
        self.assertFalse(sim.is_complete())
        await registry.call("combine_sections", agent_id="EDITOR_IN_CHIEF", sections=sections)
        self.assertTrue(sim.is_complete())
        self.assertEqual(len(sim.violations), 1)

    async def test_figure_insertion_conflicts_only_with_a_writer_on_the_same_section(self):
        for same_section in (False, True):
            sim, registry = world()
            _, figure = await prepare(registry, "a")
            await prepare(registry, "b")
            sim._delay_multiplier = 1
            sim.tool_delay = lambda tool, **kwargs: (0.02, 0.02)
            inserting = asyncio.create_task(registry.call(
                "insert_figure", agent_id="DATA_ANALYST", section_name="section_a",
                figure_id=figure.data["figure_id"],
            ))
            await asyncio.sleep(0)
            self.assertFalse(inserting.done())
            chapter = "a" if same_section else "b"
            revised = await registry.call("revise_section", agent_id=f"RESEARCHER_{chapter.upper()}",
                                          section_name=f"section_{chapter}", feedback="Revise draft.")
            self.assertTrue(revised.success)
            self.assertTrue((await inserting).success)
            self.assertEqual(len(sim.violations), int(same_section))
