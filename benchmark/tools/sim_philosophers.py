"""Parameterized dining philosophers simulation (scenarios 9E/9M/9H)."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any

from ._base import ToolResult
from .sim_base import SimContext


@dataclass
class ThinkStep:
    """A philosopher thinking step."""

    agent_id: str


@dataclass
class EatStep:
    """A philosopher eating step."""

    agent_id: str


class PhilosophersSim(SimContext):
    """Simulation for the dining philosophers problem.

    Each philosopher has a left fork and a right fork.  Eating requires
    holding BOTH forks.  Thinking requires nothing.

    Resource requirements are dynamic based on the philosopher-to-fork
    mapping passed at construction time.

    Args:
        philosopher_ids: List of philosopher agent IDs.
        forks: Mapping of philosopher_id -> (left_fork, right_fork).
        think_steps: Optional explicit think steps.  If None, one per philosopher.
        eat_steps: Optional explicit eat steps.  If None, one per philosopher.
    """

    def __init__(
        self,
        philosopher_ids: list[str],
        forks: dict[str, tuple[str, str]],
        think_steps: list[ThinkStep] | None = None,
        eat_steps: list[EatStep] | None = None,
    ) -> None:
        super().__init__()

        self._philosopher_ids = list(philosopher_ids)
        self._forks = dict(forks)  # philosopher_id -> (left_fork, right_fork)

        # Default: one think + one eat step per philosopher
        _think = think_steps if think_steps is not None else [ThinkStep(p) for p in philosopher_ids]
        _eat = eat_steps if eat_steps is not None else [EatStep(p) for p in philosopher_ids]

        self._think_steps = {s.agent_id: s for s in _think}
        self._eat_steps = {s.agent_id: s for s in _eat}

        # Initialize all unique forks as resources
        all_forks: set[str] = set()
        for left, right in forks.values():
            all_forks.add(left)
            all_forks.add(right)
        for fork in sorted(all_forks):
            self.init_resource(fork)

    # -- Resource requirements (dynamic) --

    _TOOL_RESOURCES: dict[str, list[str]] = {
        "think": [],
        # "eat" is dynamic — handled in resource_requirements()
    }

    _TOOL_DELAYS: dict[str, tuple[float, float]] = {
        "think": (0.5, 2.0),
        "eat": (1.0, 3.0),
    }

    def resource_requirements(self, tool_name: str, **kwargs: Any) -> list[str]:
        if tool_name == "eat":
            agent_id = kwargs.get("agent_id", "")
            fork_pair = self._forks.get(agent_id)
            if fork_pair:
                return list(fork_pair)
            return []
        return list(self._TOOL_RESOURCES.get(tool_name, []))

    def tool_delay(self, tool_name: str, **kwargs: Any) -> tuple[float, float]:
        return self._TOOL_DELAYS.get(tool_name, (0.0, 0.0))

    # -- Tool implementations --

    def think(self, agent_id: str, **kwargs: Any) -> ToolResult:
        """Think. No resource required."""
        result = {"philosopher": agent_id, "status": "thinking"}
        self.log_event(agent_id, "think", {},
                       success=True, result=result)
        return ToolResult(tool_name="think", success=True,
                          data=result, message=f"{agent_id} is thinking")

    def eat(self, agent_id: str, **kwargs: Any) -> ToolResult:
        """Eat. Requires holding BOTH left and right forks."""
        fork_pair = self._forks.get(agent_id)
        if fork_pair is None:
            result = {"philosopher": agent_id, "status": "error", "reason": "unknown_philosopher"}
            self.log_event(agent_id, "eat", {},
                           success=False, result=result)
            return ToolResult(tool_name="eat", success=False,
                              data=result, message=f"Unknown philosopher: {agent_id}")

        left_fork, right_fork = fork_pair

        violations = []
        if agent_id in self._think_steps and not any(
            event.agent == agent_id and event.tool == "think" and event.success
            and event.completed_at is not None for event in self._events
        ):
            violations.append(self.log_violation(
                agent_id, "eat", "eat_before_think",
                "Meal started before the diner's thinking completed",
            ))

        result = {"philosopher": agent_id, "status": "eating",
                  "left_fork": left_fork, "right_fork": right_fork}
        self.log_event(agent_id, "eat", {"left_fork": left_fork, "right_fork": right_fork},
                       success=True, result=result, violations=violations)
        return ToolResult(tool_name="eat", success=True,
                          data=result, message=f"{agent_id} is eating")

    # -- SimContext interface --

    def make_tools(self) -> dict[str, Any]:
        return {
            "think": self.think,
            "eat": self.eat,
        }

    def is_complete(self) -> bool:
        return self.progress["all_complete"]

    @property
    def progress(self) -> dict[str, Any]:
        finished = {(event.agent, event.tool) for event in self._events
                    if event.success and event.completed_at is not None and not event.violations}
        thinking = {agent: (agent, "think") in finished for agent in self._think_steps}
        eating = {agent: (agent, "eat") in finished for agent in self._eat_steps}
        return {"thinking": thinking, "eating": eating,
                "all_complete": all(thinking.values()) and all(eating.values())}
