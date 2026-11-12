"""5H world data and physical capacities, without a Hibana workflow."""
import asyncio
import importlib
import itertools
import unittest

from benchmark.tools import load_tools


class CriticalCareWorld(unittest.IsolatedAsyncioTestCase):
    def setUp(self):
        self.sim = importlib.import_module("benchmark.environments.5H.sim").MultiPatientTriageSim()
        self.sim._delay_multiplier = 0
        self.tools = load_tools("5H", sim=self.sim)

    async def test_two_beds_are_available_and_the_third_overlap_is_reported(self):
        resources = ["ICU_BEDS"]
        self.assertEqual(self.sim.begin_resource_use("first", "manage_icu", resources), [])
        self.assertEqual(self.sim.begin_resource_use("second", "manage_icu", resources), [])
        violations = self.sim.begin_resource_use("third", "manage_icu", resources)
        self.assertEqual(len(violations), 1)
        self.assertIn("ICU_BEDS", violations[0].message)
        self.sim.end_resource_use("first", "manage_icu", resources)
        self.sim.end_resource_use("second", "manage_icu", resources)
        self.assertEqual(self.sim.begin_resource_use("fourth", "manage_icu", resources), [])
        self.assertEqual(len(self.sim.violations), 1)

    async def test_overlapping_actual_icu_calls_report_only_the_exhausted_resources(self):
        call = self.tools.call
        self.assertTrue((await call("assess_patient", agent_id="ER_DOCTOR_B",
                                   patient_id="patient_b")).success)
        self.assertTrue((await call("order_tests", agent_id="ER_DOCTOR_B",
                                   patient_id="patient_b", test_type="stroke_panel,ct_angio")).success)
        for modality in ("CT", "MRI"):
            for tool in ("perform_imaging", "interpret_imaging"):
                self.assertTrue((await call(tool, agent_id="RADIOLOGIST",
                                           patient_id="patient_b", modality=modality)).success)
        self.assertTrue((await call("diagnose", agent_id="ER_DOCTOR_B",
                                   patient_id="patient_b")).success)
        self.sim._delay_multiplier = 1
        self.sim.tool_delay = lambda *args, **kwargs: (0.02, 0.02)
        start = len(self.sim.events)
        results = await asyncio.gather(*(call("manage_icu", agent_id="ICU_COORDINATOR",
                                            patient_id="patient_b") for _ in range(3)))
        self.assertTrue(all(result.success for result in results))
        events = self.sim.events[start:]
        self.assertEqual(len(events), 3)
        self.assertLess(max(event.timestamp for event in events),
                        min(event.completed_at for event in events))
        conflicts = [v for v in self.sim.violations if v.violation_type == "concurrent_resource_use"]
        self.assertEqual(sum("ICU_BEDS" in v.message for v in conflicts), 1)
        self.assertEqual(sum("VENTILATOR" in v.message for v in conflicts), 2)
        self.assertTrue((await call("manage_icu", agent_id="ICU_COORDINATOR",
                                   patient_id="patient_b")).success)
        self.assertEqual(len(self.sim.violations), 3)

    async def test_all_90_image_interleavings_preserve_patient_data(self):
        orders = [order for order in itertools.permutations(range(6))
                  if all(order.index(first) < order.index(first + 1) for first in (0, 2, 4))]
        self.assertEqual(len(orders), 90)
        images = [("patient_a", "CT"), ("patient_b", "CT"), ("patient_b", "MRI")]
        for order in orders:
            with self.subTest(order=order):
                self.setUp()
                call = self.tools.call
                for agent, patient, tests in (
                        ("ER_DOCTOR_A", "patient_a", "trauma_panel,blood_type,coag"),
                        ("ER_DOCTOR_B", "patient_b", "stroke_panel,ct_angio")):
                    self.assertTrue((await call("assess_patient", agent_id=agent,
                                               patient_id=patient)).success)
                    self.assertTrue((await call("order_tests", agent_id=agent,
                                               patient_id=patient, test_type=tests)).success)
                for operation in order:
                    patient, modality = images[operation // 2]
                    tool = "perform_imaging" if operation % 2 == 0 else "interpret_imaging"
                    self.assertTrue((await call(tool, agent_id="RADIOLOGIST",
                                               patient_id=patient, modality=modality)).success)
                for agent, patient in (("ER_DOCTOR_A", "patient_a"), ("ER_DOCTOR_B", "patient_b")):
                    self.assertTrue((await call("diagnose", agent_id=agent, patient_id=patient)).success)
                self.assertTrue((await call("prepare_anesthesia", agent_id="ANESTHESIOLOGIST",
                                           patient_id="patient_a", type="caller data")).success)
                self.assertTrue((await call("perform_surgery", agent_id="SURGEON",
                                           patient_id="patient_a", procedure="trauma_surgery")).success)
                self.assertTrue((await call("manage_icu", agent_id="ICU_COORDINATOR",
                                           patient_id="patient_b")).success)
                self.assertTrue((await call("review_prescriptions", agent_id="PHARMACIST",
                                           patient_id="patient_A_and_B", medications="caller list")).success)
                self.assertTrue(self.sim.is_complete())
                self.assertEqual(len(self.sim.events), 16)
                self.assertEqual(self.sim.violations, [])
