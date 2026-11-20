"""Score wafer fabrication from actual processing and measurement receipts."""

import json
from functools import partial
from pathlib import Path

from benchmark.tools._base import ToolResult
from benchmark.tools.sim_base import SimContext
from benchmark.tools.sim_semiconductor import SemiconductorSim as _SemiconductorSim

_METADATA = json.loads((Path(__file__).resolve().parents[2] /
                        "descriptions/15E/metadata.json").read_text())
_STEPPER = "run_stepper_exposure"
_DEP = "run_deposition"
_ETCH = "run_etch_process"
_METRO = "run_metrology_measurement"
_MONITOR = "check_monitoring"
_TOOLS = {
    _STEPPER: ("process", "stepper_processes", "stepper process complete"),
    _DEP: ("process", "depositions", "deposition complete"),
    _ETCH: ("process", "etch_processes", "etch process complete"),
    _METRO: ("measurement", "metrology_measurements", "measurement complete"),
    _MONITOR: ("check_type", "monitoring_checks", "monitoring check complete"),
}

# Keys distinguish measuring the overlay from mounting it on the stepper.
_EXPOSE = (_STEPPER, "photo_exposure")
_DOSE = (_MONITOR, "exposure_dose")
_PATTERN = (_ETCH, "pattern_transfer_etch")
_ENDPOINT = (_MONITOR, "etch_endpoint")
_GROW = (_DEP, "cvd_growth")
_RATE = (_MONITOR, "deposition_rate")
_REACTIVE = (_ETCH, "reactive_ion_etch")
_ETCH_RATE = (_MONITOR, "etch_rate")
_CD = (_METRO, "cd_measurement")
_THICKNESS = (_METRO, "thickness_measurement")
_PROFILE = (_METRO, "profile_measurement")
_VALIDATE = (_MONITOR, "cross_reference")
_MOUNT = (_STEPPER, "overlay_alignment")
_OVERLAY = (_METRO, "overlay_alignment")
_INTEGRITY = (_DEP, "film_integrity_check")
_UNIFORMITY = (_ETCH, "etch_uniformity_check")
_SPOT_CHECKS = {_INTEGRITY, _UNIFORMITY}
_FAILURES = {_ETCH: "etch_failed", _METRO: "measurement_failed", _MONITOR: "check_failed"}

# Only the inputs consumed by this effect are needed. Other wafers may be
# processed or measured before, after, or during it.
_INPUTS = {
    _EXPOSE: ("LITHOGRAPHER", ()),
    _DOSE: ("LITHOGRAPHER", ()),
    _PATTERN: ("LITHOGRAPHER", (_EXPOSE, _DOSE)),
    _ENDPOINT: ("LITHOGRAPHER", ()),
    _GROW: ("DEPOSITOR", ()),
    _RATE: ("DEPOSITOR", ()),
    _REACTIVE: ("ETCHER", ()),
    _ETCH_RATE: ("ETCHER", ()),
    _CD: ("METROLOGY_ENGINEER", (_PATTERN, _ENDPOINT)),
    _THICKNESS: ("METROLOGY_ENGINEER", (_GROW, _RATE)),
    _PROFILE: ("METROLOGY_ENGINEER", (_REACTIVE, _ETCH_RATE)),
    _VALIDATE: ("PROCESS_CONTROLLER", (_CD, _THICKNESS, _PROFILE)),
    (_METRO, "alignment_check"): ("LITHOGRAPHER", (_VALIDATE,)),
    (_METRO, "thickness_verification"): ("DEPOSITOR", (_VALIDATE,)),
    _MOUNT: ("INTEGRATION_ENGINEER", (_VALIDATE,)),
    _OVERLAY: ("INTEGRATION_ENGINEER", ()),
    _INTEGRITY: ("INTEGRATION_ENGINEER", (_MOUNT, _OVERLAY)),
    _UNIFORMITY: ("INTEGRATION_ENGINEER", (_MOUNT, _OVERLAY)),
}
_IN_SITU = {_DOSE: _EXPOSE, _ENDPOINT: _PATTERN, _RATE: _GROW,
            _ETCH_RATE: _REACTIVE, _OVERLAY: _MOUNT}
_LOADS = frozenset(_IN_SITU.values())


class SemiconductorFabSim(SimContext):
    def __init__(self, scenario=None):
        super().__init__()
        for instrument in _METADATA["resources"]:
            self.init_resource(instrument)
        self.load_from_metadata(_METADATA)
        if scenario:
            self.configure_scenario(scenario)

    def make_tools(self):
        return {tool: partial(self._operate, tool) for tool in _TOOLS}

    def tool_delay(self, tool_name, **kwargs):
        return _SemiconductorSim._TOOL_DELAYS[tool_name]

    def _evidence(self):
        """Read instrument loads and the results that still describe those wafers."""
        latest_load = {}
        unloaded = set()
        results = {}
        for event in self._events:
            if not event.success:
                continue
            key = (event.tool, event.result[_TOOLS[event.tool][0]])
            if key in _LOADS:
                latest_load[key] = event
            if event.completed_at is not None:
                results[key] = event
                if key in _IN_SITU:
                    unloaded.add(event.result["inputs"][0])
        versions = {key[1]: event.result["receipt"] for key, event in latest_load.items()}
        current = {key: event for key, event in results.items()
                   if versions.items() >= event.result["wafer_loads"].items()}
        return latest_load, unloaded, current

    def _operate(self, tool, agent_id, **args):
        field, _, status = _TOOLS[tool]
        name = args.get(field)
        key = (tool, self._normalize(name) if isinstance(name, str) else None)
        operation = _INPUTS.get(key)
        loads, unloaded, current = self._evidence()
        inputs = []
        violations = []
        error = None
        if operation is None or operation[0] != agent_id or set(args) != {field}:
            error = "unknown_wafer_operation"
        elif (key in _SPOT_CHECKS or key == _MOUNT) and any(
            e.success and e.completed_at is None
            and (e.tool, e.result[_TOOLS[e.tool][0]]) in _SPOT_CHECKS for e in self._events
        ):
            error = "reference_wafer_busy"
        elif any(load.tool == tool and (load.completed_at is None
                 or load.result["receipt"] not in unloaded) for load in loads.values()):
            error = "instrument_still_loaded"
        elif key in _IN_SITU:
            load = loads.get(_IN_SITU[key])
            if load is None or load.result["receipt"] in unloaded:
                error = "wafer_not_loaded"
            else:
                inputs.append(load)
        else:
            for source in operation[1]:
                if source not in current:
                    error = "missing_wafer_result"
                    break
                inputs.append(current[source])
        if error:
            violations.append(self.log_violation(agent_id, tool, error, f"{error}: {name}"))
            data = {field: name, "engineer": agent_id, "status": error}
        elif tool in _FAILURES and self.should_fail(tool, agent_id):
            error = _FAILURES[tool]
            data = {field: name, "engineer": agent_id, "status": error}
        else:
            receipt = len(self._events) + 1
            versions = {wafer: version for entry in inputs
                        for wafer, version in entry.result["wafer_loads"].items()}
            if key in _LOADS:
                versions[key[1]] = receipt
            data = {field: key[1], "engineer": agent_id, "status": status,
                    "receipt": receipt, "inputs": [e.result["receipt"] for e in inputs],
                    "wafer_loads": versions}
        self.log_event(agent_id, tool, args, error is None, data, violations)
        return ToolResult(tool_name=tool, success=error is None, data=data, message=data["status"])

    def is_complete(self):
        return self.progress["all_complete"]

    @property
    def progress(self):
        _, _, current = self._evidence()
        scores = {group: {} for _, group, _ in _TOOLS.values()}
        for (tool, name), (agent, _) in _INPUTS.items():
            scores[_TOOLS[tool][1]][f"{agent}_{name}"] = (tool, name) in current
        scores["all_complete"] = len(current) == len(_INPUTS)
        return scores
