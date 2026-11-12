"""Kitchen Multi-Step with Failures simulation config for task 12H.

Three chefs share oven + stovetop + prep_station.  Multi-step dish
preparation with cooking failures that force re-prep + re-cook cycles.

Deadlock traps:
  1. 3-lock contention — all three chefs need prep_station, stovetop/oven
  2. Hold-and-wait on failure — chef holds stovetop, needs prep_station
     for re-prep after cook failure
  3. Circular ingredient deps — A→C, B→A, C→B with equipment for all
"""

import json
from pathlib import Path
from typing import Any

from benchmark.tools._base import ToolResult
from benchmark.tools.sim_kitchen import DishConfig, IngredientConfig, KitchenSim

_METADATA = json.loads(
    (Path(__file__).resolve().parent.parent.parent
     / "descriptions" / "12H" / "metadata.json").read_text()
)

_EQUIPMENT = ["OVEN", "STOVETOP", "PREP_STATION"]

_DISHES = [
    DishConfig(
        chef_id="CHEF_A",
        dish_name="appetizer",
        needs_equipment=["PREP_STATION", "STOVETOP", "OVEN"],
        receives_ingredient="garnish",
        receives_from="CHEF_B",
    ),
    DishConfig(
        chef_id="CHEF_B",
        dish_name="main_course",
        needs_equipment=["PREP_STATION", "STOVETOP", "OVEN"],
        receives_ingredient="glaze",
        receives_from="CHEF_C",
    ),
    DishConfig(
        chef_id="CHEF_C",
        dish_name="dessert",
        needs_equipment=["PREP_STATION", "OVEN"],
        receives_ingredient="sauce",
        receives_from="CHEF_A",
    ),
]

_INGREDIENTS = [
    IngredientConfig(name="sauce", producer="CHEF_A", consumer="CHEF_C", needs_equipment=["STOVETOP"]),
    IngredientConfig(name="garnish", producer="CHEF_B", consumer="CHEF_A", needs_equipment=["STOVETOP"]),
    IngredientConfig(name="glaze", producer="CHEF_C", consumer="CHEF_B", needs_equipment=["OVEN"]),
]


class KitchenMultiStepSim(KitchenSim):
    """Actual prepared/cooked materials; no separate prep/cooked/done tables."""

    _DECISION_TOOLS = {
        "cook_on_stovetop": 0.3,
        "cook_in_oven": 0.3,
        "prepare_ingredient": 0.3,
    }

    def __init__(self) -> None:
        super().__init__(dishes=_DISHES, ingredients=_INGREDIENTS, equipment=_EQUIPMENT)
        self.load_from_metadata(_METADATA)

    def _target(self, chef, value, *, cooking=False):
        dish = self._dishes.get(chef)
        if dish and value in ((dish.dish_name, f"{dish.dish_name}_base") if cooking else (dish.dish_name,)):
            return dish.dish_name
        ingredient = self._ingredients.get(value) if isinstance(value, str) else None
        if ingredient and ingredient.producer == chef:
            return ingredient.name
        return None

    def _latest_cook(self, chef, target):
        for index in reversed(range(len(self.events))):
            event = self.events[index]
            if (event.agent == chef and event.tool in ("cook_on_stovetop", "cook_in_oven")
                    and event.result.get("target") == target and "prep_event" in event.result):
                return index
        return None

    def _available_prep(self, chef, target):
        for index in reversed(range(len(self.events))):
            event = self.events[index]
            if (event.agent == chef and event.tool == "prep_mise_en_place"
                    and event.success and event.completed_at is not None
                    and event.result.get("target") == target):
                # A failed actual cook consumes that prepared material. An
                # independently completed fresh prep can already be available.
                if any(attempt.agent == chef and attempt.result.get("target") == target
                       and attempt.tool in ("cook_on_stovetop", "cook_in_oven")
                       and attempt.result.get("status") == "cook_failed"
                       and attempt.result["prep_event"] >= index for attempt in self.events):
                    return None
                return index
        return None

    def prepare_ingredient(self, agent_id: str, ingredient: str = "",
                           for_chef: str = "", **kwargs: Any) -> ToolResult:
        assigned = self._ingredients.get(ingredient) if isinstance(ingredient, str) else None
        if assigned is None or (agent_id, for_chef) != (assigned.producer, assigned.consumer):
            return super().prepare_ingredient(agent_id, ingredient, for_chef)
        if self.should_fail("prepare_ingredient", agent_id):
            return self._record(agent_id, "prepare_ingredient",
                                {"ingredient": ingredient, "for_chef": for_chef},
                                {"ingredient": ingredient, "for_chef": for_chef,
                                 "status": "preparation_failed", "chef": agent_id}, success=False)
        return super().prepare_ingredient(agent_id, ingredient, for_chef)

    def prep_mise_en_place(self, agent_id: str, dish: str = "", **kwargs: Any) -> ToolResult:
        target = self._target(agent_id, dish)
        if target is None:
            return self._record(agent_id, "prep_mise_en_place", {"dish": dish},
                                {"dish": dish, "status": "error", "reason": "wrong_target"},
                                success=False)
        return self._record(agent_id, "prep_mise_en_place", {"dish": dish},
                            {"dish": dish, "target": target, "status": "mise en place done",
                             "chef": agent_id})

    def _cook(self, agent, item, tool):
        target = self._target(agent, item, cooking=True)
        if target is None or (tool == "cook_on_stovetop" and agent == "CHEF_C"):
            return self._record(agent, tool, {"item": item},
                                {"item": item, "status": "error", "reason": "wrong_target"},
                                success=False)
        prep = self._available_prep(agent, target)
        if prep is None:
            return self._record(agent, tool, {"item": item},
                                {"item": item, "status": "error", "reason": "missing_preparation"},
                                success=False)
        failed = self.should_fail(tool, agent)
        status = "cook_failed" if failed else ("cooked in oven" if tool == "cook_in_oven"
                                                else "cooked on stovetop")
        return self._record(agent, tool, {"item": item},
                            {"item": item, "target": target, "status": status,
                             "chef": agent, "prep_event": prep}, success=not failed)

    def cook_on_stovetop(self, agent_id: str, item: str = "", **kwargs: Any) -> ToolResult:
        return self._cook(agent_id, item, "cook_on_stovetop")

    def cook_in_oven(self, agent_id: str, item: str = "", **kwargs: Any) -> ToolResult:
        return self._cook(agent_id, item, "cook_in_oven")

    def plate_dish(self, agent_id: str, dish: str = "", ingredient: str = "",
                   **kwargs: Any) -> ToolResult:
        assigned = self._dishes.get(agent_id)
        arguments = {"dish": dish, "ingredient": ingredient}
        if assigned is None or (dish, ingredient) != (assigned.dish_name, assigned.receives_ingredient):
            return self._record(agent_id, "plate_dish", arguments,
                                {**arguments, "status": "error", "reason": "wrong_dish_or_ingredient"},
                                success=False)
        cooked = self._latest_cook(agent_id, dish)
        received = self._completed_ingredient(ingredient)
        if (cooked is None or not self.events[cooked].success
                or self.events[cooked].completed_at is None or received is None):
            return self._record(agent_id, "plate_dish", arguments,
                                {**arguments, "status": "error", "reason": "missing_prerequisite"},
                                success=False)
        return self._record(agent_id, "plate_dish", arguments,
                            {**arguments, "status": "dish plated", "chef": agent_id,
                             "prep_event": self.events[cooked].result["prep_event"],
                             "cook_event": cooked, "ingredient_event": received})

    def make_tools(self) -> dict[str, Any]:
        return {"prep_mise_en_place": self.prep_mise_en_place,
                "cook_on_stovetop": self.cook_on_stovetop, "cook_in_oven": self.cook_in_oven,
                "plate_dish": self.plate_dish, "prepare_ingredient": self.prepare_ingredient}

    def _completed_dishes(self):
        return {chef: self._completed(chef, "plate_dish",
                                     {"dish": dish.dish_name, "ingredient": dish.receives_ingredient})
                is not None for chef, dish in self._dishes.items()}

    @property
    def progress(self) -> dict[str, Any]:
        cooks = {chef: self._latest_cook(chef, dish.dish_name) for chef, dish in self._dishes.items()}
        return {"base_prepared": {chef: self._available_prep(chef, dish.dish_name) is not None
                                  for chef, dish in self._dishes.items()},
                "cooked": {chef: index is not None and self.events[index].success
                            and self.events[index].completed_at is not None for chef, index in cooks.items()},
                "ingredient_available": {name: self._completed_ingredient(name) is not None
                                         for name in self._ingredients},
                "dish_completed": self._completed_dishes(), "all_complete": self.is_complete()}
