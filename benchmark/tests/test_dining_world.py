"""Dining task scores preserve per-diner causality and independent schedules."""
import asyncio
import importlib
import itertools
import unittest

from benchmark.tools import load_tools


def world(task="9E"):
    module = importlib.import_module(f"benchmark.environments.{task}.sim")
    sim = {"9E": "Three", "9M": "Five", "9H": "Seven"}[task]
    sim = getattr(module, sim + "PhilosophersSim")()
    sim._delay_multiplier = 0
    return sim, load_tools(task, sim=sim), module._PHILOSOPHER_IDS


class DiningWorld(unittest.IsolatedAsyncioTestCase):
    async def test_every_order_of_three_thinks_and_three_meals(self):
        _, _, agents = world()
        steps = [(agent, tool) for agent in agents for tool in ("think", "eat")]
        valid = 0
        meal_orders = set()
        for order in itertools.permutations(steps):
            sim, tools, _ = world()
            for agent, tool in order:
                reply = await tools.call(tool, agent_id=agent, philosopher=agent)
                self.assertTrue(reply.success)
            premature = {agent for agent in agents
                         if order.index((agent, "eat")) < order.index((agent, "think"))}
            self.assertEqual(sim.is_complete(), not premature, order)
            self.assertEqual({v.agent for v in sim.violations}, premature)
            if not premature:
                valid += 1
                meal_orders.add(tuple(agent for agent, tool in order if tool == "eat"))
        self.assertEqual(valid, 90)
        self.assertEqual(meal_orders, set(itertools.permutations(agents)))

    async def test_unfinished_think_and_retry_in_each_world(self):
        for task in ("9E", "9M", "9H"):
            sim, tools, agents = world(task)
            agent = agents[-1]
            sim._delay_multiplier = 0.01
            thinking = asyncio.create_task(tools.call("think", agent_id=agent, philosopher=agent))
            await asyncio.sleep(0)
            self.assertIsNone(sim.events[-1].completed_at)
            sim._delay_multiplier = 0
            await tools.call("eat", agent_id=agent, philosopher=agent)
            await thinking
            for other in agents[:-1]:
                await tools.call("think", agent_id=other, philosopher=other)
                await tools.call("eat", agent_id=other, philosopher=other)
            self.assertFalse(sim.is_complete())
            await tools.call("eat", agent_id=agent, philosopher=agent)
            self.assertTrue(sim.is_complete())
            self.assertEqual([v.violation_type for v in sim.violations], ["eat_before_think"])

    async def test_overlap_depends_only_on_shared_forks(self):
        for task in ("9M", "9H"):
            for contender, conflicts in ((1, True), (2, False)):
                sim, tools, agents = world(task)
                first, second = agents[0], agents[contender]
                for diner in (first, second):
                    await tools.call("think", agent_id=diner, philosopher=diner)
                sim._delay_multiplier = 0.01
                eating = asyncio.create_task(tools.call("eat", agent_id=first, philosopher=first))
                await asyncio.sleep(0)
                self.assertIsNone(sim.events[-1].completed_at)
                self.assertFalse(sim.progress["eating"][first])
                sim._delay_multiplier = 0
                await tools.call("eat", agent_id=second, philosopher=second)
                await eating
                expected = ["concurrent_resource_use"] if conflicts else []
                self.assertEqual([v.violation_type for v in sim.violations], expected)
