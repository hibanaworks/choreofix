"""Drug Discovery Pipeline simulation config for task 14E.

Four scientists share hplc + mass_spec + cell_lab.  Contains three
deadlock patterns:
  1. Reverse lock ordering — BIOLOGIST (cell_lab -> hplc) vs
     TOXICOLOGIST (hplc -> cell_lab) after receiving compound
  2. Hold-across-receive — CHEMIST uses hplc at start AND end,
     tempting to hold across wait for lead who also needs hplc
  3. TOXICOLOGIST hold-and-wait — description suggests keeping
     HPLC method loaded while moving to cell_lab
"""

import json
from pathlib import Path

from benchmark.tools.sim_pharma import (
    HplcStep, MassSpecStep, CellAssayStep,
)
from benchmark.tools.sim_pharma import DiscoveryMaterialSim

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
     / "descriptions" / "14E" / "metadata.json").read_text()
)


class DrugDiscoverySim(DiscoveryMaterialSim):
    """14E's declared analyses, instruments and common material semantics."""

    def __init__(self) -> None:
        super().__init__(_METADATA, [
            ("run_hplc_analysis", "sample", _HPLC_STEPS),
            ("run_mass_spec_analysis", "sample", _MASS_SPEC_STEPS),
            ("run_cell_assay", "assay_type", _CELL_ASSAY_STEPS),
        ])
