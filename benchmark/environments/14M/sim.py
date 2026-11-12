"""Drug Discovery Pipeline with Regulatory Review simulation for task 14M.

Five scientists share hplc + mass_spec + cell_lab.  Adds:
  1. Regulatory review gate after lead's decision (approve/reject)
  2. Assay failures (cell_assay and hplc can fail)
  3. Regulatory rejection loops back to lead, who may re-request assays
"""

import json
from pathlib import Path
from benchmark.tools.sim_pharma import (
    HplcStep, MassSpecStep, CellAssayStep, DiscoveryMaterialSim,
)

_INSTRUMENTS = ["HPLC", "MASS_SPEC", "CELL_LAB"]

_HPLC_STEPS = [
    HplcStep(scientist_id="CHEMIST", sample="synthesis_qc"),
    HplcStep(scientist_id="CHEMIST", sample="documentation"),
    HplcStep(scientist_id="BIOLOGIST", sample="stability"),
    HplcStep(scientist_id="TOXICOLOGIST", sample="purity_check"),
    HplcStep(scientist_id="LEAD_SCIENTIST", sample="structural_confirmation"),
]

_MASS_SPEC_STEPS = [
    MassSpecStep(scientist_id="CHEMIST", sample="molecular_id"),
    MassSpecStep(scientist_id="TOXICOLOGIST", sample="baseline_markers"),
    MassSpecStep(scientist_id="LEAD_SCIENTIST", sample="reference_methods"),
    MassSpecStep(scientist_id="LEAD_SCIENTIST", sample="characterization"),
]

_CELL_ASSAY_STEPS = [
    CellAssayStep(scientist_id="BIOLOGIST", assay_type="culture_prep"),
    CellAssayStep(scientist_id="BIOLOGIST", assay_type="bioassay"),
    CellAssayStep(scientist_id="TOXICOLOGIST", assay_type="cytotoxicity"),
]


_METADATA = json.loads(
    (Path(__file__).resolve().parent.parent.parent
     / "descriptions" / "14M" / "metadata.json").read_text()
)


class DrugDiscoveryRegulatorySim(DiscoveryMaterialSim):
    """14M uses actual analysis and decision receipts before regulatory review."""

    _DECISION_TOOLS = {
        "run_hplc_analysis": 0.2,
        "run_cell_assay": 0.3,
        "verify_regulatory_compliance": 0.2,
    }

    def __init__(self) -> None:
        super().__init__(_METADATA, [
            ("run_hplc_analysis", "sample", _HPLC_STEPS),
            ("run_mass_spec_analysis", "sample", _MASS_SPEC_STEPS),
            ("run_cell_assay", "assay_type", _CELL_ASSAY_STEPS),
        ])
