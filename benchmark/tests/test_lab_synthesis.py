"""13E material causality and independent schedules in the common world."""
import asyncio
import importlib
import itertools
import unittest

from benchmark.tools import load_tools

RAW=('run_analysis','CHEMIST',{'sample':'raw_materials'})
CELLS=('run_separation','BIOLOGIST',{'material':'cell_cultures'})
PURIFY=('run_separation','CHEMIST',{'material':'compound_purification'})
CALIBRATE=('run_analysis','BIOLOGIST',{'sample':'fluorescence'})
ASSAY=('run_bioassay','BIOLOGIST',{'compound':'compound_purification'})
MOLECULAR=('run_analysis','ANALYST',{'sample':'molecular_detail'})
VERIFY=('run_separation','ANALYST',{'material':'verification'})
FINAL=('run_analysis','CHEMIST',{'sample':'final_validation'})


class LabWorld(unittest.IsolatedAsyncioTestCase):
    def setUp(self):
        self.sim=importlib.import_module('benchmark.environments.13E.sim').PharmaLabSim()
        self.sim._delay_multiplier=0
        self.registry=load_tools('13E',sim=self.sim)

    async def call(self, call):
        tool,actor,args=call
        return await self.registry.call(tool,agent_id=actor,**args)

    async def test_all_24_causal_orders_and_optional_culture_data(self):
        count=0
        for prefix in itertools.permutations([RAW,CELLS,PURIFY,CALIBRATE]):
            if prefix.index(RAW)>prefix.index(PURIFY) or prefix.index(CELLS)>prefix.index(CALIBRATE):
                continue
            for quality in itertools.permutations([MOLECULAR,VERIFY]):
                for cell_type in [None,'primary_cells']:
                    self.setUp()
                    assay=ASSAY if cell_type is None else (ASSAY[0],ASSAY[1],{**ASSAY[2],'cell_type':cell_type})
                    for call in [*prefix,assay,*quality,FINAL]:
                        self.assertTrue((await self.call(call)).success,call)
                    self.assertTrue(self.sim.is_complete());self.assertEqual(self.sim.violations,[])
                    count+=1
        self.assertEqual(count,24)

    async def test_missing_each_required_input_prevents_final_validation(self):
        calls=[RAW,CELLS,PURIFY,CALIBRATE,ASSAY,MOLECULAR,VERIFY]
        for missing in range(len(calls)):
            self.setUp()
            for index,call in enumerate(calls):
                if index!=missing:
                    await self.call(call)
            self.assertFalse((await self.call(FINAL)).success,calls[missing])
            self.assertFalse(self.sim.is_complete())
            for call in calls:
                self.assertTrue((await self.call(call)).success)
            self.assertFalse(self.sim.is_complete())
            self.assertTrue((await self.call(FINAL)).success)
            self.assertTrue(self.sim.is_complete());self.assertEqual(self.sim.violations,[])

    async def test_old_quality_receipts_do_not_cover_a_new_compound(self):
        for call in [RAW,CELLS,PURIFY,CALIBRATE,ASSAY,MOLECULAR,VERIFY]:
            self.assertTrue((await self.call(call)).success)
        await self.call(RAW)
        self.assertFalse((await self.call(FINAL)).success)
        for call in [PURIFY,ASSAY,MOLECULAR,VERIFY,FINAL]:
            self.assertTrue((await self.call(call)).success)
        self.assertTrue(self.sim.is_complete())

    async def test_quality_receipts_must_cover_the_same_assay_occurrence(self):
        for call in [RAW,CELLS,PURIFY,CALIBRATE,ASSAY,MOLECULAR,ASSAY,VERIFY]:
            self.assertTrue((await self.call(call)).success)
        self.assertFalse((await self.call(FINAL)).success)
        await self.call(MOLECULAR)
        self.assertTrue((await self.call(FINAL)).success)

    async def test_final_validation_cannot_consume_pending_quality_results(self):
        for call in [RAW,CELLS,PURIFY,CALIBRATE,ASSAY,MOLECULAR]:
            await self.call(call)
        self.sim._delay_multiplier=1
        self.sim.tool_delay=lambda tool,**kwargs:(0.02,0.02) if tool=='run_separation' else (0,0)
        pending=asyncio.create_task(self.call(VERIFY))
        await asyncio.sleep(0)
        self.assertFalse((await self.call(FINAL)).success)
        await pending
        self.assertFalse(self.sim.is_complete())
        self.assertTrue((await self.call(FINAL)).success)
        self.assertTrue(self.sim.is_complete());self.assertEqual(self.sim.violations,[])

    async def test_distinct_instruments_really_overlap(self):
        self.sim._delay_multiplier=0.01
        results=await asyncio.gather(self.call(RAW),self.call(CELLS))
        self.assertTrue(all(r.success and r.delay_seconds>0 for r in results))
        self.assertLess(max(e.timestamp for e in self.sim.events),min(e.completed_at for e in self.sim.events))
        self.assertEqual(self.sim.violations,[])

    async def test_same_instrument_conflict_is_not_silently_serialized(self):
        self.sim._delay_multiplier=0.01
        results=await asyncio.gather(self.call(RAW),self.call(RAW))
        self.assertTrue(all(r.success for r in results))
        self.assertIn('concurrent_resource_use',{v.violation_type for v in self.sim.violations})

    async def test_unknown_or_malformed_targets_do_not_create_materials(self):
        for sample in ['missing_sample',[],{}]:
            self.assertFalse((await self.registry.call('run_analysis',agent_id='CHEMIST',sample=sample)).success)
        self.assertFalse((await self.call(ASSAY)).success)
        self.assertFalse(self.sim.is_complete())
