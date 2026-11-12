"""Common instrument occupancy, independent of a participant's workflow."""
import asyncio
import importlib
import itertools
import unittest

from benchmark.tools import load_tools


class InstrumentWorld(unittest.IsolatedAsyncioTestCase):
    def setUp(self):
        self.sim = importlib.import_module('benchmark.environments.14E.sim').DrugDiscoverySim()
        self.sim._delay_multiplier = 0.01
        self.registry = load_tools('14E', sim=self.sim)

    async def test_same_instrument_conflict_is_observable_at_nonzero_latency(self):
        self.sim._delay_multiplier = 0
        await self.registry.call('run_hplc_analysis', agent_id='CHEMIST', sample='synthesis_qc')
        await self.registry.call('run_mass_spec_analysis', agent_id='CHEMIST', sample='molecular_id')
        self.sim._delay_multiplier = 0.01
        results = await asyncio.gather(
            self.registry.call('run_hplc_analysis', agent_id='TOXICOLOGIST', sample='purity_check'),
            self.registry.call('run_hplc_analysis', agent_id='LEAD_SCIENTIST', sample='structural_confirmation'))
        self.assertTrue(all(result.success for result in results))
        self.assertTrue(all(result.delay_seconds > 0 for result in results))
        self.assertIn('concurrent_resource_use', {v.violation_type for v in self.sim.violations})

    async def test_distinct_instruments_can_run_independently(self):
        results = await asyncio.gather(
            self.registry.call('run_hplc_analysis', agent_id='CHEMIST', sample='synthesis_qc'),
            self.registry.call('run_mass_spec_analysis', agent_id='TOXICOLOGIST', sample='baseline_markers'))
        self.assertTrue(all(result.success for result in results))
        self.assertLess(max(event.timestamp for event in self.sim.events),
                        min(event.completed_at for event in self.sim.events))
        self.assertEqual(self.sim.violations, [])

    async def test_reversing_all_analyses_is_not_a_valid_causal_execution(self):
        module = importlib.import_module('benchmark.environments.14E.sim')
        calls = [*(('run_hplc_analysis', s.scientist_id, {'sample': s.sample})
                   for s in module._HPLC_STEPS),
                 *(('run_mass_spec_analysis', s.scientist_id, {'sample': s.sample})
                   for s in module._MASS_SPEC_STEPS),
                 *(('run_cell_assay', s.scientist_id, {'assay_type': s.assay_type})
                   for s in module._CELL_ASSAY_STEPS)]
        self.sim._delay_multiplier = 0
        results = [await self.registry.call(tool, agent_id=actor, **arguments)
                   for tool, actor, arguments in reversed(calls)]
        self.assertTrue(any(not result.success for result in results))
        self.assertFalse(self.sim.is_complete())

    async def test_independent_orders_and_both_decisions_preserve_the_causal_outcome(self):
        roots = [
            ('run_hplc_analysis', 'CHEMIST', {'sample':'synthesis_qc'}),
            ('run_cell_assay', 'BIOLOGIST', {'assay_type':'culture_prep'}),
            ('run_mass_spec_analysis', 'TOXICOLOGIST', {'sample':'baseline_markers'}),
            ('run_mass_spec_analysis', 'LEAD_SCIENTIST', {'sample':'reference_methods'})]
        studies = [
            ('run_cell_assay', 'BIOLOGIST', {'assay_type':'bioassay'}),
            ('run_hplc_analysis', 'TOXICOLOGIST', {'sample':'purity_check'}),
            ('run_hplc_analysis', 'LEAD_SCIENTIST', {'sample':'structural_confirmation'}),
            ('run_mass_spec_analysis', 'LEAD_SCIENTIST', {'sample':'characterization'})]
        finishes = [
            ('run_hplc_analysis', 'BIOLOGIST', {'sample':'stability'}),
            ('run_cell_assay', 'TOXICOLOGIST', {'assay_type':'cytotoxicity'})]
        # A finite independence catalog, not all topological orders of the task.
        for first in itertools.permutations(roots):
            for middle in itertools.permutations(studies):
                for last in itertools.permutations(finishes):
                    for go in (True,False):
                        self.setUp(); self.sim._delay_multiplier = 0
                        calls = [*first, ('run_mass_spec_analysis','CHEMIST',{'sample':'molecular_id'}),
                                 *middle, *last, ('submit_decision','LEAD_SCIENTIST',{'go':go}),
                                 ('run_hplc_analysis','CHEMIST',{'sample':'documentation'})]
                        for tool, actor, arguments in calls:
                            result = await self.registry.call(tool,agent_id=actor,**arguments)
                            self.assertTrue(result.success,(tool,actor,result))
                        self.assertTrue(self.sim.is_complete())
                        self.assertEqual(self.sim.violations,[])

    async def test_preparation_in_flight_is_not_a_completed_input(self):
        self.sim._delay_multiplier=0
        await self.registry.call('run_hplc_analysis',agent_id='CHEMIST',sample='synthesis_qc')
        await self.registry.call('run_mass_spec_analysis',agent_id='CHEMIST',sample='molecular_id')
        self.sim._delay_multiplier=1
        self.sim.tool_delay=lambda *args,**kwargs:(0.02,0.02)
        prep=asyncio.create_task(self.registry.call('run_cell_assay',agent_id='BIOLOGIST',assay_type='culture_prep'))
        await asyncio.sleep(0)
        early=await self.registry.call('run_cell_assay',agent_id='BIOLOGIST',assay_type='bioassay')
        self.assertFalse(early.success)
        await prep
        self.assertTrue((await self.registry.call('run_cell_assay',agent_id='BIOLOGIST',assay_type='bioassay')).success)

    async def test_stability_can_overlap_incubation_after_actual_preparation(self):
        self.sim._delay_multiplier=0
        for tool,arguments in [('run_hplc_analysis',{'sample':'synthesis_qc'}),('run_mass_spec_analysis',{'sample':'molecular_id'})]:
            await self.registry.call(tool,agent_id='CHEMIST',**arguments)
        await self.registry.call('run_cell_assay',agent_id='BIOLOGIST',assay_type='culture_prep')
        self.sim._delay_multiplier=1
        self.sim.tool_delay=lambda *args,**kwargs:(0.02,0.02)
        assay=asyncio.create_task(self.registry.call('run_cell_assay',agent_id='BIOLOGIST',assay_type='bioassay'))
        await asyncio.sleep(0)
        stability=await self.registry.call('run_hplc_analysis',agent_id='BIOLOGIST',sample='stability')
        self.assertTrue(stability.success)
        await assay
        self.assertEqual(self.sim.violations,[])

    async def test_old_identity_does_not_identify_a_new_compound(self):
        self.sim._delay_multiplier=0
        await self.registry.call('run_hplc_analysis',agent_id='CHEMIST',sample='synthesis_qc')
        await self.registry.call('run_mass_spec_analysis',agent_id='CHEMIST',sample='molecular_id')
        await self.registry.call('run_hplc_analysis',agent_id='CHEMIST',sample='synthesis_qc')
        early=await self.registry.call('run_hplc_analysis',agent_id='TOXICOLOGIST',sample='purity_check')
        self.assertFalse(early.success)
        await self.registry.call('run_mass_spec_analysis',agent_id='CHEMIST',sample='molecular_id')
        self.assertTrue((await self.registry.call('run_hplc_analysis',agent_id='TOXICOLOGIST',sample='purity_check')).success)

    async def test_completed_decision_must_precede_documentation_and_failed_attempt_can_recover(self):
        self.sim._delay_multiplier=0
        calls=[('run_hplc_analysis','CHEMIST',{'sample':'synthesis_qc'}),
               ('run_cell_assay','BIOLOGIST',{'assay_type':'culture_prep'}),
               ('run_mass_spec_analysis','TOXICOLOGIST',{'sample':'baseline_markers'}),
               ('run_mass_spec_analysis','LEAD_SCIENTIST',{'sample':'reference_methods'}),
               ('run_mass_spec_analysis','CHEMIST',{'sample':'molecular_id'}),
               ('run_cell_assay','BIOLOGIST',{'assay_type':'bioassay'}),
               ('run_hplc_analysis','BIOLOGIST',{'sample':'stability'}),
               ('run_hplc_analysis','TOXICOLOGIST',{'sample':'purity_check'}),
               ('run_cell_assay','TOXICOLOGIST',{'assay_type':'cytotoxicity'}),
               ('run_hplc_analysis','LEAD_SCIENTIST',{'sample':'structural_confirmation'}),
               ('run_mass_spec_analysis','LEAD_SCIENTIST',{'sample':'characterization'})]
        for tool,actor,args in calls:
            self.assertTrue((await self.registry.call(tool,agent_id=actor,**args)).success)
        early=await self.registry.call('run_hplc_analysis',agent_id='CHEMIST',sample='documentation')
        self.assertFalse(early.success)
        decision=await self.registry.call('submit_decision',agent_id='LEAD_SCIENTIST',go=False)
        self.assertTrue(decision.success); self.assertFalse(self.sim.is_complete())
        document=await self.registry.call('run_hplc_analysis',agent_id='CHEMIST',sample='documentation')
        self.assertTrue(document.success); self.assertTrue(self.sim.is_complete())
        self.assertEqual(document.data['inputs']['decision'],decision.data['artifact_id'])
        self.assertEqual(self.sim.violations,[])

    async def test_each_missing_analysis_prevents_a_decision_without_poisoning_recovery(self):
        calls=[('run_hplc_analysis','CHEMIST',{'sample':'synthesis_qc'}),
               ('run_cell_assay','BIOLOGIST',{'assay_type':'culture_prep'}),
               ('run_mass_spec_analysis','TOXICOLOGIST',{'sample':'baseline_markers'}),
               ('run_mass_spec_analysis','LEAD_SCIENTIST',{'sample':'reference_methods'}),
               ('run_mass_spec_analysis','CHEMIST',{'sample':'molecular_id'}),
               ('run_cell_assay','BIOLOGIST',{'assay_type':'bioassay'}),
               ('run_hplc_analysis','BIOLOGIST',{'sample':'stability'}),
               ('run_hplc_analysis','TOXICOLOGIST',{'sample':'purity_check'}),
               ('run_cell_assay','TOXICOLOGIST',{'assay_type':'cytotoxicity'}),
               ('run_hplc_analysis','LEAD_SCIENTIST',{'sample':'structural_confirmation'}),
               ('run_mass_spec_analysis','LEAD_SCIENTIST',{'sample':'characterization'})]
        for missing in range(len(calls)):
            self.setUp();self.sim._delay_multiplier=0
            for index,(tool,actor,args) in enumerate(calls):
                if index != missing:
                    await self.registry.call(tool,agent_id=actor,**args)
            self.assertFalse((await self.registry.call('submit_decision',agent_id='LEAD_SCIENTIST',go=True)).success,calls[missing])
            self.assertFalse((await self.registry.call('run_hplc_analysis',agent_id='CHEMIST',sample='documentation')).success)
            for tool,actor,args in calls:
                self.assertTrue((await self.registry.call(tool,agent_id=actor,**args)).success)
            self.assertTrue((await self.registry.call('submit_decision',agent_id='LEAD_SCIENTIST',go=True)).success)
            await self.registry.call('run_hplc_analysis',agent_id='CHEMIST',sample='documentation')
            self.assertTrue(self.sim.is_complete());self.assertEqual(self.sim.violations,[])

    async def test_in_flight_assay_cannot_supply_a_completed_measurement_to_a_decision(self):
        self.sim._delay_multiplier=0
        for tool,actor,args in [
            ('run_hplc_analysis','CHEMIST',{'sample':'synthesis_qc'}),
            ('run_mass_spec_analysis','CHEMIST',{'sample':'molecular_id'}),
            ('run_cell_assay','BIOLOGIST',{'assay_type':'culture_prep'}),
            ('run_mass_spec_analysis','TOXICOLOGIST',{'sample':'baseline_markers'}),
            ('run_hplc_analysis','TOXICOLOGIST',{'sample':'purity_check'}),
            ('run_cell_assay','TOXICOLOGIST',{'assay_type':'cytotoxicity'}),
            ('run_mass_spec_analysis','LEAD_SCIENTIST',{'sample':'reference_methods'}),
            ('run_hplc_analysis','LEAD_SCIENTIST',{'sample':'structural_confirmation'}),
            ('run_mass_spec_analysis','LEAD_SCIENTIST',{'sample':'characterization'})]:
            self.assertTrue((await self.registry.call(tool,agent_id=actor,**args)).success)
        self.sim._delay_multiplier=1
        self.sim.tool_delay=lambda tool,**kwargs:(0.03,0.03) if tool=='run_cell_assay' else (0,0)
        pending=asyncio.create_task(self.registry.call('run_cell_assay',agent_id='BIOLOGIST',assay_type='bioassay'))
        await asyncio.sleep(0)
        self.assertTrue((await self.registry.call('run_hplc_analysis',agent_id='BIOLOGIST',sample='stability')).success)
        self.assertFalse((await self.registry.call('submit_decision',agent_id='LEAD_SCIENTIST',go=True)).success)
        await pending
        self.assertTrue((await self.registry.call('submit_decision',agent_id='LEAD_SCIENTIST',go=True)).success)
        self.assertEqual(self.sim.violations,[])
