"""Flexible manufacturing simulation config for task 11M."""

import copy
import json
from pathlib import Path

from benchmark.tools.sim_manufacturing import (
    InspectStep,
    JobStep,
    ReworkStep,
)
from benchmark.tools._base import ToolResult
from benchmark.tools.sim_base import SimContext

_METADATA = json.loads(
    (Path(__file__).resolve().parent.parent.parent
     / "descriptions" / "11M" / "metadata.json").read_text()
)

_JOB_STEPS = [
    JobStep(agent_id="WORKER_A", job_id="alpha_job_a"),
    JobStep(agent_id="WORKER_A", job_id="beta_job_a"),
    JobStep(agent_id="WORKER_B", job_id="beta_job_b"),
    JobStep(agent_id="WORKER_B", job_id="gamma_job_b"),
    JobStep(agent_id="WORKER_C", job_id="gamma_job_c"),
    JobStep(agent_id="WORKER_C", job_id="alpha_job_c"),
]

_INSPECT_STEPS = [
    InspectStep(agent_id="INSPECTOR", job_id="alpha_job_a"),
    InspectStep(agent_id="INSPECTOR", job_id="beta_job_a"),
    InspectStep(agent_id="INSPECTOR", job_id="beta_job_b"),
    InspectStep(agent_id="INSPECTOR", job_id="gamma_job_b"),
    InspectStep(agent_id="INSPECTOR", job_id="gamma_job_c"),
    InspectStep(agent_id="INSPECTOR", job_id="alpha_job_c"),
]

# Reworks are conditional (only when inspection fails) — not required for completion.
_REWORK_STEPS: list[ReworkStep] = []


class ThreeWorkerToolCabinetSim(SimContext):
    """Actual parts, measurements and work orders, independent of agent order.

    No job/inspection done table is maintained. A new physical part replaces the
    old one; quality evidence and rework orders name the actual measured part.
    The common registry observes overlapping physical use without scheduling it.
    """

    _DECISION_TOOLS = {"inspect_quality": 0.3}
    _TOOL_DELAYS = {
        "do_job": (2.0, 5.0),
        "inspect_quality": (1.0, 3.0),
        "dispatch_rework": (1.0, 3.0),
    }

    def __init__(self) -> None:
        super().__init__()
        for resource in _METADATA['resources']:
            self.init_resource(resource)
        self.load_from_metadata(_METADATA)
        physical = _METADATA['tool_resource_map']['do_job']
        if (physical.get('argument') != 'job_id'
                or set(physical.get('values', {})) != {step.job_id for step in _JOB_STEPS}
                or self._resource_capacity.get('TOOL_CABINET') != 2
                or self.resource_requirements('inspect_quality') != ['INSPECTOR']
                or any(not isinstance(resources, list)
                       or len(resources) != len(set(resources))
                       or not set(resources) <= set(_METADATA['resources'])
                       or sum(name.startswith('STATION_') for name in resources) != 1
                       for resources in physical['values'].values())):
            raise ValueError('workshop jobs need complete physical bindings and two actual tools')

    def tool_delay(self, tool_name, **kwargs):
        return self._TOOL_DELAYS[tool_name]

    def make_tools(self):
        return {'do_job': self.do_job, 'inspect_quality': self.inspect_quality,
                'dispatch_rework': self.dispatch_rework}

    def _record(self, tool, agent, arguments, success, data, violations=()):
        self.log_event(agent, tool, copy.deepcopy(arguments), success,
                       copy.deepcopy(data), list(violations))
        return ToolResult(tool, success, data)

    def _part(self, job):
        # In-flight replacement material hides old parts too. An earlier result
        # cannot certify the replacement merely because it has not finished yet.
        return next((event for event in reversed(self.events)
                     if event.tool == 'do_job' and event.success
                     and event.result['part']['job_id'] == job), None)

    def _inspection(self, job):
        return next((event for event in reversed(self.events)
                     if event.tool == 'inspect_quality' and 'receipt' in event.result
                     and event.result['input_part']['job_id'] == job), None)

    def _approval(self, job):
        part, inspection = self._part(job), self._inspection(job)
        if (part is not None and part.completed_at is not None and not part.violations
                and inspection is not None and inspection.completed_at is not None
                and inspection.success and inspection.result['input_part'] == part.result['part']):
            return inspection.result
        return None

    def do_job(self, agent_id, **arguments):
        job = arguments.get('job_id')
        assigned = next((step for step in _JOB_STEPS
                         if step.agent_id == agent_id and step.job_id == job), None)
        if assigned is None:
            return self._record('do_job', agent_id, arguments, False,
                                {'job_id': job, 'status': 'unknown_or_unassigned_job'})
        previous, inspection = self._part(job), self._inspection(job)
        order, violations = None, []
        if (previous is not None and inspection is not None and not inspection.success
                and inspection.result['input_part'] == previous.result['part']):
            order = next((event for event in reversed(self.events)
                          if event.tool == 'dispatch_rework' and event.success
                          and event.completed_at is not None
                          and event.result['inspection'] == inspection.result['receipt']
                          and event.result['input_part'] == previous.result['part']), None)
            if order is None or inspection.completed_at is None:
                violations.append(self.log_violation(
                    agent_id, 'do_job', 'missing_rework_order',
                    f'{job} replaces a rejected part without its completed current work order'))
        resources = self.resource_requirements('do_job', agent_id=agent_id, **arguments)
        attempt = 1 + sum(event.tool == 'do_job' and event.success
                          and event.result['part']['job_id'] == job for event in self.events)
        part = {'artifact_id': f'part/{job}/{attempt}', 'job_id': job,
                'worker': agent_id, 'station': resources[0], 'attempt': attempt}
        return self._record('do_job', agent_id, arguments, True,
                            {'part': part, 'status': 'completed',
                             'rework_order': order.result['order'] if order is not None else None},
                            violations)

    def inspect_quality(self, agent_id, **arguments):
        job = arguments.get('job_id')
        part = self._part(job)
        if (not any(step.agent_id == agent_id and step.job_id == job for step in _INSPECT_STEPS)
                or part is None or part.completed_at is None):
            return self._record('inspect_quality', agent_id, arguments, False,
                                {'job_id': job, 'status': 'part_not_completed'})
        previous = self._inspection(job)
        if (previous is not None and not previous.success
                and previous.result['input_part'] == part.result['part']):
            return self._record('inspect_quality', agent_id, arguments, False,
                                {'job_id': job, 'status': 'rework_required',
                                 'input_part': copy.deepcopy(part.result['part'])})
        success = not self.should_fail('inspect_quality', agent_id)
        occurrence = 1 + sum(event.tool == 'inspect_quality' and 'receipt' in event.result
                             for event in self.events)
        return self._record('inspect_quality', agent_id, arguments, success,
                            {'job_id': job, 'input_part': copy.deepcopy(part.result['part']),
                             'receipt': f'inspection/{occurrence}', 'inspector': agent_id,
                             'status': 'inspected' if success else 'failed_inspection'})

    def dispatch_rework(self, agent_id, **arguments):
        job = arguments.get('job_id')
        part, inspection = self._part(job), self._inspection(job)
        if (agent_id != 'DISPATCHER' or part is None or part.completed_at is None
                or inspection is None or inspection.completed_at is None or inspection.success
                or inspection.result['input_part'] != part.result['part']):
            return self._record('dispatch_rework', agent_id, arguments, False,
                                {'job_id': job, 'status': 'no_completed_current_failure'})
        occurrence = 1 + sum(event.tool == 'dispatch_rework' and event.success
                             for event in self.events)
        return self._record('dispatch_rework', agent_id, arguments, True,
                            {'job_id': job, 'worker': part.result['part']['worker'],
                             'input_part': copy.deepcopy(part.result['part']),
                             'inspection': inspection.result['receipt'],
                             'order': f'rework/{occurrence}', 'status': 'rework_dispatched',
                             'reason': copy.deepcopy(arguments.get('reason', ''))})

    def is_complete(self):
        return all(self._approval(step.job_id) is not None for step in _JOB_STEPS)

    @property
    def progress(self):
        parts = {step.job_id: event.result['part'] for step in _JOB_STEPS
                 if (event := self._part(step.job_id)) is not None
                 and event.completed_at is not None}
        inspections = {step.job_id: receipt for step in _JOB_STEPS
                       if (receipt := self._approval(step.job_id)) is not None}
        return copy.deepcopy({'jobs': parts, 'inspections': inspections,
                              'available_tools': self._resource_capacity['TOOL_CABINET']
                              - len(self._in_use.get('TOOL_CABINET', [])),
                              'all_complete': self.is_complete()})
