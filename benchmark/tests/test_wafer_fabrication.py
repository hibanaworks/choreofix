"""15E uses the same five physical tools and scoring rules for every method."""

import asyncio
import importlib
import itertools
import unittest

from benchmark.tools import load_tools
from benchmark.tools.sim_base import FailureSpec

Fab = importlib.import_module("benchmark.environments.15E.sim").SemiconductorFabSim
LITHO, DEP, ETCH = "LITHOGRAPHER", "DEPOSITOR", "ETCHER"
METRO, CONTROL, INTEGRATION = "METROLOGY_ENGINEER", "PROCESS_CONTROLLER", "INTEGRATION_ENGINEER"

# Public inputs from the task. The assertion side does not read the scorer's rules.
EXPOSE = ("run_stepper_exposure", LITHO, "process", "photo_exposure")
DOSE = ("check_monitoring", LITHO, "check_type", "exposure_dose")
PATTERN = ("run_etch_process", LITHO, "process", "pattern_transfer_etch")
ENDPOINT = ("check_monitoring", LITHO, "check_type", "etch_endpoint")
GROW = ("run_deposition", DEP, "process", "cvd_growth")
RATE = ("check_monitoring", DEP, "check_type", "deposition_rate")
REACTIVE = ("run_etch_process", ETCH, "process", "reactive_ion_etch")
ETCH_RATE = ("check_monitoring", ETCH, "check_type", "etch_rate")
FABRICATION = ((EXPOSE, DOSE, PATTERN, ENDPOINT), (GROW, RATE), (REACTIVE, ETCH_RATE))
REPORTS = tuple(("run_metrology_measurement", METRO, "measurement", name)
                for name in ("cd_measurement", "thickness_measurement", "profile_measurement"))
VALIDATE = ("check_monitoring", CONTROL, "check_type", "cross_reference")
ALIGN = ("run_metrology_measurement", LITHO, "measurement", "alignment_check")
VERIFY = ("run_metrology_measurement", DEP, "measurement", "thickness_verification")
MOUNT = ("run_stepper_exposure", INTEGRATION, "process", "overlay_alignment")
OVERLAY = ("run_metrology_measurement", INTEGRATION, "measurement", "overlay_alignment")
INTEGRITY = ("run_deposition", INTEGRATION, "process", "film_integrity_check")
UNIFORMITY = ("run_etch_process", INTEGRATION, "process", "etch_uniformity_check")
FINISH = (VALIDATE, MOUNT, OVERLAY, INTEGRITY, UNIFORMITY, VERIFY, ALIGN)


class WaferFabrication(unittest.IsolatedAsyncioTestCase):
    def setUp(self):
        self.sim = Fab()
        self.sim._delay_multiplier = 0
        self.tools = load_tools("15E", sim=self.sim)

    async def call(self, step):
        tool, actor, field, value = step
        return await self.tools.call(tool, agent_id=actor, **{field: value})

    async def run_steps(self, steps):
        replies = []
        for step in steps:
            reply = await self.call(step)
            self.assertTrue(reply.success, (step, reply))
            replies.append(reply.data["receipt"])
        return replies

    async def test_every_order_of_fabrication_and_measurements(self):
        for ready in itertools.permutations(FABRICATION):
            for read in itertools.permutations(REPORTS):
                self.setUp()
                for group in ready:
                    await self.run_steps(group)
                receipts = dict(zip(read, await self.run_steps(read)))
                await self.run_steps(FINISH)
                validation = next(e for e in self.sim.events if e.agent == CONTROL)
                self.assertEqual(validation.result["inputs"], [receipts[r] for r in REPORTS])
                self.assertTrue(self.sim.is_complete(), (ready, read))
                self.assertEqual(self.sim.violations, [])

    async def test_sample_can_be_remeasured_before_other_wafers_start(self):
        await self.run_steps(FABRICATION[1])
        first, second = await self.run_steps((REPORTS[1], REPORTS[1]))
        self.assertNotEqual(first, second)
        for index in (2, 0):
            await self.run_steps((*FABRICATION[index], REPORTS[index]))
        await self.run_steps((VALIDATE, ALIGN, VERIFY, MOUNT, OVERLAY, INTEGRITY, UNIFORMITY))
        self.assertTrue(self.sim.is_complete())
        self.assertIn(second, next(e for e in self.sim.events if e.agent == CONTROL).result["inputs"])
        self.assertEqual(self.sim.violations, [])

    async def test_unassigned_inputs_and_reversed_calls_do_not_complete_a_lot(self):
        steps = tuple(itertools.chain.from_iterable(FABRICATION)) + REPORTS + FINISH
        for tool, actor, field, _ in steps:
            self.assertFalse((await self.call((tool, actor, field, "unassigned"))).success)
        wrong_actor = (EXPOSE[0], DEP, EXPOSE[2], EXPOSE[3])
        self.assertFalse((await self.call(wrong_actor)).success)
        self.assertFalse(self.sim.is_complete())
        self.setUp()
        for step in reversed(steps):
            await self.call(step)
        self.assertFalse(self.sim.is_complete())
        self.assertFalse(self.sim.progress["monitoring_checks"]["PROCESS_CONTROLLER_cross_reference"])

    async def test_same_etch_chamber_cannot_be_reused_until_monitoring_finishes(self):
        await self.run_steps((EXPOSE, DOSE, PATTERN))
        self.assertEqual((await self.call(REACTIVE)).data["status"], "instrument_still_loaded")
        await self.run_steps((GROW, RATE, REPORTS[1]))
        self.sim._delay_multiplier = 0.04
        checking = asyncio.create_task(self.call(ENDPOINT))
        await asyncio.sleep(0)
        self.assertIsNone(self.sim.events[-1].completed_at)
        self.sim._delay_multiplier = 0
        self.assertEqual((await self.call(REACTIVE)).data["status"], "instrument_still_loaded")
        self.assertFalse((await self.call(REPORTS[0])).success)
        self.assertTrue((await checking).success)
        await self.run_steps((REACTIVE, ETCH_RATE, REPORTS[0], REPORTS[2], *FINISH))
        self.assertTrue(self.sim.is_complete())
        self.assertEqual(len(self.sim.violations), 3)

    async def test_in_situ_checks_can_overlap_processing_on_different_equipment(self):
        self.sim._delay_multiplier = 0.05
        growing = asyncio.create_task(self.call(GROW))
        await asyncio.sleep(0)
        self.assertIsNone(self.sim.events[-1].completed_at)
        self.sim._delay_multiplier = 0
        await self.run_steps((EXPOSE, RATE, DOSE))
        self.assertFalse((await self.call(REPORTS[1])).success)
        self.assertTrue((await growing).success)
        await self.run_steps((REPORTS[1], REACTIVE, ETCH_RATE, REPORTS[2],
                              PATTERN, ENDPOINT, REPORTS[0], *FINISH))
        self.assertTrue(self.sim.is_complete())
        self.assertEqual([v.violation_type for v in self.sim.violations], ["missing_wafer_result"])

    async def test_two_loaded_wafers_share_one_monitor_in_either_order(self):
        for first, second in itertools.permutations((RATE, DOSE)):
            with self.subTest(first=first[1]):
                self.setUp()
                await self.run_steps((GROW, EXPOSE, first, second))
                self.assertEqual(self.sim.violations, [])

                self.setUp()
                await self.run_steps((GROW, EXPOSE))
                self.sim._delay_multiplier = 0.05
                sampling = asyncio.create_task(self.call(first))
                await asyncio.sleep(0)
                self.sim._delay_multiplier = 0
                self.assertIsNone(self.sim.events[-1].completed_at)
                self.assertTrue((await self.call(second)).success)
                self.assertFalse(sampling.done())
                self.assertTrue((await sampling).success)
                self.assertEqual([(v.agent, v.tool, v.violation_type)
                                  for v in self.sim.violations],
                                 [(second[1], second[0], "concurrent_resource_use")])

    async def test_new_material_invalidates_only_its_dependent_evidence(self):
        for group in FABRICATION:
            await self.run_steps(group)
        await self.run_steps((*REPORTS, *FINISH))
        self.assertTrue(self.sim.is_complete())
        await self.run_steps((GROW, RATE))
        self.assertFalse(self.sim.is_complete())
        measurements = self.sim.progress["metrology_measurements"]
        self.assertTrue(measurements["METROLOGY_ENGINEER_cd_measurement"])
        self.assertTrue(measurements["METROLOGY_ENGINEER_profile_measurement"])
        self.assertFalse(measurements["METROLOGY_ENGINEER_thickness_measurement"])
        self.assertFalse((await self.call(VALIDATE)).success)
        await self.run_steps((REPORTS[1], *FINISH))
        self.assertTrue(self.sim.is_complete())

    async def test_either_spot_check_can_go_first_on_the_same_reference_wafer(self):
        for first, second in itertools.permutations((INTEGRITY, UNIFORMITY)):
            self.setUp()
            for group in FABRICATION:
                await self.run_steps(group)
            await self.run_steps((*REPORTS, VALIDATE, ALIGN, VERIFY))
            mount, overlay = await self.run_steps((MOUNT, OVERLAY))
            self.sim._delay_multiplier = 0.03
            checking = asyncio.create_task(self.call(first))
            await asyncio.sleep(0)
            self.sim._delay_multiplier = 0
            self.assertEqual((await self.call(second)).data["status"], "reference_wafer_busy")
            self.assertEqual((await self.call(MOUNT)).data["status"], "reference_wafer_busy")
            self.assertEqual((await checking).data["inputs"], [mount, overlay])
            self.assertFalse(self.sim.is_complete())
            self.assertEqual((await self.call(second)).data["inputs"], [mount, overlay])
            self.assertTrue(self.sim.is_complete())
            self.assertEqual([v.violation_type for v in self.sim.violations], ["reference_wafer_busy"] * 2)

    async def test_failed_in_situ_reading_can_be_retried_on_the_loaded_wafer(self):
        self.sim._failure_specs.append(FailureSpec(tool_name=RATE[0], agent_id=DEP, call_index=0))
        load, = await self.run_steps((GROW,))
        failed = await self.call(RATE)
        self.assertFalse(failed.success)
        self.assertEqual(failed.data["status"], "check_failed")
        retried = await self.call(RATE)
        self.assertTrue(retried.success)
        self.assertEqual(retried.data["inputs"], [load])
        await self.run_steps((REPORTS[1],))
        self.assertEqual(self.sim.violations, [])
