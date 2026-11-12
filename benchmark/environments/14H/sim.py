"""14H: actual formulations, finite samples and clinical project decisions."""
import json
from pathlib import Path

from benchmark.tools.sim_pharma import MultiRoundMaterialSim


class DrugDiscoveryMultiRoundSim(MultiRoundMaterialSim):
    """The common world observes material outcomes, not an agent schedule."""

    _DECISION_TOOLS = {
        'run_hplc_analysis': 0.2,
        'run_cell_assay': 0.3,
        'prepare_formulation': 0.2,
        'verify_regulatory_compliance': 0.2,
    }

    def __init__(self):
        super().__init__(json.loads(
            (Path(__file__).resolve().parent.parent.parent
             / 'descriptions' / '14H' / 'metadata.json').read_text()))
