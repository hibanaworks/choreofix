"""Pharma Lab simulation config for task 13E.

Three scientists share SPECTROMETER + CENTRIFUGE.  Contains two
deadlock patterns:
  1. Reverse lock ordering (CHEMIST: spec->cent, BIOLOGIST: cent->spec)
  2. Hold-and-wait (CHEMIST holds spec while waiting for ANALYST who needs spec)
"""

import json
from pathlib import Path

from benchmark.tools.sim_lab import (
    AnalysisStep, SeparationStep, TestStep,
)
from benchmark.tools._base import ToolResult
from benchmark.tools.sim_base import SimContext

_METADATA = json.loads(
    (Path(__file__).resolve().parent.parent.parent
     / "descriptions" / "13E" / "metadata.json").read_text()
)

_INSTRUMENTS = ["SPECTROMETER", "CENTRIFUGE"]

_ANALYSES = [
    AnalysisStep(scientist_id="CHEMIST", sample="raw_materials"),
    AnalysisStep(scientist_id="CHEMIST", sample="final_validation"),
    AnalysisStep(scientist_id="BIOLOGIST", sample="fluorescence"),
    AnalysisStep(scientist_id="ANALYST", sample="molecular_detail"),
]

_SEPARATIONS = [
    SeparationStep(scientist_id="CHEMIST", material="compound_purification"),
    SeparationStep(scientist_id="BIOLOGIST", material="cell_cultures"),
    SeparationStep(scientist_id="ANALYST", material="verification"),
]

_TESTS = [
    TestStep(scientist_id="BIOLOGIST", test_type="bioassay"),
]


class PharmaLabSim(SimContext):
    """Material and measurement receipts, independent of one task schedule.

    Preparation/calibration create actual assay inputs. Quality measurements
    bind one completed assay of the same purified compound. Final validation
    consumes both completed quality measurements; later evidence cannot repair
    an earlier failed attempt. No additional workflow cursor is maintained.
    """
    _DECISION_TOOLS: dict = {}

    def __init__(self) -> None:
        super().__init__()
        for instrument in _INSTRUMENTS:
            self.init_resource(instrument)
        self.load_from_metadata(_METADATA)

    def tool_delay(self, tool_name, **kwargs):
        return tuple(_METADATA['tool_delays'][tool_name])

    def _find(self, actor, tool, key, value, **inputs):
        return next((index for index in range(len(self.events)-1,-1,-1)
                     if (event:=self.events[index]).agent==actor and event.tool==tool
                     and event.success and event.completed_at is not None
                     and event.args.get(key)==value
                     and all(event.result['inputs'].get(k)==v for k,v in inputs.items())),None)

    def _compound(self):
        raw=self._find('CHEMIST','run_analysis','sample','raw_materials')
        if raw is None:
            return None
        return self._find('CHEMIST','run_separation','material','compound_purification',raw=raw)

    def _assay(self, compound):
        if compound is None:
            return None
        return self._find('BIOLOGIST','run_bioassay','compound','compound_purification',compound=compound)

    def _record(self, actor, tool, arguments, inputs, valid):
        missing=[name for name,receipt in inputs.items() if receipt is None]
        success=valid and not missing
        data={'scientist':actor,**arguments,'inputs':inputs,
              'artifact_id':len(self.events) if success else None,
              'status':'complete' if success else 'missing_or_invalid_input',
              'missing_inputs':missing}
        self.log_event(actor,tool,arguments,success,data)
        return ToolResult(tool,success,data)

    def run_analysis(self, agent_id, sample):
        valid=isinstance(sample,str) and (agent_id,sample) in {(s.scientist_id,s.sample) for s in _ANALYSES}
        inputs={}
        if (agent_id,sample)==('BIOLOGIST','fluorescence'):
            inputs['prepared_cells']=self._find('BIOLOGIST','run_separation','material','cell_cultures')
        elif (agent_id,sample) in [('ANALYST','molecular_detail'),('CHEMIST','final_validation')]:
            compound=self._compound();assay=self._assay(compound)
            inputs={'compound':compound,'bioassay':assay}
            if agent_id=='CHEMIST':
                inputs['molecular']=self._find('ANALYST','run_analysis','sample','molecular_detail',compound=compound,bioassay=assay)
                inputs['verification']=self._find('ANALYST','run_separation','material','verification',compound=compound,bioassay=assay)
        return self._record(agent_id,'run_analysis',{'sample':sample},inputs,valid)

    def run_separation(self, agent_id, material):
        valid=isinstance(material,str) and (agent_id,material) in {(s.scientist_id,s.material) for s in _SEPARATIONS}
        inputs={}
        if (agent_id,material)==('CHEMIST','compound_purification'):
            inputs['raw']=self._find('CHEMIST','run_analysis','sample','raw_materials')
        elif (agent_id,material)==('ANALYST','verification'):
            compound=self._compound()
            inputs={'compound':compound,'bioassay':self._assay(compound)}
        return self._record(agent_id,'run_separation',{'material':material},inputs,valid)

    def run_bioassay(self, agent_id, compound, cell_type=''):
        valid=agent_id=='BIOLOGIST' and compound=='compound_purification' and isinstance(cell_type,str)
        prepared=self._find('BIOLOGIST','run_separation','material','cell_cultures')
        calibration=self._find('BIOLOGIST','run_analysis','sample','fluorescence',prepared_cells=prepared)
        return self._record(agent_id,'run_bioassay',{'compound':compound,'cell_type':cell_type},
                            {'compound':self._compound(),'prepared_cells':prepared,'calibration':calibration},valid)

    def make_tools(self):
        return {'run_analysis':self.run_analysis,'run_separation':self.run_separation,'run_bioassay':self.run_bioassay}

    def is_complete(self):
        return self._find('CHEMIST','run_analysis','sample','final_validation') is not None

    @property
    def progress(self):
        return {'completed_artifacts':[e.result for e in self.events if e.success and e.completed_at is not None],
                'all_complete':self.is_complete()}
