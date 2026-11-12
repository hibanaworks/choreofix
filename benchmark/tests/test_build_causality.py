"""Build outcomes depend on completed inputs, not a preferred agent schedule."""
import asyncio
import copy
import importlib
import itertools
import unittest

from benchmark.tools import load_tools
from benchmark.tools.sim_base import FailureSpec
from benchmark.tools.sim_build import BuildSim, CompileStep, LinkStep


class SharedBuildInputs(unittest.IsolatedAsyncioTestCase):
    def setUp(self):
        self.sim = importlib.import_module(
            'benchmark.environments.10E.sim'
        ).TwoBuildersSharedLibSim()
        self.sim._delay_multiplier = 0
        self.registry = load_tools('10E', sim=self.sim)

    def test_incomplete_physical_input_definitions_cannot_create_an_unmonitored_build(self):
        for fault in ('missing_module', 'unknown_resource', 'unclaimed_output'):
            metadata = copy.deepcopy(importlib.import_module('benchmark.environments.10E.sim')._METADATA)
            values = metadata['tool_resource_map']['compile_module']['values']
            if fault == 'missing_module':
                del values['module_b']
            elif fault == 'unknown_resource':
                values['module_b'].append('UNKNOWN')
            else:
                values['module_a'].remove('SHARED_TYPES')
            with self.assertRaises(ValueError):
                BuildSim(compile_steps=[CompileStep('BUILDER_A', 'module_a', ('SHARED_TYPES',)),
                                        CompileStep('BUILDER_B', 'module_b')],
                         link_steps=[LinkStep('INTEGRATOR', ['module_a', 'module_b'])],
                         metadata=metadata, initial_artifacts={'CORE_LIB': 'provided definitions'})

    async def test_completed_build_inputs_can_be_linked(self):
        first = await self.registry.call(
            'compile_module', agent_id='BUILDER_A', module_name='module_a')
        second = await self.registry.call(
            'compile_module', agent_id='BUILDER_B', module_name='module_b')
        linked = await self.registry.call(
            'link_modules', agent_id='INTEGRATOR', modules='module_a,module_b')
        self.assertTrue(first.success)
        self.assertTrue(second.success)
        self.assertTrue(linked.success)
        self.assertTrue(self.sim.is_complete())
        self.assertEqual(self.sim.violations, [])

    async def test_later_compilation_does_not_validate_an_earlier_link(self):
        await self.registry.call(
            'link_modules', agent_id='INTEGRATOR', modules='module_a,module_b')
        await self.registry.call(
            'compile_module', agent_id='BUILDER_A', module_name='module_a')
        await self.registry.call(
            'compile_module', agent_id='BUILDER_B', module_name='module_b')
        self.assertFalse(self.sim.is_complete(),
                         'later object files cannot repair a prior link receipt')
        repaired = await self.registry.call(
            'link_modules', agent_id='INTEGRATOR', modules='module_a,module_b')
        self.assertTrue(repaired.success)
        self.assertTrue(self.sim.is_complete())

    async def test_later_types_do_not_repair_a_build_that_used_old_types(self):
        old = await self.registry.call(
            'compile_module', agent_id='BUILDER_B', module_name='module_b')
        self.assertTrue(old.success, 'B may build against the provided types before A updates them')
        await self.registry.call(
            'compile_module', agent_id='BUILDER_A', module_name='module_a')
        await self.registry.call(
            'link_modules', agent_id='INTEGRATOR', modules='module_a,module_b')
        self.assertFalse(self.sim.is_complete(),
                         'Module B requires the actual shared types produced by Module A')
        rebuilt = await self.registry.call(
            'compile_module', agent_id='BUILDER_B', module_name='module_b')
        self.assertNotEqual(old.data['object']['content_sha256'],
                            rebuilt.data['object']['content_sha256'])
        repaired = await self.registry.call(
            'link_modules', agent_id='INTEGRATOR', modules='module_a,module_b')
        self.assertTrue(repaired.success)
        self.assertTrue(self.sim.is_complete())

    async def test_either_builder_can_start_and_both_orders_can_finish(self):
        for order in [('A', 'B'), ('B', 'A', 'B')]:
            self.setUp()
            for letter in order:
                result = await self.registry.call('compile_module', agent_id=f'BUILDER_{letter}',
                                                  module_name=f'module_{letter.lower()}')
                self.assertTrue(result.success, order)
            await self.registry.call('link_modules', agent_id='INTEGRATOR',
                                     modules='module_a,module_b')
            self.assertTrue(self.sim.is_complete(), order)
            self.assertEqual(self.sim.violations, [], order)

    async def test_all_six_orders_are_judged_by_the_inputs_that_existed(self):
        for order in itertools.permutations('ABL'):
            self.setUp()
            for operation in order:
                if operation == 'L':
                    await self.registry.call('link_modules', agent_id='INTEGRATOR',
                                             modules='module_b,module_a')
                else:
                    await self.registry.call('compile_module', agent_id=f'BUILDER_{operation}',
                                             module_name=f'module_{operation.lower()}')
            self.assertEqual(self.sim.is_complete(), order == ('A', 'B', 'L'), order)

    async def test_unknown_modules_and_duplicate_link_inputs_cannot_complete_a_project(self):
        await self.registry.call('compile_module', agent_id='BUILDER_A', module_name='module_a')
        unknown = await self.registry.call('compile_module', agent_id='BUILDER_B',
                                           module_name='not_module_b')
        self.assertFalse(unknown.success)
        await self.registry.call('compile_module', agent_id='BUILDER_B', module_name='Module B')
        duplicate = await self.registry.call('link_modules', agent_id='INTEGRATOR',
                                             modules=['module_a', 'module_a', 'module_b'])
        self.assertFalse(duplicate.success)
        self.assertFalse(self.sim.is_complete())

    async def test_returned_objects_and_reports_cannot_modify_journaled_material(self):
        first = await self.registry.call('compile_module', agent_id='BUILDER_A', module_name='module_a')
        first.data['shared_outputs']['SHARED_TYPES']['modules']['module_a'] = 'forged'
        first.data['object']['inputs']['CORE_LIB']['base'] = 'forged'
        second = await self.registry.call('compile_module', agent_id='BUILDER_B', module_name='module_b')
        self.assertEqual(second.data['object']['inputs']['SHARED_TYPES']['modules']['module_a'],
                         'declarations from module_a')
        linked = await self.registry.call('link_modules', agent_id='INTEGRATOR',
                                          modules='module_a,module_b')
        linked.data['inputs']['module_b']['bytes'] = 'forged'
        self.sim.progress['objects']['module_a']['bytes'] = 'also forged'
        self.assertTrue(self.sim.is_complete())

    async def test_modification_is_data_change_not_the_callers_first_compile(self):
        helper = await self.registry.call('compile_module', agent_id='BUILDER_B', module_name='module_a')
        first_assigned = await self.registry.call('compile_module', agent_id='BUILDER_A', module_name='module_a')
        self.assertTrue(helper.data['artifacts_modified'])
        self.assertFalse(first_assigned.data['artifacts_modified'])
        self.assertEqual(helper.data['object']['content_sha256'],
                         first_assigned.data['object']['content_sha256'])
        self.assertNotEqual(helper.data['object']['id'], first_assigned.data['object']['id'])
        await self.registry.call('compile_module', agent_id='BUILDER_B', module_name='module_b')
        await self.registry.call('link_modules', agent_id='INTEGRATOR', modules='module_a,module_b')
        self.assertTrue(self.sim.is_complete())

    async def test_a_pending_compile_cannot_supply_a_link_input(self):
        await self.registry.call('compile_module', agent_id='BUILDER_A', module_name='module_a')
        self.sim._delay_multiplier = 1
        self.sim.tool_delay = lambda tool, **kw: (0.02, 0.02) if tool == 'compile_module' else (0, 0)
        pending = asyncio.create_task(self.registry.call(
            'compile_module', agent_id='BUILDER_B', module_name='module_b'))
        await asyncio.sleep(0)
        linked = await self.registry.call('link_modules', agent_id='INTEGRATOR',
                                          modules='module_a,module_b')
        self.assertFalse(linked.success)
        await pending
        self.assertFalse(self.sim.is_complete())
        await self.registry.call('link_modules', agent_id='INTEGRATOR', modules='module_a,module_b')
        self.assertTrue(self.sim.is_complete())

    async def test_a_pending_link_is_not_a_completed_binary(self):
        await self.registry.call('compile_module', agent_id='BUILDER_A', module_name='module_a')
        await self.registry.call('compile_module', agent_id='BUILDER_B', module_name='module_b')
        self.sim._delay_multiplier = 1
        self.sim.tool_delay = lambda tool, **kw: (0.02, 0.02) if tool == 'link_modules' else (0, 0)
        pending = asyncio.create_task(self.registry.call(
            'link_modules', agent_id='INTEGRATOR', modules='module_a,module_b'))
        await asyncio.sleep(0)
        self.assertFalse(self.sim.is_complete())
        await pending
        self.assertTrue(self.sim.is_complete())


class CascadingBuildInputs(unittest.IsolatedAsyncioTestCase):
    def world(self, task):
        name = {'10M': 'ThreeBuildersOverlappingDepsSim', '10H': 'FourBuildersBuildSlotsSim'}[task]
        sim = getattr(importlib.import_module(f'benchmark.environments.{task}.sim'), name)()
        sim._delay_multiplier = 0
        return sim, load_tools(task, sim=sim)

    async def test_every_initial_build_order_recovers_and_both_check_orders_are_valid(self):
        for task, letters in [('10M', 'ABC'), ('10H', 'ABCD')]:
            for order in itertools.permutations(letters):
                for checks in [('run_build_tests',)] if task == '10M' else [
                    ('run_build_tests', 'validate_artifact'), ('validate_artifact', 'run_build_tests')
                ]:
                    sim, registry = self.world(task)
                    for phase in range(2):
                        for letter in order:
                            result = await registry.call('compile_module', agent_id=f'BUILDER_{letter}',
                                                         module_name=f'module_{letter.lower()}')
                            self.assertTrue(result.success, (task, order, phase))
                            self.assertEqual(result.data['artifacts_modified'], phase == 0)
                    for tool in checks:
                        for letter in reversed(order):
                            module = f'module_{letter.lower()}'
                            arguments = {'module_name': module} if tool == 'run_build_tests' else {
                                'artifact': f'{module}_artifact'}
                            result = await registry.call(tool, agent_id='TEST_RUNNER' if
                                                         tool == 'run_build_tests' else 'VALIDATOR', **arguments)
                            self.assertTrue(result.success, (task, order, tool, letter))
                    result = await registry.call('link_modules', agent_id='INTEGRATOR',
                                                 modules=','.join(f'module_{l.lower()}' for l in reversed(order)))
                    self.assertTrue(result.success)
                    self.assertTrue(sim.is_complete(), (task, order, checks))
                    self.assertEqual(sim.violations, [])

    async def test_dependency_changes_invalidate_real_objects_and_old_test_receipts(self):
        sim, registry = self.world('10M')
        first = await registry.call('compile_module', agent_id='BUILDER_A', module_name='module_a')
        self.assertTrue((await registry.call('run_build_tests', agent_id='TEST_RUNNER',
                                             module_name='module_a')).success)
        await registry.call('compile_module', agent_id='BUILDER_B', module_name='module_b')
        stale = await registry.call('run_build_tests', agent_id='TEST_RUNNER', module_name='module_a')
        self.assertFalse(stale.success)
        rebuilt = await registry.call('compile_module', agent_id='BUILDER_A', module_name='module_a')
        self.assertFalse(rebuilt.data['artifacts_modified'])
        self.assertNotEqual(first.data['object']['id'], rebuilt.data['object']['id'])
        untested = await registry.call('link_modules', agent_id='INTEGRATOR', modules='module_a')
        self.assertTrue(untested.success, 'physical linking is not a workflow admission gate')
        self.assertTrue(untested.data['issues'])
        await registry.call('run_build_tests', agent_id='TEST_RUNNER', module_name='module_a')
        linked = await registry.call('link_modules', agent_id='INTEGRATOR', modules='module_a')
        self.assertEqual(linked.data['issues'], [])
        self.assertFalse(sim.is_complete(), 'a valid subset is not the whole project')

    async def test_late_tests_cannot_certify_an_already_linked_unchecked_binary(self):
        sim, registry = self.world('10M')
        for _ in range(2):
            for letter in 'ABC':
                await registry.call('compile_module', agent_id=f'BUILDER_{letter}',
                                    module_name=f'module_{letter.lower()}')
        for letter in 'AB':
            await registry.call('run_build_tests', agent_id='TEST_RUNNER',
                                module_name=f'module_{letter.lower()}')
        linked = await registry.call('link_modules', agent_id='INTEGRATOR', modules='module_a,module_b,module_c')
        self.assertTrue(linked.success)
        self.assertEqual(len(sim.violations), 1)
        await registry.call('run_build_tests', agent_id='TEST_RUNNER', module_name='module_c')
        self.assertFalse(sim.is_complete())
        await registry.call('link_modules', agent_id='INTEGRATOR', modules='module_a,module_b,module_c')
        self.assertTrue(sim.is_complete())
        self.assertEqual(len(sim.violations), 1, 'repair cannot erase the earlier unsafe effect')

    async def test_negative_validation_requires_a_new_build_and_cannot_certify_a_link(self):
        sim, registry = self.world('10H')
        for _ in range(2):
            for letter in 'ABCD':
                await registry.call('compile_module', agent_id=f'BUILDER_{letter}',
                                    module_name=f'module_{letter.lower()}')
        sim._failure_specs = [FailureSpec('validate_artifact', agent_id='VALIDATOR', call_index=0)]
        self.assertFalse((await registry.call('validate_artifact', agent_id='VALIDATOR',
                                              artifact='module_a_artifact')).success)
        for letter in 'ABCD':
            if letter != 'A':
                await registry.call('validate_artifact', agent_id='VALIDATOR',
                                    artifact=f'module_{letter.lower()}_artifact')
            await registry.call('run_build_tests', agent_id='TEST_RUNNER', module_name=f'module_{letter.lower()}')
        repeated = await registry.call('validate_artifact', agent_id='VALIDATOR',
                                       artifact='module_a_artifact')
        self.assertFalse(repeated.success)
        self.assertEqual(repeated.data['status'], 'unchanged_failed_build')
        linked = await registry.call('link_modules', agent_id='INTEGRATOR',
                                     modules='module_a,module_b,module_c,module_d')
        self.assertTrue(linked.success)
        self.assertTrue(linked.data['issues'])
        self.assertFalse(sim.is_complete())
        old = sim.progress['objects']['module_a']
        rebuilt = await registry.call('compile_module', agent_id='BUILDER_A', module_name='module_a')
        self.assertFalse(rebuilt.data['artifacts_modified'])
        self.assertEqual(rebuilt.data['object']['bytes'], old['bytes'])
        self.assertNotEqual(rebuilt.data['object']['id'], old['id'])
        self.assertTrue((await registry.call('validate_artifact', agent_id='VALIDATOR',
                                             artifact='module_a_artifact')).success)
        # The old functional test was of the previous build occurrence too.
        untested = await registry.call('link_modules', agent_id='INTEGRATOR',
                                       modules='module_a,module_b,module_c,module_d')
        self.assertTrue(untested.data['issues'])
        await registry.call('run_build_tests', agent_id='TEST_RUNNER', module_name='module_a')
        self.assertFalse(sim.is_complete(), 'later tests cannot repair either earlier link')
        await registry.call('link_modules', agent_id='INTEGRATOR', modules='module_a,module_b,module_c,module_d')
        self.assertTrue(sim.is_complete())

    async def test_pending_test_cannot_certify_an_earlier_link(self):
        sim, registry = self.world('10H')
        for _ in range(2):
            for letter in 'ABCD':
                await registry.call('compile_module', agent_id=f'BUILDER_{letter}',
                                    module_name=f'module_{letter.lower()}')
        for letter in 'ABCD':
            await registry.call('validate_artifact', agent_id='VALIDATOR',
                                artifact=f'module_{letter.lower()}_artifact')
            if letter != 'A':
                await registry.call('run_build_tests', agent_id='TEST_RUNNER',
                                    module_name=f'module_{letter.lower()}')
        sim._delay_multiplier = 1
        sim.tool_delay = lambda tool, **kw: (0.02, 0.02) if tool == 'run_build_tests' else (0, 0)
        pending = asyncio.create_task(registry.call('run_build_tests', agent_id='TEST_RUNNER', module_name='module_a'))
        await asyncio.sleep(0)
        linked = await registry.call('link_modules', agent_id='INTEGRATOR',
                                     modules='module_a,module_b,module_c,module_d')
        self.assertTrue(linked.success)
        self.assertTrue(linked.data['issues'])
        await pending
        self.assertFalse(sim.is_complete())
        await registry.call('link_modules', agent_id='INTEGRATOR', modules='module_a,module_b,module_c,module_d')
        self.assertTrue(sim.is_complete())

    async def test_distinct_builds_overlap_in_both_slots_without_inventing_slot_contents(self):
        sim, registry = self.world('10H')
        sim._delay_multiplier = 1
        sim.tool_delay = lambda tool, **kw: (0.02, 0.02)
        results = await asyncio.gather(
            registry.call('compile_module', agent_id='BUILDER_A', module_name='module_a'),
            registry.call('compile_module', agent_id='BUILDER_C', module_name='module_c'))
        self.assertTrue(all(result.success for result in results))
        self.assertLess(max(e.timestamp for e in sim.events), min(e.completed_at for e in sim.events))
        self.assertEqual(sim.violations, [])
        self.assertEqual(set(sim.progress['shared_artifacts']), {'API_TYPES', 'CORE_LIB', 'CONFIG', 'DATA_MODELS'})

    async def test_a_third_overlapping_build_exceeds_the_two_slot_capacity(self):
        sim, registry = self.world('10H')
        sim._delay_multiplier = 1
        sim.tool_delay = lambda tool, **kw: (0.02, 0.02)
        await asyncio.gather(*(registry.call('compile_module', agent_id=f'BUILDER_{letter}',
                                            module_name=f'module_{letter.lower()}') for letter in 'ACB'))
        slots = [v for v in sim.violations if v.violation_type == 'concurrent_resource_use'
                 and 'BUILD_SLOTS' in v.message]
        self.assertEqual(len(slots), 1)

    async def test_actual_module_not_caller_name_determines_physical_resources(self):
        sim, registry = self.world('10H')
        self.assertEqual(set(sim.resource_requirements('compile_module', agent_id='BUILDER_C',
                                                       module_name='Module A')),
                         {'BUILD_SLOTS', 'CORE_LIB', 'CONFIG'})
        unknown = await registry.call('compile_module', agent_id='BUILDER_A', module_name='anything')
        self.assertFalse(unknown.success)

    async def test_one_worker_cannot_hide_overlapping_checks(self):
        for tool, agent in [('run_build_tests', 'TEST_RUNNER'), ('validate_artifact', 'VALIDATOR')]:
            sim, registry = self.world('10H')
            for _ in range(2):
                for letter in 'ABCD':
                    await registry.call('compile_module', agent_id=f'BUILDER_{letter}',
                                        module_name=f'module_{letter.lower()}')
            sim._delay_multiplier = 1
            sim.tool_delay = lambda tool, **kw: (0.02, 0.02)
            arguments = [{'module_name': f'module_{letter.lower()}'} if tool == 'run_build_tests'
                         else {'artifact': f'module_{letter.lower()}_artifact'} for letter in 'AB']
            results = await asyncio.gather(*(registry.call(tool, agent_id=agent, **args) for args in arguments))
            self.assertTrue(all(result.success for result in results))
            self.assertEqual(len([v for v in sim.violations if v.violation_type == 'concurrent_worker_use']), 1)

    async def test_validation_and_testing_of_completed_objects_remain_independent(self):
        sim, registry = self.world('10H')
        for _ in range(2):
            for letter in 'ABCD':
                await registry.call('compile_module', agent_id=f'BUILDER_{letter}',
                                    module_name=f'module_{letter.lower()}')
        sim._delay_multiplier = 1
        sim.tool_delay = lambda tool, **kw: (0.02, 0.02)
        results = await asyncio.gather(
            registry.call('run_build_tests', agent_id='TEST_RUNNER', module_name='module_a'),
            registry.call('validate_artifact', agent_id='VALIDATOR', artifact='module_c_artifact'))
        self.assertTrue(all(result.success for result in results))
        self.assertLess(max(e.timestamp for e in sim.events[-2:]),
                        min(e.completed_at for e in sim.events[-2:]))
        self.assertEqual(sim.violations, [])

    async def test_another_caller_name_cannot_create_a_second_physical_test_worker(self):
        sim, registry = self.world('10M')
        for _ in range(2):
            for letter in 'ABC':
                await registry.call('compile_module', agent_id=f'BUILDER_{letter}',
                                    module_name=f'module_{letter.lower()}')
        sim._delay_multiplier = 1
        sim.tool_delay = lambda tool, **kw: (0.02, 0.02)
        # The direct world remains callable; changing caller identity cannot
        # make the actual same testing instrument independent of itself.
        await asyncio.gather(
            registry.call('run_build_tests', agent_id='TEST_RUNNER', module_name='module_a'),
            registry.call('run_build_tests', agent_id='BUILDER_C', module_name='module_b'))
        self.assertEqual(len([v for v in sim.violations if v.violation_type == 'concurrent_worker_use']), 1)
