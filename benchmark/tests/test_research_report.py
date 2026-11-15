"""3E scores actual report inputs and receipts for every orchestration method."""
import asyncio
import importlib
import itertools
import unittest

from benchmark.tools import load_tools
from benchmark.tools.sim_base import FailureSpec


def world():
    sim = importlib.import_module("benchmark.environments.3E.sim").TwoAuthorReportSim()
    sim._delay_multiplier = 0
    return sim, load_tools("3E", sim=sim)


class ResearchReportCausality(unittest.IsolatedAsyncioTestCase):
    async def test_free_research_does_not_supply_someone_elses_assigned_input(self):
        sim, tools = world()
        extra = await tools.call("research_topic", agent_id="RESEARCHER_A",
                                 topic="an independently chosen supplementary topic")
        self.assertTrue(extra.success)
        await tools.call("research_topic", agent_id="RESEARCHER_B", topic="subtopic_a")
        for tool in ["write_section", "update_references"]:
            self.assertFalse((await tools.call(tool, agent_id="RESEARCHER_A",
                                              section_name="section_a")).success)
        research = await tools.call("research_topic", agent_id="RESEARCHER_A",
                                    topic="subtopic_a")
        written = await tools.call("write_section", agent_id="RESEARCHER_A",
                                   section_name="section_a", content_notes="actual findings A")
        self.assertTrue(written.success)
        self.assertEqual(written.data["artifact"]["content"], "actual findings A")
        self.assertEqual(written.data["artifact"]["research_inputs"]["subtopic_a"],
                         research.data["artifact"])
        self.assertEqual(sim.violations, [])

    async def test_author_write_reference_and_review_orders_remain_free(self):
        # Each section needs only its own actual research and prepared material.
        # No preferred author, resource order or choreography enters this world.
        for authors, preparations, reviews in itertools.product(
            itertools.permutations([("RESEARCHER_A", "section_a", "subtopic_a"),
                                    ("RESEARCHER_B", "section_b", "subtopic_b")]),
            itertools.permutations(["write_section", "update_references"]),
            itertools.permutations(["section_a", "section_b"]),
        ):
            sim, tools = world()
            for actor, section, topic in authors:
                await tools.call("research_topic", agent_id=actor, topic=topic)
                for operation in preparations:
                    self.assertTrue((await tools.call(operation, agent_id=actor,
                                                       section_name=section)).success)
            for section in reviews:
                self.assertTrue((await tools.call("review_sections", agent_id="EDITOR",
                                                  section_name=section)).success)
            await tools.call("combine_sections", agent_id="EDITOR",
                             sections=list(reversed(reviews)))
            self.assertTrue(sim.is_complete(), (authors, preparations, reviews))
            self.assertEqual(sim.violations, [])

    async def test_future_material_cannot_retroactively_supply_a_review(self):
        sim, tools = world()
        for section in ["section_a", "section_b"]:
            self.assertFalse((await tools.call("review_sections", agent_id="EDITOR",
                                               section_name=section)).success)
        for actor, section, topic in [("RESEARCHER_A", "section_a", "subtopic_a"),
                                      ("RESEARCHER_B", "section_b", "subtopic_b")]:
            await tools.call("research_topic", agent_id=actor, topic=topic)
            await tools.call("write_section", agent_id=actor, section_name=section)
            await tools.call("update_references", agent_id=actor, section_name=section)
        await tools.call("combine_sections", agent_id="EDITOR",
                         sections=["section_a", "section_b"])
        self.assertFalse(sim.is_complete())
        self.assertEqual(len(sim.violations), 1)

    async def test_negative_review_needs_rework_and_approval_is_bound_to_its_section(self):
        sim, tools = world()
        sim._failure_specs = [FailureSpec("review_sections", "EDITOR", 0)]
        for actor, section, topic in [("RESEARCHER_A", "section_a", "subtopic_a"),
                                      ("RESEARCHER_B", "section_b", "subtopic_b")]:
            await tools.call("research_topic", agent_id=actor, topic=topic)
            await tools.call("write_section", agent_id=actor, section_name=section)
            await tools.call("update_references", agent_id=actor, section_name=section)
        for _ in range(2):
            self.assertFalse((await tools.call("review_sections", agent_id="EDITOR",
                                               section_name="section_a")).success)
        approved_b = await tools.call("review_sections", agent_id="EDITOR",
                                      section_name="section_b")
        self.assertTrue(approved_b.success)
        await tools.call("revise_section", agent_id="RESEARCHER_A", section_name="section_a",
                         feedback="account for the conflicting observation")
        approved_a = await tools.call("review_sections", agent_id="EDITOR",
                                      section_name="section_a")
        self.assertTrue(approved_a.success)
        combined = await tools.call("combine_sections", agent_id="EDITOR",
                                    sections=["section_b", "section_a"])
        self.assertEqual(combined.data["review_receipts"], {
            "section_a": approved_a.data["review_id"],
            "section_b": approved_b.data["review_id"],
        })
        self.assertTrue(sim.is_complete())
        self.assertEqual(sim.violations, [])

    async def test_later_research_invalidates_prior_draft_references_and_reviews(self):
        sim, tools = world()
        for actor, section, topic in [("RESEARCHER_A", "section_a", "subtopic_a"),
                                      ("RESEARCHER_B", "section_b", "subtopic_b")]:
            await tools.call("research_topic", agent_id=actor, topic=topic)
            await tools.call("write_section", agent_id=actor, section_name=section)
            await tools.call("update_references", agent_id=actor, section_name=section)
            await tools.call("review_sections", agent_id="EDITOR", section_name=section)
        await tools.call("research_topic", agent_id="RESEARCHER_A", topic="subtopic_a")
        self.assertFalse((await tools.call("review_sections", agent_id="EDITOR",
                                           section_name="section_a")).success)
        await tools.call("write_section", agent_id="RESEARCHER_A", section_name="section_a")
        self.assertFalse((await tools.call("review_sections", agent_id="EDITOR",
                                           section_name="section_a")).success)
        await tools.call("update_references", agent_id="RESEARCHER_A", section_name="section_a")
        approved = await tools.call("review_sections", agent_id="EDITOR", section_name="section_a")
        self.assertTrue(approved.success)
        await tools.call("combine_sections", agent_id="EDITOR", sections=["section_a", "section_b"])
        self.assertTrue(sim.is_complete())
        self.assertEqual(sim.violations, [])

    async def test_later_approval_cannot_erase_an_unsafe_publication(self):
        sim, tools = world()
        for actor, section, topic in [("RESEARCHER_A", "section_a", "subtopic_a"),
                                      ("RESEARCHER_B", "section_b", "subtopic_b")]:
            await tools.call("research_topic", agent_id=actor, topic=topic)
            await tools.call("write_section", agent_id=actor, section_name=section)
            await tools.call("update_references", agent_id=actor, section_name=section)
        await tools.call("combine_sections", agent_id="EDITOR", sections=["section_a", "section_b"])
        for section in ["section_b", "section_a"]:
            await tools.call("review_sections", agent_id="EDITOR", section_name=section)
        self.assertFalse(sim.is_complete())
        self.assertEqual(len(sim.violations), 1)
        await tools.call("combine_sections", agent_id="EDITOR", sections=["section_b", "section_a"])
        self.assertTrue(sim.is_complete())
        self.assertEqual(len(sim.violations), 1)

    async def test_pending_research_and_review_are_not_completed_evidence(self):
        sim, tools = world()
        sim._delay_multiplier = 1
        sim.tool_delay = lambda tool, **kwargs: (0.02, 0.02) if tool in [
            "research_topic", "review_sections"] else (0, 0)
        pending = asyncio.create_task(tools.call("research_topic", agent_id="RESEARCHER_A",
                                                 topic="subtopic_a"))
        await asyncio.sleep(0)
        self.assertFalse((await tools.call("write_section", agent_id="RESEARCHER_A",
                                           section_name="section_a")).success)
        await pending
        for actor, section, topic in [("RESEARCHER_A", "section_a", "subtopic_a"),
                                      ("RESEARCHER_B", "section_b", "subtopic_b")]:
            if actor == "RESEARCHER_B":
                await tools.call("research_topic", agent_id=actor, topic=topic)
            await tools.call("write_section", agent_id=actor, section_name=section)
            await tools.call("update_references", agent_id=actor, section_name=section)
        await tools.call("review_sections", agent_id="EDITOR", section_name="section_a")
        pending = asyncio.create_task(tools.call("review_sections", agent_id="EDITOR",
                                                 section_name="section_b"))
        await asyncio.sleep(0)
        await tools.call("combine_sections", agent_id="EDITOR", sections=["section_a", "section_b"])
        await pending
        self.assertFalse(sim.is_complete())
        self.assertEqual(len(sim.violations), 1)

    async def test_independent_document_and_database_effects_really_overlap(self):
        sim, tools = world()
        for actor, topic in [("RESEARCHER_A", "subtopic_a"), ("RESEARCHER_B", "subtopic_b")]:
            await tools.call("research_topic", agent_id=actor, topic=topic)
        sim._delay_multiplier = 1
        sim.tool_delay = lambda tool, **kwargs: (0.02, 0.02)
        written, references = await asyncio.gather(
            tools.call("write_section", agent_id="RESEARCHER_A", section_name="section_a"),
            tools.call("update_references", agent_id="RESEARCHER_B", section_name="section_b"),
        )
        self.assertTrue(written.success)
        self.assertTrue(references.success)
        first, second = sim.events[-2:]
        self.assertLess(max(first.timestamp, second.timestamp),
                        min(first.completed_at, second.completed_at))
        self.assertEqual(sim.violations, [])
