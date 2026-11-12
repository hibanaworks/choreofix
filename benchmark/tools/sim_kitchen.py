"""Kitchen materials and effects, without prescribing a chef's operation order."""
from __future__ import annotations

from copy import deepcopy
from dataclasses import dataclass, field
from typing import Any

from ._base import ToolResult
from .sim_base import SimContext


@dataclass
class DishConfig:
    chef_id: str
    dish_name: str
    needs_equipment: list[str]
    receives_ingredient: str
    receives_from: str


@dataclass
class IngredientConfig:
    name: str
    producer: str
    consumer: str
    needs_equipment: list[str] = field(default_factory=list)


class KitchenSim(SimContext):
    """12E/12M score completed dishes from the actual completed input effects.

    The registry observes physical occupancy; this world never locks, queues or
    schedules calls. Unavailable or malformed requests create no material.
    """

    def __init__(self, dishes: list[DishConfig], ingredients: list[IngredientConfig],
                 equipment: list[str]) -> None:
        super().__init__()
        self._dishes = {dish.chef_id: dish for dish in dishes}
        self._ingredients = {ingredient.name: ingredient for ingredient in ingredients}
        for resource in equipment:
            self.init_resource(resource)

    def _record(self, agent, tool, arguments, data, *, success=True):
        # Caller-owned replies cannot rewrite the simulator's actual effects.
        self.log_event(agent, tool, deepcopy(arguments), success, deepcopy(data))
        return ToolResult(tool_name=tool, success=success, data=deepcopy(data))

    def _completed(self, agent, tool, arguments):
        for index in reversed(range(len(self.events))):
            event = self.events[index]
            if (event.agent == agent and event.tool == tool
                    and event.args == arguments and event.success
                    and event.completed_at is not None):
                return index
        return None

    def _completed_base(self, chef):
        dish = self._dishes[chef]
        return self._completed(chef, 'prepare_base_dish', {'dish': dish.dish_name})

    def _completed_ingredient(self, name):
        ingredient = self._ingredients[name]
        return self._completed(ingredient.producer, 'prepare_ingredient',
                               {'ingredient': name, 'for_chef': ingredient.consumer})

    def prepare_base_dish(self, agent_id: str, dish: str) -> ToolResult:
        assigned = self._dishes.get(agent_id)
        arguments = {'dish': dish}
        if assigned is None or dish != assigned.dish_name:
            return self._record(agent_id, 'prepare_base_dish', arguments,
                                {**arguments, 'status': 'error', 'reason': 'wrong_dish'}, success=False)
        return self._record(agent_id, 'prepare_base_dish', arguments,
                            {**arguments, 'chef': agent_id, 'status': 'base dish prepared'})

    def prepare_ingredient(self, agent_id: str, ingredient: str, for_chef: str) -> ToolResult:
        assigned = self._ingredients.get(ingredient) if isinstance(ingredient, str) else None
        arguments = {'ingredient': ingredient, 'for_chef': for_chef}
        if assigned is None or (agent_id, for_chef) != (assigned.producer, assigned.consumer):
            return self._record(agent_id, 'prepare_ingredient', arguments,
                                {**arguments, 'status': 'error', 'reason': 'wrong_ingredient_target'},
                                success=False)
        return self._record(agent_id, 'prepare_ingredient', arguments,
                            {**arguments, 'chef': agent_id, 'status': 'ingredient prepared'})

    def combine_dish(self, agent_id: str, dish: str, ingredient: str) -> ToolResult:
        assigned = self._dishes.get(agent_id)
        arguments = {'dish': dish, 'ingredient': ingredient}
        if assigned is None or (dish, ingredient) != (assigned.dish_name, assigned.receives_ingredient):
            return self._record(agent_id, 'combine_dish', arguments,
                                {**arguments, 'status': 'error', 'reason': 'wrong_dish_or_ingredient'},
                                success=False)
        base = self._completed_base(agent_id)
        received = self._completed_ingredient(ingredient)
        if base is None or received is None:
            return self._record(agent_id, 'combine_dish', arguments,
                                {**arguments, 'status': 'error', 'reason': 'missing_prerequisite'},
                                success=False)
        return self._record(agent_id, 'combine_dish', arguments,
                            {**arguments, 'chef': agent_id, 'status': 'dish completed',
                             'base_event': base, 'ingredient_event': received})

    def resource_requirements(self, tool_name: str, **kwargs: Any) -> list[str]:
        if tool_name == 'prepare_ingredient':
            name = kwargs.get('ingredient', '')
            ingredient = self._ingredients.get(name) if isinstance(name, str) else None
            return list(ingredient.needs_equipment) if ingredient else []
        return super().resource_requirements(tool_name, **kwargs)

    def make_tools(self) -> dict[str, Any]:
        return {'prepare_base_dish': self.prepare_base_dish,
                'prepare_ingredient': self.prepare_ingredient, 'combine_dish': self.combine_dish}

    def _completed_dishes(self):
        return {chef: self._completed(chef, 'combine_dish',
                                     {'dish': dish.dish_name, 'ingredient': dish.receives_ingredient})
                is not None for chef, dish in self._dishes.items()}

    def is_complete(self) -> bool:
        return all(self._completed_dishes().values())

    @property
    def progress(self) -> dict[str, Any]:
        return {'base_prepared': {chef: self._completed_base(chef) is not None for chef in self._dishes},
                'ingredient_available': {name: self._completed_ingredient(name) is not None
                                         for name in self._ingredients},
                'dish_completed': self._completed_dishes(), 'all_complete': self.is_complete()}
