"""CI/CD worlds grade completed artifacts and publication-time evidence."""
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


class PipelineWorld(unittest.IsolatedAsyncioTestCase):
    frontend = "FRONTEND_DEV"
    backend = "BACKEND_DEV"
    qa = "QA_ENGINEER"
    release = "RELEASE_ENGINEER"
    build_calls = (
        (frontend, "compile_code", {"component": "frontend"}),
        (frontend, "run_unit_tests", {"component": "frontend"}),
        (frontend, "publish_artifact", {"artifact": "frontend_image"}),
        (backend, "compile_code", {"component": "backend"}),
        (backend, "run_unit_tests", {"component": "backend"}),
        (backend, "publish_artifact", {"artifact": "backend_image"}),
    )
    qa_pull = (qa, "pull_artifacts", {"target": "test_harness"})
    qa_checks = (
        (qa, "run_integration_tests", {"suite": "api_integration"}),
        (qa, "run_e2e_tests", {"suite": "user_flows"}),
    )
    stage = (
        (release, "pull_artifacts", {"target": "staging_deploy"}),
        (release, "deploy_staging", {"component": "full_stack"}),
        (release, "run_smoke_tests", {"target": "staging"}),
    )
    production = (
        (release, "deploy_production", {"component": "full_stack"}),
        (release, "run_smoke_tests", {"target": "production"}),
    )
    ordinary = (*build_calls, qa_pull, *qa_checks, *stage, *production)

    def setUp(self):
        self.sim = importlib.import_module("benchmark.environments.16M.sim").CICDPipelineSim()
        self.sim._delay_multiplier = 0
        self.registry = load_tools("16M", sim=self.sim)

    async def call(self, request):
        agent, tool, arguments = request
        return await self.registry.call(tool, agent_id=agent, **arguments)

    async def execute(self, requests):
        for request in requests:
            result = await self.call(request)
            self.assertTrue(result.success, (request, result))
        return result

    async def test_exact_reverse_order_leaves_only_compilation_effects(self):
        reverse = (*self.production[::-1], *self.stage[:0:-1], *self.qa_checks[::-1],
                   self.stage[0], self.qa_pull, *self.build_calls[2::3],
                   *self.build_calls[1::3], *self.build_calls[::3])
        successes = []
        for request in reverse:
            if (await self.call(request)).success:
                successes.append(request[1])
        self.assertEqual(successes, ["compile_code", "compile_code"])
        self.assertEqual(len(self.sim.events), 14)
        self.assertFalse(self.sim.is_complete())
        self.assertEqual(self.sim.progress["published"], {})
        self.assertIsNone(self.sim.progress["production"])

    async def test_independent_builds_qa_suites_and_release_preparation_can_be_reordered(self):
        front = self.build_calls[:3]
        back = self.build_calls[3:]
        for build_order, checks, stage_first in itertools.product(
                ((*front, *back), (*back, *front)),
                (self.qa_checks, self.qa_checks[::-1]),
                (False, True)):
            self.setUp()
            await self.execute(build_order)
            qa = (self.qa_pull, *checks)
            await self.execute((*self.stage, *qa) if stage_first else (*qa, *self.stage))
            await self.execute(self.production)
            self.assertTrue(self.sim.is_complete())
            self.assertFalse(self.sim.has_violations)

    async def test_production_before_checks_remains_unvalidated_after_tests_pass(self):
        await self.execute((*self.build_calls, self.stage[0]))
        early = await self.execute(self.production)
        self.assertGreater(early.data["deployment"], 0)
        deployment = self.sim.progress["production"]
        self.assertEqual(set(deployment["qa"].values()), {None})
        self.assertIsNone(deployment["staging_smoke"])
        self.assertEqual(self.sim.violations[0].violation_type, "unvalidated_publication")
        await self.execute((self.qa_pull, *self.qa_checks, *self.stage[1:]))
        self.assertFalse(self.sim.is_complete())
        await self.execute(self.production)
        self.assertTrue(self.sim.is_complete())
        self.assertEqual(len(self.sim.violations), 1)

    async def test_rebuilt_image_cannot_inherit_the_old_pair_tests_or_smoke(self):
        await self.execute(self.ordinary)
        original = self.sim.progress["production"]["artifacts"]
        await self.execute(self.build_calls[3:])
        self.assertFalse(self.sim.is_complete())
        await self.call((self.release, "pull_artifacts", {"target": "production_deploy"}))
        result = await self.call(self.production[0])
        self.assertTrue(result.success)
        self.assertEqual(result.data["artifacts"]["frontend_image"], original["frontend_image"])
        self.assertNotEqual(result.data["artifacts"]["backend_image"], original["backend_image"])
        self.assertIsNone(result.data["staging_smoke"])
        self.assertEqual(set(result.data["qa"].values()), {None})
        await self.execute((self.production[1], self.qa_pull, *self.qa_checks, *self.stage))
        self.assertFalse(self.sim.is_complete())
        await self.execute(self.production)
        self.assertTrue(self.sim.is_complete())

    async def test_pending_build_and_staging_smoke_are_unavailable(self):
        self.sim.tool_delay = lambda tool, **kwargs: (0.02, 0.02) if tool in (
            "compile_code", "run_smoke_tests") else (0, 0)
        self.sim._delay_multiplier = 1
        build = asyncio.create_task(self.call(self.build_calls[0]))
        await asyncio.sleep(0)
        try:
            tested = await self.call(self.build_calls[1])
            self.assertFalse(tested.success)
            self.assertIsNone(tested.data["build"])
        finally:
            self.assertTrue((await build).success)

        await self.execute(self.ordinary[1:11])
        smoke = asyncio.create_task(self.call(self.stage[2]))
        await asyncio.sleep(0)
        try:
            deployed = await self.call(self.production[0])
            self.assertTrue(deployed.success)
            self.assertIsNone(deployed.data["staging_smoke"])
        finally:
            self.assertTrue((await smoke).success)

        await self.call(self.production[1])
        self.assertFalse(self.sim.is_complete())
        self.assertIn("concurrent_resource_use", {v.violation_type for v in self.sim.violations})

    async def test_failed_staging_smoke_cannot_authorize_production(self):
        self.sim.configure_scenario("staging_fail")
        await self.execute(self.ordinary[:-3])
        failed = await self.call(self.stage[2])
        self.assertFalse(failed.success)
        deployed = await self.call(self.production[0])
        self.assertTrue(deployed.success)
        self.assertIsNone(deployed.data["staging_smoke"])
        self.assertFalse(self.sim.is_complete())

    async def test_build_and_qa_failures_apply_to_real_inputs_and_allow_recovery(self):
        self.sim.configure_scenario("build_fail")
        await self.execute(self.build_calls[:3])
        self.assertFalse((await self.call(self.build_calls[3])).success)
        self.assertFalse((await self.call(self.build_calls[4])).success)
        await self.execute(self.build_calls[3:])

        self.sim.configure_scenario("qa_fail")
        self.assertFalse((await self.call(self.qa_checks[0])).success)
        await self.call(self.qa_pull)
        failure = await self.call(self.qa_checks[0])
        self.assertFalse(failure.success)
        self.assertIsNotNone(failure.data["artifacts"])
        await self.execute((*self.build_calls[3:], self.qa_pull, *self.qa_checks,
                            *self.stage, *self.production))
        self.assertTrue(self.sim.is_complete())
        self.assertFalse(self.sim.has_violations)

    async def test_artifact_published_before_unit_tests_retains_that_fact(self):
        await self.execute((self.build_calls[0], self.build_calls[2], self.build_calls[1],
                            *self.ordinary[3:]))
        self.assertFalse(self.sim.is_complete())
        self.assertIsNone(self.sim.progress["production"]["unit_tests"]["frontend_image"])
        self.assertEqual([v.violation_type for v in self.sim.violations],
                         ["unvalidated_artifact", "unvalidated_publication"])

    async def test_smoke_uses_the_deployment_it_actually_tested(self):
        await self.execute(self.ordinary)
        first = self.sim.progress["production"]["receipt"]
        next_deployment = await self.call(self.production[0])
        self.assertNotEqual(next_deployment.data["receipt"], first)
        self.assertFalse(self.sim.is_complete())
        smoke = await self.call(self.production[1])
        self.assertEqual(smoke.data["deployment"], next_deployment.data["receipt"])
        smoke.data["deployment"] = first
        self.assertTrue(self.sim.is_complete())
