"""13M QC receipts bind actual measurements, not a preferred work schedule."""
import asyncio
import importlib
import itertools
import unittest

from benchmark.tools import load_tools
from benchmark.tools.sim_base import FailureSpec


class LabQualityWorld(unittest.IsolatedAsyncioTestCase):
    def setUp(self):
        self.sim = importlib.import_module('benchmark.environments.13M.sim').PharmaLabQCSim()
        self.sim.set_scenario_depth(0)
        self.sim._delay_multiplier = 0
        self.registry = load_tools('13M', sim=self.sim)

    async def call(self, tool, actor, **arguments):
        return await self.registry.call(tool, agent_id=actor, **arguments)

    async def prepared_compound_and_cells(self, name='compound_one'):
        await self.call('run_synthesis', 'CHEMIST', compound=name)
        await self.call('run_analysis', 'CHEMIST', sample='synthesized_compound')
        await self.call('run_separation', 'CHEMIST', material='compound_purification')
        await self.call('run_separation', 'BIOLOGIST', material='cell_cultures')
        await self.call('run_analysis', 'BIOLOGIST', sample='fluorescence')
        return await self.call('run_bioassay', 'BIOLOGIST', compound=name)

    async def qc(self, scientist):
        return await self.call('verify_result', 'QC_CONTROLLER', scientist=scientist,
                               data_type='purification' if scientist == 'CHEMIST' else 'bioassay')

    async def quality(self, reverse=False):
        calls = [('run_analysis', {'sample': 'molecular_detail'}),
                 ('run_separation', {'material': 'verification'})]
        for tool, args in reversed(calls) if reverse else calls:
            self.assertTrue((await self.call(tool, 'ANALYST', **args)).success)

    async def final(self):
        return await self.call('run_analysis', 'CHEMIST', sample='final_validation')

    async def test_early_qc_and_final_validation_do_not_create_approval(self):
        for scientist in ['CHEMIST', 'BIOLOGIST']:
            self.assertFalse((await self.qc(scientist)).success)
        self.assertFalse((await self.final()).success)
        self.assertFalse(self.sim.is_complete())

    async def test_eight_independent_qc_quality_and_optional_input_combinations(self):
        count = 0
        for order in itertools.permutations(['CHEMIST', 'BIOLOGIST']):
            for reverse in [False, True]:
                for culture in [None, 'my_culture']:
                    self.setUp()
                    await self.prepared_compound_and_cells('sample_alpha')
                    # Redoing the assay before QC is a new actual specimen trial.
                    args = {'compound': 'sample_alpha'}
                    if culture is not None:
                        args['cell_type'] = culture
                    self.assertTrue((await self.call('run_bioassay', 'BIOLOGIST', **args)).success)
                    for scientist in order:
                        approval = await self.qc(scientist)
                        self.assertTrue(approval.success)
                        self.assertEqual(approval.data['scientist'], scientist)
                        self.assertEqual(approval.data['qc_agent'], 'QC_CONTROLLER')
                    await self.quality(reverse)
                    self.assertTrue((await self.final()).success)
                    self.assertTrue(self.sim.is_complete())
                    self.assertEqual(self.sim.violations, [])
                    count += 1
        self.assertEqual(count, 8)

    async def test_rejected_purification_requires_a_new_measurement(self):
        await self.prepared_compound_and_cells()
        self.sim._failure_specs = [FailureSpec('verify_result', call_index=0)]
        rejected = await self.qc('CHEMIST')
        self.assertFalse(rejected.success)
        self.assertFalse((await self.qc('CHEMIST')).success)
        old_compound = self.sim.events[2].result
        await self.call('run_analysis', 'CHEMIST', sample='synthesized_compound')
        approved = await self.qc('CHEMIST')
        self.assertTrue(approved.success)
        self.assertEqual(approved.data['inputs']['purification'], old_compound['artifact_id'])
        self.assertNotEqual(approved.data['inputs']['analysis'], rejected.data['inputs']['analysis'])
        self.assertTrue((await self.qc('BIOLOGIST')).success)
        await self.quality()
        self.assertTrue((await self.final()).success)

    async def test_rejected_bioassay_requires_a_new_trial_and_new_quality_results(self):
        await self.prepared_compound_and_cells()
        self.assertTrue((await self.qc('CHEMIST')).success)
        await self.quality()
        self.sim._failure_specs = [FailureSpec('verify_result', call_index=1)]
        rejected = await self.qc('BIOLOGIST')
        self.assertFalse(rejected.success)
        self.assertFalse((await self.qc('BIOLOGIST')).success)
        await self.call('run_bioassay', 'BIOLOGIST', compound='compound_one')
        approved = await self.qc('BIOLOGIST')
        self.assertTrue(approved.success)
        self.assertNotEqual(approved.data['inputs']['bioassay'], rejected.data['inputs']['bioassay'])
        self.assertFalse((await self.final()).success)
        await self.quality(reverse=True)
        self.assertTrue((await self.final()).success)

    async def test_quality_may_precede_qc_but_final_needs_both_actual_approvals(self):
        await self.prepared_compound_and_cells()
        await self.quality()
        self.assertFalse((await self.final()).success)
        self.assertTrue((await self.qc('BIOLOGIST')).success)
        self.assertFalse((await self.final()).success)
        self.assertTrue((await self.qc('CHEMIST')).success)
        self.assertFalse(self.sim.is_complete())
        self.assertTrue((await self.final()).success)
        self.assertTrue(self.sim.is_complete())

    async def test_new_synthesis_cannot_use_old_approvals_or_old_quality(self):
        await self.prepared_compound_and_cells()
        await self.qc('CHEMIST'); await self.qc('BIOLOGIST'); await self.quality()
        await self.call('run_synthesis', 'CHEMIST', compound='compound_two')
        self.assertFalse((await self.final()).success)
        self.assertFalse((await self.call('run_bioassay', 'BIOLOGIST', compound='compound_one')).success)
        await self.call('run_analysis', 'CHEMIST', sample='synthesized_compound')
        await self.call('run_separation', 'CHEMIST', material='compound_purification')
        await self.call('run_bioassay', 'BIOLOGIST', compound='compound_two')
        await self.qc('BIOLOGIST'); await self.qc('CHEMIST'); await self.quality()
        self.assertTrue((await self.final()).success)

    async def test_a_later_failed_assay_cannot_be_hidden_by_an_old_success(self):
        await self.prepared_compound_and_cells()
        self.sim._failure_specs = [FailureSpec('run_bioassay', call_index=1)]
        self.assertFalse((await self.call('run_bioassay', 'BIOLOGIST', compound='compound_one')).success)
        self.assertFalse((await self.qc('BIOLOGIST')).success)
        self.assertFalse((await self.final()).success)
        self.assertTrue((await self.call('run_bioassay', 'BIOLOGIST', compound='compound_one')).success)
        self.assertTrue((await self.qc('BIOLOGIST')).success)

    async def test_pending_qc_is_not_a_completed_approval(self):
        await self.prepared_compound_and_cells()
        await self.qc('CHEMIST'); await self.quality()
        self.sim._delay_multiplier = 1
        self.sim.tool_delay = lambda tool, **kwargs: (0.02, 0.02) if tool == 'verify_result' else (0, 0)
        pending = asyncio.create_task(self.qc('BIOLOGIST'))
        await asyncio.sleep(0)
        self.assertFalse((await self.final()).success)
        await pending
        self.assertFalse(self.sim.is_complete())
        self.assertTrue((await self.final()).success)

    async def test_depth_zero_one_and_two_allow_reconsideration_to_completion(self):
        for depth in [0, 1, 2]:
            self.setUp(); self.sim.set_scenario_depth(depth)
            assay = await self.prepared_compound_and_cells()
            for _ in range(depth + 1):
                if assay.success:
                    break
                assay = await self.call('run_bioassay', 'BIOLOGIST', compound='compound_one')
            self.assertTrue(assay.success)
            for scientist in ['BIOLOGIST', 'CHEMIST']:
                for _ in range(depth + 1):
                    checked = await self.qc(scientist)
                    if checked.success:
                        break
                    if scientist == 'CHEMIST':
                        await self.call('run_analysis', 'CHEMIST', sample='synthesized_compound')
                    else:
                        await self.call('run_bioassay', 'BIOLOGIST', compound='compound_one')
                self.assertTrue(checked.success)
            await self.quality()
            self.assertTrue((await self.final()).success)
            self.assertTrue(self.sim.is_complete())

    async def test_qc_and_instrument_free_bioassay_can_overlap(self):
        await self.prepared_compound_and_cells()
        self.sim._delay_multiplier = 0.01
        replies = await asyncio.gather(self.qc('CHEMIST'),
            self.call('run_bioassay', 'BIOLOGIST', compound='compound_one'))
        self.assertTrue(all(reply.success and reply.delay_seconds > 0 for reply in replies))
        events = self.sim.events[-2:]
        self.assertLess(max(e.timestamp for e in events), min(e.completed_at for e in events))
        self.assertEqual(self.sim.violations, [])

    async def test_four_rejections_can_switch_subjects_without_a_fixed_qc_schedule(self):
        for subjects in [('CHEMIST', 'BIOLOGIST', 'CHEMIST', 'BIOLOGIST'),
                         ('BIOLOGIST', 'CHEMIST', 'BIOLOGIST', 'CHEMIST'),
                         ('CHEMIST', 'CHEMIST', 'BIOLOGIST', 'BIOLOGIST'),
                         ('BIOLOGIST', 'BIOLOGIST', 'CHEMIST', 'CHEMIST')]:
            for final_order in itertools.permutations(['CHEMIST', 'BIOLOGIST']):
                self.setUp()
                await self.prepared_compound_and_cells()
                self.sim._failure_specs = [FailureSpec('verify_result', call_index=i) for i in range(4)]
                for scientist in subjects:
                    rejected = await self.qc(scientist)
                    self.assertFalse(rejected.success)
                    self.assertEqual(rejected.data['status'], 'rejected')
                    if scientist == 'CHEMIST':
                        await self.call('run_analysis', 'CHEMIST', sample='synthesized_compound')
                    else:
                        await self.call('run_bioassay', 'BIOLOGIST', compound='compound_one')
                for scientist in final_order:
                    self.assertTrue((await self.qc(scientist)).success)
                await self.quality()
                self.assertTrue((await self.final()).success)
                self.assertTrue(self.sim.is_complete())

    async def test_qc_requires_both_actual_instruments(self):
        await self.prepared_compound_and_cells()
        self.sim._delay_multiplier = 0.01
        await asyncio.gather(self.qc('CHEMIST'),
            self.call('run_analysis', 'CHEMIST', sample='synthesized_compound'),
            self.call('run_synthesis', 'CHEMIST', compound='next_batch'))
        conflicts = [v for v in self.sim.violations if v.violation_type == 'concurrent_resource_use']
        self.assertEqual(len(conflicts), 2)

    async def test_wrong_qc_subject_and_malformed_targets_have_no_artifact(self):
        await self.prepared_compound_and_cells()
        for scientist, data_type in [('ANALYST', 'purification'), ('CHEMIST', 'bioassay'),
                                     ('BIOLOGIST', 'purification'), ([], {})]:
            reply = await self.call('verify_result', 'QC_CONTROLLER', scientist=scientist, data_type=data_type)
            self.assertFalse(reply.success)
        self.assertFalse(self.sim.is_complete())
