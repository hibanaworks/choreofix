"""13H scores material causality, finite stock and actual instrument overlap."""
import asyncio
import importlib
import itertools
import unittest

from benchmark.tools import load_tools
from benchmark.tools.sim_base import FailureSpec

SYNTHESIS = ('run_synthesis', 'CHEMIST', {'compound': 'batch_one'})
ANALYSIS = ('run_analysis', 'CHEMIST', {'sample': 'synthesized_compound'})
PURIFICATION = ('run_separation', 'CHEMIST', {'material': 'compound_purification'})
CELLS = ('run_separation', 'BIOLOGIST', {'material': 'cell_cultures'})
CALIBRATION = ('run_analysis', 'BIOLOGIST', {'sample': 'fluorescence'})
ASSAY = ('run_bioassay', 'BIOLOGIST', {'compound': 'batch_one'})
CHROMATOGRAPHY = ('run_chromatography', 'ANALYST', {'sample': 'batch_one'})
MOLECULAR = ('run_analysis', 'ANALYST', {'sample': 'molecular_detail'})
VERIFICATION = ('run_separation', 'ANALYST', {'material': 'verification'})
QC = ('verify_result', 'QC_CONTROLLER', {'scientist': 'BIOLOGIST', 'data_type': 'bioassay'})
INSPECTION = ('perform_inspection', 'SAFETY_OFFICER', {})
FINAL = ('run_analysis', 'CHEMIST', {'sample': 'final_validation'})
PREPARATION = [SYNTHESIS, ANALYSIS, PURIFICATION, CELLS, CALIBRATION, ASSAY]
QUALITY = [CHROMATOGRAPHY, MOLECULAR, VERIFICATION, QC]


class LabCascadingWorld(unittest.IsolatedAsyncioTestCase):
    def setUp(self):
        self.sim = importlib.import_module('benchmark.environments.13H.sim').PharmaLabCascadingSim()
        self.sim.set_scenario_depth(0)
        self.sim._delay_multiplier = 0
        self.registry = load_tools('13H', sim=self.sim)

    async def call(self, operation):
        tool, actor, arguments = operation
        return await self.registry.call(tool, agent_id=actor, **arguments)

    async def test_all_120_preparation_quality_orders_and_independent_inspection(self):
        checked = 0
        inspection_positions = set()
        for preparation in itertools.permutations(PREPARATION[:5]):
            if not (preparation.index(SYNTHESIS) < preparation.index(ANALYSIS)
                    < preparation.index(PURIFICATION)
                    and preparation.index(CELLS) < preparation.index(CALIBRATION)):
                continue
            for quality in itertools.permutations(QUALITY):
                if quality.index(CHROMATOGRAPHY) > quality.index(MOLECULAR):
                    continue
                self.setUp()
                schedule = [*preparation, ASSAY, *quality, FINAL]
                position = checked % (len(schedule) + 1)
                inspection_positions.add(position)
                schedule.insert(position, INSPECTION)
                for operation in schedule:
                    self.assertTrue((await self.call(operation)).success, operation)
                self.assertTrue(self.sim.is_complete())
                self.assertEqual(self.sim.violations, [])
                self.assertEqual(self.sim.progress['reagent_units_remaining'], 2)
                checked += 1
        self.assertEqual(checked, 120)
        self.assertEqual(inspection_positions, set(range(12)))

    async def test_each_missing_material_or_inspection_prevents_completion(self):
        schedule = [*PREPARATION, *QUALITY, INSPECTION, FINAL]
        for missing in range(len(schedule)):
            self.setUp()
            for index, operation in enumerate(schedule):
                if index != missing:
                    await self.call(operation)
            self.assertFalse(self.sim.is_complete(), schedule[missing])

    async def test_rejected_bioassay_requires_a_new_batch_not_another_old_assay(self):
        for operation in [*PREPARATION, *QUALITY[:3], INSPECTION]:
            self.assertTrue((await self.call(operation)).success)
        self.sim._failure_specs = [FailureSpec('verify_result', call_index=0)]
        rejected = await self.call(QC)
        self.assertEqual(rejected.data['status'], 'rejected')
        self.assertFalse((await self.call(QC)).success)
        self.assertTrue((await self.call(ASSAY)).success)
        self.assertFalse((await self.call(QC)).success)
        self.assertFalse((await self.call(FINAL)).success)

        second = ('run_synthesis', 'CHEMIST', {'compound': 'batch_two'})
        for operation in [second, ANALYSIS, PURIFICATION,
                          ('run_bioassay', 'BIOLOGIST', {'compound': 'batch_two'}), QC]:
            self.assertTrue((await self.call(operation)).success, operation)
        # Old analytical results cannot certify the replacement material.
        self.assertFalse((await self.call(FINAL)).success)
        for operation in [('run_chromatography', 'ANALYST', {'sample': 'batch_two'}),
                          VERIFICATION, MOLECULAR, FINAL]:
            self.assertTrue((await self.call(operation)).success, operation)
        self.assertTrue(self.sim.is_complete())
        self.assertEqual(self.sim.progress['reagent_units_remaining'], 1)
        self.assertNotEqual(rejected.data['inputs']['synthesis'],
                            self.sim.events[-1].result['inputs']['synthesis'])

    async def test_failed_syntheses_consume_exactly_three_units_and_invalid_calls_do_not(self):
        self.assertFalse((await self.call(('run_synthesis', 'CHEMIST', {'compound': []}))).success)
        self.assertEqual(self.sim.progress['reagent_units_remaining'], 3)
        self.sim._failure_specs = [FailureSpec('run_synthesis', call_index=i) for i in range(3)]
        for remaining in [2, 1, 0]:
            reply = await self.call(SYNTHESIS)
            self.assertEqual(reply.data['status'], 'synthesis_failed')
            self.assertIsNone(reply.data['artifact_id'])
            self.assertEqual(reply.data['reagent_units_remaining'], remaining)
            self.assertEqual(self.sim.progress['reagent_units_remaining'], remaining)
        exhausted = await self.call(SYNTHESIS)
        self.assertEqual(exhausted.data['status'], 'reagent_exhausted')
        self.assertIsNone(exhausted.data['artifact_id'])
        self.assertEqual(self.sim.progress['reagent_units_remaining'], 0)
        self.assertFalse((await self.call(ANALYSIS)).success)

    async def test_a_pending_synthesis_spends_stock_but_cannot_supply_material(self):
        self.sim._delay_multiplier = 1
        self.sim.tool_delay = lambda tool, **kwargs: (0.02, 0.02) if tool == 'run_synthesis' else (0, 0)
        pending = asyncio.create_task(self.call(SYNTHESIS))
        await asyncio.sleep(0)
        self.assertEqual(self.sim.progress['reagent_units_remaining'], 2)
        self.assertFalse((await self.call(ANALYSIS)).success)
        self.assertFalse((await self.call(PURIFICATION)).success)
        await pending
        self.assertTrue((await self.call(ANALYSIS)).success)
        self.assertTrue((await self.call(PURIFICATION)).success)

    async def test_inspection_occupies_all_four_actual_instruments(self):
        for operation in PREPARATION:
            self.assertTrue((await self.call(operation)).success)
        self.sim._delay_multiplier = 0.01
        await asyncio.gather(*(self.call(operation) for operation in
                             [INSPECTION, SYNTHESIS, ANALYSIS, CELLS, CHROMATOGRAPHY]))
        conflicts = [v for v in self.sim.violations if v.violation_type == 'concurrent_resource_use']
        self.assertEqual(len(conflicts), 4)
        for instrument in ['SPECTROMETER', 'CENTRIFUGE', 'FUME_HOOD', 'CHROMATOGRAPH']:
            self.assertTrue(any(instrument in violation.message for violation in conflicts))

    async def test_pending_inspection_is_not_complete_and_needs_no_scientific_order(self):
        for operation in [*PREPARATION, *QUALITY, FINAL]:
            self.assertTrue((await self.call(operation)).success)
        self.assertFalse(self.sim.is_complete())
        self.sim._delay_multiplier = 1
        self.sim.tool_delay = lambda tool, **kwargs: (0.02, 0.02) if tool == 'perform_inspection' else (0, 0)
        pending = asyncio.create_task(self.call(INSPECTION))
        await asyncio.sleep(0)
        self.assertFalse(self.sim.is_complete())
        await pending
        self.assertTrue(self.sim.is_complete())

    async def test_new_material_invalidates_old_final_and_old_sample_arguments(self):
        for operation in [*PREPARATION, *QUALITY, INSPECTION, FINAL]:
            self.assertTrue((await self.call(operation)).success)
        self.assertTrue(self.sim.is_complete())
        await self.call(('run_synthesis', 'CHEMIST', {'compound': 'batch_two'}))
        self.assertFalse(self.sim.is_complete())
        for operation in [ASSAY, CHROMATOGRAPHY, FINAL]:
            self.assertFalse((await self.call(operation)).success, operation)
