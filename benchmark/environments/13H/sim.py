"""13H actual batches, finite reagents, cascading QC and physical inspection.

No choreography is consulted. Rejection cannot be cleared with an old assay,
and independent safety inspections may happen anywhere in the work schedule.
"""
import json
from pathlib import Path

from benchmark.tools.sim_lab_qc import LabQualitySim

_METADATA = json.loads((Path(__file__).resolve().parents[2]
                        / "descriptions" / "13H" / "metadata.json").read_text())


class PharmaLabCascadingSim(LabQualitySim):
    _DECISION_TOOLS = {"run_synthesis": 0.3, "verify_result": 0.3}

    def __init__(self):
        super().__init__(_METADATA)

    def _reagents(self):
        # Reagent inventory is material: every real attempt consumes one unit,
        # even if it fails or its output is still being processed.
        return self._metadata["resource_capacity"]["REAGENT_SUPPLY"] - sum(
            event.tool == "run_synthesis"
            and event.result.get("status") in ("complete", "synthesis_failed")
            for event in self.events)

    def run_synthesis(self, agent_id, compound):
        valid = agent_id == "CHEMIST" and isinstance(compound, str) and bool(compound.strip())
        remaining = self._reagents()
        if valid and remaining == 0:
            return self._record(agent_id, "run_synthesis", {"compound": compound}, {},
                                valid=False, extra={"status": "reagent_exhausted",
                                                    "reagent_units_remaining": 0})
        return self._record(agent_id, "run_synthesis", {"compound": compound}, {},
                            valid=valid, failure="synthesis_failed",
                            extra={"reagent_units_remaining": remaining - int(valid)})

    def _bio_rejection(self):
        latest = self._find("QC_CONTROLLER", "verify_result", "scientist", "BIOLOGIST",
                            include_failure=True)
        return latest if latest is not None and not self.events[latest].success else None

    def _assay(self, compound):
        assay = super()._assay(compound)
        rejected = self._bio_rejection()
        synthesis = self._synthesis()
        if rejected is not None and (synthesis is None or synthesis <= rejected):
            return None
        return assay

    def run_chromatography(self, agent_id, sample):
        synthesis = self._synthesis()
        compound = self._compound(synthesis)
        valid = (agent_id == "ANALYST" and isinstance(sample, str) and synthesis is not None
                 and sample == self.events[synthesis].args["compound"])
        return self._record(agent_id, "run_chromatography", {"sample": sample},
                            {"synthesis": synthesis, "purification": compound,
                             "bioassay": self._assay(compound)}, valid=valid)

    def _analysis_inputs(self, agent_id, sample):
        inputs = super()._analysis_inputs(agent_id, sample)
        if (agent_id, sample) in [("ANALYST", "molecular_detail"), ("CHEMIST", "final_validation")]:
            inputs["chromatography"] = self._find(
                "ANALYST", "run_chromatography", synthesis=inputs["synthesis"],
                purification=inputs["purification"], bioassay=inputs["bioassay"])
        if (agent_id, sample) == ("CHEMIST", "final_validation"):
            # 13H asks QC to approve the bioassay; unlike 13M it does not
            # require a separate purification-QC request by the chemist.
            inputs.pop("purification_qc")
        return inputs

    def perform_inspection(self, agent_id, notes=""):
        return self._record(agent_id, "perform_inspection", {"notes": notes}, {},
                            valid=agent_id == "SAFETY_OFFICER" and isinstance(notes, str),
                            extra={"instruments": ["CENTRIFUGE", "CHROMATOGRAPH",
                                                   "FUME_HOOD", "SPECTROMETER"]})

    def make_tools(self):
        return {**super().make_tools(), "run_chromatography": self.run_chromatography,
                "perform_inspection": self.perform_inspection}

    def is_complete(self):
        final_inputs = self._analysis_inputs("CHEMIST", "final_validation")
        return (all(receipt is not None for receipt in final_inputs.values())
                and self._find("CHEMIST", "run_analysis", "sample", "final_validation",
                               **final_inputs) is not None
                and self._find("SAFETY_OFFICER", "perform_inspection") is not None)

    @property
    def progress(self):
        return {**super().progress, "reagent_units_remaining": self._reagents()}
