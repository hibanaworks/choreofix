"""Actual analysis/decision receipts, independent of any agent implementation."""
import asyncio
import importlib
import itertools
import unittest

from benchmark.tools import load_tools
from benchmark.tools.sim_base import FailureSpec


ANALYSES = [
    ('CHEMIST', 'run_hplc_analysis', {'sample': 'synthesis_qc'}),
    ('BIOLOGIST', 'run_cell_assay', {'assay_type': 'culture_prep'}),
    ('TOXICOLOGIST', 'run_mass_spec_analysis', {'sample': 'baseline_markers'}),
    ('LEAD_SCIENTIST', 'run_mass_spec_analysis', {'sample': 'reference_methods'}),
    ('CHEMIST', 'run_mass_spec_analysis', {'sample': 'molecular_id'}),
    ('BIOLOGIST', 'run_cell_assay', {'assay_type': 'bioassay'}),
    ('BIOLOGIST', 'run_hplc_analysis', {'sample': 'stability'}),
    ('TOXICOLOGIST', 'run_hplc_analysis', {'sample': 'purity_check'}),
    ('TOXICOLOGIST', 'run_cell_assay', {'assay_type': 'cytotoxicity'}),
    ('LEAD_SCIENTIST', 'run_hplc_analysis', {'sample': 'structural_confirmation'}),
    ('LEAD_SCIENTIST', 'run_mass_spec_analysis', {'sample': 'characterization'}),
]


class RegulatoryDiscovery(unittest.IsolatedAsyncioTestCase):
    def setUp(self):
        self.sim = importlib.import_module('benchmark.environments.14M.sim').DrugDiscoveryRegulatorySim()
        self.sim._delay_multiplier = 0
        self.sim.set_difficulty(0)
        self.registry = load_tools('14M', sim=self.sim)

    async def call(self, actor, tool, arguments):
        return await self.registry.call(tool, agent_id=actor, **arguments)

    async def analyses(self):
        for actor, tool, args in ANALYSES:
            self.assertTrue((await self.call(actor, tool, args)).success, (actor, tool, args))

    async def decide(self, go=True):
        return await self.registry.call('submit_decision', agent_id='LEAD_SCIENTIST', go=go)

    async def review(self):
        return await self.registry.call('verify_regulatory_compliance',
                                        agent_id='REGULATORY_SPECIALIST', decision='go_no_go')

    async def document(self):
        return await self.registry.call('run_hplc_analysis', agent_id='CHEMIST', sample='documentation')

    async def test_early_regulatory_approval_and_reverse_analyses_cannot_complete(self):
        self.assertFalse((await self.review()).success)
        self.assertFalse((await self.document()).success)
        self.assertFalse((await self.decide()).success)
        failed = [not (await self.call(actor, tool, args)).success
                  for actor, tool, args in reversed(ANALYSES)]
        self.assertTrue(any(failed))
        self.assertFalse(self.sim.is_complete())
        await self.analyses()
        self.assertTrue((await self.decide()).success)
        self.assertFalse((await self.document()).success)
        self.assertTrue((await self.review()).success)
        self.assertTrue((await self.document()).success)
        self.assertTrue(self.sim.is_complete())

    async def test_independent_order_catalog_and_both_decisions(self):
        orders = [order for order in itertools.permutations(range(5, 11))
                  if order.index(5) < order.index(6) and order.index(7) < order.index(8)]
        self.assertEqual(len(orders), 180)
        cases = 0
        for roots in itertools.permutations(range(4)):
            for studies in orders:
                for go in [False, True]:
                    self.setUp()
                    for at in (*roots, 4, *studies):
                        self.assertTrue((await self.call(*ANALYSES[at])).success, (roots, studies, at))
                    self.assertTrue((await self.decide(go)).success)
                    decision = len(self.sim.events) - 1
                    review = await self.review()
                    self.assertTrue(review.success)
                    self.assertEqual(review.data['inputs']['decision'], decision)
                    result = await self.document()
                    self.assertTrue(result.success)
                    self.assertEqual(result.data['inputs']['decision'], decision)
                    self.assertEqual(result.data['inputs']['regulatory'], decision + 1)
                    self.assertTrue(self.sim.is_complete())
                    self.assertEqual(self.sim.violations, [])
                    cases += 1
        self.assertEqual(cases, 8640)

    async def test_each_missing_analysis_blocks_decision_and_can_recover(self):
        for absent in range(len(ANALYSES)):
            self.setUp()
            for at, call in enumerate(ANALYSES):
                if at != absent:
                    await self.call(*call)
            self.assertFalse((await self.decide()).success, absent)
            self.assertFalse((await self.review()).success, absent)
            self.assertFalse((await self.document()).success, absent)
            self.assertFalse(self.sim.is_complete())
            await self.analyses()
            self.assertTrue((await self.decide()).success)
            self.assertTrue((await self.review()).success)
            self.assertTrue((await self.document()).success)
            self.assertTrue(self.sim.is_complete())

    async def test_rejection_needs_new_decision_but_does_not_prescribe_extra_assays(self):
        for rejections in [1, 2, 3]:
            for changed in [None, 5, 8]:
                self.setUp()
                self.sim._failure_specs = [FailureSpec('verify_regulatory_compliance',
                    agent_id='REGULATORY_SPECIALIST', call_index=i) for i in range(rejections)]
                await self.analyses()
                for _ in range(rejections):
                    self.assertTrue((await self.decide()).success)
                    self.assertFalse((await self.review()).success)
                    self.assertFalse((await self.review()).success, 'same rejected decision reused')
                    self.assertFalse((await self.document()).success)
                    if changed is not None:
                        self.assertTrue((await self.call(*ANALYSES[changed])).success)
                        if changed == 5:
                            self.assertTrue((await self.call(*ANALYSES[6])).success)
                self.assertTrue((await self.decide(False)).success)
                self.assertTrue((await self.review()).success)
                self.assertTrue((await self.document()).success)
                self.assertTrue(self.sim.is_complete())

    async def test_failure_and_pending_replacements_do_not_reuse_old_positive_data(self):
        for changed in [0, 1, 5, 6, 7, 8, 9]:
            self.setUp()
            await self.analyses()
            self.assertTrue((await self.decide()).success)
            self.assertTrue((await self.review()).success)
            self.assertTrue((await self.document()).success)
            actor, tool, args = ANALYSES[changed]
            index = self.sim._call_counts.get(f'{tool}:{actor}', 0)
            self.sim._failure_specs = [FailureSpec(tool, agent_id=actor, call_index=index)]
            self.assertFalse((await self.call(actor, tool, args)).success)
            self.assertFalse(self.sim.is_complete(), changed)
            self.assertFalse((await self.review()).success)
            self.assertFalse((await self.document()).success)
            self.sim._failure_specs = []
            await self.analyses()
            self.assertTrue((await self.decide()).success)
            self.assertTrue((await self.review()).success)
            self.assertTrue((await self.document()).success)
            self.assertTrue(self.sim.is_complete())

    async def test_every_failible_analysis_retries_without_losing_its_inputs(self):
        for changed in [0, 1, 5, 6, 7, 8, 9]:
            for depth in [1, 2, 3]:
                self.setUp()
                actor, tool, args = ANALYSES[changed]
                # The scientist may have used the same instrument for an
                # earlier different analysis; identify the actual call count.
                for call in ANALYSES[:changed]:
                    self.assertTrue((await self.call(*call)).success)
                first = self.sim._call_counts.get(f'{tool}:{actor}', 0)
                self.sim._failure_specs = [FailureSpec(tool, agent_id=actor, call_index=first+i)
                                           for i in range(depth)]
                for _ in range(depth):
                    self.assertFalse((await self.call(actor, tool, args)).success)
                self.assertTrue((await self.call(actor, tool, args)).success)
                for call in ANALYSES[changed+1:]:
                    self.assertTrue((await self.call(*call)).success)
                self.assertTrue((await self.decide()).success)
                self.assertTrue((await self.review()).success)
                self.assertTrue((await self.document()).success)
                self.assertTrue(self.sim.is_complete())

    async def test_pending_decision_and_approval_are_not_completed_inputs(self):
        for pending_tool in ['submit_decision', 'verify_regulatory_compliance']:
            self.setUp()
            await self.analyses()
            if pending_tool != 'submit_decision':
                self.assertTrue((await self.decide()).success)
            self.sim._delay_multiplier = 1
            self.sim.tool_delay = lambda tool, **kwargs: (0.02, 0.02) if tool == pending_tool else (0, 0)
            pending = asyncio.create_task(self.decide() if pending_tool == 'submit_decision' else self.review())
            await asyncio.sleep(0)
            self.assertFalse((await self.document()).success)
            self.assertFalse(self.sim.is_complete())
            self.assertTrue((await pending).success)
            if pending_tool == 'submit_decision':
                self.assertTrue((await self.review()).success)
            self.assertTrue((await self.document()).success)
            self.assertTrue(self.sim.is_complete())

    async def test_world_observes_conflicts_without_scheduling_the_callers(self):
        for same_instrument in [False, True]:
            self.setUp()
            if same_instrument:
                await self.analyses()
                calls = [ANALYSES[7], ANALYSES[9]]
            else:
                calls = [ANALYSES[0], ANALYSES[2]]
            self.sim._delay_multiplier = 1
            self.sim.tool_delay = lambda tool, **kwargs: (0.02, 0.02)
            results = await asyncio.gather(*(self.call(*call) for call in calls))
            self.assertTrue(all(result.success for result in results))
            events = self.sim.events[-2:]
            self.assertLess(max(e.timestamp for e in events), min(e.completed_at for e in events))
            self.assertEqual('concurrent_resource_use' in {v.violation_type for v in self.sim.violations},
                             same_instrument)

    async def test_new_data_and_new_decisions_invalidate_previous_approval(self):
        await self.analyses()
        self.assertTrue((await self.decide()).success)
        self.assertTrue((await self.review()).success)
        self.assertTrue((await self.document()).success)

        self.assertTrue((await self.decide(False)).success)
        self.assertFalse(self.sim.is_complete())
        self.assertFalse((await self.document()).success)
        self.assertTrue((await self.review()).success)
        self.assertTrue((await self.document()).success)
        self.assertTrue((await self.call(*ANALYSES[5])).success)
        self.assertFalse((await self.review()).success)
        self.assertFalse((await self.decide()).success, 'stability refers to prior bioassay')
        self.assertTrue((await self.call(*ANALYSES[6])).success)
        self.assertTrue((await self.decide()).success)
        self.assertFalse((await self.document()).success)
        self.assertTrue((await self.review()).success)
        self.assertTrue((await self.document()).success)


    async def test_every_replaced_or_pending_input_invalidates_old_approval(self):
        for changed in range(len(ANALYSES)):
            for pending_result in [False, True]:
                self.setUp()
                await self.analyses()
                self.assertTrue((await self.decide()).success)
                self.assertTrue((await self.review()).success)
                self.assertTrue((await self.document()).success)
                if pending_result:
                    tool_name = ANALYSES[changed][1]
                    self.sim._delay_multiplier = 1
                    self.sim.tool_delay = lambda tool, **kwargs: (0.02, 0.02) if tool == tool_name else (0, 0)
                    pending = asyncio.create_task(self.call(*ANALYSES[changed]))
                    await asyncio.sleep(0)
                    self.assertFalse(self.sim.is_complete(), (changed, 'pending'))
                    self.assertFalse((await self.review()).success)
                    self.assertTrue((await pending).success)
                    self.sim._delay_multiplier = 0
                else:
                    self.assertTrue((await self.call(*ANALYSES[changed])).success)
                self.assertFalse(self.sim.is_complete(), (changed, 'completed replacement'))
                self.assertFalse((await self.review()).success)
                self.assertFalse((await self.document()).success)
                await self.analyses()
                self.assertTrue((await self.decide()).success)
                self.assertTrue((await self.review()).success)
                self.assertTrue((await self.document()).success)
                self.assertTrue(self.sim.is_complete())
    async def test_wrong_json_values_and_roles_fail_without_advancing(self):
        for value in [None, False, 3, [], {}, 'wrong_target']:
            for actor, tool, key in [('CHEMIST', 'run_hplc_analysis', 'sample'),
                                     ('CHEMIST', 'run_mass_spec_analysis', 'sample'),
                                     ('BIOLOGIST', 'run_cell_assay', 'assay_type')]:
                self.assertFalse((await self.call(actor, tool, {key: value})).success)
            self.assertFalse((await self.registry.call('submit_decision', agent_id='LEAD_SCIENTIST', go=value)).success)
            self.assertFalse((await self.registry.call('verify_regulatory_compliance',
                agent_id='REGULATORY_SPECIALIST', decision=value)).success)
        self.assertFalse((await self.registry.call('submit_decision', agent_id='BIOLOGIST', go=True)).success)
        self.assertFalse(self.sim.is_complete())
        await self.analyses()
        result = await self.decide()
        self.assertTrue(result.success)
        result.data['inputs'].clear()
        self.assertTrue((await self.review()).success, 'returned JSON must not mutate the journal')
        self.assertTrue((await self.document()).success)
        self.assertTrue(self.sim.is_complete())
