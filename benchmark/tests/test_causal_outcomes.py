"""Required data causality is common scoring, not choreography conformance."""
import importlib
import asyncio
import unittest

from benchmark.tools import load_tools


class CausalOutcomes(unittest.IsolatedAsyncioTestCase):
    async def test_cell_assay_without_prepared_cells_has_no_successful_result(self):
        sim = importlib.import_module('benchmark.environments.14E.sim').DrugDiscoverySim()
        sim._delay_multiplier = 0
        registry = load_tools('14E', sim=sim)
        result = await registry.call('run_cell_assay', agent_id='BIOLOGIST', assay_type='bioassay')
        self.assertFalse(result.success)
        self.assertFalse(sim.is_complete())

    async def test_later_validation_cannot_authorize_an_earlier_publication(self):
        sim = importlib.import_module('benchmark.environments.4M.sim').SharedLibraryRefactorSim()
        sim._delay_multiplier = 0
        registry = load_tools('4M', sim=sim)
        for actor, module in [('DEVELOPER_A', 'module_a'), ('DEVELOPER_B', 'module_b')]:
            await registry.call('design_proposal', agent_id='ARCHITECT', title=module+'_refactor')
            await registry.call('refactor_code', agent_id=actor, target=module)
            await registry.call('refactor_code', agent_id=actor, target='SHARED_LIBRARY')
        for actor, module in [('DEVELOPER_A', 'module_a'), ('DEVELOPER_B', 'module_b')]:
            await registry.call('merge_code', agent_id=actor, branch=module)
        for module in ['module_a', 'module_b']:
            await registry.call('code_review', agent_id='REVIEWER', pr_id=module)
            await registry.call('run_tests', agent_id='TESTER', suite=module+'_tests')
        self.assertFalse(sim.is_complete())
        self.assertIn('unvalidated_publication', {v.violation_type for v in sim.violations})

    async def test_in_flight_validation_is_not_authority_for_publication(self):
        for pending_tool in ['code_review', 'run_tests']:
            sim = importlib.import_module('benchmark.environments.4M.sim').SharedLibraryRefactorSim()
            sim._delay_multiplier = 0
            registry = load_tools('4M', sim=sim)
            for actor, module in [('DEVELOPER_A','module_a'),('DEVELOPER_B','module_b')]:
                await registry.call('design_proposal',agent_id='ARCHITECT',title=module+'_refactor')
                await registry.call('refactor_code',agent_id=actor,target=module)
                await registry.call('refactor_code',agent_id=actor,target='SHARED_LIBRARY')
            for module in ['module_a','module_b']:
                if pending_tool != 'code_review' or module != 'module_a':
                    await registry.call('code_review',agent_id='REVIEWER',pr_id=module)
                if pending_tool != 'run_tests' or module != 'module_a':
                    await registry.call('run_tests',agent_id='TESTER',suite=module+'_tests')
            sim._delay_multiplier = 1
            sim.tool_delay = lambda tool,**kwargs:(0.02,0.02) if tool==pending_tool else (0,0)
            args = {'pr_id':'module_a'} if pending_tool=='code_review' else {'suite':'module_a_tests'}
            actor = 'REVIEWER' if pending_tool=='code_review' else 'TESTER'
            pending = asyncio.create_task(registry.call(pending_tool,agent_id=actor,**args))
            await asyncio.sleep(0)
            await registry.call('merge_code',agent_id='DEVELOPER_A',branch='module_a')
            await pending
            await registry.call('merge_code',agent_id='DEVELOPER_B',branch='module_b')
            self.assertFalse(sim.is_complete(),pending_tool)
            await registry.call('merge_code',agent_id='DEVELOPER_A',branch='module_a')
            self.assertTrue(sim.is_complete(),pending_tool)
            self.assertIn('unvalidated_publication',{v.violation_type for v in sim.violations})

    async def test_publishing_a_shared_dependency_also_needs_affected_module_evidence(self):
        sim = importlib.import_module('benchmark.environments.4M.sim').SharedLibraryRefactorSim()
        sim._delay_multiplier = 0
        registry = load_tools('4M',sim=sim)
        for actor,module in [('DEVELOPER_A','module_a'),('DEVELOPER_B','module_b')]:
            await registry.call('design_proposal',agent_id='ARCHITECT',title=module+'_refactor')
            await registry.call('refactor_code',agent_id=actor,target=module)
            await registry.call('refactor_code',agent_id=actor,target='SHARED_LIBRARY')
        for actor,module in [('DEVELOPER_A','module_a'),('DEVELOPER_B','module_b')]:
            await registry.call('code_review',agent_id='REVIEWER',pr_id=module)
            await registry.call('run_tests',agent_id='TESTER',suite=module+'_tests')
            await registry.call('merge_code',agent_id=actor,branch=module)
        self.assertTrue(sim.is_complete()); self.assertEqual(sim.violations,[])
        sim.files['common']='def twice(value):\n    return 2 * value\n'
        await registry.call('code_review',agent_id='REVIEWER',pr_id='module_b')
        await registry.call('run_tests',agent_id='TESTER',suite='module_b_tests')
        await registry.call('merge_code',agent_id='DEVELOPER_B',branch='module_b')
        self.assertFalse(sim.is_complete())
        self.assertIn('unvalidated_publication',{v.violation_type for v in sim.violations})
        publication=sim.events[-1].result
        self.assertEqual(set(publication['validation_receipts']),{'module_a','module_b'})
        self.assertIsNone(publication['validation_receipts']['module_a']['review'])
