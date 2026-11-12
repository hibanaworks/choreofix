"""Kitchen world inputs and completed material causality, independent of EPF."""
import asyncio
import importlib
import itertools
import unittest

from benchmark.tools import load_tools
from benchmark.tools.sim_base import FailureSpec


CHEFS = ('CHEF_A', 'CHEF_B', 'CHEF_C')
DISHES = ('appetizer', 'main_course', 'dessert')
INGREDIENTS = ('sauce', 'garnish', 'glaze')
RECIPIENTS = ('CHEF_C', 'CHEF_A', 'CHEF_B')
RECEIVED = ('garnish', 'glaze', 'sauce')
WORLD_TYPES = (('12E', 'KitchenBasicSim'), ('12M', 'KitchenAdvancedSim'))


def serial_orders(prefix=(), remaining=tuple(range(9))):
    """All nine-effect orders satisfying only each dish's two material inputs."""
    inputs = {6: (0, 4), 7: (1, 5), 8: (2, 3)}
    if not remaining:
        yield prefix
    for index in remaining:
        if all(value in prefix for value in inputs.get(index, ())):
            yield from serial_orders(prefix + (index,),
                                     tuple(value for value in remaining if value != index))


class KitchenWorld(unittest.IsolatedAsyncioTestCase):
    def world(self, task='12E', name='KitchenBasicSim'):
        sim = getattr(importlib.import_module(f'benchmark.environments.{task}.sim'), name)()
        sim._delay_multiplier = 0
        return sim, load_tools(task, sim=sim)

    async def operation(self, registry, index):
        chef = CHEFS[index % 3]
        if index < 3:
            return await registry.call('prepare_base_dish', agent_id=chef,
                                       dish=DISHES[index])
        if index < 6:
            return await registry.call('prepare_ingredient', agent_id=chef,
                                       ingredient=INGREDIENTS[index - 3],
                                       for_chef=RECIPIENTS[index - 3])
        return await registry.call('combine_dish', agent_id=chef,
                                   dish=DISHES[index - 6], ingredient=RECEIVED[index - 6])

    async def test_wrong_dishes_cannot_make_an_assigned_base(self):
        for task, name in WORLD_TYPES:
            sim, registry = self.world(task, name)
            for index, chef in enumerate(CHEFS):
                for dish in (*DISHES, 'unknown'):
                    if dish != DISHES[index]:
                        result = await registry.call('prepare_base_dish', agent_id=chef, dish=dish)
                        self.assertFalse(result.success, (task, chef, dish))
                self.assertFalse(sim.progress['base_prepared'][chef])

    async def test_ingredient_is_bound_to_its_producer_and_recipient(self):
        for task, name in WORLD_TYPES:
            sim, registry = self.world(task, name)
            for index, ingredient in enumerate(INGREDIENTS):
                for chef in CHEFS:
                    for recipient in (*CHEFS, 'unknown'):
                        if (chef, recipient) == (CHEFS[index], RECIPIENTS[index]):
                            continue
                        result = await registry.call('prepare_ingredient', agent_id=chef,
                                                     ingredient=ingredient, for_chef=recipient)
                        self.assertFalse(result.success, (task, chef, ingredient, recipient))
                self.assertFalse(sim.progress['ingredient_available'][ingredient])
            self.assertFalse((await registry.call('prepare_ingredient', agent_id='CHEF_A',
                                                 ingredient='unknown', for_chef='CHEF_C')).success)

    async def test_combination_requires_the_exact_assigned_dish_and_ingredient(self):
        for task, name in WORLD_TYPES:
            sim, registry = self.world(task, name)
            for index in range(6):
                await self.operation(registry, index)
            for index, chef in enumerate(CHEFS):
                for dish in (*DISHES, 'unknown'):
                    for ingredient in (*INGREDIENTS, 'unknown'):
                        if (dish, ingredient) == (DISHES[index], RECEIVED[index]):
                            continue
                        self.assertFalse((await registry.call('combine_dish', agent_id=chef,
                                                              dish=dish, ingredient=ingredient)).success)
                self.assertFalse(sim.progress['dish_completed'][chef])

    async def test_unknown_chef_cannot_create_material_or_a_completed_dish(self):
        for task, name in WORLD_TYPES:
            sim, registry = self.world(task, name)
            for tool, arguments in (
                ('prepare_base_dish', {'dish': 'appetizer'}),
                ('prepare_ingredient', {'ingredient': 'sauce', 'for_chef': 'CHEF_C'}),
                ('combine_dish', {'dish': 'appetizer', 'ingredient': 'garnish'}),
            ):
                self.assertFalse((await registry.call(tool, agent_id='unknown', **arguments)).success)
            self.assertFalse(sim.is_complete())

    async def test_malformed_json_values_return_failure_without_crashing_the_world(self):
        for task, name in (*WORLD_TYPES, ('12H', 'KitchenMultiStepSim')):
            sim, registry = self.world(task, name)
            calls = [('prepare_ingredient', {'ingredient': 'sauce', 'for_chef': 'CHEF_C'})]
            if task == '12H':
                calls += [('prep_mise_en_place', {'dish': 'appetizer'}),
                          ('cook_in_oven', {'item': 'appetizer_base'}),
                          ('plate_dish', {'dish': 'appetizer', 'ingredient': 'garnish'})]
            else:
                calls += [('prepare_base_dish', {'dish': 'appetizer'}),
                          ('combine_dish', {'dish': 'appetizer', 'ingredient': 'garnish'})]
            for tool, arguments in calls:
                for key in arguments:
                    for value in (None, False, 1, [], {}):
                        bad = {**arguments, key: value}
                        self.assertFalse((await registry.call(tool, agent_id='CHEF_A', **bad)).success,
                                         (task, tool, bad))
            self.assertFalse(sim.is_complete())

    async def test_each_missing_material_is_unavailable_and_can_be_retried(self):
        for task, name in WORLD_TYPES:
            for chef in range(3):
                for first in (chef, 3 + (chef + 1) % 3):
                    sim, registry = self.world(task, name)
                    self.assertFalse((await self.operation(registry, 6 + chef)).success)
                    await self.operation(registry, first)
                    self.assertFalse((await self.operation(registry, 6 + chef)).success)
                    other = 3 + (chef + 1) % 3 if first == chef else chef
                    await self.operation(registry, other)
                    self.assertTrue((await self.operation(registry, 6 + chef)).success)
                    self.assertEqual(sim.violations, [])

    async def test_pending_base_is_not_a_completed_material(self):
        for task, name in WORLD_TYPES:
            for chef in range(3):
                sim, registry = self.world(task, name)
                await self.operation(registry, 3 + (chef + 1) % 3)
                sim.tool_delay = lambda tool, **kwargs: (0.02, 0.02) if tool == 'prepare_base_dish' else (0, 0)
                sim._delay_multiplier = 1
                pending = asyncio.create_task(self.operation(registry, chef))
                await asyncio.sleep(0)
                self.assertFalse(sim.progress['base_prepared'][CHEFS[chef]])
                self.assertFalse((await self.operation(registry, 6 + chef)).success)
                await pending
                self.assertTrue((await self.operation(registry, 6 + chef)).success)
                self.assertEqual(sim.violations, [])

    async def test_pending_ingredient_is_not_a_completed_material(self):
        for task, name in WORLD_TYPES:
            for chef in range(3):
                sim, registry = self.world(task, name)
                await self.operation(registry, chef)
                sim.tool_delay = lambda tool, **kwargs: (0.02, 0.02) if tool == 'prepare_ingredient' else (0, 0)
                sim._delay_multiplier = 1
                pending = asyncio.create_task(self.operation(registry, 3 + (chef + 1) % 3))
                await asyncio.sleep(0)
                self.assertFalse((await self.operation(registry, 6 + chef)).success)
                await pending
                self.assertTrue((await self.operation(registry, 6 + chef)).success)
                self.assertEqual(sim.violations, [])

    async def test_task_completion_waits_for_the_final_combination_result(self):
        for task, name in WORLD_TYPES:
            sim, registry = self.world(task, name)
            for index in range(8):
                await self.operation(registry, index)
            sim.tool_delay = lambda tool, **kwargs: (0.02, 0.02)
            sim._delay_multiplier = 1
            pending = asyncio.create_task(self.operation(registry, 8))
            await asyncio.sleep(0)
            self.assertFalse(sim.is_complete())
            await pending
            self.assertTrue(sim.is_complete())

    async def test_all_material_causal_serial_orders_are_accepted(self):
        count = 0
        for order in serial_orders():
            for task, name in WORLD_TYPES:
                sim, registry = self.world(task, name)
                for index in order:
                    self.assertTrue((await self.operation(registry, index)).success, (task, order))
                self.assertTrue(sim.is_complete(), (task, order))
                self.assertEqual(sim.violations, [], (task, order))
            count += 1
        self.assertEqual(count, 13_440)

    async def test_base_and_equipment_free_ingredient_can_really_overlap(self):
        sim, registry = self.world()
        sim.tool_delay = lambda *args, **kwargs: (0.02, 0.02)
        sim._delay_multiplier = 1
        results = await asyncio.gather(self.operation(registry, 0), self.operation(registry, 4))
        self.assertTrue(all(result.success for result in results))
        self.assertLess(max(e.timestamp for e in sim.events), min(e.completed_at for e in sim.events))
        self.assertEqual(sim.violations, [])

    async def test_same_equipment_conflict_is_observed_without_serializing_calls(self):
        for task, name in WORLD_TYPES:
            sim, registry = self.world(task, name)
            sim.tool_delay = lambda *args, **kwargs: (0.02, 0.02)
            sim._delay_multiplier = 1
            results = await asyncio.gather(self.operation(registry, 0), self.operation(registry, 2))
            self.assertTrue(all(result.success for result in results))
            self.assertLess(max(e.timestamp for e in sim.events), min(e.completed_at for e in sim.events))
            conflicts = [v for v in sim.violations if v.violation_type == 'concurrent_resource_use']
            self.assertEqual(len(conflicts), 1)
            self.assertIn('OVEN', conflicts[0].message)
            for index in (1, 3, 4, 5, 6, 7, 8):
                await self.operation(registry, index)
            self.assertTrue(sim.is_complete())
            self.assertEqual(sim.violations, conflicts)

    async def test_advanced_ingredient_equipment_is_observed_but_not_locked(self):
        sim, registry = self.world('12M', 'KitchenAdvancedSim')
        sim.tool_delay = lambda *args, **kwargs: (0.02, 0.02)
        sim._delay_multiplier = 1
        results = await asyncio.gather(self.operation(registry, 0), self.operation(registry, 4))
        self.assertTrue(all(result.success for result in results))
        self.assertEqual({v.violation_type for v in sim.violations}, {'concurrent_resource_use'})
        self.assertTrue(any('STOVETOP' in v.message for v in sim.violations))

    async def test_returned_results_cannot_forge_the_world_journal(self):
        sim, registry = self.world()
        base = await self.operation(registry, 0)
        ingredient = await self.operation(registry, 4)
        base.data['dish'] = 'dessert'
        ingredient.data['for_chef'] = 'CHEF_C'
        combined = await self.operation(registry, 6)
        self.assertTrue(combined.success)
        self.assertEqual(sim.events[0].result['dish'], 'appetizer')
        self.assertEqual(sim.events[1].result['for_chef'], 'CHEF_A')
        combined.data['dish'] = 'unknown'
        self.assertEqual(sim.events[-1].result['dish'], 'appetizer')

    async def test_completed_combination_names_its_actual_input_occurrences(self):
        sim, registry = self.world()
        await self.operation(registry, 0)
        await self.operation(registry, 4)
        result = await self.operation(registry, 6)
        self.assertEqual(result.data['base_event'], 0)
        self.assertEqual(result.data['ingredient_event'], 1)
        for index in (result.data['base_event'], result.data['ingredient_event']):
            self.assertLessEqual(sim.events[index].completed_at, sim.events[-1].timestamp)

    async def test_existing_multistep_success_path_remains_available(self):
        sim, registry = self.world('12H', 'KitchenMultiStepSim')
        sim.set_scenario_depth(0)
        for chef, dish in zip(CHEFS, DISHES):
            self.assertTrue((await registry.call('prep_mise_en_place', agent_id=chef, dish=dish)).success)
            self.assertTrue((await registry.call('cook_in_oven', agent_id=chef, item=f'{dish}_base')).success)
        for chef, ingredient, recipient in zip(CHEFS, INGREDIENTS, RECIPIENTS):
            self.assertTrue((await registry.call('prepare_ingredient', agent_id=chef,
                                                 ingredient=ingredient, for_chef=recipient)).success)
        for chef, dish, ingredient in zip(CHEFS, DISHES, RECEIVED):
            self.assertTrue((await registry.call('plate_dish', agent_id=chef,
                                                 dish=dish, ingredient=ingredient)).success)
        self.assertTrue(sim.is_complete())


class MultiStepKitchenWorld(unittest.IsolatedAsyncioTestCase):
    def world(self, depth=0):
        sim = importlib.import_module('benchmark.environments.12H.sim').KitchenMultiStepSim()
        sim._delay_multiplier = 0
        sim.set_scenario_depth(depth)
        return sim, load_tools('12H', sim=sim)

    async def test_wrong_ingredient_arguments_do_not_supply_material_or_consume_a_real_failure(self):
        sim, registry = self.world(1)
        for index, ingredient in enumerate(INGREDIENTS):
            for chef in CHEFS:
                for recipient in (*CHEFS, 'unknown'):
                    if (chef, recipient) == (CHEFS[index], RECIPIENTS[index]):
                        continue
                    self.assertFalse((await registry.call('prepare_ingredient', agent_id=chef,
                                                          ingredient=ingredient, for_chef=recipient)).success)
            self.assertFalse(sim.progress['ingredient_available'][ingredient])
            first = await registry.call('prepare_ingredient', agent_id=CHEFS[index],
                                        ingredient=ingredient, for_chef=RECIPIENTS[index])
            self.assertFalse(first.success)
            self.assertEqual(first.data['status'], 'preparation_failed')
            self.assertTrue((await registry.call('prepare_ingredient', agent_id=CHEFS[index],
                                                 ingredient=ingredient, for_chef=RECIPIENTS[index])).success)

    async def test_each_physical_equipment_use_is_observed_without_a_global_order(self):
        sim, registry = self.world()
        for tool, resource in (('prep_mise_en_place', 'PREP_STATION'),
                               ('cook_on_stovetop', 'STOVETOP'),
                               ('cook_in_oven', 'OVEN'), ('plate_dish', 'PREP_STATION')):
            self.assertEqual(sim.resource_requirements(tool, agent_id='CHEF_A'), [resource])
        await registry.call('prep_mise_en_place', agent_id='CHEF_A', dish='appetizer')
        await registry.call('prep_mise_en_place', agent_id='CHEF_C', dish='dessert')
        sim.tool_delay = lambda *args, **kwargs: (0.02, 0.02)
        sim._delay_multiplier = 1
        results = await asyncio.gather(
            registry.call('cook_in_oven', agent_id='CHEF_A', item='appetizer_base'),
            registry.call('prep_mise_en_place', agent_id='CHEF_B', dish='main_course'))
        self.assertTrue(all(result.success for result in results))
        self.assertEqual(sim.violations, [])
        self.assertLess(sim.events[-1].timestamp, sim.events[-2].completed_at)
        results = await asyncio.gather(
            registry.call('cook_in_oven', agent_id='CHEF_A', item='appetizer_base'),
            registry.call('cook_in_oven', agent_id='CHEF_C', item='dessert_base'))
        self.assertTrue(all(result.success for result in results))
        self.assertEqual(len(sim.violations), 1)
        self.assertEqual(sim.violations[0].violation_type, 'concurrent_resource_use')
        self.assertIn('OVEN', sim.violations[0].message)

    async def test_missing_ingredient_can_be_retried_and_final_plating_must_complete(self):
        sim, registry = self.world()
        for chef, dish in zip(CHEFS, DISHES):
            await registry.call('prep_mise_en_place', agent_id=chef, dish=dish)
            await registry.call('cook_in_oven', agent_id=chef, item=f'{dish}_base')
            self.assertFalse((await registry.call('plate_dish', agent_id=chef, dish=dish,
                                                  ingredient=RECEIVED[CHEFS.index(chef)])).success)
        for chef, ingredient, recipient in zip(CHEFS, INGREDIENTS, RECIPIENTS):
            await registry.call('prepare_ingredient', agent_id=chef, ingredient=ingredient, for_chef=recipient)
        for chef, dish, ingredient in zip(CHEFS[:2], DISHES[:2], RECEIVED[:2]):
            await registry.call('plate_dish', agent_id=chef, dish=dish, ingredient=ingredient)
        sim.tool_delay = lambda tool, **kwargs: (0.02, 0.02) if tool == 'plate_dish' else (0, 0)
        sim._delay_multiplier = 1
        pending = asyncio.create_task(registry.call('plate_dish', agent_id='CHEF_C',
                                                    dish='dessert', ingredient='sauce'))
        await asyncio.sleep(0)
        self.assertFalse(sim.is_complete())
        await pending
        self.assertTrue(sim.is_complete())
        self.assertEqual(sim.violations, [])

    async def test_wrong_preparation_or_cooking_target_creates_no_dish_material(self):
        for index, chef in enumerate(CHEFS):
            sim, registry = self.world()
            for dish in (*DISHES, *INGREDIENTS, 'unknown'):
                if dish in (DISHES[index], INGREDIENTS[index]):
                    continue
                self.assertFalse((await registry.call('prep_mise_en_place', agent_id=chef, dish=dish)).success)
            for item in (*(f'{dish}_base' for dish in DISHES), *INGREDIENTS, 'unknown'):
                self.assertFalse((await registry.call('cook_in_oven', agent_id=chef, item=item)).success)
            self.assertFalse(sim.progress['base_prepared'][chef])
            self.assertFalse(sim.progress['cooked'][chef])

    async def test_cooking_requires_its_own_completed_preparation(self):
        for index, chef in enumerate(CHEFS):
            sim, registry = self.world()
            await registry.call('prep_mise_en_place', agent_id=chef, dish=INGREDIENTS[index])
            self.assertFalse((await registry.call('cook_in_oven', agent_id=chef,
                                                 item=f'{DISHES[index]}_base')).success)
            self.assertTrue((await registry.call('cook_in_oven', agent_id=chef,
                                                item=INGREDIENTS[index])).success)
            self.assertFalse(sim.progress['cooked'][chef])

    async def test_wrong_plating_arguments_and_unknown_chef_do_not_complete_a_dish(self):
        sim, registry = self.world()
        for chef, dish in zip(CHEFS, DISHES):
            await registry.call('prep_mise_en_place', agent_id=chef, dish=dish)
            await registry.call('cook_in_oven', agent_id=chef, item=f'{dish}_base')
        for chef, ingredient, recipient in zip(CHEFS, INGREDIENTS, RECIPIENTS):
            await registry.call('prepare_ingredient', agent_id=chef, ingredient=ingredient, for_chef=recipient)
        for index, chef in enumerate(CHEFS):
            for dish in (*DISHES, 'unknown'):
                for ingredient in (*INGREDIENTS, 'unknown'):
                    if (dish, ingredient) == (DISHES[index], RECEIVED[index]):
                        continue
                    self.assertFalse((await registry.call('plate_dish', agent_id=chef,
                                                          dish=dish, ingredient=ingredient)).success)
        for tool, arguments in (
            ('prep_mise_en_place', {'dish': 'appetizer'}),
            ('cook_in_oven', {'item': 'appetizer_base'}),
            ('plate_dish', {'dish': 'appetizer', 'ingredient': 'garnish'}),
        ):
            self.assertFalse((await registry.call(tool, agent_id='unknown', **arguments)).success)
        self.assertFalse(sim.is_complete())

    async def test_failed_cooking_needs_fresh_preparation_and_can_recover(self):
        for depth in (1, 2, 3):
            for index, chef, tool in (
                (0, 'CHEF_A', 'cook_on_stovetop'), (0, 'CHEF_A', 'cook_in_oven'),
                (1, 'CHEF_B', 'cook_on_stovetop'), (1, 'CHEF_B', 'cook_in_oven'),
                (2, 'CHEF_C', 'cook_in_oven'),
            ):
                sim, registry = self.world(depth)
                for attempt in range(depth + 1):
                    await registry.call('prep_mise_en_place', agent_id=chef, dish=DISHES[index])
                    result = await registry.call(tool, agent_id=chef, item=f'{DISHES[index]}_base')
                    self.assertEqual(result.success, attempt == depth)
                    if attempt < depth:
                        self.assertFalse(sim.progress['base_prepared'][chef])
                        missing = await registry.call(tool, agent_id=chef, item=f'{DISHES[index]}_base')
                        self.assertFalse(missing.success)
                        self.assertEqual(missing.data['reason'], 'missing_preparation')
                self.assertTrue(sim.progress['cooked'][chef])
                self.assertEqual(sim.violations, [])

    async def test_older_success_cannot_hide_a_later_failed_cook_of_that_material(self):
        sim, registry = self.world()
        sim._failure_specs = [FailureSpec('cook_in_oven', agent_id='CHEF_A', call_index=1)]
        await registry.call('prepare_ingredient', agent_id='CHEF_B', ingredient='garnish', for_chef='CHEF_A')
        await registry.call('prep_mise_en_place', agent_id='CHEF_A', dish='appetizer')
        self.assertTrue((await registry.call('cook_in_oven', agent_id='CHEF_A', item='appetizer_base')).success)
        self.assertFalse((await registry.call('cook_in_oven', agent_id='CHEF_A', item='appetizer_base')).success)
        self.assertFalse((await registry.call('plate_dish', agent_id='CHEF_A', dish='appetizer', ingredient='garnish')).success)
        await registry.call('prep_mise_en_place', agent_id='CHEF_A', dish='appetizer')
        await registry.call('cook_in_oven', agent_id='CHEF_A', item='appetizer_base')
        self.assertTrue((await registry.call('plate_dish', agent_id='CHEF_A', dish='appetizer', ingredient='garnish')).success)
        self.assertEqual(sim.violations, [])

    async def test_pending_preparation_cannot_feed_a_cook(self):
        sim, registry = self.world()
        sim.tool_delay = lambda tool, **kwargs: (0.02, 0.02) if tool == 'prep_mise_en_place' else (0, 0)
        sim._delay_multiplier = 1
        pending = asyncio.create_task(registry.call('prep_mise_en_place', agent_id='CHEF_A', dish='appetizer'))
        await asyncio.sleep(0)
        self.assertFalse((await registry.call('cook_in_oven', agent_id='CHEF_A', item='appetizer_base')).success)
        await pending
        self.assertTrue((await registry.call('cook_in_oven', agent_id='CHEF_A', item='appetizer_base')).success)

    async def test_pending_cook_cannot_feed_a_plate(self):
        sim, registry = self.world()
        await registry.call('prepare_ingredient', agent_id='CHEF_B', ingredient='garnish', for_chef='CHEF_A')
        await registry.call('prep_mise_en_place', agent_id='CHEF_A', dish='appetizer')
        sim.tool_delay = lambda tool, **kwargs: (0.02, 0.02) if tool == 'cook_in_oven' else (0, 0)
        sim._delay_multiplier = 1
        pending = asyncio.create_task(registry.call('cook_in_oven', agent_id='CHEF_A', item='appetizer_base'))
        await asyncio.sleep(0)
        self.assertFalse((await registry.call('plate_dish', agent_id='CHEF_A', dish='appetizer', ingredient='garnish')).success)
        await pending
        result = await registry.call('plate_dish', agent_id='CHEF_A', dish='appetizer', ingredient='garnish')
        self.assertTrue(result.success)
        for key in ('prep_event', 'cook_event', 'ingredient_event'):
            event = sim.events[result.data[key]]
            self.assertTrue(event.success)
            self.assertLessEqual(event.completed_at, sim.events[-1].timestamp)

    async def test_fresh_independent_prep_can_overlap_the_failed_cook_response(self):
        sim, registry = self.world(1)
        await registry.call('prep_mise_en_place', agent_id='CHEF_A', dish='appetizer')
        sim.tool_delay = lambda tool, **kwargs: (0.02, 0.02) if tool == 'cook_in_oven' else (0, 0)
        sim._delay_multiplier = 1
        pending = asyncio.create_task(registry.call('cook_in_oven', agent_id='CHEF_A', item='appetizer_base'))
        await asyncio.sleep(0)
        prep = await registry.call('prep_mise_en_place', agent_id='CHEF_A', dish='appetizer')
        self.assertTrue(prep.success)
        self.assertFalse((await pending).success)
        self.assertTrue((await registry.call('cook_in_oven', agent_id='CHEF_A', item='appetizer_base')).success)
        self.assertEqual(sim.violations, [])

    async def test_failed_ingredient_is_not_available_and_repreparation_can_recover(self):
        sim, registry = self.world(1)
        result = await registry.call('prepare_ingredient', agent_id='CHEF_B', ingredient='garnish', for_chef='CHEF_A')
        self.assertFalse(result.success)
        result.data['status'] = 'ingredient prepared'
        self.assertFalse(sim.progress['ingredient_available']['garnish'])
        self.assertTrue((await registry.call('prepare_ingredient', agent_id='CHEF_B',
                                            ingredient='garnish', for_chef='CHEF_A')).success)
        self.assertTrue(sim.progress['ingredient_available']['garnish'])

    async def test_chef_orders_cooking_choices_and_recovery_remain_free(self):
        count = 0
        for order in itertools.permutations(range(3)):
            for on_stove in itertools.product((False, True), repeat=2):
                for depth in (0, 1, 2):
                    for ingredients_first in (False, True):
                        sim, registry = self.world(depth)
                        async def ingredients():
                            for index in reversed(order):
                                for attempt in range(depth + 1):
                                    result = await registry.call('prepare_ingredient', agent_id=CHEFS[index],
                                                                 ingredient=INGREDIENTS[index], for_chef=RECIPIENTS[index])
                                    self.assertEqual(result.success, attempt == depth)
                        if ingredients_first:
                            await ingredients()
                        for index in order:
                            tool = 'cook_on_stovetop' if index < 2 and on_stove[index] else 'cook_in_oven'
                            for attempt in range(depth + 1):
                                await registry.call('prep_mise_en_place', agent_id=CHEFS[index], dish=DISHES[index])
                                cooked = await registry.call(tool, agent_id=CHEFS[index], item=f'{DISHES[index]}_base')
                                self.assertEqual(cooked.success, attempt == depth)
                        if not ingredients_first:
                            await ingredients()
                        for index in reversed(order):
                            self.assertTrue((await registry.call('plate_dish', agent_id=CHEFS[index],
                                                                 dish=DISHES[index], ingredient=RECEIVED[index])).success)
                        self.assertTrue(sim.is_complete())
                        self.assertEqual(sim.violations, [])
                        count += 1
        self.assertEqual(count, 144)

    async def test_cooking_the_dish_or_its_named_base_refers_to_the_same_material(self):
        for item in ('appetizer', 'appetizer_base'):
            sim, registry = self.world()
            await registry.call('prep_mise_en_place', agent_id='CHEF_A', dish='appetizer')
            result = await registry.call('cook_in_oven', agent_id='CHEF_A', item=item)
            self.assertTrue(result.success)
            self.assertEqual(result.data['target'], 'appetizer')
            self.assertTrue(sim.progress['cooked']['CHEF_A'])
