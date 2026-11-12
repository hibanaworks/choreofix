"""Actual research, factual evidence and publication, independent of an agent runtime."""
import asyncio
import importlib
import itertools
import unittest

from benchmark.tools import load_tools
from benchmark.tools.sim_base import FailureSpec


def world():
    sim = importlib.import_module("benchmark.environments.3M.sim").MultiAuthorPaperSim()
    sim._delay_multiplier = 0
    return sim, load_tools("3M", sim=sim)


class ResearchPaperCausality(unittest.IsolatedAsyncioTestCase):
    async def test_extra_research_is_free_but_does_not_replace_assigned_evidence(self):
        _, tools = world()
        extra = await tools.call("research_topic", agent_id="RESEARCHER_A",
                                 topic="A freely chosen additional topic")
        self.assertTrue(extra.success)
        await tools.call("research_topic", agent_id="RESEARCHER_B", topic="subtopic_a")
        self.assertFalse((await tools.call("write_section", agent_id="RESEARCHER_A",
                                          section_name="section_a")).success)
        actual = await tools.call("research_topic", agent_id="RESEARCHER_A", topic="subtopic_a")
        written = await tools.call("write_section", agent_id="RESEARCHER_A", section_name="section_a")
        self.assertTrue(written.success)
        self.assertEqual(written.data["artifact"]["research_inputs"],
                         {"subtopic_a": actual.data["artifact"]})

    async def test_later_evidence_cannot_authorize_an_earlier_publication(self):
        sim, tools = world()
        await tools.call("combine_sections", agent_id="EDITOR",
                         sections=["section_a", "section_b", "section_c"])
        for suffix in "abc":
            author = f"RESEARCHER_{suffix.upper()}"
            section = f"section_{suffix}"
            await tools.call("research_topic", agent_id=author, topic=f"subtopic_{suffix}")
            await tools.call("write_section", agent_id=author, section_name=section)
            await tools.call("update_references", agent_id=author, section_name=section)
            await tools.call("fact_check", agent_id="FACT_CHECKER", section_name=section)
            await tools.call("review_sections", agent_id="EDITOR", scope=section)
        self.assertFalse(sim.is_complete())
        self.assertTrue(sim.violations)
        self.assertTrue(any(e.tool == "combine_sections" for e in sim.events))

    async def test_each_completed_section_can_flow_before_other_authors_start(self):
        for order in itertools.permutations("abc"):
            for reference_first in itertools.product((False, True), repeat=3):
                sim, tools = world()
                for index, suffix in enumerate(order):
                    author = f"RESEARCHER_{suffix.upper()}"
                    section = f"section_{suffix}"
                    research = await tools.call("research_topic", agent_id=author,
                                                topic=f"subtopic_{suffix}")
                    names = ("update_references", "write_section") if reference_first[index] else (
                        "write_section", "update_references")
                    for name in names:
                        result = await tools.call(name, agent_id=author, section_name=section,
                                                  content_notes=f"Actual {section} findings")
                        self.assertTrue(result.success)
                        self.assertEqual(result.data["artifact"]["research_inputs"],
                                         {f"subtopic_{suffix}": research.data["artifact"]})
                    fact = await tools.call("fact_check", agent_id="FACT_CHECKER",
                                            section_name=section)
                    review = await tools.call("review_sections", agent_id="EDITOR", scope=section)
                    self.assertTrue(fact.success)
                    self.assertTrue(review.success)
                    self.assertEqual(review.data["fact_receipts"], {section: fact.data["check_id"]})
                combined = await tools.call("combine_sections", agent_id="EDITOR",
                                            sections=[f"section_{s}" for s in reversed(order)])
                self.assertEqual(len(combined.data["fact_receipts"]), 3)
                self.assertEqual(len(combined.data["review_receipts"]), 3)
                self.assertTrue(sim.is_complete())
                self.assertEqual(sim.violations, [])

    async def test_fact_check_and_editor_require_completed_current_inputs(self):
        sim, tools = world()
        for name, actor, args in [
            ("write_section", "RESEARCHER_A", {"section_name": "section_a"}),
            ("update_references", "RESEARCHER_A", {"section_name": "section_a"}),
            ("fact_check", "FACT_CHECKER", {"section_name": "section_a"}),
            ("review_sections", "EDITOR", {"scope": "section_a"}),
        ]:
            self.assertFalse((await tools.call(name, agent_id=actor, **args)).success)
        await tools.call("research_topic", agent_id="RESEARCHER_A", topic="subtopic_a")
        await tools.call("write_section", agent_id="RESEARCHER_A", section_name="section_a")
        self.assertFalse((await tools.call("fact_check", agent_id="FACT_CHECKER",
                                          section_name="section_a")).success)
        await tools.call("update_references", agent_id="RESEARCHER_A", section_name="section_a")
        self.assertFalse((await tools.call("review_sections", agent_id="EDITOR",
                                          scope="section_a")).success)
        await tools.call("fact_check", agent_id="FACT_CHECKER", section_name="section_a")
        self.assertTrue((await tools.call("review_sections", agent_id="EDITOR",
                                         scope="section_a")).success)
        await tools.call("revise_section", agent_id="RESEARCHER_A",
                         section_name="section_a", feedback="Changed factual claims")
        self.assertFalse((await tools.call("review_sections", agent_id="EDITOR",
                                          scope="section_a")).success)
        self.assertFalse(sim.is_complete())

    async def test_repeated_rejections_need_changed_material_and_fresh_fact_checks(self):
        for tool, agent in [("fact_check", "FACT_CHECKER"), ("review_sections", "EDITOR")]:
            for failures in (1, 2, 3):
                sim, tools = world()
                sim._failure_specs = [FailureSpec(tool_name=tool, agent_id=agent,
                                                 call_index=index)
                                      for index in range(failures)]
                await tools.call("research_topic", agent_id="RESEARCHER_A", topic="subtopic_a")
                await tools.call("write_section", agent_id="RESEARCHER_A", section_name="section_a")
                await tools.call("update_references", agent_id="RESEARCHER_A", section_name="section_a")
                for attempt in range(failures + 1):
                    fact = await tools.call("fact_check", agent_id="FACT_CHECKER", section_name="section_a")
                    if tool == "fact_check":
                        outcome = fact
                    else:
                        self.assertTrue(fact.success)
                        outcome = await tools.call("review_sections", agent_id="EDITOR", scope="section_a")
                    self.assertEqual(outcome.success, attempt == failures)
                    if not outcome.success:
                        await tools.call("revise_section", agent_id="RESEARCHER_A", section_name="section_a",
                                         feedback=f"Actual correction {attempt}")
                self.assertEqual(sim.violations, [])

    async def test_pending_research_cannot_supply_a_write_and_results_are_owned_snapshots(self):
        sim, tools = world()
        sim._delay_multiplier = 1
        sim.tool_delay = lambda tool, **kwargs: (0.02, 0.02) if tool == "research_topic" else (0, 0)
        pending = asyncio.create_task(tools.call("research_topic", agent_id="RESEARCHER_A",
                                                 topic="subtopic_a"))
        await asyncio.sleep(0)
        self.assertFalse(pending.done())
        self.assertFalse((await tools.call("write_section", agent_id="RESEARCHER_A",
                                          section_name="section_a")).success)
        research = await pending
        original = dict(research.data["artifact"])
        research.data["artifact"]["content"] = "Forged caller-side result"
        written = await tools.call("write_section", agent_id="RESEARCHER_A", section_name="section_a")
        self.assertEqual(written.data["artifact"]["research_inputs"], {"subtopic_a": original})
        self.assertNotEqual(sim.events[0].result["artifact"]["content"], "Forged caller-side result")

    async def test_physical_conflicts_are_reported_without_scheduling_independent_calls(self):
        for first, second, resource in [
            ("update_references", "update_references", "DATABASE"),
            ("write_section", "write_section", "DOCUMENT"),
            ("write_section", "update_references", None),
        ]:
            sim, tools = world()
            for suffix in "ab":
                await tools.call("research_topic", agent_id=f"RESEARCHER_{suffix.upper()}",
                                 topic=f"subtopic_{suffix}")
            sim._delay_multiplier = 1
            sim.tool_delay = lambda tool, **kwargs: (0.02, 0.02)
            before = len(sim.events)
            a = asyncio.create_task(tools.call(first, agent_id="RESEARCHER_A", section_name="section_a"))
            b = asyncio.create_task(tools.call(second, agent_id="RESEARCHER_B", section_name="section_b"))
            await asyncio.sleep(0)
            self.assertEqual(len(sim.events), before + 2, "the world must not serialize callers")
            await asyncio.gather(a, b)
            if resource:
                self.assertTrue(any(v.violation_type == "concurrent_resource_use"
                                    for v in sim.violations))
            else:
                self.assertEqual(sim.violations, [])
