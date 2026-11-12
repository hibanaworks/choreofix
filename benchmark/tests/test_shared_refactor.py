"""Method-neutral world tests: effects are free; outputs and receipts are factual."""
import asyncio
import importlib
import itertools
import unittest
from benchmark.tools import load_tools


class SharedLibraryWorld(unittest.IsolatedAsyncioTestCase):
    def setUp(self):
        self.sim = importlib.import_module('benchmark.environments.4M.sim').SharedLibraryRefactorSim()
        self.sim._delay_multiplier = 0
        self.registry = load_tools('4M', sim=self.sim)

    async def call(self, agent, tool, **arguments):
        return await self.registry.call(tool, agent_id=agent, **arguments)

    async def changes(self):
        for agent, own in [('DEVELOPER_A', 'module_a'), ('DEVELOPER_B', 'module_b')]:
            for target in (own, 'SHARED_LIBRARY'):
                self.assertTrue((await self.call(agent, 'refactor_code', target=target)).success)

    async def designs(self):
        for module in ('module_a', 'module_b'):
            await self.call('ARCHITECT', 'design_proposal', title=module+'_refactor')

    async def reviews(self):
        for module in ('module_a', 'module_b'):
            await self.call('REVIEWER', 'code_review', pr_id=module)

    async def suites(self):
        for module in ('module_a', 'module_b'):
            await self.call('TESTER', 'run_tests', suite=module+'_tests')

    async def merges(self):
        for agent, module in [('DEVELOPER_A', 'module_a'), ('DEVELOPER_B', 'module_b')]:
            await self.call(agent, 'merge_code', branch=module)

    async def test_all_actions_accept_calls_without_a_workflow_gate(self):
        for agent, tool, args in [
            ('DEVELOPER_A','merge_code',{'branch':'module_a'}),
            ('REVIEWER','code_review',{'pr_id':'module_b'}),
            ('TESTER','run_tests',{'suite':'module_a_tests'}),
            ('DEVELOPER_B','refactor_code',{'target':'SHARED_LIBRARY'}),
            ('ARCHITECT','design_proposal',{'title':'module_b_refactor'})]:
            self.assertTrue((await self.call(agent,tool,**args)).success)
        self.assertFalse(self.sim.is_complete())
        self.assertIn('unvalidated_publication', {v.violation_type for v in self.sim.violations})

    async def test_reviews_and_tests_may_reorder_but_publication_requires_both(self):
        operations = (
            ('REVIEWER', 'code_review', {'pr_id': 'module_a'}),
            ('TESTER', 'run_tests', {'suite': 'module_a_tests'}),
            ('DEVELOPER_A', 'merge_code', {'branch': 'module_a'}),
            ('REVIEWER', 'code_review', {'pr_id': 'module_b'}),
            ('TESTER', 'run_tests', {'suite': 'module_b_tests'}),
            ('DEVELOPER_B', 'merge_code', {'branch': 'module_b'}),
        )
        valid_orders = 0
        for order in itertools.permutations(range(len(operations))):
            self.setUp()
            await self.changes()
            for index in order:
                actor, tool, arguments = operations[index]
                await self.call(actor, tool, **arguments)
            await self.designs()
            # Only each publication's own two prerequisites constrain order.
            # The other module may be reviewed, tested or published anywhere.
            valid = all(order.index(before) < order.index(after)
                        for before, after in ((0, 2), (1, 2), (3, 5), (4, 5)))
            if valid:
                valid_orders += 1
                self.assertTrue(self.sim.is_complete(), order)
                self.assertEqual(self.sim.violations, [])
            else:
                self.assertFalse(self.sim.is_complete(), order)
                self.assertIn('unvalidated_publication', {v.violation_type for v in self.sim.violations})
        self.assertEqual(valid_orders, 80)

    async def test_repair_can_complete_but_cannot_erase_an_unsafe_publication(self):
        await self.merges(); await self.reviews(); await self.suites()
        await self.changes(); await self.designs()
        self.assertFalse(self.sim.is_complete())
        await self.suites(); await self.reviews(); await self.merges()
        self.assertTrue(self.sim.is_complete())
        self.assertIn('unvalidated_publication', {v.violation_type for v in self.sim.violations})

    async def test_receipts_for_different_bytes_do_not_cover_new_artifacts(self):
        await self.reviews(); await self.suites(); await self.changes()
        await self.merges(); await self.designs()
        self.assertFalse(self.sim.is_complete())
        await self.suites()
        await self.call('REVIEWER', 'code_review', pr_id='module_b')
        self.sim.tool_delay = lambda *args, **kwargs: (0.02, 0.02)
        self.sim._delay_multiplier = 1
        pending = asyncio.create_task(self.registry.call(
            'code_review', agent_id='REVIEWER', pr_id='module_a'))
        await asyncio.sleep(0)
        identity = self.sim._identity(self.sim.published, 'module_a')
        self.assertFalse(any(r['candidate_sha256'] == identity for r in self.sim.reviews))
        self.assertFalse(self.sim.is_complete())
        await pending
        self.assertFalse(self.sim.is_complete())
        await self.merges()
        self.assertTrue(self.sim.is_complete())

    async def test_test_tool_executes_actual_code_and_detects_a_wrong_result(self):
        self.sim.files['module_a'] = 'def compute(value):\n    return 0\n'
        result = await self.call('TESTER','run_tests',suite='module_a_tests')
        self.assertFalse(result.success)
        self.assertNotEqual(result.data['outputs'][0]['actual'], result.data['outputs'][0]['expected'])

    async def test_same_content_does_not_become_stale_due_to_an_extra_call(self):
        await self.changes(); await self.reviews(); await self.suites()
        before = list(self.sim.reviews)
        await self.call('DEVELOPER_B','refactor_code',target='SHARED_LIBRARY')
        await self.merges(); await self.designs()
        self.assertEqual(before,self.sim.reviews)
        self.assertTrue(self.sim.is_complete())

    async def test_unmerged_experiments_do_not_change_the_published_outcome(self):
        await self.changes(); await self.reviews(); await self.suites()
        await self.merges(); await self.designs()
        self.sim.files['module_a'] = 'def compute(value):\n    return -1\n'
        self.assertTrue(self.sim.is_complete())
        self.sim.designs.clear()
        self.assertFalse(self.sim.is_complete())

    async def test_physical_resource_conflicts_are_observed_without_serializing_calls(self):
        self.sim.tool_delay = lambda *args, **kwargs: (0.02,0.02)
        self.sim._delay_multiplier = 1
        results = await asyncio.gather(
            self.registry.call('refactor_code',agent_id='DEVELOPER_A',target='module_a'),
            self.registry.call('refactor_code',agent_id='DEVELOPER_B',target='module_b'))
        self.assertTrue(all(result.success for result in results))
        self.assertIn('concurrent_resource_use',{v.violation_type for v in self.sim.violations})
