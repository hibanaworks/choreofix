"""11M scores actual parts and quality evidence without prescribing job order."""
import asyncio
import importlib
import itertools
import unittest

from benchmark.tools import load_tools
from benchmark.tools.sim_base import FailureSpec


JOBS = (
    ('WORKER_A', 'alpha_job_a', 'STATION_ALPHA', True),
    ('WORKER_A', 'beta_job_a', 'STATION_BETA', False),
    ('WORKER_B', 'beta_job_b', 'STATION_BETA', True),
    ('WORKER_B', 'gamma_job_b', 'STATION_GAMMA', False),
    ('WORKER_C', 'gamma_job_c', 'STATION_GAMMA', True),
    ('WORKER_C', 'alpha_job_c', 'STATION_ALPHA', False),
)


class ManufacturingReworkWorld(unittest.IsolatedAsyncioTestCase):
    def setUp(self):
        self.sim = importlib.import_module(
            'benchmark.environments.11M.sim').ThreeWorkerToolCabinetSim()
        self.sim.set_scenario_depth(0)
        self.sim._delay_multiplier = 0
        self.registry = load_tools('11M', sim=self.sim)

    async def job(self, index):
        worker, job, _, _ = JOBS[index]
        return await self.registry.call('do_job', agent_id=worker, job_id=job)

    async def inspect(self, index):
        return await self.registry.call('inspect_quality', agent_id='INSPECTOR',
                                        job_id=JOBS[index][1])

    async def dispatch(self, index, **arguments):
        return await self.registry.call('dispatch_rework', agent_id='DISPATCHER',
                                        job_id=JOBS[index][1], **arguments)

    async def test_later_jobs_cannot_authorize_earlier_inspections(self):
        for index in range(6):
            self.assertFalse((await self.inspect(index)).success)
        for index in range(6):
            await self.job(index)
        self.assertFalse(self.sim.is_complete())
        for index in reversed(range(6)):
            self.assertTrue((await self.inspect(index)).success)
        self.assertTrue(self.sim.is_complete())

    def test_actual_jobs_name_their_station_and_optional_tool(self):
        for worker, job, station, tool in JOBS:
            expected = [station] + (['TOOL_CABINET'] if tool else [])
            self.assertEqual(self.sim.resource_requirements(
                'do_job', agent_id=worker, job_id=job), expected)
        self.assertEqual(self.sim.resource_requirements(
            'inspect_quality', agent_id='INSPECTOR', job_id=JOBS[0][1]), ['INSPECTOR'])

    async def test_all_job_orders_and_all_inspection_orders_remain_free(self):
        for order in itertools.permutations(range(6)):
            self.setUp()
            for index in order:
                self.assertTrue((await self.job(index)).success)
                self.assertTrue((await self.inspect(index)).success)
            self.assertTrue(self.sim.is_complete(), order)
            self.assertEqual(self.sim.violations, [])

            self.setUp()
            for index in range(6):
                await self.job(index)
            for index in order:
                self.assertTrue((await self.inspect(index)).success)
            self.assertTrue(self.sim.is_complete(), order)
            self.assertEqual(self.sim.violations, [])

    async def test_two_tools_allow_independent_work_but_three_is_an_observed_conflict(self):
        for indices, conflicts in (((0, 2, 3), 0), ((0, 2, 4), 1)):
            self.setUp()
            self.sim.tool_delay = lambda *args, **kwargs: (0.02, 0.02)
            self.sim._delay_multiplier = 1
            results = await asyncio.gather(*(self.job(i) for i in indices))
            self.assertTrue(all(result.success for result in results))
            self.assertLess(max(e.timestamp for e in self.sim.events),
                            min(e.completed_at for e in self.sim.events))
            violations = [v for v in self.sim.violations
                          if v.violation_type == 'concurrent_resource_use']
            self.assertEqual(len(violations), conflicts)
            if violations:
                self.assertIn('TOOL_CABINET', violations[0].message)
            self.assertEqual(self.sim.progress['available_tools'], 2)

    async def test_station_and_inspector_conflicts_are_observed_without_serializing_calls(self):
        for pair in ((0, 5), (1, 2), (3, 4)):
            self.setUp()
            self.sim.tool_delay = lambda *args, **kwargs: (0.02, 0.02)
            self.sim._delay_multiplier = 1
            await asyncio.gather(*(self.job(i) for i in pair))
            self.assertEqual(len(self.sim.violations), 1)
            self.assertIn(JOBS[pair[0]][2], self.sim.violations[0].message)
        self.setUp()
        await self.job(0)
        await self.job(3)
        self.sim.tool_delay = lambda *args, **kwargs: (0.02, 0.02)
        self.sim._delay_multiplier = 1
        await asyncio.gather(self.inspect(0), self.inspect(3))
        self.assertEqual(len(self.sim.violations), 1)
        self.assertIn('INSPECTOR', self.sim.violations[0].message)

    async def test_each_failed_part_requires_actual_dispatched_rework(self):
        for index in range(6):
            for count in (1, 2, 3):
                self.setUp()
                self.sim.set_scenario_depth(count)
                part = await self.job(index)
                for _ in range(count):
                    rejected = await self.inspect(index)
                    self.assertFalse(rejected.success)
                    self.assertEqual(rejected.data['input_part'], part.data['part'])
                    self.assertFalse((await self.inspect(index)).success,
                                     'rechecking a rejected physical part is not rework')
                    order = await self.dispatch(index, reason='measured defect')
                    self.assertTrue(order.success)
                    self.assertEqual(order.data['inspection'], rejected.data['receipt'])
                    replaced = await self.job(index)
                    self.assertNotEqual(replaced.data['part'], part.data['part'])
                    self.assertEqual(replaced.data['rework_order'], order.data['order'])
                    part = replaced
                approved = await self.inspect(index)
                self.assertTrue(approved.success)
                self.assertEqual(approved.data['input_part'], part.data['part'])
                self.assertEqual(self.sim.violations, [])

    async def test_missing_or_old_work_orders_remain_violations_after_later_repair(self):
        await self.job(0)
        self.sim._failure_specs = [FailureSpec('inspect_quality', call_index=0),
                                   FailureSpec('inspect_quality', call_index=1)]
        first_failure = await self.inspect(0)
        first_order = await self.dispatch(0)
        self.assertEqual(first_order.data['inspection'], first_failure.data['receipt'])
        await self.job(0)
        second_failure = await self.inspect(0)
        self.assertFalse(second_failure.success)
        self.assertNotEqual(second_failure.data['receipt'], first_failure.data['receipt'])
        await self.job(0)  # Free physical action; the missing current work order is scored.
        self.assertEqual(len(self.sim.violations), 1)
        self.assertEqual(self.sim.violations[0].violation_type, 'missing_rework_order')
        await self.inspect(0)
        for index in range(1, 6):
            await self.job(index)
            await self.inspect(index)
        self.assertEqual(len(self.sim.violations), 1,
                         'later successful material cannot erase an earlier unsafe effect')

    async def test_new_and_pending_material_cannot_keep_old_quality_evidence(self):
        for index in range(6):
            await self.job(index)
            await self.inspect(index)
        self.assertTrue(self.sim.is_complete())
        self.sim.tool_delay = lambda tool, **kwargs: (0.02, 0.02) if tool == 'do_job' else (0, 0)
        self.sim._delay_multiplier = 1
        pending = asyncio.create_task(self.job(0))
        await asyncio.sleep(0)
        try:
            self.assertFalse(self.sim.is_complete())
            self.assertFalse((await self.inspect(0)).success)
        finally:
            await pending
        self.assertFalse(self.sim.is_complete())
        self.assertTrue((await self.inspect(0)).success)
        self.assertTrue(self.sim.is_complete())

    async def test_pending_inspection_and_dispatch_are_not_completed_receipts(self):
        await self.job(0)
        self.sim._failure_specs = [FailureSpec('inspect_quality', call_index=0)]
        self.sim._delay_multiplier = 1
        self.sim.tool_delay = lambda tool, **kwargs: (0.02, 0.02) if tool in (
            'inspect_quality', 'dispatch_rework') else (0, 0)
        pending = asyncio.create_task(self.inspect(0))
        await asyncio.sleep(0)
        try:
            self.assertFalse((await self.dispatch(0)).success)
        finally:
            await pending
        order = asyncio.create_task(self.dispatch(0))
        await asyncio.sleep(0)
        try:
            await self.job(0)
            self.assertEqual(len(self.sim.violations), 1)
            self.assertEqual(self.sim.violations[0].violation_type, 'missing_rework_order')
        finally:
            await order

    async def test_pending_positive_inspection_is_not_a_completed_approval(self):
        for index in range(6):
            await self.job(index)
            if index != 5:
                await self.inspect(index)
        self.sim._delay_multiplier = 1
        self.sim.tool_delay = lambda *args, **kwargs: (0.02, 0.02)
        for _ in range(2):
            pending = asyncio.create_task(self.inspect(5))
            await asyncio.sleep(0)
            try:
                self.assertFalse(self.sim.is_complete())
            finally:
                await pending
            self.assertTrue(self.sim.is_complete())

    async def test_wrong_jobs_create_no_parts_and_returned_values_cannot_change_the_world(self):
        self.assertFalse((await self.registry.call(
            'do_job', agent_id='WORKER_A', job_id='unknown')).success)
        self.assertFalse((await self.registry.call(
            'do_job', agent_id='WORKER_A', job_id=JOBS[2][1])).success)
        self.assertFalse((await self.dispatch(0)).success)
        part = await self.job(0)
        original = dict(part.data['part'])
        part.data['part']['job_id'] = 'forged'
        inspection = await self.inspect(0)
        self.assertEqual(inspection.data['input_part'], original)
        inspection.data['input_part']['job_id'] = 'also forged'
        self.sim.progress['jobs'][JOBS[0][1]]['job_id'] = 'again forged'
        self.assertEqual(self.sim.progress['jobs'][JOBS[0][1]], original)

