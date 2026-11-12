"""13M material and QC configuration for the common scientific world."""
import json
from pathlib import Path

from benchmark.tools.sim_lab_qc import LabQualitySim

_METADATA = json.loads((Path(__file__).resolve().parents[2]
                        / "descriptions" / "13M" / "metadata.json").read_text())


class PharmaLabQCSim(LabQualitySim):
    def __init__(self):
        super().__init__(_METADATA)
