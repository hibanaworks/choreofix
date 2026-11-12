"""11E world behavior: machine use and actual report contents, never a workflow."""
import asyncio
import importlib
import itertools
import unittest

from benchmark.tools import load_tools


JOBS = (
    ('TECHNICIAN_A', 'lathe_job_a', 'LATHE'),
    ('TECHNICIAN_A', 'mill_job_a', 'MILL'),
    ('TECHNICIAN_B', 'lathe_job_b', 'LATHE'),
    ('TECHNICIAN_B', 'mill_job_b', 'MILL'),
)


class WorkshopWorld(unittest.IsolatedAsyncioTestCase):
    def setUp(self):
        self.sim = importlib.import_module('benchmark.environments.11E.sim').TwoTechnicianWorkshopSim()
        self.sim._delay_multiplier = 0
        self.registry = load_tools('11E', sim=self.sim)

    async def job(self, index):
        agent, job, _ = JOBS[index]
        return await self.registry.call('do_job', agent_id=agent, job_id=job)

    async def report(self):
        return await self.registry.call('collect_reports', agent_id='SUPERVISOR')

    def test_each_known_job_uses_exactly_its_own_machine(self):
        for agent, job, machine in JOBS:
            self.assertEqual(self.sim.resource_requirements(
                'do_job', agent_id=agent, job_id=job), [machine])

    async def test_distinct_machines_really_overlap_without_a_false_conflict(self):
        for pair in ((0, 3), (1, 2)):
            self.setUp()
            self.sim.tool_delay = lambda *args, **kwargs: (0.02, 0.02)
            self.sim._delay_multiplier = 1
            results = await asyncio.gather(*(self.job(index) for index in pair))
            self.assertTrue(all(result.success for result in results))
            self.assertLess(max(e.timestamp for e in self.sim.events),
                            min(e.completed_at for e in self.sim.events))
            self.assertEqual(self.sim.violations, [])

    async def test_same_machine_conflict_is_observed_without_serializing_the_calls(self):
        self.sim.tool_delay = lambda *args, **kwargs: (0.02, 0.02)
        self.sim._delay_multiplier = 1
        results = await asyncio.gather(self.job(0), self.job(2))
        self.assertTrue(all(result.success for result in results))
        conflicts = [v for v in self.sim.violations if v.violation_type == 'concurrent_resource_use']
        self.assertEqual(len(conflicts), 1)
        self.assertIn('LATHE', conflicts[0].message)

    async def test_one_actor_cannot_hide_two_overlapping_uses_of_one_machine(self):
        self.sim.tool_delay = lambda *args, **kwargs: (0.02, 0.02)
        self.sim._delay_multiplier = 1
        results = await asyncio.gather(self.job(0), self.job(0))
        self.assertTrue(all(result.success for result in results))
        conflicts = [v for v in self.sim.violations if v.violation_type == 'concurrent_resource_use']
        self.assertEqual(len(conflicts), 1)
        self.assertIn('LATHE', conflicts[0].message)
        await self.job(0)
        self.assertEqual(len(self.sim.violations), 1)

    async def test_one_actor_may_use_two_distinct_machines_concurrently(self):
        self.sim.tool_delay = lambda *args, **kwargs: (0.02, 0.02)
        self.sim._delay_multiplier = 1
        results = await asyncio.gather(self.job(0), self.job(1))
        self.assertTrue(all(result.success for result in results))
        self.assertEqual(self.sim.violations, [])

    async def test_all_twenty_four_job_orders_produce_the_same_valid_report(self):
        for order in itertools.permutations(range(4)):
            self.setUp()
            for index in order:
                self.assertTrue((await self.job(index)).success)
            report = await self.report()
            self.assertTrue(report.success)
            self.assertEqual({(j['agent'], j['job_id']) for j in report.data['jobs']},
                             {(agent, job) for agent, job, _ in JOBS})
            self.assertTrue(self.sim.is_complete(), order)
            self.assertEqual(self.sim.violations, [])

    async def test_early_report_is_allowed_and_can_be_replaced_with_actual_results(self):
        early = await self.report()
        self.assertTrue(early.success)
        self.assertEqual(early.data['jobs'], [])
        for index in range(4):
            await self.job(index)
        self.assertFalse(self.sim.is_complete())
        self.assertTrue((await self.report()).success)
        self.assertTrue(self.sim.is_complete())

    async def test_a_report_does_not_include_an_inflight_job(self):
        for index in range(3):
            await self.job(index)
        self.sim.tool_delay = lambda tool, **kwargs: (0.02, 0.02) if tool == 'do_job' else (0, 0)
        self.sim._delay_multiplier = 1
        pending = asyncio.create_task(self.job(3))
        await asyncio.sleep(0)
        report = await self.report()
        self.assertEqual(len(report.data['jobs']), 3)
        await pending
        self.assertFalse(self.sim.is_complete())
        await self.report()
        self.assertTrue(self.sim.is_complete())

    async def test_unknown_job_creates_no_completed_part(self):
        result = await self.registry.call('do_job', agent_id='TECHNICIAN_A', job_id='unknown')
        self.assertFalse(result.success)
        self.assertFalse(self.sim.is_complete())

    async def test_a_failed_operation_does_not_leave_its_machine_occupied(self):
        def unavailable(*args, **kwargs):
            raise OSError('workstation I/O failed')

        self.registry._tool_fns['do_job'] = unavailable
        with self.assertRaises(OSError):
            await self.job(0)
        self.registry._tool_fns['do_job'] = self.sim.do_job
        self.assertTrue((await self.job(2)).success)
        self.assertEqual(self.sim.violations, [])
