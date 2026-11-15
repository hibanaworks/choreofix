"""16E grades completed artifacts and publication-time evidence for every method."""
import asyncio
import importlib
import itertools
import unittest
from benchmark.tools import load_tools


class CICDWorld(unittest.IsolatedAsyncioTestCase):
    def setUp(self):
        self.sim = importlib.import_module("benchmark.environments.16E.sim").SimpleCICDSim()
        self.sim._delay_multiplier = 0
        self.registry = load_tools("16E", sim=self.sim)

    async def call(self, agent, tool, **arguments):
        return await self.registry.call(tool, agent_id=agent, **arguments)

    async def build(self):
        return await self.call("DEVELOPER", "compile_and_test", component="application")

    async def integration(self):
        return await self.call("TESTER", "run_integration_tests", suite="integration")

    async def deploy(self):
        return await self.call("DEPLOYER", "deploy_application", target="production")

    async def prepare(self):
        await self.build()
        await self.integration()
        return await self.deploy()

    async def test_all_six_serial_orders_remain_callable(self):
        for order in itertools.permutations(range(3)):
            self.setUp()
            operations = (self.build, self.integration, self.deploy)
            for index in order:
                await operations[index]()
            self.assertEqual(len(self.sim.events), 3)
            self.assertEqual(self.sim.is_complete(), order == (0, 1, 2), order)
            unsafe = order.index(0) < order.index(2) and order != (0, 1, 2)
            self.assertEqual(len(self.sim.violations), int(unsafe), order)

    async def test_early_deployment_is_real_but_later_tests_do_not_validate_it(self):
        await self.build()
        result = await self.deploy()
        self.assertTrue(result.success)
        self.assertIn("production/application.bin", self.sim.files)
        self.assertIsNone(result.data["integration"])
        self.assertEqual(self.sim.violations[0].violation_type, "unvalidated_publication")
        await self.integration()
        self.assertFalse(self.sim.is_complete())
        self.assertTrue((await self.deploy()).success)
        self.assertTrue(self.sim.is_complete())
        # Fixing the current deployment does not erase the earlier real effect.
        self.assertEqual(len(self.sim.violations), 1)
        self.assertEqual(len(self.sim.events[1].violations), 1)

    async def test_early_test_remains_failed_after_a_later_build(self):
        failed = await self.integration()
        self.assertFalse(failed.success)
        await self.build()
        await self.deploy()
        self.assertFalse(self.sim.is_complete())
        self.assertIsNone(failed.data["artifact_sha256"])

    async def test_built_code_is_executed(self):
        self.sim.files["application.py"] = b"def application(value):\n    return 0\n"
        self.assertFalse((await self.build()).success)
        self.assertFalse((await self.integration()).success)
        self.assertFalse(self.sim.is_complete())

    async def test_wrong_source_and_binary_are_not_covered_by_old_evidence(self):
        await self.prepare()
        self.assertTrue(self.sim.is_complete())
        original = self.sim.files["application.py"]
        self.sim.files["application.py"] += b"# another source\n"
        self.assertFalse(self.sim.is_complete())
        self.sim.files["application.py"] = original
        self.assertTrue(self.sim.is_complete())
        self.sim.files["production/application.bin"] = b"another artifact"
        self.assertFalse(self.sim.is_complete())

    async def test_identical_rebuilds_and_retests_do_not_invalidate_equal_content(self):
        await self.prepare()
        files = dict(self.sim.files)
        await self.integration()
        await self.build()
        self.assertEqual(files, self.sim.files)
        self.assertTrue(self.sim.is_complete())

    async def test_in_flight_build_and_tests_are_not_available_evidence(self):
        self.sim.tool_delay = lambda *args, **kwargs: (0.02, 0.02)
        self.sim._delay_multiplier = 1
        building = asyncio.create_task(self.build())
        await asyncio.sleep(0)
        tested = await self.integration()
        await building
        self.assertFalse(tested.success)
        testing = asyncio.create_task(self.integration())
        await asyncio.sleep(0)
        deployed = await self.deploy()
        await testing
        self.assertIsNone(deployed.data["integration"])
        self.assertFalse(self.sim.is_complete())
        await self.deploy()
        self.assertTrue(self.sim.is_complete())

    async def test_mutating_a_reply_or_progress_does_not_rewrite_the_world(self):
        built = await self.build()
        built.data["artifact_sha256"] = "forged"
        await self.integration()
        result = await self.deploy()
        result.data["integration"] = None
        progress = self.sim.progress
        progress["deployments"][0]["integration"] = None
        self.assertTrue(self.sim.is_complete())

    async def test_wrong_targets_do_not_complete_the_assignment(self):
        self.assertFalse((await self.call("DEVELOPER", "compile_and_test", component="other")).success)
        await self.build()
        self.assertFalse((await self.call("TESTER", "run_integration_tests", suite="other")).success)
        self.assertFalse((await self.call("DEPLOYER", "deploy_application", target="other")).success)
        self.assertFalse(self.sim.is_complete())

    async def test_physical_build_server_conflicts_remain_independent_of_scoring(self):
        await self.build()
        self.sim.tool_delay = lambda *args, **kwargs: (0.02, 0.02)
        self.sim._delay_multiplier = 1
        await asyncio.gather(self.build(), self.integration())
        self.assertIn("concurrent_resource_use", {v.violation_type for v in self.sim.violations})
        await self.deploy()
        self.assertTrue(self.sim.is_complete())
