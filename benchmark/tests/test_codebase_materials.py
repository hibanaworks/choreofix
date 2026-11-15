"""Code material and result causality, independent of any method's schedule."""
import asyncio
import importlib
import itertools
import unittest

from benchmark.tools import load_tools
from benchmark.tools.sim_base import FailureSpec


def world(task):
    module = importlib.import_module(f"benchmark.environments.{task}.sim")
    sim = module.SharedCodebaseSim()
    sim._delay_multiplier = 0
    sim.set_scenario_depth(0)
    inputs = {step.agent_id: {
        "feature": step.feature,
        "modules": next(row.modules for row in module._COMMIT_STEPS
                        if row.agent_id == step.agent_id),
    } for step in module._DESIGN_STEPS}
    return sim, load_tools(task, sim=sim), inputs


def calls(data):
    return [
        ("design_feature", {"feature_name": data["feature"]}),
        ("implement_code", {"feature_name": data["feature"]}),
        ("commit_changes", {"modules": ", ".join(data["modules"])}),
        ("run_local_tests", {"feature_name": data["feature"]}),
    ]


async def execute(tools, agent, work):
    return [await tools.call(tool, agent_id=agent, **arguments) for tool, arguments in work]


class CodebaseMaterialWorld(unittest.IsolatedAsyncioTestCase):
    async def test_all_24_local_orders_remain_callable_without_retroactive_completion(self):
        for task in ["1E", "1M", "1H"]:
            for order in itertools.permutations(range(4)):
                sim, tools, inputs = world(task)
                work = calls(inputs["DEVELOPER_A"])
                await execute(tools, "DEVELOPER_A", [work[i] for i in order])
                for agent, data in inputs.items():
                    if agent != "DEVELOPER_A":
                        self.assertTrue(all(r.success for r in await execute(tools, agent, calls(data))))
                self.assertEqual(sim.is_complete(), order == (0, 1, 2, 3), (task, order))
                if not sim.is_complete():
                    self.assertTrue(all(r.success for r in await execute(tools, "DEVELOPER_A", work)))
                self.assertTrue(sim.is_complete())
                self.assertEqual(sim.violations, [])

    async def test_author_orders_and_interleaved_preparation_have_the_same_goal(self):
        for task in ["1E", "1M", "1H"]:
            _, _, assignments = world(task)
            agents = list(assignments)
            for offset in range(len(agents)):
                sim, tools, inputs = world(task)
                order = agents[offset:] + agents[:offset]
                for agent in reversed(order):
                    await execute(tools, agent, calls(inputs[agent])[:2])
                for agent in order:
                    await execute(tools, agent, calls(inputs[agent])[2:])
                self.assertTrue(sim.is_complete(), (task, order))
                self.assertEqual(sim.violations, [])

    async def test_commit_and_test_contain_actual_owned_input_snapshots(self):
        for task in ["1E", "1M", "1H"]:
            sim, tools, inputs = world(task)
            for agent, data in inputs.items():
                design, source, commit, test = await execute(tools, agent, calls(data))
                self.assertEqual(source.data["artifact"]["design"], design.data["artifact"])
                self.assertEqual(commit.data["artifact"]["source"], source.data["artifact"])
                self.assertEqual(test.data["input_snapshot"], commit.data["artifact"])
                self.assertEqual(sim.events[-1].args, {"feature_name": data["feature"]})
                self.assertEqual(sim.events[-2].args, {"modules": ", ".join(data["modules"])})
                test.data["input_snapshot"]["content"] = "caller changed its own JSON"
                self.assertNotEqual(sim.events[-1].result["input_snapshot"]["content"],
                                    "caller changed its own JSON")
            self.assertTrue(sim.is_complete())

    async def test_same_content_rewrites_preserve_matching_test_evidence(self):
        for task in ["1E", "1M", "1H"]:
            sim, tools, inputs = world(task)
            for agent, data in inputs.items():
                await execute(tools, agent, calls(data))
            before = sim.progress
            for agent, data in inputs.items():
                await execute(tools, agent, calls(data)[:3])
            self.assertTrue(sim.is_complete())
            self.assertEqual(sim.progress, before)

    async def test_changed_specification_needs_current_source_commit_and_test(self):
        sim, tools, inputs = world("1H")
        for agent, data in inputs.items():
            await execute(tools, agent, calls(data))
        data = inputs["DEVELOPER_A"]
        sim.specifications[data["feature"]] = "Changed authentication contract"
        self.assertFalse(sim.is_complete())
        for tool, arguments in calls(data)[:3]:
            self.assertTrue((await tools.call(tool, agent_id="DEVELOPER_A", **arguments)).success)
            self.assertFalse(sim.is_complete())
        await execute(tools, "DEVELOPER_A", calls(data)[3:])
        self.assertTrue(sim.is_complete())

    async def test_injected_failure_requires_actual_repaired_source_and_another_commit(self):
        sim, tools, inputs = world("1H")
        sim._failure_specs = [FailureSpec("run_local_tests", "DEVELOPER_A", 0)]
        for agent, data in inputs.items():
            replies = await execute(tools, agent, calls(data))
            self.assertEqual(replies[-1].success, agent != "DEVELOPER_A")
        failed = sim.events[3].result
        data = inputs["DEVELOPER_A"]
        self.assertFalse(sim.is_complete())
        premature = (await execute(tools, "DEVELOPER_A", calls(data)[3:]))[0]
        self.assertFalse(premature.success)
        repair = (await execute(tools, "DEVELOPER_A", calls(data)[1:2]))[0]
        self.assertEqual(repair.data["artifact"]["repair"]["test_id"], failed["test_id"])
        self.assertEqual(repair.data["artifact"]["repair"]["tested_commit"], failed["input_snapshot"]["id"])
        self.assertFalse((await execute(tools, "DEVELOPER_A", calls(data)[3:]))[0].success)
        self.assertTrue(all(r.success for r in await execute(tools, "DEVELOPER_A", calls(data)[2:])))
        self.assertTrue(sim.is_complete())

    async def test_depth_zero_one_two_and_four_keep_original_failure_counts(self):
        for depth in [0, 1, 2, 4]:
            sim, tools, inputs = world("1H")
            sim.set_scenario_depth(depth)
            for agent, data in inputs.items():
                await execute(tools, agent, calls(data)[:1])
                for attempt in range(depth + 1):
                    replies = await execute(tools, agent, calls(data)[1:])
                    self.assertTrue(all(r.success for r in replies[:-1]))
                    self.assertEqual(replies[-1].success, attempt == depth)
            self.assertTrue(sim.is_complete())
            self.assertEqual(sum(not e.success for e in sim.events), depth * 7)

    async def test_a_later_failure_cannot_reuse_an_older_passing_receipt(self):
        sim, tools, inputs = world("1H")
        for agent, data in inputs.items():
            await execute(tools, agent, calls(data))
        self.assertTrue(sim.is_complete())
        sim._failure_specs = [FailureSpec("run_local_tests", "DEVELOPER_A", 1)]
        self.assertFalse((await execute(tools, "DEVELOPER_A", calls(inputs["DEVELOPER_A"])[3:]))[0].success)
        self.assertFalse(sim.is_complete())
        self.assertTrue(all(r.success for r in await execute(tools, "DEVELOPER_A", calls(inputs["DEVELOPER_A"])[1:])))
        self.assertTrue(sim.is_complete())

    async def test_unfinished_design_and_commit_cannot_supply_other_operations(self):
        for pending_tool, dependent_index in [("design_feature", 1), ("commit_changes", 3)]:
            sim, tools, inputs = world("1E")
            data = inputs["DEVELOPER_A"]
            work = calls(data)
            await execute(tools, "DEVELOPER_A", work[:0 if pending_tool == "design_feature" else 2])
            sim._delay_multiplier = 0.03
            tool, arguments = next(pair for pair in work if pair[0] == pending_tool)
            pending = asyncio.create_task(tools.call(tool, agent_id="DEVELOPER_A", **arguments))
            while not any(e.tool == pending_tool for e in sim.events):
                await asyncio.sleep(0)
            dependent = (await execute(tools, "DEVELOPER_A", work[dependent_index:dependent_index + 1]))[0]
            self.assertFalse(dependent.success)
            self.assertTrue((await pending).success)
            self.assertTrue((await execute(tools, "DEVELOPER_A", work[dependent_index:dependent_index + 1]))[0].success)

    async def test_disjoint_commits_overlap_and_shared_module_conflicts_are_observed(self):
        for agents, conflict in [(["DEVELOPER_A", "DEVELOPER_C"], False),
                                 (["DEVELOPER_A", "DEVELOPER_B"], True)]:
            sim, tools, inputs = world("1M")
            for agent in agents:
                await execute(tools, agent, calls(inputs[agent])[:2])
            sim._delay_multiplier = 0.05
            replies = await asyncio.gather(*(execute(tools, agent, calls(inputs[agent])[2:3])
                                             for agent in agents))
            self.assertTrue(all(reply[0].success for reply in replies))
            commits = [e for e in sim.events if e.tool == "commit_changes"]
            self.assertLess(max(e.timestamp for e in commits), min(e.completed_at for e in commits))
            self.assertEqual(bool(sim.violations), conflict)

    async def test_other_features_and_wrong_module_pairs_do_not_complete_assigned_work(self):
        for task in ["1E", "1M", "1H"]:
            sim, tools, inputs = world(task)
            await tools.call("design_feature", agent_id="DEVELOPER_A", feature_name="extra feature")
            await tools.call("implement_code", agent_id="DEVELOPER_A", feature_name="extra feature")
            self.assertFalse((await execute(tools, "DEVELOPER_A", calls(inputs["DEVELOPER_A"])[2:3]))[0].success)
            await execute(tools, "DEVELOPER_A", calls(inputs["DEVELOPER_A"])[:2])
            module = inputs["DEVELOPER_A"]["modules"][0]
            self.assertFalse((await tools.call("commit_changes", agent_id="DEVELOPER_A", modules=f"{module},{module}")).success)
            for agent, data in inputs.items():
                self.assertTrue(all(r.success for r in await execute(tools, agent, calls(data))))
            self.assertTrue(sim.is_complete())
