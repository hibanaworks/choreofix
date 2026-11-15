"""4E scores the actual contract and integrated files, independently of Hibana."""
import asyncio
from hashlib import sha256
import importlib
import itertools
import json
import unittest

from benchmark.tools import load_tools


class APICoordinationWorld(unittest.IsolatedAsyncioTestCase):
    def setUp(self):
        self.sim = importlib.import_module(
            "benchmark.environments.4E.sim").FrontendBackendAPISim()
        self.sim._delay_multiplier = 0
        self.registry = load_tools("4E", sim=self.sim)

    async def call(self, agent, tool, **arguments):
        return await self.registry.call(tool, agent_id=agent, **arguments)

    async def define(self):
        return await self.call("BACKEND_DEV", "define_api", api_name="feature_api_spec")

    async def backend(self):
        return await self.call("BACKEND_DEV", "implement_feature", feature_name="backend_endpoint")

    async def frontend(self):
        return await self.call("FRONTEND_DEV", "implement_feature", feature_name="frontend_component")

    async def run_suite(self):
        return await self.call("TESTER", "run_tests", suite="integration_tests")

    async def prepare(self):
        await self.define()
        await self.backend()
        await self.frontend()

    async def test_all_24_serial_orders_and_both_implementation_orders(self):
        valid = 0
        for order in itertools.permutations(range(4)):
            self.setUp()
            operations = (self.define, self.backend, self.frontend, self.run_suite)
            for index in order:
                await operations[index]()
            expected = (order.index(0) < order.index(1) < order.index(3)
                        and order.index(0) < order.index(2) < order.index(3))
            self.assertEqual(self.sim.is_complete(), expected, order)
            if expected:
                valid += 1
                self.assertEqual(self.sim.violations, [])
        self.assertEqual(valid, 2)

    async def test_early_effects_are_executed_but_do_not_retroactively_pass(self):
        before = await self.backend()
        self.assertTrue(before.success)
        self.assertIn("backend.py", self.sim.files)
        self.assertIsNone(before.data["api_source"])
        early = await self.run_suite()
        self.assertFalse(early.success)
        await self.define()
        await self.frontend()
        self.assertFalse((await self.run_suite()).success)
        await self.backend()
        self.assertFalse(self.sim.is_complete())
        self.assertTrue((await self.run_suite()).success)
        self.assertTrue(self.sim.is_complete())

    async def test_tests_execute_code_and_reject_wrong_backend_or_frontend_values(self):
        for path, replacement in [
            ("backend.py", "def endpoint(value):\n    return 0\n"),
            ("frontend.py", "def render(value):\n    return 'invented result'\n"),
        ]:
            self.setUp()
            await self.prepare()
            original = self.sim.files[path]
            contract_line = original.splitlines(keepends=True)[0]
            self.sim.files[path] = contract_line + replacement
            checked = await self.run_suite()
            self.assertFalse(checked.success, path)
            self.assertIn("outputs", checked.data["validation"])
            self.assertFalse(self.sim.is_complete())

    async def test_finished_test_of_old_bytes_does_not_cover_changed_sources(self):
        await self.prepare()
        checked = await self.run_suite()
        self.assertTrue(self.sim.is_complete())
        source = self.sim.files["backend.py"]
        self.sim.files["backend.py"] = source + "# another actual candidate\n"
        self.assertFalse(self.sim.is_complete())
        self.assertNotEqual(checked.data["repository_sha256"], self.sim._identity(self.sim.files))
        self.sim.files["backend.py"] = source
        self.assertTrue(self.sim.is_complete())

    async def test_redundant_definition_and_implementation_do_not_invalidate_equal_bytes(self):
        await self.prepare()
        await self.run_suite()
        identity = self.sim._identity(self.sim.files)
        await self.define()
        await self.frontend()
        await self.backend()
        self.assertEqual(self.sim._identity(self.sim.files), identity)
        self.assertTrue(self.sim.is_complete())

    async def test_in_flight_definition_or_implementation_is_not_a_completed_input(self):
        self.sim.tool_delay = lambda *args, **kwargs: (0.02, 0.02)
        self.sim._delay_multiplier = 1
        pending = asyncio.create_task(self.define())
        await asyncio.sleep(0)
        result = await self.frontend()
        self.assertIsNone(result.data["api_source"])
        await pending
        await self.backend()
        await self.frontend()
        pending = asyncio.create_task(self.run_suite())
        await asyncio.sleep(0)
        self.assertFalse(self.sim.is_complete())
        self.assertTrue((await pending).success)
        self.assertTrue(self.sim.is_complete())

    async def test_reply_mutation_cannot_rewrite_world_or_test_evidence(self):
        defined = await self.define()
        defined.data["source"] = "forged definition"
        self.assertNotEqual(self.sim.files["api.json"], defined.data["source"])
        await self.backend()
        await self.frontend()
        tested = await self.run_suite()
        tested.data["files"]["backend.py"] = "forged implementation"
        tested.data["validation"]["success"] = False
        self.assertTrue(self.sim.is_complete())
        progress = self.sim.progress
        progress["test_results"][0]["validation"]["success"] = False
        self.assertTrue(self.sim.is_complete())

    async def test_unknown_targets_and_wrong_test_subject_do_not_satisfy_the_task(self):
        self.assertFalse((await self.call(
            "BACKEND_DEV", "define_api", api_name="another API")).success)
        self.assertFalse((await self.call(
            "BACKEND_DEV", "implement_feature", feature_name="another feature")).success)
        await self.prepare()
        self.assertFalse((await self.call(
            "TESTER", "run_tests", suite="another suite")).success)
        self.assertFalse(self.sim.is_complete())

    async def test_contract_identity_and_concurrent_resource_conflicts_are_independent(self):
        await self.define()
        expected = sha256(self.sim.files["api.json"].encode()).hexdigest()
        self.sim.tool_delay = lambda *args, **kwargs: (0.02, 0.02)
        self.sim._delay_multiplier = 1
        results = await asyncio.gather(self.backend(), self.frontend())
        self.assertTrue(all(result.success for result in results))
        self.assertTrue(all(expected in result.data["source"] for result in results))
        self.assertIn("concurrent_resource_use", {v.violation_type for v in self.sim.violations})
        self.assertTrue((await self.run_suite()).success)
        self.assertTrue(self.sim.is_complete())

    async def test_snapshot_requires_the_actual_current_contract(self):
        await self.prepare()
        original = self.sim.files["frontend.py"]
        self.sim.files["frontend.py"] = original.replace("API_SHA256 = '", "API_SHA256 = 'wrong")
        checked = await self.run_suite()
        self.assertFalse(checked.success)
        self.assertFalse(checked.data["validation"]["same_contract"])
        self.assertEqual(json.loads(self.sim.files["api.json"])["api_name"], "feature_api_spec")
