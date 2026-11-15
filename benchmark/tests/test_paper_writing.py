"""Actual input causality, free schedules and physical sections for all methods."""
import asyncio
import importlib
import itertools
import unittest

from benchmark.tools import load_tools
from benchmark.tools.sim_base import FailureSpec


def world(task):
    module = importlib.import_module(f"benchmark.environments.{task}.sim")
    sim = module.ResearchPaperSim()
    sim._delay_multiplier = 0
    inputs = {step.agent_id: {
        "topic": step.topic,
        "section": next(row.section for row in module._DRAFT_STEPS
                        if row.agent_id == step.agent_id),
        "sections": next(row.sections for row in module._WRITE_STEPS
                         if row.agent_id == step.agent_id),
    } for step in module._READ_STEPS}
    return sim, load_tools(task, sim=sim), inputs


async def complete(tools, agent, inputs):
    replies = []
    for tool, arguments in [
        ("read_literature", {"topic": inputs["topic"]}),
        ("draft_content", {"section_name": inputs["section"]}),
        ("write_to_document", {"sections": ", ".join(inputs["sections"])}),
        ("review_own_work", {"section_name": inputs["section"]}),
    ]:
        replies.append(await tools.call(tool, agent_id=agent, **arguments))
    return replies


class PaperWritingWorld(unittest.IsolatedAsyncioTestCase):
    async def test_late_inputs_cannot_retroactively_complete_any_local_tool_order(self):
        for task in ["2E", "2M", "2H"]:
            for order in itertools.permutations(range(4)):
                sim, tools, inputs = world(task)
                agent = "RESEARCHER_A"
                item = inputs[agent]
                calls = [
                    ("read_literature", {"topic": item["topic"]}),
                    ("draft_content", {"section_name": item["section"]}),
                    ("write_to_document", {"sections": ",".join(item["sections"])}),
                    ("review_own_work", {"section_name": item["section"]}),
                ]
                for index in order:
                    tool, arguments = calls[index]
                    await tools.call(tool, agent_id=agent, **arguments)
                for other, data in inputs.items():
                    if other != agent:
                        self.assertTrue(all(r.success for r in await complete(tools, other, data)))
                self.assertEqual(sim.is_complete(), order == (0, 1, 2, 3), (task, order))
                if not sim.is_complete():
                    self.assertTrue(all(r.success for r in await complete(tools, agent, item)))
                self.assertTrue(sim.is_complete())
                self.assertEqual(sim.violations, [])

    async def test_rotated_and_reversed_author_orders_have_the_same_goal(self):
        for task in ["2E", "2M", "2H"]:
            _, _, assignments = world(task)
            agents = list(assignments)
            for offset in range(len(agents)):
                for order in [agents[offset:] + agents[:offset],
                              list(reversed(agents[offset:] + agents[:offset]))]:
                    sim, tools, inputs = world(task)
                    for agent in order:
                        replies = await complete(tools, agent, inputs[agent])
                        self.assertTrue(all(reply.success for reply in replies))
                    self.assertTrue(sim.is_complete(), (task, order))
                    self.assertEqual(sim.violations, [])

    async def test_written_material_contains_actual_completed_literature_and_draft(self):
        for task in ["2E", "2M", "2H"]:
            sim, tools, inputs = world(task)
            for agent, data in inputs.items():
                read, draft, written, reviewed = await complete(tools, agent, data)
                self.assertEqual(draft.data["artifact"]["literature"][data["topic"]],
                                 read.data["artifact"])
                self.assertEqual(written.data["artifact"]["draft"], draft.data["artifact"])
                self.assertEqual(reviewed.data["input_snapshot"], written.data["artifact"])
                self.assertEqual(sim.events[-3].args, {"section_name": data["section"]})
                self.assertEqual(sim.events[-2].args, {"sections": ", ".join(data["sections"])})
                reviewed.data["input_snapshot"]["content"] = "caller-owned changed value"
                self.assertNotEqual(sim.events[-1].result["input_snapshot"]["content"],
                                    "caller-owned changed value")
            self.assertTrue(sim.is_complete())

    async def test_new_literature_requires_current_draft_write_and_review(self):
        sim, tools, inputs = world("2E")
        for agent, data in inputs.items():
            await complete(tools, agent, data)
        self.assertTrue(sim.is_complete())
        data = inputs["RESEARCHER_A"]
        sim.library[data["topic"]] = "New actual literature edition"
        await tools.call("read_literature", agent_id="RESEARCHER_A", topic=data["topic"])
        self.assertFalse(sim.is_complete())
        await tools.call("draft_content", agent_id="RESEARCHER_A", section_name=data["section"])
        self.assertFalse(sim.is_complete())
        await tools.call("write_to_document", agent_id="RESEARCHER_A", sections=data["sections"])
        self.assertFalse(sim.is_complete())
        await tools.call("review_own_work", agent_id="RESEARCHER_A", section_name=data["section"])
        self.assertTrue(sim.is_complete())

    async def test_identical_material_rewrites_keep_equal_content_evidence(self):
        sim, tools, inputs = world("2E")
        for agent, data in inputs.items():
            await complete(tools, agent, data)
        before = sim.progress
        data = inputs["RESEARCHER_A"]
        await tools.call("read_literature", agent_id="RESEARCHER_A", topic=data["topic"])
        await tools.call("draft_content", agent_id="RESEARCHER_A", section_name=data["section"])
        await tools.call("write_to_document", agent_id="RESEARCHER_A",
                         sections=" , ".join(reversed(data["sections"])))
        self.assertTrue(sim.is_complete())
        self.assertEqual(sim.progress, before)

    async def test_an_unfinished_replacement_cannot_supply_another_operation(self):
        sim, tools, inputs = world("2E")
        data = inputs["RESEARCHER_A"]
        sim._delay_multiplier = 0.1
        reading = asyncio.create_task(tools.call(
            "read_literature", agent_id="RESEARCHER_A", topic=data["topic"]))
        await asyncio.sleep(0.01)
        sim._delay_multiplier = 0
        draft = await tools.call("draft_content", agent_id="RESEARCHER_A",
                                 section_name=data["section"])
        self.assertFalse(reading.done())
        self.assertFalse(draft.success)
        await reading
        self.assertTrue((await tools.call("draft_content", agent_id="RESEARCHER_A",
                                         section_name=data["section"])).success)

    async def test_terminal_failed_review_is_not_replaced_by_an_older_approval(self):
        sim, tools, inputs = world("2E")
        for agent, data in inputs.items():
            await complete(tools, agent, data)
        self.assertTrue(sim.is_complete())
        sim._failure_specs = [FailureSpec("review_own_work", "RESEARCHER_A", 1)]
        rejected = await tools.call("review_own_work", agent_id="RESEARCHER_A",
                                    section_name=inputs["RESEARCHER_A"]["section"])
        self.assertFalse(rejected.success)
        self.assertIn("input_snapshot", rejected.data)
        self.assertFalse(sim.is_complete())

    async def test_disjoint_writes_overlap_and_shared_section_conflicts_stay_visible(self):
        for agents, collision in [(["RESEARCHER_A", "RESEARCHER_C"], False),
                                  (["RESEARCHER_A", "RESEARCHER_B"], True)]:
            sim, tools, inputs = world("2M")
            for agent in agents:
                await tools.call("read_literature", agent_id=agent, topic=inputs[agent]["topic"])
                await tools.call("draft_content", agent_id=agent,
                                 section_name=inputs[agent]["section"])
            sim._delay_multiplier = 0.1
            writes = [asyncio.create_task(tools.call(
                "write_to_document", agent_id=agent, sections=inputs[agent]["sections"]))
                for agent in agents]
            await asyncio.sleep(0.01)
            self.assertTrue(all(not write.done() for write in writes))
            results = await asyncio.gather(*writes)
            self.assertTrue(all(result.success for result in results))
            self.assertEqual(bool(sim.violations), collision)

    async def test_supplementary_research_and_wrong_sections_do_not_supply_assignments(self):
        sim, tools, inputs = world("2E")
        data = inputs["RESEARCHER_A"]
        self.assertTrue((await tools.call("read_literature", agent_id="RESEARCHER_A",
                                         topic="independent supplementary reading")).success)
        self.assertFalse((await tools.call("draft_content", agent_id="RESEARCHER_A",
                                          section_name=data["section"])).success)
        await tools.call("read_literature", agent_id="RESEARCHER_A", topic=data["topic"])
        await tools.call("draft_content", agent_id="RESEARCHER_A", section_name=data["section"])
        self.assertFalse((await tools.call("write_to_document", agent_id="RESEARCHER_A",
                                          sections=data["sections"] + data["sections"])).success)
        self.assertIsNone(sim.progress["contributions"][data["section"]])
