"""Parameterized pharmaceutical discovery simulation shared by 3A+ tasks."""

from __future__ import annotations

from dataclasses import dataclass

from ._base import ToolResult
from .sim_base import SimContext


@dataclass
class HplcStep:
    """An HPLC analysis step for one scientist."""

    scientist_id: str
    sample: str


@dataclass
class MassSpecStep:
    """A mass spectrometry analysis step for one scientist."""

    scientist_id: str
    sample: str


@dataclass
class CellAssayStep:
    """A cell-based assay step for one scientist."""

    scientist_id: str
    assay_type: str


class ScientificResults(SimContext):
    """Owned scientific results and their actual input occurrences."""
    _DECISION_TOOLS: dict = {}

    def __init__(self, metadata) -> None:
        super().__init__()
        for instrument in metadata["resources"]:
            self.init_resource(instrument)
        self.load_from_metadata(metadata)
        self._tool_delays = metadata["tool_delays"]

    def tool_delay(self, tool_name: str, **kwargs) -> tuple[float, float]:
        # A common benchmark latency model, not measured laboratory time.
        # Every method uses the same values and caller-selected multiplier.
        return tuple(self._tool_delays[tool_name])

    def _latest(self, actor, tool, key, value, *, inputs=None, completed=True):
        for index in range(len(self.events)-1, -1, -1):
            event = self.events[index]
            if (event.agent == actor and event.tool == tool
                    and (key is None or event.args.get(key) == value)
                    and all(event.result.get("inputs", {}).get(k) == v
                            for k, v in (inputs or {}).items())):
                # A failed or unfinished replacement is not the older positive
                # result. Inputs identify the actual material, not a done flag.
                return index if not completed or event.completed_at is not None else None
        return None

    def _find(self, actor, tool, key, value, *, inputs=None, completed=True):
        at = self._latest(actor, tool, key, value, inputs=inputs, completed=completed)
        return at if at is not None and self.events[at].success else None

    def _record(self, actor, tool, arguments, inputs, *, valid=True, extra=None,
                failure_status='injected_failure'):
        missing = [name for name, receipt in inputs.items() if receipt is None]
        injected = valid and not missing and tool in self._DECISION_TOOLS and self.should_fail(tool, actor)
        success = valid and not missing and not injected
        result = {'scientist': actor, **arguments, 'inputs': inputs,
                  'event_id': len(self.events),
                  'artifact_id': len(self.events) if success else None,
                  'status': ('complete' if success else failure_status if injected
                             else 'missing_or_invalid_input'),
                  'missing_inputs': missing, **(extra or {})}
        self.log_event(actor, tool, arguments, success, result)
        return ToolResult(tool, success, result)


class DiscoveryMaterialSim(ScientificResults):
    """14E/14M material, decision and approval receipts."""

    def __init__(self, metadata, analyses):
        super().__init__(metadata)
        self._analyses = {
            tool: frozenset((step.scientist_id, getattr(step, argument)) for step in steps)
            for tool, argument, steps in analyses
        }

    def _compound(self):
        qc = self._find('CHEMIST', 'run_hplc_analysis', 'sample', 'synthesis_qc')
        if qc is None:
            return None
        return self._find('CHEMIST', 'run_mass_spec_analysis', 'sample', 'molecular_id',
                          inputs={'qc': qc})

    def _decision(self, compound):
        if compound is None:
            return None
        inputs = self._decision_inputs(compound)
        if any(value is None for value in inputs.values()):
            return None
        return next((index for index in range(len(self.events)-1, -1, -1)
                     if (event := self.events[index]).agent == 'LEAD_SCIENTIST'
                     and event.tool == 'submit_decision' and event.success
                     and isinstance(event.args.get('go'), bool)
                     and event.completed_at is not None
                     and event.result.get('inputs') == inputs), None)

    def _approval(self, compound, decision):
        if compound is None or decision is None:
            return None
        return self._find('REGULATORY_SPECIALIST', 'verify_regulatory_compliance',
                          'decision', 'go_no_go', inputs={'compound': compound, 'decision': decision})

    def run_hplc_analysis(self, agent_id, sample):
        inputs = {}
        valid = isinstance(sample, str) and (agent_id, sample) in self._analyses["run_hplc_analysis"]
        if (agent_id, sample) != ('CHEMIST', 'synthesis_qc'):
            compound = self._compound()
            inputs['compound'] = compound
            if (agent_id, sample) == ('BIOLOGIST', 'stability'):
                inputs['medium'] = self._find('BIOLOGIST', 'run_cell_assay', 'assay_type',
                                              'bioassay', inputs={'compound': compound},
                                              completed=False) if compound is not None else None
            elif (agent_id, sample) == ('CHEMIST', 'documentation'):
                inputs['decision'] = self._decision(compound)
                if 'REGULATORY_SPECIALIST' in self._agent_resources:
                    inputs['regulatory'] = self._approval(compound, inputs['decision'])
        return self._record(agent_id, 'run_hplc_analysis', {'sample': sample}, inputs, valid=valid)

    def run_mass_spec_analysis(self, agent_id, sample):
        inputs = {}
        valid = isinstance(sample, str) and (agent_id, sample) in self._analyses["run_mass_spec_analysis"]
        if (agent_id, sample) == ('CHEMIST', 'molecular_id'):
            inputs['qc'] = self._find('CHEMIST', 'run_hplc_analysis', 'sample', 'synthesis_qc')
        elif (agent_id, sample) == ('LEAD_SCIENTIST', 'characterization'):
            inputs['compound'] = self._compound()
            inputs['reference'] = self._find('LEAD_SCIENTIST', 'run_mass_spec_analysis',
                                              'sample', 'reference_methods')
        return self._record(agent_id, 'run_mass_spec_analysis', {'sample': sample}, inputs, valid=valid)

    def run_cell_assay(self, agent_id, assay_type):
        inputs = {}
        valid = isinstance(assay_type, str) and (agent_id, assay_type) in self._analyses["run_cell_assay"]
        if (agent_id, assay_type) != ('BIOLOGIST', 'culture_prep'):
            compound = self._compound()
            inputs['compound'] = compound
            inputs['prepared_cells'] = self._find('BIOLOGIST', 'run_cell_assay',
                                                  'assay_type', 'culture_prep')
            if agent_id == 'TOXICOLOGIST':
                inputs['baseline'] = self._find('TOXICOLOGIST', 'run_mass_spec_analysis',
                                                'sample', 'baseline_markers')
                inputs['purity'] = self._find('TOXICOLOGIST', 'run_hplc_analysis', 'sample',
                                              'purity_check', inputs={'compound': compound})
        return self._record(agent_id, 'run_cell_assay', {'assay_type': assay_type}, inputs, valid=valid)

    def _decision_inputs(self, compound):
        cells = self._find('BIOLOGIST', 'run_cell_assay', 'assay_type', 'culture_prep')
        baseline = self._find('TOXICOLOGIST', 'run_mass_spec_analysis', 'sample', 'baseline_markers')
        reference = self._find('LEAD_SCIENTIST', 'run_mass_spec_analysis', 'sample', 'reference_methods')
        purity = self._find('TOXICOLOGIST', 'run_hplc_analysis', 'sample', 'purity_check',
                            inputs={'compound': compound})
        assay = self._find('BIOLOGIST', 'run_cell_assay', 'assay_type', 'bioassay',
                           inputs={'compound': compound, 'prepared_cells': cells}) if compound is not None else None
        return {
            'compound': compound,
            'bioassay': assay,
            'stability': self._find('BIOLOGIST', 'run_hplc_analysis', 'sample', 'stability',
                                     inputs={'compound': compound, 'medium': assay}) if assay is not None else None,
            'toxicity': self._find('TOXICOLOGIST', 'run_cell_assay', 'assay_type', 'cytotoxicity',
                                    inputs={'compound': compound, 'prepared_cells': cells,
                                            'baseline': baseline, 'purity': purity}),
            'structural': self._find('LEAD_SCIENTIST', 'run_hplc_analysis', 'sample',
                                      'structural_confirmation', inputs={'compound': compound}),
            'characterization': self._find('LEAD_SCIENTIST', 'run_mass_spec_analysis', 'sample',
                                            'characterization', inputs={'compound': compound,
                                                                         'reference': reference}),
        }

    def submit_decision(self, agent_id, go):
        inputs = self._decision_inputs(self._compound())
        return self._record(agent_id, 'submit_decision', {'go': go}, inputs,
                            valid=agent_id == 'LEAD_SCIENTIST' and isinstance(go, bool))

    def verify_regulatory_compliance(self, agent_id, decision):
        compound = self._compound()
        current = self._decision(compound)
        prior = next((event for event in reversed(self.events)
                      if event.agent == 'REGULATORY_SPECIALIST'
                      and event.tool == 'verify_regulatory_compliance'
                      and event.result.get('inputs', {}).get('decision') == current
                      and event.result.get('status') in ('complete', 'injected_failure')), None)
        # Rejection needs an actual new lead decision. Extra assays remain
        # optional, as in the task; their updated evidence enters that decision.
        inputs = {'compound': compound, 'decision': current}
        if prior is not None and not prior.success:
            inputs['reconsidered_decision'] = None
        return self._record(agent_id, 'verify_regulatory_compliance', {'decision': decision}, inputs,
                            valid=(agent_id == 'REGULATORY_SPECIALIST' and decision == 'go_no_go'
                                   and 'REGULATORY_SPECIALIST' in self._agent_resources))

    def make_tools(self):
        tools = {'run_hplc_analysis': self.run_hplc_analysis,
                 'run_mass_spec_analysis': self.run_mass_spec_analysis,
                 'run_cell_assay': self.run_cell_assay, 'submit_decision': self.submit_decision}
        if 'REGULATORY_SPECIALIST' in self._agent_resources:
            tools['verify_regulatory_compliance'] = self.verify_regulatory_compliance
        return tools

    def is_complete(self):
        compound = self._compound()
        decision = self._decision(compound)
        if compound is None or decision is None:
            return False
        inputs = {'compound': compound, 'decision': decision}
        if 'REGULATORY_SPECIALIST' in self._agent_resources:
            inputs['regulatory'] = self._approval(compound, decision)
            if inputs['regulatory'] is None:
                return False
        return self._find('CHEMIST', 'run_hplc_analysis', 'sample', 'documentation', inputs=inputs) is not None

    @property
    def progress(self):
        return {'completed_artifacts': [event.result for event in self.events
                                       if event.success and event.completed_at is not None],
                'all_complete': self.is_complete()}


class MultiRoundMaterialSim(ScientificResults):
    """Actual 14H material/results and three consumable biological samples.

    No scientist schedule or choreography is consulted. Completed negative
    measurements remain evidence for an abort; they never become positive
    measurements or replenish the stock. Culture preparation consumes no unit.
    """

    def _compound(self):
        qc = self._find('CHEMIST', 'run_hplc_analysis', 'sample', 'synthesis_qc')
        return self._find('CHEMIST', 'run_mass_spec_analysis', 'sample', 'molecular_id',
                          inputs={'qc': qc}) if qc is not None else None

    def _formulation(self):
        compound = self._compound()
        return self._find('FORMULATION_SCIENTIST', 'prepare_formulation', 'compound',
                          f'compound-{compound}', inputs={'compound': compound}) if compound is not None else None

    @property
    def samples_remaining(self):
        used = {event.result['sample_unit'] for event in self.events
                if event.result.get('sample_unit') is not None}
        return self._resource_capacity['BIOLOGICAL_SAMPLES'] - len(used)

    def _assay(self, actor, assay, inputs):
        at = self._latest(actor, 'run_cell_assay', 'assay_type', assay, inputs=inputs)
        return at if at is not None and self.events[at].result.get('status') in {
            'complete', 'injected_failure', 'exhausted'} else None

    def _clinical_inputs(self):
        formulation = self._formulation()
        cells = self._find('BIOLOGIST', 'run_cell_assay', 'assay_type', 'culture_prep')
        baseline = self._find('TOXICOLOGIST', 'run_mass_spec_analysis', 'sample', 'baseline_markers')
        purity = self._find('TOXICOLOGIST', 'run_hplc_analysis', 'sample', 'purity_check',
                            inputs={'formulation': formulation})
        return {
            'formulation': formulation,
            'bioassay': self._assay('BIOLOGIST', 'bioassay',
                                   {'formulation': formulation, 'cells': cells}),
            'stability': self._find('BIOLOGIST', 'run_hplc_analysis', 'sample', 'stability',
                                    inputs={'formulation': formulation, 'cells': cells}),
            'purity': purity,
            'toxicity': self._assay('TOXICOLOGIST', 'cytotoxicity',
                                    {'formulation': formulation, 'cells': cells,
                                     'baseline': baseline, 'purity': purity}),
        }

    def _clinical(self):
        inputs = self._clinical_inputs()
        if any(value is None for value in inputs.values()):
            return None
        return self._find('CLINICAL_LEAD', 'review_clinical_data', 'data_type',
                          'combined_review', inputs=inputs)

    def _regulatory(self):
        clinical = self._clinical()
        if clinical is None:
            return None
        at = self._latest('REGULATORY_SPECIALIST', 'verify_regulatory_compliance',
                          None, None, inputs={'clinical': clinical})
        return at if at is not None and self.events[at].result.get('status') in {
            'complete', 'clinical_hold'} else None

    def _project_inputs(self):
        return {'compound': self._compound(), 'formulation': self._formulation(),
                'clinical': self._clinical(), 'regulatory': self._regulatory()}

    def _terminal_inputs(self):
        inputs = self._project_inputs()
        decision = self._latest('PROJECT_DIRECTOR', 'submit_project_decision', None, None,
                                inputs=inputs)
        if (decision is None or not self.events[decision].success
                or self.events[decision].args.get('outcome') not in ('continue', 'abort')):
            decision = None
        return {**inputs, 'decision': decision}

    def run_hplc_analysis(self, agent_id, sample):
        valid = (agent_id in {'CHEMIST', 'BIOLOGIST', 'TOXICOLOGIST', 'REGULATORY_SPECIALIST'}
                 and isinstance(sample, str) and bool(sample))
        inputs = {}
        if (agent_id, sample) == ('CHEMIST', 'documentation'):
            inputs = self._terminal_inputs()
        elif (agent_id, sample) != ('CHEMIST', 'synthesis_qc'):
            inputs['formulation'] = self._formulation()
            if (agent_id, sample) == ('BIOLOGIST', 'stability'):
                inputs['cells'] = self._find('BIOLOGIST', 'run_cell_assay',
                                             'assay_type', 'culture_prep')
        return self._record(agent_id, 'run_hplc_analysis', {'sample': sample}, inputs, valid=valid)

    def run_mass_spec_analysis(self, agent_id, sample):
        valid = (agent_id in {'CHEMIST', 'TOXICOLOGIST', 'CLINICAL_LEAD', 'REGULATORY_SPECIALIST'}
                 and isinstance(sample, str) and bool(sample))
        inputs = {}
        if (agent_id, sample) == ('CHEMIST', 'molecular_id'):
            inputs['qc'] = self._find('CHEMIST', 'run_hplc_analysis', 'sample', 'synthesis_qc')
        elif (agent_id, sample) != ('TOXICOLOGIST', 'baseline_markers'):
            inputs['formulation'] = self._formulation()
        extra = {'compound': f'compound-{len(self.events)}'} if (
            agent_id, sample) == ('CHEMIST', 'molecular_id') else None
        return self._record(agent_id, 'run_mass_spec_analysis', {'sample': sample}, inputs,
                            valid=valid, extra=extra)

    def prepare_formulation(self, agent_id, compound):
        actual = self._compound()
        return self._record(agent_id, 'prepare_formulation', {'compound': compound},
                            {'compound': actual},
                            valid=(agent_id == 'FORMULATION_SCIENTIST' and actual is not None
                                   and compound == f'compound-{actual}'),
                            extra={'formulation': f'formulation-{len(self.events)}'})

    def run_cell_assay(self, agent_id, assay_type):
        valid = isinstance(assay_type, str) and (agent_id, assay_type) in {
            ('BIOLOGIST', 'culture_prep'), ('BIOLOGIST', 'bioassay'),
            ('TOXICOLOGIST', 'cytotoxicity')}
        if (agent_id, assay_type) == ('BIOLOGIST', 'culture_prep'):
            return self._record(agent_id, 'run_cell_assay', {'assay_type': assay_type}, {})
        inputs = {'formulation': self._formulation(),
                  'cells': self._find('BIOLOGIST', 'run_cell_assay', 'assay_type', 'culture_prep')}
        if agent_id == 'TOXICOLOGIST':
            inputs['baseline'] = self._find('TOXICOLOGIST', 'run_mass_spec_analysis',
                                           'sample', 'baseline_markers')
            inputs['purity'] = self._find('TOXICOLOGIST', 'run_hplc_analysis', 'sample',
                                         'purity_check', inputs={'formulation': inputs['formulation']})
        used = {event.result['sample_unit'] for event in self.events
                if event.result.get('sample_unit') is not None}
        unit = next((unit for unit in range(self._resource_capacity['BIOLOGICAL_SAMPLES'])
                     if unit not in used), None)
        actual = valid and all(value is not None for value in inputs.values())
        if actual and unit is None:
            return self._record(agent_id, 'run_cell_assay', {'assay_type': assay_type},
                                inputs, valid=False,
                                extra={'status': 'exhausted', 'sample_unit': None})
        return self._record(agent_id, 'run_cell_assay', {'assay_type': assay_type}, inputs,
                            valid=valid, extra={'sample_unit': unit if actual else None})

    def review_clinical_data(self, agent_id, data_type):
        inputs = self._clinical_inputs()
        supported = all(at is not None and self.events[at].success for at in inputs.values())
        return self._record(agent_id, 'review_clinical_data', {'data_type': data_type}, inputs,
                            valid=agent_id == 'CLINICAL_LEAD' and data_type == 'combined_review',
                            extra={'assessment': 'supported' if supported else 'insufficient_data'})

    def verify_regulatory_compliance(self, agent_id, review_type):
        clinical = self._clinical()
        data = self.events[clinical].result['inputs'] if clinical is not None else None
        previous = next((event for event in reversed(self.events)
                         if event.agent == 'REGULATORY_SPECIALIST'
                         and event.tool == 'verify_regulatory_compliance'
                         and event.result.get('clinical_data') == data
                         and event.result.get('status') in {'complete', 'clinical_hold'}
                         and event.completed_at is not None), None)
        supported = (clinical is not None
                     and self.events[clinical].result['assessment'] == 'supported')
        held = previous is not None and not previous.success
        actual = (agent_id == 'REGULATORY_SPECIALIST' and isinstance(review_type, str)
                  and review_type in {'initial_review', 'hold_review'} and clinical is not None)
        admissible = actual and supported and (not held or review_type == 'hold_review')
        extra = {'clinical_data': data}
        if actual and not admissible:
            extra['status'] = 'clinical_hold'
        return self._record(agent_id, 'verify_regulatory_compliance',
                            {'review_type': review_type}, {'clinical': clinical},
                            valid=admissible, extra=extra, failure_status='clinical_hold')

    def submit_project_decision(self, agent_id, outcome):
        inputs = self._project_inputs()
        approved = (inputs['regulatory'] is not None
                    and self.events[inputs['regulatory']].success)
        return self._record(agent_id, 'submit_project_decision', {'outcome': outcome}, inputs,
                            valid=(agent_id == 'PROJECT_DIRECTOR' and isinstance(outcome, str)
                                   and outcome in {'continue', 'retry', 'abort'}
                                   and (outcome != 'continue' or approved)),
                            extra={'samples_remaining': self.samples_remaining})

    def make_tools(self):
        return {'run_hplc_analysis': self.run_hplc_analysis,
                'run_mass_spec_analysis': self.run_mass_spec_analysis,
                'run_cell_assay': self.run_cell_assay,
                'prepare_formulation': self.prepare_formulation,
                'review_clinical_data': self.review_clinical_data,
                'verify_regulatory_compliance': self.verify_regulatory_compliance,
                'submit_project_decision': self.submit_project_decision}

    def is_complete(self):
        inputs = self._terminal_inputs()
        return (all(value is not None for value in inputs.values())
                and self._find('CHEMIST', 'run_hplc_analysis', 'sample',
                               'documentation', inputs=inputs) is not None)

    @property
    def progress(self):
        decision = self._terminal_inputs()['decision']
        return {'completed_results': [event.result for event in self.events
                                      if event.completed_at is not None],
                'samples_remaining': self.samples_remaining,
                'final_outcome': self.events[decision].args['outcome'] if decision is not None else None,
                'all_complete': self.is_complete()}
