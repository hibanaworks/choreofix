"""Parallel build simulation config for task 10H."""

import json
from pathlib import Path

from benchmark.tools.sim_build import (
    BuildSim,
    BuildTestStep,
    CompileStep,
    LinkStep,
    ValidateStep,
)

_METADATA = json.loads(
    (Path(__file__).resolve().parent.parent.parent
     / "descriptions" / "10H" / "metadata.json").read_text()
)

_RESOURCES = _METADATA["resources"]

_COMPILE_STEPS = [
    CompileStep(agent_id="BUILDER_A", module="module_a", outputs=("CONFIG", "CORE_LIB")),
    CompileStep(agent_id="BUILDER_B", module="module_b", outputs=("CORE_LIB", "DATA_MODELS")),
    CompileStep(agent_id="BUILDER_C", module="module_c", outputs=("API_TYPES", "DATA_MODELS")),
    CompileStep(agent_id="BUILDER_D", module="module_d", outputs=("API_TYPES", "CONFIG")),
]

_BUILD_TEST_STEPS = [
    BuildTestStep(agent_id="TEST_RUNNER", module="module_a"),
    BuildTestStep(agent_id="TEST_RUNNER", module="module_b"),
    BuildTestStep(agent_id="TEST_RUNNER", module="module_c"),
    BuildTestStep(agent_id="TEST_RUNNER", module="module_d"),
]

_VALIDATE_STEPS = [
    ValidateStep(agent_id="VALIDATOR", artifact="module_a_artifact", module="module_a"),
    ValidateStep(agent_id="VALIDATOR", artifact="module_b_artifact", module="module_b"),
    ValidateStep(agent_id="VALIDATOR", artifact="module_c_artifact", module="module_c"),
    ValidateStep(agent_id="VALIDATOR", artifact="module_d_artifact", module="module_d"),
]

_LINK_STEPS = [
    LinkStep(agent_id="INTEGRATOR", modules=["module_a", "module_b", "module_c", "module_d"]),
]


class FourBuildersBuildSlotsSim(BuildSim):
    """10H: Four builders with build slots and cascading rebuilds,
    an artifact validator, test runner, and integrator.

    BUILD_SLOTS is a counting semaphore (capacity=2) — at most 2 builds
    run concurrently.  Each builder acquires a slot before compiling and
    releases it afterward. compile_module uses the actual module's metadata
    resources. BUILD_SLOTS is execution capacity, not shared source material.
    """

    def __init__(self) -> None:
        super().__init__(
            compile_steps=_COMPILE_STEPS,
            build_test_steps=_BUILD_TEST_STEPS,
            validate_steps=_VALIDATE_STEPS,
            link_steps=_LINK_STEPS,
            metadata=_METADATA,
            initial_artifacts={name: f"provided definitions for {name}"
                               for name in _RESOURCES if name != "BUILD_SLOTS"},
            resource_capacities={"BUILD_SLOTS": 2},
        )
