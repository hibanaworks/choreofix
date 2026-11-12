"""Scientific material, measurement and QC receipts shared by lab tasks.

The journal contains actual inputs and completed results, not a task cursor or
preferred instrument order. Task metadata names physical tools and resources.
"""
from ._base import ToolResult
from .sim_base import SimContext

_ANALYSES = {("CHEMIST", "synthesized_compound"), ("CHEMIST", "final_validation"),
             ("BIOLOGIST", "fluorescence"), ("ANALYST", "molecular_detail")}
_SEPARATIONS = {("CHEMIST", "compound_purification"), ("BIOLOGIST", "cell_cultures"),
                ("ANALYST", "verification")}


class LabQualitySim(SimContext):
    """Scientific data survives a rejected measurement; approvals name inputs.

    A repeated chemist analysis measures the same purified specimen and does
    not manufacture a new batch. Re-synthesis/purification does. Bioassays and
    QC are independently retryable; final validation needs matching completed
    approvals and quality data. No per-scientist progress table is maintained.
    """
    _DECISION_TOOLS = {"run_bioassay": 0.3, "verify_result": 0.3}

    def __init__(self, metadata):
        super().__init__()
        self._metadata = metadata
        for instrument in metadata["resources"]:
            self.init_resource(instrument)
        self.load_from_metadata(metadata)

    def tool_delay(self, tool_name, **kwargs):
        return tuple(self._metadata.get("tool_delays", {}).get(tool_name, (0.0, 0.0)))

    def _find(self, actor, tool, key=None, value=None, *, include_failure=False, **inputs):
        return next((index for index in range(len(self.events) - 1, -1, -1)
                     if (event := self.events[index]).agent == actor and event.tool == tool
                     and event.completed_at is not None
                     and (event.success or (include_failure and event.result.get("status")
                                            in ("bioassay_failed", "rejected")))
                     and (key is None or event.args.get(key) == value)
                     and all(event.result.get("inputs", {}).get(k) == v
                             for k, v in inputs.items())), None)

    def _synthesis(self):
        return self._find("CHEMIST", "run_synthesis")

    def _analysis(self, synthesis):
        if synthesis is None:
            return None
        return self._find("CHEMIST", "run_analysis", "sample", "synthesized_compound",
                          synthesis=synthesis)

    def _compound(self, synthesis):
        if synthesis is None:
            return None
        return self._find("CHEMIST", "run_separation", "material", "compound_purification",
                          synthesis=synthesis)

    def _assay(self, compound):
        if compound is None:
            return None
        latest = self._find("BIOLOGIST", "run_bioassay", include_failure=True,
                            purification=compound)
        return latest if latest is not None and self.events[latest].success else None

    def _approval(self, scientist, **inputs):
        latest = self._find("QC_CONTROLLER", "verify_result", "scientist", scientist,
                            include_failure=True, **inputs)
        return latest if latest is not None and self.events[latest].success else None

    def _record(self, actor, tool, arguments, inputs, *, valid=True, failure=None, extra=None):
        missing = [name for name, receipt in inputs.items() if receipt is None]
        ready = valid and not missing
        # Invalid/missing data is not an actual QC decision or biological trial
        # and cannot spend a valid-attempt failure injection or mint an artifact.
        rejected = ready and failure is not None and self.should_fail(tool, actor)
        success = ready and not rejected
        status = failure if rejected else ("complete" if success else "missing_or_invalid_input")
        result = {"scientist": actor, **arguments, "inputs": inputs,
                  "artifact_id": len(self.events) if success else None,
                  "status": status, "missing_inputs": missing, **(extra or {})}
        self.log_event(actor, tool, arguments, success, result)
        return ToolResult(tool, success, result)

    def run_synthesis(self, agent_id, compound):
        return self._record(agent_id, "run_synthesis", {"compound": compound}, {},
                            valid=agent_id == "CHEMIST" and isinstance(compound, str))

    def _analysis_inputs(self, agent_id, sample):
        """Actual material read by this instrument, independent of scheduling."""
        inputs = {}
        synthesis = self._synthesis()
        if (agent_id, sample) == ("CHEMIST", "synthesized_compound"):
            inputs["synthesis"] = synthesis
        elif (agent_id, sample) == ("BIOLOGIST", "fluorescence"):
            inputs["prepared_cells"] = self._find("BIOLOGIST", "run_separation",
                                                   "material", "cell_cultures")
        elif (agent_id, sample) in [("ANALYST", "molecular_detail"), ("CHEMIST", "final_validation")]:
            compound = self._compound(synthesis)
            assay = self._assay(compound)
            inputs = {"synthesis": synthesis, "purification": compound, "bioassay": assay}
            if agent_id == "CHEMIST":
                analysis = self._analysis(synthesis)
                inputs.update({
                    "purification_qc": self._approval("CHEMIST", synthesis=synthesis,
                                                       purification=compound, analysis=analysis),
                    "bioassay_qc": self._approval("BIOLOGIST", synthesis=synthesis,
                                                   purification=compound, bioassay=assay),
                    "molecular": self._find("ANALYST", "run_analysis", "sample", "molecular_detail",
                                             purification=compound, bioassay=assay),
                    "verification": self._find("ANALYST", "run_separation", "material", "verification",
                                                purification=compound, bioassay=assay),
                })
        return inputs

    def run_analysis(self, agent_id, sample):
        valid = isinstance(sample, str) and (agent_id, sample) in _ANALYSES
        return self._record(agent_id, "run_analysis", {"sample": sample},
                            self._analysis_inputs(agent_id, sample), valid=valid)

    def run_separation(self, agent_id, material):
        valid = isinstance(material, str) and (agent_id, material) in _SEPARATIONS
        inputs = {}
        synthesis = self._synthesis()
        if (agent_id, material) == ("CHEMIST", "compound_purification"):
            inputs = {"synthesis": synthesis, "analysis": self._analysis(synthesis)}
        elif (agent_id, material) == ("ANALYST", "verification"):
            compound = self._compound(synthesis)
            inputs = {"synthesis": synthesis, "purification": compound, "bioassay": self._assay(compound)}
        extra = {"compound": self.events[synthesis].args["compound"]} if synthesis is not None else {}
        return self._record(agent_id, "run_separation", {"material": material}, inputs,
                            valid=valid, extra=extra)

    def run_bioassay(self, agent_id, compound, cell_type=""):
        synthesis = self._synthesis()
        valid = (agent_id == "BIOLOGIST" and isinstance(compound, str) and isinstance(cell_type, str)
                 and synthesis is not None and compound == self.events[synthesis].args["compound"])
        prepared = self._find("BIOLOGIST", "run_separation", "material", "cell_cultures")
        calibration = self._find("BIOLOGIST", "run_analysis", "sample", "fluorescence",
                                  prepared_cells=prepared)
        return self._record(agent_id, "run_bioassay", {"compound": compound, "cell_type": cell_type},
                            {"synthesis": synthesis, "purification": self._compound(synthesis),
                             "prepared_cells": prepared, "calibration": calibration},
                            valid=valid, failure="bioassay_failed")

    def verify_result(self, agent_id, scientist="", data_type=""):
        synthesis = self._synthesis()
        compound = self._compound(synthesis)
        inputs = {"synthesis": synthesis, "purification": compound}
        valid = agent_id == "QC_CONTROLLER"
        if scientist == "CHEMIST" and data_type == "purification":
            current = self._analysis(synthesis)
            inputs["analysis"] = current
        elif scientist == "BIOLOGIST" and data_type == "bioassay":
            current = self._assay(compound)
            inputs["bioassay"] = current
        else:
            current = None
            valid = False
        previous = self._find("QC_CONTROLLER", "verify_result", "scientist", scientist,
                               include_failure=True, synthesis=synthesis, purification=compound)
        if (previous is not None and not self.events[previous].success
                and (current is None or current <= previous)):
            valid = False
        return self._record(agent_id, "verify_result", {"scientist": scientist, "data_type": data_type},
                            inputs, valid=valid, failure="rejected", extra={"qc_agent": agent_id})

    def make_tools(self):
        return {"run_analysis": self.run_analysis, "run_separation": self.run_separation,
                "run_bioassay": self.run_bioassay, "run_synthesis": self.run_synthesis,
                "verify_result": self.verify_result}

    def is_complete(self):
        return self._find("CHEMIST", "run_analysis", "sample", "final_validation") is not None

    @property
    def progress(self):
        return {"completed_artifacts": [e.result for e in self.events
                                       if e.success and e.completed_at is not None],
                "all_complete": self.is_complete()}
