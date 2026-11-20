"""Check 4H's common effects with no choreography or prescribed schedule."""

import asyncio
import importlib
import itertools
import unittest

from benchmark.tools import load_tools

_WORLD = importlib.import_module("benchmark.environments.4H.sim").MicroserviceMigrationSim
_SERVICES = ("microservice_a", "microservice_b", "microservice_c")


class MicroserviceMigrationWorld(unittest.IsolatedAsyncioTestCase):
    def setUp(self):
        self.world = _WORLD()
        self.world._delay_multiplier = 0
        self.tools = load_tools("4H", sim=self.world)

    async def plan(self, **kwargs):
        return await self.tools.call(
            "design_proposal", agent_id="ARCHITECT", title="migration_plan", **kwargs,
        )

    async def act(self, action, service):
        tool, agent, args = {
            "build": ("implement_feature", f"DEVELOPER_{service[-1].upper()}", {"feature_name": service}),
            "review": ("code_review", "REVIEWER", {"pr_id": service}),
            "test": ("run_tests", "TESTER", {"suite": service}),
            "schema": ("migrate_schema", "DEVOPS", {"migration_name": f"{service}_schema"}),
            "stage": ("deploy_service", "DEVOPS", {"service": service, "environment": "staging"}),
            "rollback": ("rollback_service", "DEVOPS", {"service": service}),
            "register": ("register_service", "DEVOPS", {"service_name": service}),
        }[action]
        return await self.tools.call(tool, agent_id=agent, **args)

    async def prepare(self, service):
        for action in ("build", "review", "test"):
            result = await self.act(action, service)
            self.assertTrue(result.success, result.data)

    async def test_reverse_order_cannot_be_certified_by_later_successes(self):
        for service in _SERVICES:
            for step in ("register", "stage", "schema", "test", "review", "build"):
                self.assertFalse((await self.act(step, service)).success)
        self.assertTrue((await self.plan()).success)
        self.assertFalse(self.world.is_complete())

    async def test_independent_services_accept_every_staging_order(self):
        for migrations_first in (False, True):
            for deployment_order in itertools.permutations(_SERVICES):
                self.setUp()
                self.assertTrue((await self.plan()).success)
                for service in reversed(deployment_order):
                    await self.prepare(service)
                for service in deployment_order:
                    operations = ("schema", "stage") if migrations_first else ("stage", "schema")
                    for action in (*operations, "register"):
                        self.assertTrue((await self.act(action, service)).success)
                self.assertTrue(self.world.is_complete(), deployment_order)
                self.assertEqual(self.world.violations, [])

    async def test_dependency_graph_comes_from_the_plan_and_rejects_cycles(self):
        a, b, c = _SERVICES
        self.assertFalse((await self.plan(dependencies={a: [b], b: [a]})).success)
        self.assertFalse((await self.plan(dependencies={a: [a]})).success)
        self.assertTrue((await self.plan(dependencies={a: [c]})).success)
        for service in _SERVICES:
            await self.prepare(service)
        early = await self.act("stage", a)
        self.assertTrue(early.success)
        self.assertEqual(early.data["dependencies"], {c: None})
        self.assertEqual(self.world.violations[-1].violation_type, "unvalidated_deployment")
        self.assertTrue((await self.act("stage", b)).success)
        prerequisite = await self.act("stage", c)
        self.assertTrue(prerequisite.success)
        deployment = await self.act("stage", a)
        self.assertTrue(deployment.success)
        self.assertEqual(deployment.data["dependencies"], {c: prerequisite.data["receipt"]})
        for service in (b, c, a):
            self.assertTrue((await self.act("schema", service)).success)
            self.assertTrue((await self.act("register", service)).success)
        self.assertTrue(self.world.is_complete())

    async def test_all_service_graphs_and_deployment_registration_orders(self):
        edges = tuple(itertools.permutations(_SERVICES, 2))
        orders = tuple(itertools.permutations(_SERVICES))
        actions = tuple(itertools.product(("stage", "register"), _SERVICES))
        schedules = [
            schedule for schedule in itertools.permutations(actions)
            if all(schedule.index(("stage", service)) < schedule.index(("register", service))
                   for service in _SERVICES)
        ]
        self.assertEqual(len(schedules), 90)
        acyclic = qualified = early_registrations = 0
        for bits in itertools.product((False, True), repeat=len(edges)):
            dependencies = {service: [] for service in _SERVICES}
            for (service, prerequisite), included in zip(edges, bits):
                if included:
                    dependencies[service].append(prerequisite)
            allowed = {
                order for order in orders
                if all(order.index(prerequisite) < order.index(service)
                       for service, prerequisites in dependencies.items()
                       for prerequisite in prerequisites)
            }
            self.setUp()
            plan = await self.plan(dependencies=dependencies)
            self.assertEqual(plan.success, bool(allowed), dependencies)
            if not allowed:
                continue
            acyclic += 1
            for schedule in schedules:
                order = tuple(service for step, service in schedule if step == "stage")
                self.setUp()
                self.assertTrue((await self.plan(dependencies=dependencies)).success)
                for service in reversed(order):
                    await self.prepare(service)
                    self.assertTrue((await self.act("schema", service)).success)
                for step, service in schedule:
                    self.assertTrue((await self.act(step, service)).success)
                with self.subTest(dependencies=dependencies, schedule=schedule):
                    self.assertEqual(self.world.is_complete(), order in allowed)
                    self.assertEqual(bool(self.world.violations), order not in allowed)
                if self.world.is_complete():
                    qualified += 1
                    early_registrations += any(step == "register" for step, _ in schedule[:3])
        self.assertEqual((acyclic, qualified, early_registrations), (25, 720, 432))

    async def test_unsafe_staging_and_cutover_remain_observable_after_repair(self):
        a, b, c = _SERVICES
        await self.plan()
        for service in (b, c):
            await self.prepare(service)
            for step in ("schema", "stage", "register"):
                self.assertTrue((await self.act(step, service)).success)
        await self.act("build", a)
        early_test = await self.act("test", a)
        self.assertTrue(early_test.success)
        self.assertIsNone(early_test.data["review"])
        await self.act("review", a)
        early_stage = await self.act("stage", a)
        self.assertTrue(early_stage.success)
        self.assertIsNone(early_stage.data["test"])
        await self.act("schema", a)
        await self.act("test", a)
        cutover = await self.act("register", a)
        self.assertTrue(cutover.success)
        self.assertEqual(cutover.data["deployment"], early_stage.data["receipt"])
        self.assertIsNone(cutover.data["staging_test"])
        self.assertFalse(self.world.is_complete())
        await self.act("stage", a)
        await self.act("register", a)
        self.assertTrue(self.world.is_complete())
        self.assertEqual([v.violation_type for v in self.world.violations],
                         ["unvalidated_deployment", "unvalidated_cutover"])

    async def test_failed_attempts_require_rework_and_actual_rollback(self):
        a = _SERVICES[0]
        self.world.set_scenario_depth(1)
        await self.plan()
        await self.act("build", a)
        self.assertEqual((await self.act("review", a)).data["status"], "changes_requested")
        self.assertEqual((await self.act("review", a)).data["status"], "rework_required")
        await self.act("build", a)
        self.assertTrue((await self.act("review", a)).success)
        self.assertEqual((await self.act("test", a)).data["status"], "tests_failed")
        self.assertEqual((await self.act("test", a)).data["status"], "rework_required")
        await self.prepare(a)
        failed = await self.act("stage", a)
        self.assertEqual(failed.data["status"], "deployment_failed")
        self.assertEqual((await self.act("build", a)).data["status"], "rollback_required")
        self.assertEqual((await self.act("register", a)).data["status"], "rollback_required")
        rolled_back = await self.act("rollback", a)
        self.assertEqual(rolled_back.data["deployment"], failed.data["receipt"])
        self.assertFalse((await self.act("rollback", a)).success)
        self.assertEqual((await self.act("stage", a)).data["status"], "rework_required")
        await self.prepare(a)
        recovered = await self.act("stage", a)
        self.assertTrue(recovered.success)
        self.assertNotEqual(recovered.data["candidate"], failed.data["candidate"])

    async def test_revised_code_needs_new_receipts_before_registration(self):
        a = _SERVICES[0]
        await self.plan()
        await self.prepare(a)
        old = await self.act("stage", a)
        self.assertTrue(old.success)
        self.assertTrue((await self.act("schema", a)).success)
        await self.prepare(a)
        self.assertEqual((await self.act("register", a)).data["status"], "missing_deployment")
        new = await self.act("stage", a)
        registered = await self.act("register", a)
        self.assertTrue(registered.success)
        self.assertEqual(registered.data["deployment"], new.data["receipt"])
        self.assertNotEqual(new.data["candidate"], old.data["candidate"])

    async def test_pending_deployment_does_not_authorize_a_dependent_service(self):
        a, b, _ = _SERVICES
        await self.plan(dependencies={b: [a]})
        for service in (a, b):
            await self.prepare(service)
        self.world.tool_delay = lambda *args, **kwargs: (0.02, 0.02)
        self.world._delay_multiplier = 1
        pending = asyncio.create_task(self.act("stage", a))
        await asyncio.sleep(0)
        self.assertFalse(pending.done())
        self.assertEqual((await self.act("stage", b)).data["dependencies"], {a: None})
        self.assertTrue((await pending).success)
        self.assertEqual([v.violation_type for v in self.world.violations],
                         ["concurrent_resource_use", "unvalidated_deployment"])
        self.world._delay_multiplier = 0
        self.assertTrue((await self.act("stage", b)).success)
