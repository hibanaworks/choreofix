"""Parameterized medical consultation simulation (scenarios 5E/5M/5H)."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any

from ._base import ToolResult
from .sim_base import SimContext


@dataclass
class AssessStep:
    """A patient assessment step."""

    agent_id: str
    patient_id: str


@dataclass
class OrderTestsStep:
    """A test ordering step."""

    agent_id: str
    patient_id: str
    tests: list[str]


@dataclass
class ImagingStep:
    """An imaging step (perform or interpret)."""

    agent_id: str
    patient_id: str
    modality: str  # e.g. "CT", "MRI", "X-ray"


@dataclass
class DiagnoseStep:
    """A diagnosis step."""

    agent_id: str
    patient_id: str


@dataclass
class PrescriptionReviewStep:
    """A prescription review step."""

    agent_id: str
    patient_id: str


@dataclass
class CaseConferenceStep:
    """A case conference step."""

    agent_id: str
    case_id: str


@dataclass
class SurgeryStep:
    """A surgery step."""

    agent_id: str
    patient_id: str
    procedure: str


@dataclass
class AnesthesiaStep:
    """An anesthesia preparation step."""

    agent_id: str
    patient_id: str


@dataclass
class ICUStep:
    """An ICU management step."""

    agent_id: str
    patient_id: str


class MedicalSim(SimContext):
    """Actual patient results and physical resource use, shared by all methods.

    Completion is derived from completed effect receipts and their current
    inputs. There is no expected-next-operation cursor or persistent done table.
    This abstract tool set has no prescribing or drug-administration operation;
    a review of caller-supplied medications does not certify those absent effects.
    """

    def __init__(
        self,
        assess_steps: list[AssessStep] | None = None,
        order_tests_steps: list[OrderTestsStep] | None = None,
        imaging_steps: list[ImagingStep] | None = None,
        interpret_imaging_steps: list[ImagingStep] | None = None,
        diagnose_steps: list[DiagnoseStep] | None = None,
        prescription_review_steps: list[PrescriptionReviewStep] | None = None,
        case_conference_steps: list[CaseConferenceStep] | None = None,
        surgery_steps: list[SurgeryStep] | None = None,
        anesthesia_steps: list[AnesthesiaStep] | None = None,
        icu_steps: list[ICUStep] | None = None,
        resources: list[str] | None = None,
    ) -> None:
        super().__init__()
        # Immutable task data describes patients, test panels and real objects.
        # It does not prescribe one total order of tool invocations.
        self._steps = {
            "assess_patient": tuple(assess_steps or []),
            "order_tests": tuple(order_tests_steps or []),
            "perform_imaging": tuple(imaging_steps or []),
            "interpret_imaging": tuple(interpret_imaging_steps or []),
            "diagnose": tuple(diagnose_steps or []),
            "review_prescriptions": tuple(prescription_review_steps or []),
            "conduct_case_conference": tuple(case_conference_steps or []),
            "perform_surgery": tuple(surgery_steps or []),
            "prepare_anesthesia": tuple(anesthesia_steps or []),
            "manage_icu": tuple(icu_steps or []),
        }
        for resource in resources or []:
            self.init_resource(resource)

    # -- Decision tools (probabilistic failure for either/or branches) --

    _DECISION_TOOLS: dict[str, float] = {
        "interpret_imaging": 0.2,
        "diagnose": 0.2,
        "review_prescriptions": 0.3,
    }

    # -- Resource requirements --

    _TOOL_RESOURCES: dict[str, list[str]] = {
        "assess_patient": ["PATIENT_RECORD"],
        "order_tests": ["PATIENT_RECORD"],
        "perform_imaging": ["IMAGING_SUITE"],
        "interpret_imaging": ["IMAGING_SUITE"],
        "diagnose": ["PATIENT_RECORD"],
        "review_prescriptions": ["PHARMACY"],
        "conduct_case_conference": [],
        "perform_surgery": ["OPERATING_ROOM"],
        "prepare_anesthesia": ["OPERATING_ROOM"],
        "manage_icu": ["ICU"],
    }

    # Modality → IR resource name mapping for imaging tools.
    _MODALITY_RESOURCES: dict[str, str] = {
        "CT": "CT_SCANNER",
        "MRI": "MRI",
    }

    def resource_requirements(self, tool_name: str, **kwargs: Any) -> list[str]:
        # Prefer metadata-driven map (loaded via load_from_metadata) when available.
        if self._tool_resource_map:
            base = super().resource_requirements(tool_name, **kwargs)
            # For imaging tools, resolve modality-specific resource dynamically.
            if tool_name in ("perform_imaging", "interpret_imaging"):
                modality = str(kwargs.get("modality", "")).upper()
                res = self._MODALITY_RESOURCES.get(modality)
                if res:
                    return [res]
            return base
        return list(self._TOOL_RESOURCES.get(tool_name, []))

    # -- Simulated delays --

    _TOOL_DELAYS: dict[str, tuple[float, float]] = {
        "assess_patient": (0.5, 1.5),
        "order_tests": (0.5, 1.0),
        "perform_imaging": (2.0, 5.0),
        "interpret_imaging": (1.0, 3.0),
        "diagnose": (1.0, 2.0),
        "review_prescriptions": (0.5, 1.5),
        "conduct_case_conference": (2.0, 4.0),
        "perform_surgery": (5.0, 10.0),
        "prepare_anesthesia": (1.0, 3.0),
        "manage_icu": (1.0, 2.0),
    }

    def tool_delay(self, tool_name: str, **kwargs: Any) -> tuple[float, float]:
        return self._TOOL_DELAYS.get(tool_name, (0.0, 0.0))

    def _step(self, tool, agent, arguments):
        patient = arguments.get("patient_id")
        if not isinstance(patient, str) or not patient:
            return None
        for step in self._steps[tool]:
            target = step.case_id if tool == "conduct_case_conference" else step.patient_id
            if step.agent_id != agent or target != patient:
                continue
            if tool in ("perform_imaging", "interpret_imaging"):
                modality = arguments.get("modality")
                if not isinstance(modality, str) or modality.upper() != step.modality:
                    continue
            if tool == "perform_surgery" and arguments.get("procedure") != step.procedure:
                continue
            return step
        return None

    def _latest(self, tool, step):
        target = step.case_id if tool == "conduct_case_conference" else step.patient_id
        for event in reversed(self.events):
            if (event.completed_at is None or event.tool != tool
                    or event.agent != step.agent_id or not event.result.get("artifact_id")
                    or event.result.get("patient_id") != target):
                continue
            if tool in ("perform_imaging", "interpret_imaging") and event.result.get("modality") != step.modality:
                continue
            if tool == "perform_surgery" and event.result.get("procedure") != step.procedure:
                continue
            return event
        return None

    def _assessment(self, patient):
        for step in self._steps["assess_patient"]:
            if step.patient_id == patient:
                event = self._latest("assess_patient", step)
                if event is not None and event.success:
                    return event.result
        return None

    def _panel(self, step):
        inputs = self._inputs("order_tests", step)
        if inputs is None:
            return {}
        tests = {}
        for event in self.events:
            if (event.completed_at is not None and event.success
                    and event.tool == "order_tests" and event.agent == step.agent_id
                    and event.result.get("patient_id") == step.patient_id
                    and event.result.get("inputs") == inputs):
                for name in event.result["tests"]:
                    tests[name] = event.result["artifact_id"]
        return tests

    def _current(self, tool, step):
        event = self._latest(tool, step)
        inputs = self._inputs(tool, step)
        if event is not None and event.success and inputs is not None and event.result.get("inputs") == inputs:
            return event.result
        return None

    def _inputs(self, tool, step):
        if tool in ("assess_patient", "review_prescriptions"):
            return {}
        patient = step.case_id if tool == "conduct_case_conference" else step.patient_id
        assessment = self._assessment(patient)
        if assessment is None:
            return None
        inputs = {"assessment": assessment["artifact_id"]}
        if tool == "interpret_imaging":
            image = next((self._current("perform_imaging", item)
                          for item in self._steps["perform_imaging"]
                          if item.patient_id == patient and item.modality == step.modality), None)
            if image is None:
                return None
            inputs["image"] = image["artifact_id"]
        elif tool == "diagnose":
            panels = [item for item in self._steps["order_tests"]
                      if item.patient_id == patient and item.agent_id == step.agent_id]
            for panel in panels:
                actual = self._panel(panel)
                if not all(name in actual for name in panel.tests):
                    return None
                inputs.update({f"test:{name}": actual[name] for name in panel.tests})
            for image in self._steps["interpret_imaging"]:
                if image.patient_id == patient:
                    actual = self._current("interpret_imaging", image)
                    if actual is None:
                        return None
                    inputs[f"imaging:{image.modality}"] = actual["artifact_id"]
        elif tool in ("conduct_case_conference", "perform_surgery", "prepare_anesthesia", "manage_icu"):
            diagnoses = [item for item in self._steps["diagnose"] if item.patient_id == patient]
            if not diagnoses:
                return None
            for diagnosis in diagnoses:
                actual = self._current("diagnose", diagnosis)
                if actual is None:
                    return None
                inputs[f"diagnosis:{diagnosis.agent_id}"] = actual["artifact_id"]
            if tool == "perform_surgery":
                for preparation in self._steps["prepare_anesthesia"]:
                    if preparation.patient_id == patient:
                        actual = self._current("prepare_anesthesia", preparation)
                        if actual is None:
                            return None
                        inputs["anesthesia"] = actual["artifact_id"]
        return inputs

    def _record(self, tool, agent, arguments, result, success=True):
        result = {**result, "agent": agent, "artifact_id": f"{tool}:{len(self.events)}"}
        self.log_event(agent, tool, dict(arguments), success=success, result=result)
        return ToolResult(tool_name=tool, success=success, data=result,
                          message=result["status"])

    def _reject(self, tool, agent, arguments, reason):
        result = {"error": reason}
        self.log_event(agent, tool, dict(arguments), success=False, result=result)
        return ToolResult(tool_name=tool, success=False, data=result, message=reason)

    def assess_patient(self, agent_id: str, **kwargs: Any) -> ToolResult:
        step = self._step("assess_patient", agent_id, kwargs)
        if step is None:
            return self._reject("assess_patient", agent_id, kwargs, "unknown assigned patient")
        return self._record("assess_patient", agent_id, kwargs, {
            "patient_id": step.patient_id, "status": "assessed", "inputs": {}})

    def order_tests(self, agent_id: str, **kwargs: Any) -> ToolResult:
        step = self._step("order_tests", agent_id, kwargs)
        raw = kwargs.get("test_type")
        if step is None or not isinstance(raw, str):
            return self._reject("order_tests", agent_id, kwargs, "unknown patient or test_type")
        tests = list(dict.fromkeys(self._parse_list(raw)))
        if not tests or any(name not in step.tests for name in tests):
            return self._reject("order_tests", agent_id, kwargs, "unknown or foreign test")
        inputs = self._inputs("order_tests", step)
        if inputs is None:
            return self._reject("order_tests", agent_id, kwargs, "required patient results are absent or pending")
        return self._record("order_tests", agent_id, kwargs, {
            "patient_id": step.patient_id, "tests": tests, "status": "ordered", "inputs": inputs})

    def perform_imaging(self, agent_id: str, **kwargs: Any) -> ToolResult:
        step = self._step("perform_imaging", agent_id, kwargs)
        inputs = self._inputs("perform_imaging", step) if step is not None else None
        if inputs is None:
            return self._reject("perform_imaging", agent_id, kwargs, "unknown imaging target or missing assessment")
        return self._record("perform_imaging", agent_id, kwargs, {
            "patient_id": step.patient_id, "modality": step.modality,
            "status": "completed", "inputs": inputs})

    def interpret_imaging(self, agent_id: str, **kwargs: Any) -> ToolResult:
        step = self._step("interpret_imaging", agent_id, kwargs)
        inputs = self._inputs("interpret_imaging", step) if step is not None else None
        if inputs is None:
            return self._reject("interpret_imaging", agent_id, kwargs, "matching completed image is absent")
        success = not self.should_fail("interpret_imaging", agent_id)
        return self._record("interpret_imaging", agent_id, kwargs, {
            "patient_id": step.patient_id, "modality": step.modality,
            "status": "interpreted" if success else "inconclusive", "inputs": inputs}, success)

    def diagnose(self, agent_id: str, **kwargs: Any) -> ToolResult:
        step = self._step("diagnose", agent_id, kwargs)
        inputs = self._inputs("diagnose", step) if step is not None else None
        if inputs is None:
            return self._reject("diagnose", agent_id, kwargs, "current completed test results are absent")
        success = not self.should_fail("diagnose", agent_id)
        return self._record("diagnose", agent_id, kwargs, {
            "patient_id": step.patient_id, "symptoms": kwargs.get("symptoms"),
            "status": "diagnosed" if success else "inconclusive", "inputs": inputs}, success)

    def review_prescriptions(self, agent_id: str, **kwargs: Any) -> ToolResult:
        step = self._step("review_prescriptions", agent_id, kwargs)
        medications = kwargs.get("medications")
        if step is None or medications is not None and not isinstance(medications, str):
            return self._reject("review_prescriptions", agent_id, kwargs, "unknown patient or malformed medications")
        success = not self.should_fail("review_prescriptions", agent_id)
        return self._record("review_prescriptions", agent_id, kwargs, {
            "patient_id": step.patient_id, "medications": medications,
            "status": "reviewed" if success else "flagged", "inputs": {}}, success)

    def conduct_case_conference(self, agent_id: str, **kwargs: Any) -> ToolResult:
        step = self._step("conduct_case_conference", agent_id, kwargs)
        inputs = self._inputs("conduct_case_conference", step) if step is not None else None
        if inputs is None:
            return self._reject("conduct_case_conference", agent_id, kwargs, "current case results are absent")
        return self._record("conduct_case_conference", agent_id, kwargs, {
            "patient_id": step.case_id, "case_id": step.case_id, "attendees": kwargs.get("attendees"),
            "status": "conference held", "inputs": inputs})

    def perform_surgery(self, agent_id: str, **kwargs: Any) -> ToolResult:
        step = self._step("perform_surgery", agent_id, kwargs)
        inputs = self._inputs("perform_surgery", step) if step is not None else None
        if inputs is None:
            return self._reject("perform_surgery", agent_id, kwargs, "current diagnosis or preparation is absent")
        return self._record("perform_surgery", agent_id, kwargs, {
            "patient_id": step.patient_id, "procedure": step.procedure,
            "status": "completed", "inputs": inputs})

    def prepare_anesthesia(self, agent_id: str, **kwargs: Any) -> ToolResult:
        step = self._step("prepare_anesthesia", agent_id, kwargs)
        inputs = self._inputs("prepare_anesthesia", step) if step is not None else None
        if inputs is None:
            return self._reject("prepare_anesthesia", agent_id, kwargs, "current diagnosis is absent")
        return self._record("prepare_anesthesia", agent_id, kwargs, {
            "patient_id": step.patient_id, "type": kwargs.get("type"),
            "status": "anesthesia prepared", "inputs": inputs})

    def manage_icu(self, agent_id: str, **kwargs: Any) -> ToolResult:
        step = self._step("manage_icu", agent_id, kwargs)
        inputs = self._inputs("manage_icu", step) if step is not None else None
        if inputs is None:
            return self._reject("manage_icu", agent_id, kwargs, "current diagnosis is absent")
        return self._record("manage_icu", agent_id, kwargs, {
            "patient_id": step.patient_id, "status": "icu managed", "inputs": inputs})

    def make_tools(self) -> dict[str, Any]:
        return {tool: getattr(self, tool) for tool in self._steps}

    def _step_complete(self, tool, step):
        if tool == "order_tests":
            actual = self._panel(step)
            return all(name in actual for name in step.tests)
        return self._current(tool, step) is not None

    def is_complete(self) -> bool:
        return all(self._step_complete(tool, step)
                   for tool, steps in self._steps.items() for step in steps)

    @property
    def progress(self) -> dict[str, Any]:
        categories = {
            "assess_patient": "assessments", "order_tests": "test_orders",
            "perform_imaging": "imaging", "interpret_imaging": "imaging_interpretations",
            "diagnose": "diagnoses", "review_prescriptions": "prescription_reviews",
            "conduct_case_conference": "case_conferences", "perform_surgery": "surgeries",
            "prepare_anesthesia": "anesthesia_preps", "manage_icu": "icu_management",
        }
        result = {}
        for tool, steps in self._steps.items():
            values = {}
            for step in steps:
                target = step.case_id if tool == "conduct_case_conference" else step.patient_id
                suffix = (f"_{step.modality}" if tool in ("perform_imaging", "interpret_imaging")
                          else f"_{step.procedure}" if tool == "perform_surgery" else "")
                values[f"{step.agent_id}_{target}{suffix}"] = self._step_complete(tool, step)
            result[categories[tool]] = values
        return {**result, "all_complete": self.is_complete()}

