"""Flexible manufacturing simulation config for task 11E."""

import json
from pathlib import Path

from benchmark.tools.sim_manufacturing import (
    CollectReportsStep,
    JobStep,
    ManufacturingSim,
)
from benchmark.tools.sim_base import SimContext
from benchmark.tools._base import ToolResult

_METADATA = json.loads(
    (Path(__file__).resolve().parent.parent.parent
     / "descriptions" / "11E" / "metadata.json").read_text()
)

_JOB_STEPS = [
    JobStep(agent_id="TECHNICIAN_A", job_id="lathe_job_a"),
    JobStep(agent_id="TECHNICIAN_A", job_id="mill_job_a"),
    JobStep(agent_id="TECHNICIAN_B", job_id="lathe_job_b"),
    JobStep(agent_id="TECHNICIAN_B", job_id="mill_job_b"),
]

_COLLECT_REPORTS_STEPS = [
    CollectReportsStep(agent_id="SUPERVISOR", scope="all_jobs"),
]


class TwoTechnicianWorkshopSim(SimContext):
    """11E: Two technicians completing jobs on two workstations (lathe and
    mill) with a supervisor collecting completion reports."""

    # 11E has no inspect_quality tool — override inherited _DECISION_TOOLS.
    _DECISION_TOOLS: dict = {}

    def __init__(self) -> None:
        super().__init__()
        for resource in _METADATA["resources"]:
            self.init_resource(resource)
        self.load_from_metadata(_METADATA)

    def tool_delay(self, tool_name: str, **kwargs) -> tuple[float, float]:
        return ManufacturingSim._TOOL_DELAYS.get(tool_name, (0.0, 0.0))

    @property
    def completed_jobs(self) -> list[dict]:
        return [event.result for event in self.events
                if event.tool == "do_job" and event.success and event.completed_at is not None]

    def do_job(self, agent_id: str, **kwargs) -> ToolResult:
        job = kwargs.get("job_id")
        machine = self.resource_requirements("do_job", agent_id=agent_id, **kwargs)
        success = isinstance(job, str) and bool(machine)
        data = {"job_id": job, "agent": agent_id,
                "machine": machine[0] if success else None,
                "status": "completed" if success else "unknown_job"}
        self.log_event(agent_id, "do_job", {"job_id": job}, success, data)
        return ToolResult("do_job", success, data)

    def collect_reports(self, agent_id: str, **kwargs) -> ToolResult:
        # Reading is always allowed. Its contents describe completed real jobs;
        # it cannot stand in for reports of jobs that finish in the future.
        scope = kwargs.get("scope", "all_jobs")
        data = {"scope": scope, "status": "collected", "agent": agent_id,
                "jobs": list(self.completed_jobs)}
        self.log_event(agent_id, "collect_reports", {"scope": scope}, True, data)
        return ToolResult("collect_reports", True, data)

    def make_tools(self) -> dict:
        return {"do_job": self.do_job, "collect_reports": self.collect_reports}

    def is_complete(self) -> bool:
        required = {(step.agent_id, step.job_id) for step in _JOB_STEPS}
        for event in self.events:
            if event.tool != "collect_reports" or not event.success or event.completed_at is None or event.agent != "SUPERVISOR":
                continue
            reported = {(job["agent"], job["job_id"]) for job in event.result["jobs"]}
            if required.issubset(reported):
                return True
        return False

    @property
    def progress(self) -> dict:
        return {"completed_jobs": self.completed_jobs,
                "reports": [event.result for event in self.events
                            if event.tool == "collect_reports" and event.completed_at is not None],
                "all_complete": self.is_complete()}
