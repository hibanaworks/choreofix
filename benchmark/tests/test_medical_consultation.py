"""Method-neutral patient inputs and physical occupancy for the 5E world."""
import asyncio
import importlib
import itertools
import unittest

from benchmark.tools import load_tools


ASSESS = ("assess_patient", "ER_DOCTOR", {"patient_id": "patient_1"})
BASIC = ("order_tests", "ER_DOCTOR", {
    "patient_id": "patient_1", "test_type": "vitals,basic_labs"})
CARDIAC = ("order_tests", "CARDIOLOGIST", {
    "patient_id": "patient_1", "test_type": "ecg,cardiac_enzymes"})
IMAGE = ("perform_imaging", "ER_DOCTOR", {
    "patient_id": "patient_1", "modality": "CT"})
DIAGNOSE = ("diagnose", "CARDIOLOGIST", {"patient_id": "patient_1"})
REVIEW = ("review_prescriptions", "PHARMACIST", {
    "patient_id": "patient_1", "medications": "proposed_medications"})


class MedicalWorld(unittest.IsolatedAsyncioTestCase):
    def setUp(self):
        self.sim = importlib.import_module(
            "benchmark.environments.5E.sim").DualSpecialistSim()
        self.sim._delay_multiplier = 0
        self.registry = load_tools("5E", sim=self.sim)

    async def call(self, operation):
        tool, agent, arguments = operation
        return await self.registry.call(tool, agent_id=agent, **arguments)

    async def test_diagnosis_cannot_precede_its_actual_tests(self):
        self.assertFalse((await self.call(DIAGNOSE)).success)
        for operation in [ASSESS, BASIC]:
            self.assertTrue((await self.call(operation)).success)
        self.assertFalse((await self.call(DIAGNOSE)).success)
        self.assertTrue((await self.call(CARDIAC)).success)
        self.assertTrue((await self.call(DIAGNOSE)).success)

    async def test_review_preserves_caller_data_without_an_invented_diagnosis_gate(self):
        # No prescribe/administer tool exists in this task. Review the supplied
        # data without inventing a hidden diagnosis->prescription workflow.
        result = await self.call(REVIEW)
        self.assertTrue(result.success)
        self.assertEqual(result.data["medications"], "proposed_medications")
        result = await self.registry.call("review_prescriptions", agent_id="PHARMACIST",
            patient_id="patient_1")
        self.assertTrue(result.success)
        self.assertIsNone(result.data.get("medications"))

    async def test_tests_and_imaging_need_an_assessed_patient(self):
        for operation in [BASIC, CARDIAC, IMAGE]:
            self.assertFalse((await self.call(operation)).success, operation)
        self.assertFalse(self.sim.is_complete())

    async def test_public_test_type_reaches_the_actual_panel(self):
        self.assertTrue((await self.call(ASSESS)).success)
        result = await self.call(BASIC)
        self.assertTrue(result.success)
        self.assertEqual(result.data["tests"], ["vitals", "basic_labs"])
        result = await self.call(CARDIAC)
        self.assertTrue(result.success)
        self.assertEqual(result.data["tests"], ["ecg", "cardiac_enzymes"])

    async def test_empty_unknown_and_foreign_tests_do_not_supply_inputs(self):
        await self.call(ASSESS)
        for test_type in ["", "unknown_test", "ecg", [], {}]:
            result = await self.registry.call("order_tests", agent_id="ER_DOCTOR",
                patient_id="patient_1", test_type=test_type)
            self.assertFalse(result.success, test_type)
        self.assertFalse(self.sim.progress["test_orders"]["ER_DOCTOR_patient_1"])
        self.assertFalse((await self.call(DIAGNOSE)).success)
        self.assertFalse(self.sim.is_complete())

    async def test_unknown_patients_cannot_complete_the_assigned_case(self):
        for patient in ["patient_other", "", [], {}]:
            result = await self.registry.call("assess_patient", agent_id="ER_DOCTOR",
                patient_id=patient)
            self.assertFalse(result.success, patient)
        for operation in [BASIC, CARDIAC, DIAGNOSE, IMAGE]:
            self.assertFalse((await self.call(operation)).success)
        self.assertFalse(self.sim.is_complete())

    async def test_partial_test_orders_are_real_inputs_not_whole_panel_completion(self):
        await self.call(ASSESS)
        first = await self.registry.call("order_tests", agent_id="ER_DOCTOR",
            patient_id="patient_1", test_type="vitals")
        self.assertTrue(first.success)
        self.assertEqual(first.data["tests"], ["vitals"])
        self.assertFalse(self.sim.progress["test_orders"]["ER_DOCTOR_patient_1"])
        self.assertTrue((await self.call(CARDIAC)).success)
        second = await self.registry.call("order_tests", agent_id="ER_DOCTOR",
            patient_id="patient_1", test_type="basic_labs")
        self.assertTrue(second.success)
        self.assertTrue(self.sim.progress["test_orders"]["ER_DOCTOR_patient_1"])
        self.assertTrue((await self.call(DIAGNOSE)).success)

    async def test_diagnosis_needs_the_complete_cardiac_panel_not_general_tests(self):
        await self.call(ASSESS)
        self.assertTrue((await self.registry.call("order_tests", agent_id="CARDIOLOGIST",
            patient_id="patient_1", test_type="ecg")).success)
        self.assertFalse((await self.call(DIAGNOSE)).success)
        self.assertTrue((await self.registry.call("order_tests", agent_id="CARDIOLOGIST",
            patient_id="patient_1", test_type="cardiac_enzymes")).success)
        self.assertTrue((await self.call(DIAGNOSE)).success)
        self.assertFalse(self.sim.progress["test_orders"]["ER_DOCTOR_patient_1"])

    async def test_all_five_imaging_positions_are_allowed(self):
        completed = 0
        for operations in itertools.permutations([BASIC, CARDIAC, DIAGNOSE, REVIEW, IMAGE]):
            order = [operation[0:2] for operation in operations]
            if not (order.index(BASIC[0:2]) < order.index(CARDIAC[0:2])
                    < order.index(DIAGNOSE[0:2]) < order.index(REVIEW[0:2])):
                continue
            self.setUp()
            self.assertTrue((await self.call(ASSESS)).success)
            for operation in operations:
                self.assertTrue((await self.call(operation)).success, operation)
            self.assertTrue(self.sim.is_complete())
            self.assertEqual(self.sim.violations, [])
            completed += 1
        self.assertEqual(completed, 5)

    async def test_all_72_data_valid_orders_including_independent_panels_and_review(self):
        completed = 0
        for operations in itertools.permutations([ASSESS, BASIC, CARDIAC, IMAGE, DIAGNOSE, REVIEW]):
            order = [operation[0:2] for operation in operations]
            if not (order.index(ASSESS[0:2]) < order.index(BASIC[0:2])
                    and order.index(ASSESS[0:2]) < order.index(CARDIAC[0:2])
                    < order.index(DIAGNOSE[0:2])
                    and order.index(ASSESS[0:2]) < order.index(IMAGE[0:2])):
                continue
            self.setUp()
            for operation in operations:
                self.assertTrue((await self.call(operation)).success, operation)
            self.assertTrue(self.sim.is_complete())
            self.assertEqual(self.sim.violations, [])
            completed += 1
        self.assertEqual(completed, 72)

    async def test_independent_general_tests_do_not_invalidate_cardiac_results(self):
        for operation in [ASSESS, BASIC, CARDIAC, IMAGE, DIAGNOSE, REVIEW]:
            await self.call(operation)
        self.assertTrue(self.sim.is_complete())
        await self.registry.call("order_tests", agent_id="ER_DOCTOR",
            patient_id="patient_1", test_type="vitals")
        self.assertTrue(self.sim.is_complete())

    async def test_medium_case_uses_completed_matching_images_and_real_public_fields(self):
        sim = importlib.import_module("benchmark.environments.5M.sim").MultiSpecialtyCaseSim()
        sim._delay_multiplier = 0
        tools = load_tools("5M", sim=sim)
        self.assertFalse((await tools.call("interpret_imaging", agent_id="RADIOLOGIST",
            patient_id="patient_1", modality="CT")).success)
        self.assertFalse((await tools.call("conduct_case_conference", agent_id="ER_DOCTOR",
            patient_id="patient_1")).success)
        operations = [ASSESS, BASIC, CARDIAC,
            ("order_tests", "NEUROLOGIST", {"patient_id": "patient_1", "test_type": "neuro_exam,nerve_conduction"}),
            ("perform_imaging", "RADIOLOGIST", {"patient_id": "patient_1", "modality": "MRI"}),
            ("interpret_imaging", "RADIOLOGIST", {"patient_id": "patient_1", "modality": "MRI"}),
            ("perform_imaging", "RADIOLOGIST", {"patient_id": "patient_1", "modality": "CT"}),
            ("interpret_imaging", "RADIOLOGIST", {"patient_id": "patient_1", "modality": "CT"}),
            DIAGNOSE, ("diagnose", "NEUROLOGIST", {"patient_id": "patient_1"}),
            ("conduct_case_conference", "ER_DOCTOR", {"patient_id": "patient_1", "attendees": "actual specialists"}), REVIEW]
        for tool, agent, arguments in operations:
            self.assertTrue((await tools.call(tool, agent_id=agent, **arguments)).success,
                (tool, agent, arguments))
        self.assertTrue(sim.is_complete())
        await tools.call("perform_imaging", agent_id="RADIOLOGIST", patient_id="patient_1", modality="CT")
        self.assertFalse(sim.is_complete())
        self.assertFalse((await tools.call("conduct_case_conference", agent_id="ER_DOCTOR",
            patient_id="patient_1")).success)
        self.assertTrue((await tools.call("interpret_imaging", agent_id="RADIOLOGIST",
            patient_id="patient_1", modality="CT")).success)
        for agent in ["NEUROLOGIST", "CARDIOLOGIST"]:
            self.assertTrue((await tools.call("diagnose", agent_id=agent, patient_id="patient_1")).success)
        self.assertTrue((await tools.call("conduct_case_conference", agent_id="ER_DOCTOR",
            patient_id="patient_1")).success)
        self.assertTrue(sim.is_complete())

    async def test_two_patients_keep_separate_results_and_preparation(self):
        sim = importlib.import_module("benchmark.environments.5H.sim").MultiPatientTriageSim()
        sim._delay_multiplier = 0
        tools = load_tools("5H", sim=sim)
        self.assertFalse((await tools.call("perform_surgery", agent_id="SURGEON",
            patient_id="patient_a", procedure="trauma_surgery")).success)
        for agent, patient, tests in [("ER_DOCTOR_B", "patient_b", "stroke_panel,ct_angio"),
                                      ("ER_DOCTOR_A", "patient_a", "trauma_panel,blood_type,coag")]:
            self.assertTrue((await tools.call("assess_patient", agent_id=agent, patient_id=patient)).success)
            self.assertTrue((await tools.call("order_tests", agent_id=agent,
                patient_id=patient, test_type=tests)).success)
        for patient, modality in [("patient_b", "MRI"), ("patient_a", "CT"), ("patient_b", "CT")]:
            self.assertTrue((await tools.call("perform_imaging", agent_id="RADIOLOGIST",
                patient_id=patient, modality=modality)).success)
            self.assertTrue((await tools.call("interpret_imaging", agent_id="RADIOLOGIST",
                patient_id=patient, modality=modality)).success)
        self.assertTrue((await tools.call("diagnose", agent_id="ER_DOCTOR_B", patient_id="patient_b")).success)
        self.assertFalse((await tools.call("perform_surgery", agent_id="SURGEON",
            patient_id="patient_a", procedure="trauma_surgery")).success)
        self.assertTrue((await tools.call("diagnose", agent_id="ER_DOCTOR_A", patient_id="patient_a")).success)
        self.assertFalse((await tools.call("perform_surgery", agent_id="SURGEON",
            patient_id="patient_a", procedure="trauma_surgery")).success)
        self.assertTrue((await tools.call("prepare_anesthesia", agent_id="ANESTHESIOLOGIST",
            patient_id="patient_a", type="caller supplied preparation")).success)
        self.assertTrue((await tools.call("perform_surgery", agent_id="SURGEON",
            patient_id="patient_a", procedure="trauma_surgery")).success)
        self.assertTrue((await tools.call("manage_icu", agent_id="ICU_COORDINATOR", patient_id="patient_b")).success)
        self.assertTrue((await tools.call("review_prescriptions", agent_id="PHARMACIST",
            patient_id="patient_A_and_B", medications="caller list")).success)
        self.assertTrue(sim.is_complete())
        self.assertEqual(sim.violations, [])
        self.assertTrue((await tools.call("assess_patient", agent_id="ER_DOCTOR_B", patient_id="patient_b")).success)
        self.assertFalse(sim.is_complete())
        self.assertTrue(sim.progress["surgeries"]["SURGEON_patient_a_trauma_surgery"])
        self.assertTrue((await tools.call("order_tests", agent_id="ER_DOCTOR_B",
            patient_id="patient_b", test_type="stroke_panel,ct_angio")).success)
        for modality in ["CT", "MRI"]:
            for tool in ["perform_imaging", "interpret_imaging"]:
                self.assertTrue((await tools.call(tool, agent_id="RADIOLOGIST",
                    patient_id="patient_b", modality=modality)).success)
        self.assertTrue((await tools.call("diagnose", agent_id="ER_DOCTOR_B", patient_id="patient_b")).success)
        self.assertTrue((await tools.call("manage_icu", agent_id="ICU_COORDINATOR", patient_id="patient_b")).success)
        self.assertTrue(sim.is_complete())

    async def test_pending_tests_cannot_supply_a_diagnosis(self):
        for operation in [ASSESS, BASIC]:
            await self.call(operation)
        self.sim._delay_multiplier = 1
        self.sim.tool_delay = lambda tool, **kwargs: (
            (0.02, 0.02) if tool == "order_tests" else (0, 0))
        pending = asyncio.create_task(self.call(CARDIAC))
        await asyncio.sleep(0)
        self.assertFalse((await self.call(DIAGNOSE)).success)
        await pending
        self.assertTrue((await self.call(DIAGNOSE)).success)

    async def test_independent_panels_and_imaging_really_overlap(self):
        await self.call(ASSESS)
        self.sim._delay_multiplier = 1
        self.sim.tool_delay = lambda tool, **kwargs: (0.02, 0.02)
        start = len(self.sim.events)
        results = await asyncio.gather(self.call(CARDIAC), self.call(BASIC), self.call(IMAGE))
        self.assertTrue(all(result.success for result in results))
        effects = self.sim.events[start:]
        self.assertEqual(len(effects), 3)
        self.assertLess(max(event.timestamp for event in effects),
            min(event.completed_at for event in effects))
        self.assertEqual(self.sim.violations, [])

    async def test_new_assessment_invalidates_old_patient_results(self):
        for operation in [ASSESS, BASIC, CARDIAC, IMAGE, DIAGNOSE, REVIEW]:
            self.assertTrue((await self.call(operation)).success)
        self.assertTrue(self.sim.is_complete())
        self.assertTrue((await self.call(ASSESS)).success)
        self.assertFalse(self.sim.is_complete())
        self.assertFalse((await self.call(DIAGNOSE)).success)
        for operation in [BASIC, CARDIAC, IMAGE, DIAGNOSE, REVIEW]:
            self.assertTrue((await self.call(operation)).success)
        self.assertTrue(self.sim.is_complete())

    async def test_scanner_conflicts_are_reported_not_serialized(self):
        await self.call(ASSESS)
        self.sim._delay_multiplier = 1
        self.sim.tool_delay = lambda tool, **kwargs: (0.02, 0.02)
        results = await asyncio.gather(self.call(IMAGE), self.call(IMAGE))
        self.assertTrue(all(result.success for result in results))
        self.assertIn("concurrent_resource_use", {
            violation.violation_type for violation in self.sim.violations})
