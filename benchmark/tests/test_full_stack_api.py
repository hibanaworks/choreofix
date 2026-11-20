"""Exercise the shared 8M score without a choreography or a prescribed recipe."""

import asyncio
import importlib
import itertools
import unittest

from benchmark.tools import load_tools

_OPERATIONS = {
    "schema": ("DB_ADMIN", "migrate_schema", {"migration_name": "feature_schema"}),
    "define": ("BACKEND_DEV", "define_api", {"endpoint": "feature_api_spec"}),
    "backend": ("BACKEND_DEV", "implement_backend", {"component": "api_endpoints"}),
    "frontend": ("FRONTEND_DEV", "implement_frontend", {"component": "user_interface"}),
    "backend_review": ("REVIEWER", "code_review", {"pr_id": "backend_api"}),
    "frontend_review": ("REVIEWER", "code_review", {"pr_id": "frontend_ui"}),
    "test": ("TESTER", "run_integration_tests", {"suite": "full_stack_integration"}),
}


class FullStackAPIWorld(unittest.IsolatedAsyncioTestCase):
    def setUp(self):
        self.world = importlib.import_module(
            "benchmark.environments.8M.sim"
        ).FullStackAPIDevelopmentSim()
        self.world._delay_multiplier = 0
        self.tools = load_tools("8M", sim=self.world)

    async def perform(self, name, **overrides):
        actor, tool, args = _OPERATIONS[name]
        return await self.tools.call(tool, agent_id=actor, **(args | overrides))

    async def prepare(self):
        for operation in ("schema", "define", "backend", "backend_review",
                          "frontend", "frontend_review"):
            self.assertTrue((await self.perform(operation)).success)

    async def test_all_serial_orders_preserve_only_required_input_dependencies(self):
        completed = 0
        for order in itertools.permutations(_OPERATIONS):
            self.setUp()
            for operation in order:
                await self.perform(operation)
            position = {operation: index for index, operation in enumerate(order)}
            expected = (
                position["schema"] < position["backend"] < position["backend_review"]
                < position["frontend"] < position["frontend_review"] < position["test"]
                and position["define"] < position["backend_review"]
            )
            self.assertEqual(self.world.is_complete(), expected, order)
            completed += self.world.is_complete()
        self.assertEqual(completed, 3)

    async def test_later_work_does_not_retroactively_complete_early_receipts(self):
        self.assertEqual((await self.perform("test")).data["status"], "unavailable_inputs")
        self.assertEqual((await self.perform("backend_review")).data["status"], "unavailable_inputs")
        early_frontend = await self.perform("frontend")
        self.assertIsNone(early_frontend.data["inputs"]["backend_review"])
        for step in ("schema", "backend", "define", "backend_review"):
            self.assertTrue((await self.perform(step)).success)
        self.assertEqual((await self.perform("frontend_review")).data["status"], "unavailable_inputs")
        self.assertFalse(self.world.is_complete())
        for step in ("frontend", "frontend_review", "test"):
            self.assertTrue((await self.perform(step)).success)
        self.assertTrue(self.world.is_complete())

    async def test_schema_change_invalidates_the_downstream_receipts(self):
        await self.prepare()
        self.assertTrue((await self.perform("test")).success)
        new_schema = await self.perform("schema")
        self.assertFalse(self.world.is_complete())
        self.assertFalse((await self.perform("backend_review")).success)
        backend = await self.perform("backend")
        self.assertEqual(backend.data["inputs"]["schema"], new_schema.data["revision"])
        self.assertTrue((await self.perform("backend_review")).success)
        self.assertFalse((await self.perform("frontend_review")).success)
        self.assertFalse((await self.perform("test")).success)
        for step in ("frontend", "frontend_review", "test"):
            self.assertTrue((await self.perform(step)).success)
        self.assertTrue(self.world.is_complete())

    async def test_spec_change_invalidates_backend_review_and_frontend_input(self):
        await self.prepare()
        self.assertTrue((await self.perform("test")).success)
        await self.perform("define", method="POST")
        self.assertFalse(self.world.is_complete())
        self.assertFalse((await self.perform("test")).success)
        review = await self.perform("backend_review")
        self.assertTrue(review.success)
        self.assertEqual(review.data["inputs"]["api"]["method"], "POST")
        self.assertFalse((await self.perform("frontend_review")).success)
        for step in ("frontend", "frontend_review", "test"):
            self.assertTrue((await self.perform(step)).success)
        self.assertTrue(self.world.is_complete())

    async def test_review_and_test_rejections_require_actual_rework(self):
        self.world.set_scenario_depth(1)
        for step in ("schema", "define", "backend"):
            await self.perform(step)
        self.assertEqual((await self.perform("backend_review")).data["status"], "changes_requested")
        self.assertEqual((await self.perform("backend_review")).data["status"], "rework_required")
        await self.perform("backend")
        for step in ("backend_review", "frontend", "frontend_review"):
            self.assertTrue((await self.perform(step)).success)
        failed = await self.perform("test")
        self.assertEqual(failed.data["status"], "tests_failed")
        self.assertEqual((await self.perform("test")).data["status"], "rework_required")
        # An internal backend repair keeps the already approved, compatible UI.
        await self.perform("backend")
        self.assertTrue((await self.perform("backend_review")).success)
        result = await self.perform("test")
        self.assertTrue(result.success)
        self.assertNotEqual(result.data["inputs"]["candidate"]["backend"],
                            failed.data["inputs"]["candidate"]["backend"])
        self.assertEqual(result.data["inputs"]["candidate"]["frontend"],
                         failed.data["inputs"]["candidate"]["frontend"])
        self.assertTrue(self.world.is_complete())

    async def test_pending_inputs_and_validation_do_not_count_as_completed(self):
        self.world.tool_delay = lambda *args, **kwargs: (0.02, 0.02)
        self.world._delay_multiplier = 1
        pending = asyncio.create_task(self.perform("schema"))
        await asyncio.sleep(0)
        self.world._delay_multiplier = 0
        early = await self.perform("backend")
        self.assertIsNone(early.data["inputs"]["schema"])
        await pending
        await self.prepare()
        self.world._delay_multiplier = 1
        pending = asyncio.create_task(self.perform("test"))
        await asyncio.sleep(0)
        self.assertFalse(self.world.is_complete())
        self.assertTrue((await pending).success)
        self.assertTrue(self.world.is_complete())

    async def test_coincident_requests_report_conflict_without_serializing(self):
        self.world.tool_delay = lambda *args, **kwargs: (0.02, 0.02)
        self.world._delay_multiplier = 1
        first = asyncio.create_task(self.perform("schema"))
        await asyncio.sleep(0)
        definition = asyncio.create_task(self.perform("define"))
        await asyncio.sleep(0)
        self.assertFalse(first.done())
        self.assertFalse(definition.done())
        self.assertEqual(self.world.violations, [])
        second = asyncio.create_task(self.perform("schema"))
        await asyncio.sleep(0)
        self.assertEqual([v.violation_type for v in self.world.violations], ["concurrent_resource_use"])
        self.assertFalse(first.done())
        self.assertFalse(second.done())
        original, contract, competing = await asyncio.gather(first, definition, second)
        self.assertTrue(contract.success)
        self.assertNotEqual(original.data["revision"], competing.data["revision"])

    async def test_reply_mutation_and_wrong_test_targets_cannot_forge_score(self):
        await self.prepare()
        wrong = await self.perform("test", suite="frontend only")
        self.assertFalse(wrong.success)
        self.assertFalse(self.world.is_complete())
        result = await self.perform("test")
        self.assertTrue(result.success)
        result.data["inputs"]["candidate"]["backend"] = -1
        result.data["inputs"]["candidate"]["api"]["method"] = "POST"
        self.assertTrue(self.world.is_complete())
