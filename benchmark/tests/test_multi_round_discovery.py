"""Scientific material and finite-sample outcomes, independent of choreography."""
import importlib
import asyncio
import itertools
import json
import unittest

from benchmark.tools import load_tools
from benchmark.tools.sim_base import FailureSpec


REVERSE_STUDY = [
    ('REGULATORY_SPECIALIST', 'verify_regulatory_compliance', {'review_type': 'initial_review'}),
    ('CHEMIST', 'run_hplc_analysis', {'sample': 'documentation'}),
    ('CLINICAL_LEAD', 'review_clinical_data', {'data_type': 'combined_review'}),
    ('FORMULATION_SCIENTIST', 'prepare_formulation', {'compound': 'unidentified'}),
    ('TOXICOLOGIST', 'run_cell_assay', {'assay_type': 'cytotoxicity'}),
    ('BIOLOGIST', 'run_cell_assay', {'assay_type': 'bioassay'}),
    ('BIOLOGIST', 'run_hplc_analysis', {'sample': 'stability'}),
    ('TOXICOLOGIST', 'run_hplc_analysis', {'sample': 'purity_check'}),
    ('CLINICAL_LEAD', 'run_mass_spec_analysis', {'sample': 'combined_review'}),
    ('CHEMIST', 'run_mass_spec_analysis', {'sample': 'molecular_id'}),
    ('TOXICOLOGIST', 'run_mass_spec_analysis', {'sample': 'baseline_markers'}),
    ('BIOLOGIST', 'run_cell_assay', {'assay_type': 'culture_prep'}),
    ('CHEMIST', 'run_hplc_analysis', {'sample': 'synthesis_qc'}),
]


class MultiRoundDiscovery(unittest.IsolatedAsyncioTestCase):
    def setUp(self):
        self.sim = importlib.import_module('benchmark.environments.14H.sim').DrugDiscoveryMultiRoundSim()
        self.sim._delay_multiplier = 0
        self.sim.set_difficulty(0)
        self.registry = load_tools('14H', sim=self.sim)

    async def call(self, actor, tool, **arguments):
        return await self.registry.call(tool, agent_id=actor, **arguments)

    async def prepare(self, order=(0, 1, 2)):
        roots = [
            ('CHEMIST', 'run_hplc_analysis', {'sample': 'synthesis_qc'}),
            ('BIOLOGIST', 'run_cell_assay', {'assay_type': 'culture_prep'}),
            ('TOXICOLOGIST', 'run_mass_spec_analysis', {'sample': 'baseline_markers'}),
        ]
        for at in order:
            actor, tool, arguments = roots[at]
            self.assertTrue((await self.call(actor, tool, **arguments)).success)
        identity = await self.call('CHEMIST', 'run_mass_spec_analysis', sample='molecular_id')
        self.assertTrue(identity.success)
        self.assertTrue((await self.call('FORMULATION_SCIENTIST', 'prepare_formulation',
                                        compound=identity.data['compound'])).success)
        return identity.data['compound']

    async def studies(self, order=(0, 1, 2, 3)):
        analyses = [
            ('BIOLOGIST', 'run_cell_assay', {'assay_type': 'bioassay'}),
            ('BIOLOGIST', 'run_hplc_analysis', {'sample': 'stability'}),
            ('TOXICOLOGIST', 'run_hplc_analysis', {'sample': 'purity_check'}),
            ('TOXICOLOGIST', 'run_cell_assay', {'assay_type': 'cytotoxicity'}),
        ]
        for at in order:
            actor, tool, arguments = analyses[at]
            self.assertTrue((await self.call(actor, tool, **arguments)).success)

    async def clinical(self):
        result = await self.call('CLINICAL_LEAD', 'review_clinical_data', data_type='combined_review')
        self.assertTrue(result.success)
        return result

    async def regulatory(self, review_type='initial_review'):
        return await self.call('REGULATORY_SPECIALIST', 'verify_regulatory_compliance',
                               review_type=review_type)

    async def decide(self, outcome):
        return await self.call('PROJECT_DIRECTOR', 'submit_project_decision', outcome=outcome)

    async def document(self):
        return await self.call('CHEMIST', 'run_hplc_analysis', sample='documentation')

    async def test_early_approval_documentation_and_reverse_study_cannot_complete(self):
        results = []
        for actor, tool, arguments in REVERSE_STUDY:
            result = await self.registry.call(tool, agent_id=actor, **arguments)
            results.append(result.to_dict())
        self.assertFalse(self.sim.is_complete(), json.dumps({
            'calls': REVERSE_STUDY, 'results': results,
            'complete': self.sim.is_complete(),
            'violations': [v.violation_type for v in self.sim.violations],
        }, sort_keys=True))

    async def test_independent_order_catalog_and_both_terminal_outcomes(self):
        orders = [p for p in itertools.permutations(range(4)) if p.index(2) < p.index(3)]
        self.assertEqual(len(orders), 12)
        cases = 0
        for roots in itertools.permutations(range(3)):
            for studies in orders:
                for outcome in ['continue', 'abort']:
                    self.setUp()
                    await self.prepare(roots)
                    await self.studies(studies)
                    self.assertEqual((await self.clinical()).data['assessment'], 'supported')
                    self.assertTrue((await self.regulatory()).success)
                    self.assertFalse((await self.document()).success, 'missing actual director decision')
                    self.assertTrue((await self.decide(outcome)).success)
                    self.assertTrue((await self.document()).success)
                    self.assertTrue(self.sim.is_complete())
                    self.assertEqual(self.sim.progress['final_outcome'], outcome)
                    self.assertEqual(self.sim.samples_remaining, 1)
                    self.assertEqual(self.sim.violations, [])
                    cases += 1
        self.assertEqual(cases, 144)

    async def test_each_required_material_review_or_decision_must_exist_before_archival(self):
        for absent in range(13):
            self.setUp()
            compound = 'missing'
            operations = [
                ('CHEMIST', 'run_hplc_analysis', {'sample': 'synthesis_qc'}),
                ('BIOLOGIST', 'run_cell_assay', {'assay_type': 'culture_prep'}),
                ('TOXICOLOGIST', 'run_mass_spec_analysis', {'sample': 'baseline_markers'}),
                ('CHEMIST', 'run_mass_spec_analysis', {'sample': 'molecular_id'}),
                ('FORMULATION_SCIENTIST', 'prepare_formulation', {}),
                ('BIOLOGIST', 'run_cell_assay', {'assay_type': 'bioassay'}),
                ('BIOLOGIST', 'run_hplc_analysis', {'sample': 'stability'}),
                ('TOXICOLOGIST', 'run_hplc_analysis', {'sample': 'purity_check'}),
                ('TOXICOLOGIST', 'run_cell_assay', {'assay_type': 'cytotoxicity'}),
                ('CLINICAL_LEAD', 'review_clinical_data', {'data_type': 'combined_review'}),
                ('REGULATORY_SPECIALIST', 'verify_regulatory_compliance', {'review_type': 'initial_review'}),
                ('PROJECT_DIRECTOR', 'submit_project_decision', {'outcome': 'continue'}),
                ('CHEMIST', 'run_hplc_analysis', {'sample': 'documentation'}),
            ]
            for at, (actor, tool, arguments) in enumerate(operations):
                if at == absent:
                    continue
                if tool == 'prepare_formulation':
                    arguments = {'compound': compound}
                result = await self.call(actor, tool, **arguments)
                if at == 3:
                    compound = result.data['compound']
            self.assertFalse(self.sim.is_complete(), absent)

    async def test_every_new_current_result_requires_fresh_downstream_receipts(self):
        replacements = [
            ('CHEMIST', 'run_hplc_analysis', {'sample': 'synthesis_qc'}),
            ('BIOLOGIST', 'run_cell_assay', {'assay_type': 'culture_prep'}),
            ('TOXICOLOGIST', 'run_mass_spec_analysis', {'sample': 'baseline_markers'}),
            ('CHEMIST', 'run_mass_spec_analysis', {'sample': 'molecular_id'}),
            ('FORMULATION_SCIENTIST', 'prepare_formulation', {}),
            ('BIOLOGIST', 'run_cell_assay', {'assay_type': 'bioassay'}),
            ('BIOLOGIST', 'run_hplc_analysis', {'sample': 'stability'}),
            ('TOXICOLOGIST', 'run_hplc_analysis', {'sample': 'purity_check'}),
            ('TOXICOLOGIST', 'run_cell_assay', {'assay_type': 'cytotoxicity'}),
            ('CLINICAL_LEAD', 'review_clinical_data', {'data_type': 'combined_review'}),
            ('REGULATORY_SPECIALIST', 'verify_regulatory_compliance', {'review_type': 'initial_review'}),
            ('PROJECT_DIRECTOR', 'submit_project_decision', {'outcome': 'continue'}),
        ]
        for actor, tool, arguments in replacements:
            for pending in [False, True]:
                self.setUp()
                compound = await self.prepare()
                await self.studies()
                await self.clinical()
                self.assertTrue((await self.regulatory()).success)
                self.assertTrue((await self.decide('continue')).success)
                self.assertTrue((await self.document()).success)
                self.assertTrue(self.sim.is_complete())
                if tool == 'prepare_formulation':
                    arguments = {'compound': compound}
                if pending:
                    self.sim._delay_multiplier = 1
                    self.sim.tool_delay = lambda tool, **kwargs: (0.01, 0.01)
                    task = asyncio.create_task(self.call(actor, tool, **arguments))
                    await asyncio.sleep(0)
                    self.assertFalse(self.sim.is_complete(), (tool, pending))
                    self.assertTrue((await task).success)
                else:
                    self.assertTrue((await self.call(actor, tool, **arguments)).success)
                self.assertFalse(self.sim.is_complete(), (tool, pending))

    async def test_failed_replacements_do_not_fall_back_to_prior_positive_results(self):
        for actor, tool, arguments in [
            ('CHEMIST', 'run_hplc_analysis', {'sample': 'synthesis_qc'}),
            ('BIOLOGIST', 'run_cell_assay', {'assay_type': 'culture_prep'}),
            ('FORMULATION_SCIENTIST', 'prepare_formulation', {}),
            ('BIOLOGIST', 'run_cell_assay', {'assay_type': 'bioassay'}),
            ('BIOLOGIST', 'run_hplc_analysis', {'sample': 'stability'}),
            ('TOXICOLOGIST', 'run_hplc_analysis', {'sample': 'purity_check'}),
            ('TOXICOLOGIST', 'run_cell_assay', {'assay_type': 'cytotoxicity'}),
            ('REGULATORY_SPECIALIST', 'verify_regulatory_compliance', {'review_type': 'initial_review'}),
        ]:
            self.setUp()
            compound = await self.prepare()
            await self.studies()
            await self.clinical()
            self.assertTrue((await self.regulatory()).success)
            self.assertTrue((await self.decide('continue')).success)
            self.assertTrue((await self.document()).success)
            self.assertTrue(self.sim.is_complete())
            if tool == 'prepare_formulation':
                arguments = {'compound': compound}
            call = self.sim._call_counts.get(f'{tool}:{actor}', 0)
            self.sim._failure_specs = [FailureSpec(tool, agent_id=actor, call_index=call)]
            self.assertFalse((await self.call(actor, tool, **arguments)).success)
            self.assertFalse(self.sim.is_complete(), (actor, tool))

    async def test_actual_sample_consumption_includes_failure_and_never_culture_preparation(self):
        await self.prepare()
        for _ in range(4):
            self.assertTrue((await self.call('BIOLOGIST', 'run_cell_assay',
                                            assay_type='culture_prep')).success)
        self.assertEqual(self.sim.samples_remaining, 3)
        self.sim._failure_specs = [FailureSpec('run_cell_assay', agent_id='BIOLOGIST',
                                               call_index=5+i) for i in range(3)]
        for remaining in [2, 1, 0]:
            result = await self.call('BIOLOGIST', 'run_cell_assay', assay_type='bioassay')
            self.assertFalse(result.success)
            self.assertEqual(result.data['status'], 'injected_failure')
            self.assertEqual(self.sim.samples_remaining, remaining)
        result = await self.call('BIOLOGIST', 'run_cell_assay', assay_type='bioassay')
        self.assertFalse(result.success)
        self.assertEqual(result.data['status'], 'exhausted')
        units = [e.result['sample_unit'] for e in self.sim.events if e.result.get('sample_unit') is not None]
        self.assertEqual(units, [0, 1, 2])

    async def test_exhaustion_supports_an_honest_abort_not_continuation(self):
        await self.prepare()
        self.sim._failure_specs = [FailureSpec('run_cell_assay', agent_id='BIOLOGIST',
                                               call_index=1+i) for i in range(3)]
        for _ in range(3):
            self.assertFalse((await self.call('BIOLOGIST', 'run_cell_assay', assay_type='bioassay')).success)
        self.assertTrue((await self.call('BIOLOGIST', 'run_hplc_analysis', sample='stability')).success)
        self.assertTrue((await self.call('TOXICOLOGIST', 'run_hplc_analysis', sample='purity_check')).success)
        exhausted = await self.call('TOXICOLOGIST', 'run_cell_assay', assay_type='cytotoxicity')
        self.assertEqual(exhausted.data['status'], 'exhausted')
        self.assertEqual((await self.clinical()).data['assessment'], 'insufficient_data')
        self.assertEqual((await self.regulatory()).data['status'], 'clinical_hold')
        self.assertFalse((await self.decide('continue')).success)
        self.assertTrue((await self.decide('abort')).success)
        self.assertTrue((await self.document()).success)
        self.assertTrue(self.sim.is_complete())
        self.assertEqual(self.sim.samples_remaining, 0)
        self.assertEqual(self.sim.progress['final_outcome'], 'abort')

    async def test_negative_outcomes_have_distinct_actual_occurrences_for_clinical_inputs(self):
        await self.prepare()
        self.sim._failure_specs = [FailureSpec('run_cell_assay', agent_id='BIOLOGIST',
                                               call_index=1+i) for i in range(3)]
        failures = [await self.call('BIOLOGIST', 'run_cell_assay', assay_type='bioassay')
                    for _ in range(3)]
        bio = await self.call('BIOLOGIST', 'run_cell_assay', assay_type='bioassay')
        for result in [*failures, bio]:
            self.assertFalse(result.success)
            self.assertIsNone(result.data['artifact_id'])
            at = result.data['event_id']
            self.assertEqual(self.sim.events[at].result, result.data)
        self.assertEqual(len({result.data['event_id'] for result in [*failures, bio]}), 4)
        await self.call('BIOLOGIST', 'run_hplc_analysis', sample='stability')
        await self.call('TOXICOLOGIST', 'run_hplc_analysis', sample='purity_check')
        tox = await self.call('TOXICOLOGIST', 'run_cell_assay', assay_type='cytotoxicity')
        clinical = await self.clinical()
        self.assertEqual(clinical.data['inputs']['bioassay'], bio.data['event_id'])
        self.assertEqual(clinical.data['inputs']['toxicity'], tox.data['event_id'])
        self.assertEqual(clinical.data['artifact_id'], clinical.data['event_id'])
        self.assertNotEqual(bio.data['event_id'], tox.data['event_id'])
        actual_bio = bio.data['event_id']
        bio.data['event_id'] = tox.data['event_id']
        self.assertEqual((await self.clinical()).data['inputs']['bioassay'], actual_bio)
        held = await self.regulatory()
        self.assertFalse(held.success)
        self.assertEqual(self.sim.events[held.data['event_id']].result, held.data)
        self.assertEqual([event.result['event_id'] for event in self.sim.events],
                         list(range(len(self.sim.events))))
        self.assertEqual(self.sim.violations, [])

    async def test_retry_round_changes_material_and_does_not_replenish_the_pool(self):
        for first in ['BIOLOGIST', 'TOXICOLOGIST']:
            self.setUp()
            await self.prepare()
            await self.studies()
            await self.clinical()
            self.assertTrue((await self.regulatory()).success)
            self.assertTrue((await self.decide('retry')).success)
            self.assertFalse((await self.document()).success)
            self.assertTrue((await self.call('CHEMIST', 'run_hplc_analysis', sample='synthesis_qc')).success)
            identity = await self.call('CHEMIST', 'run_mass_spec_analysis', sample='molecular_id')
            self.assertTrue((await self.call('FORMULATION_SCIENTIST', 'prepare_formulation',
                                            compound=identity.data['compound'])).success)
            self.assertTrue((await self.call('BIOLOGIST', 'run_hplc_analysis', sample='stability')).success)
            self.assertTrue((await self.call('TOXICOLOGIST', 'run_hplc_analysis', sample='purity_check')).success)
            for actor in [first, 'TOXICOLOGIST' if first == 'BIOLOGIST' else 'BIOLOGIST']:
                result = await self.call(actor, 'run_cell_assay', assay_type=(
                    'bioassay' if actor == 'BIOLOGIST' else 'cytotoxicity'))
                self.assertEqual(result.success, actor == first)
            self.assertEqual((await self.clinical()).data['assessment'], 'insufficient_data')
            self.assertFalse((await self.regulatory()).success)
            self.assertFalse((await self.decide('continue')).success)
            self.assertTrue((await self.decide('abort')).success)
            self.assertTrue((await self.document()).success)
            self.assertTrue(self.sim.is_complete())
            self.assertEqual(self.sim.samples_remaining, 0)

    async def test_hold_needs_deeper_review_of_unchanged_evidence(self):
        for depth in [1, 2, 3]:
            self.setUp()
            await self.prepare()
            await self.studies()
            await self.clinical()
            self.sim._failure_specs = [FailureSpec('verify_regulatory_compliance',
                agent_id='REGULATORY_SPECIALIST', call_index=i) for i in range(depth)]
            for at in range(depth):
                held = await self.regulatory('initial_review' if at == 0 else 'hold_review')
                self.assertFalse(held.success)
                self.assertEqual(held.data['status'], 'clinical_hold')
                self.assertFalse((await self.regulatory()).success, 'routine review cannot erase a hold')
                self.assertFalse((await self.decide('continue')).success)
            self.assertTrue((await self.regulatory('hold_review')).success)
            self.assertTrue((await self.decide('continue')).success)
            self.assertTrue((await self.document()).success)
            self.assertTrue(self.sim.is_complete())

    async def test_new_scientific_evidence_can_resolve_hold_without_a_prescribed_review_path(self):
        await self.prepare()
        self.sim._failure_specs = [FailureSpec('run_cell_assay', agent_id='BIOLOGIST', call_index=1)]
        self.assertFalse((await self.call('BIOLOGIST', 'run_cell_assay', assay_type='bioassay')).success)
        await self.studies((1, 2, 3))
        self.assertEqual((await self.clinical()).data['assessment'], 'insufficient_data')
        self.assertFalse((await self.regulatory()).success)
        self.assertTrue((await self.call('BIOLOGIST', 'run_cell_assay', assay_type='bioassay')).success)
        self.assertEqual((await self.clinical()).data['assessment'], 'supported')
        self.assertTrue((await self.regulatory()).success)
        self.assertTrue((await self.decide('continue')).success)
        self.assertTrue((await self.document()).success)
        self.assertTrue(self.sim.is_complete())
        self.assertEqual(self.sim.samples_remaining, 0)

    async def test_pending_and_replaced_material_cannot_reuse_approval_or_final_profile(self):
        compound = await self.prepare()
        await self.studies()
        await self.clinical()
        self.assertTrue((await self.regulatory()).success)
        self.assertTrue((await self.decide('continue')).success)
        self.assertTrue((await self.document()).success)
        self.assertTrue(self.sim.is_complete())
        self.sim._delay_multiplier = 1
        self.sim.tool_delay = lambda tool, **kwargs: (0.02, 0.02)
        pending = asyncio.create_task(self.call('FORMULATION_SCIENTIST', 'prepare_formulation',
                                               compound=compound))
        await asyncio.sleep(0)
        self.assertFalse(self.sim.is_complete())
        self.assertFalse((await self.document()).success)
        self.assertTrue((await pending).success)
        self.assertFalse(self.sim.is_complete())
        self.assertFalse((await self.regulatory()).success)

    async def test_returned_json_cannot_forge_clinical_evidence(self):
        await self.prepare()
        self.sim._failure_specs = [FailureSpec('run_cell_assay', agent_id='BIOLOGIST', call_index=1)]
        self.assertFalse((await self.call('BIOLOGIST', 'run_cell_assay', assay_type='bioassay')).success)
        await self.studies((1, 2, 3))
        result = await self.clinical()
        result.data['assessment'] = 'supported'
        result.data['inputs']['bioassay'] = 9999
        self.assertFalse((await self.regulatory()).success)
        self.assertFalse((await self.decide('continue')).success)

    async def test_malformed_or_unassigned_requests_never_consume_a_sample(self):
        await self.prepare()
        for value in [None, [], {}, 7]:
            for actor, tool, key in [
                ('BIOLOGIST', 'run_cell_assay', 'assay_type'),
                ('CHEMIST', 'run_hplc_analysis', 'sample'),
                ('FORMULATION_SCIENTIST', 'prepare_formulation', 'compound'),
                ('CLINICAL_LEAD', 'review_clinical_data', 'data_type'),
                ('REGULATORY_SPECIALIST', 'verify_regulatory_compliance', 'review_type'),
                ('PROJECT_DIRECTOR', 'submit_project_decision', 'outcome'),
            ]:
                self.assertFalse((await self.call(actor, tool, **{key: value})).success)
        self.assertFalse((await self.call('CHEMIST', 'run_cell_assay', assay_type='bioassay')).success)
        self.assertEqual(self.sim.samples_remaining, 3)

    async def test_hold_uses_both_instruments_and_world_detects_conflicts_without_serializing(self):
        await self.prepare()
        await self.studies()
        await self.clinical()
        self.sim._delay_multiplier = 1
        self.sim.tool_delay = lambda tool, **kwargs: (0.02, 0.02)
        hold = asyncio.create_task(self.regulatory('hold_review'))
        await asyncio.sleep(0)
        await asyncio.gather(
            self.call('CHEMIST', 'run_hplc_analysis', sample='independent_extra'),
            self.call('TOXICOLOGIST', 'run_mass_spec_analysis', sample='baseline_markers'),
        )
        await hold
        conflicts = [v for v in self.sim.violations if v.violation_type == 'concurrent_resource_use']
        self.assertEqual(len(conflicts), 2)
        self.assertTrue(any('HPLC' in v.message for v in conflicts))
        self.assertTrue(any('MASS_SPEC' in v.message for v in conflicts))
