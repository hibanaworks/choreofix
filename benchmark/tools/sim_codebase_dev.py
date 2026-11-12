"""Repository contents and completed validation data for scenarios 6E/6M/6H.

Calls are not gated by a workflow. Publication and deployment effects retain
which completed evidence actually existed for their bytes at the effect time.
These are abstract development tools, not a compiler or a proof of source code.
"""
from __future__ import annotations

from dataclasses import dataclass
from copy import deepcopy
from hashlib import sha256
import json
from typing import Any

from ._base import ToolResult
from .sim_base import SimContext


@dataclass
class ImplementStep:
    agent_id: str
    feature: str


@dataclass
class WriteTestsStep:
    agent_id: str
    component: str


@dataclass
class CodeReviewStep:
    agent_id: str
    target: str


@dataclass
class RunTestsStep:
    agent_id: str
    suite: str
    feature: str


@dataclass
class MergeStep:
    agent_id: str
    branch: str


@dataclass
class DeployStep:
    agent_id: str
    service: str
    dependencies: tuple[str, ...] = ()


@dataclass
class RollbackStep:
    agent_id: str
    service: str


@dataclass
class ValidateStagingStep:
    agent_id: str
    service: str


class CodebaseDevSim(SimContext):
    """Model world data, physical use and outcomes, never a method's protocol."""
    _DECISION_TOOLS = {"code_review": 0.3, "run_tests": 0.3, "validate_staging": 0.2}
    _TOOL_RESOURCES = {
        "implement_feature": ["REPO"], "write_tests": ["TEST_ENV"],
        "code_review": [], "run_tests": ["BUILD_SERVER"], "merge_code": ["REPO"],
        "deploy_service": ["STAGING_ENV"], "rollback_service": ["STAGING_ENV"],
        "validate_staging": ["STAGING_ENV"],
    }
    _TOOL_DELAYS = {
        "implement_feature": (1.5, 4.0), "write_tests": (1.0, 2.5),
        "code_review": (1.0, 2.0), "run_tests": (2.0, 5.0),
        "merge_code": (0.5, 1.0), "deploy_service": (1.0, 3.0),
        "rollback_service": (0.5, 1.0), "validate_staging": (1.5, 3.0),
    }

    def __init__(self, implement_steps=None, write_tests_steps=None,
                 code_review_steps=None, run_tests_steps=None, merge_steps=None,
                 deploy_steps=None, rollback_steps=None, validate_staging_steps=None,
                 resources=None) -> None:
        super().__init__()
        self._implement_steps = tuple(implement_steps or ())
        self._write_tests_steps = tuple(write_tests_steps or ())
        self._code_review_steps = tuple(code_review_steps or ())
        self._run_tests_steps = tuple(run_tests_steps or ())
        self._merge_steps = tuple(merge_steps or ())
        self._deploy_steps = tuple(deploy_steps or ())
        self._rollback_steps = tuple(rollback_steps or ())
        self._validate_staging_steps = tuple(validate_staging_steps or ())
        self.files: dict[str, str] = {}
        self.test_files: dict[str, dict] = {}
        self.published: dict[str, dict] = {}
        self.deployments: dict[str, dict] = {}
        for resource in resources or ():
            self.init_resource(resource)

    def resource_requirements(self, tool_name: str, **kwargs: Any) -> list[str]:
        return list(self._TOOL_RESOURCES.get(tool_name, ()))

    def tool_delay(self, tool_name: str, **kwargs: Any) -> tuple[float, float]:
        return self._TOOL_DELAYS[tool_name]

    @staticmethod
    def _identity(data) -> str:
        return sha256(json.dumps(data, sort_keys=True, separators=(",", ":")).encode()).hexdigest()

    def _result(self, actor, tool, args, data, success, violations=()) -> ToolResult:
        data = {**data, "artifact_id": f"development:{len(self.events)}"}
        self.log_event(actor, tool, deepcopy(args), success, deepcopy(data), list(violations))
        return ToolResult(tool, success, data)

    def _completed(self, tool, predicate, before=float("inf")):
        return next(((i, e) for i, e in reversed(list(enumerate(self.events)))
                     if e.tool == tool and e.success and e.completed_at is not None
                     and e.completed_at < before and predicate(e)), None)

    def _candidate(self, feature):
        if not isinstance(feature, str) or feature not in self.files:
            return None
        identity = self._identity({"feature": feature, "source": self.files[feature]})
        job = next((s for s in self._implement_steps if s.feature == feature), None)
        if job is None:
            return None
        return self._completed("implement_feature", lambda e:
                               e.agent == job.agent_id
                               and e.result.get("candidate_sha256") == identity)

    def _review(self, feature, identity, before=float("inf")):
        job = next((s for s in self._code_review_steps if s.target == feature), None)
        if job is None:
            return None
        return self._completed("code_review", lambda e:
                               e.agent == job.agent_id and e.result.get("target") == feature
                               and e.result.get("candidate_sha256") == identity, before)

    def _tests(self, feature, identity):
        job = next((s for s in self._write_tests_steps if s.component == feature), None)
        if job is None or feature not in self.test_files:
            return None
        test = self.test_files[feature]
        return self._completed("write_tests", lambda e:
                               e.agent == job.agent_id
                               and e.result.get("test_sha256") == test["test_sha256"]
                               and e.result.get("candidate_sha256") == identity)

    def _repository(self):
        return {feature: value["candidate_sha256"] for feature, value in self.published.items()}

    def _ci(self, identity, before=float("inf")):
        publications = [self.events[value["publication_event"]] for value in self.published.values()]
        return self._completed("run_tests", lambda e:
            any(e.agent == s.agent_id and e.result.get("suite") == s.suite
                for s in self._run_tests_steps)
            and e.result.get("repository_sha256") == identity
            and e.result.get("publication_events") == {
                f: value["publication_event"] for f, value in self.published.items()}
            and self._ci_inputs_valid(e)
            and all(publication.completed_at is not None and publication.completed_at < e.timestamp
                    for publication in publications), before)

    def _receipt(self, index, tool, before, predicate):
        if type(index) is not int or not 0 <= index < len(self.events):
            return False
        event = self.events[index]
        return (event.tool == tool and event.success and event.completed_at is not None
                and event.completed_at < before and predicate(event))

    def _ci_inputs_valid(self, event):
        """Check a CI observation's actual completed publication and test inputs."""
        data = event.result
        candidates = data.get("published_candidates")
        publications = data.get("publication_events")
        tests = data.get("test_receipts")
        inputs = data.get("test_inputs")
        if (not isinstance(candidates, dict) or not candidates
                or not isinstance(publications, dict) or set(publications) != set(candidates)
                or not isinstance(tests, dict) or set(tests) != set(candidates)
                or not isinstance(inputs, dict) or set(inputs) != set(candidates)):
            return False
        for feature, identity in candidates.items():
            writer = next((s for s in self._write_tests_steps if s.component == feature), None)
            if (writer is None
                    or not self._receipt(publications[feature], "merge_code", event.timestamp,
                        lambda e: e.result.get("feature") == feature
                        and e.result.get("candidate_sha256") == identity)
                    or not self._receipt(tests[feature], "write_tests", event.timestamp,
                        lambda e: e.agent == writer.agent_id
                        and e.result.get("candidate_sha256") == identity
                        and e.result == inputs[feature])):
                return False
        return True

    def _repair_failure(self, feature, candidate, review, base, publications,
                        before=float("inf"), failure_receipt=None):
        """Find data proving a fresh repair of the last failed publication.

        This records provenance, not tool admission. Unrelated and stale
        publications still execute and retain their violations.
        """
        if (candidate is None or review is None or not publications
                or feature not in publications
                or publications[feature] != max(publications.values())
                or review[1].result.get("source_receipt") != candidate[0]
                or candidate[1].completed_at is None
                or not candidate[1].completed_at < review[1].timestamp
                or review[1].completed_at is None or not review[1].completed_at < before):
            return None
        publication = self.events[publications[feature]]
        author = next((s for s in self._implement_steps if s.feature == feature), None)
        if (author is None or candidate[0] <= publications[feature]
                or candidate[1].tool != "implement_feature" or not candidate[1].success
                or candidate[1].agent != author.agent_id
                or candidate[1].result.get("feature") != feature
                or publication.completed_at is None
                or not publication.completed_at < candidate[1].completed_at):
            return None
        job = next((s for s in self._run_tests_steps if s.feature == feature), None)
        if job is None:
            return None
        for index, event in reversed(list(enumerate(self.events))):
            if failure_receipt is not None and index != failure_receipt:
                continue
            if (event.tool == "run_tests" and not event.success
                    and event.agent == job.agent_id and event.result.get("suite") == job.suite
                    and event.completed_at is not None
                    and event.completed_at < before
                    and event.result.get("repository_sha256") == base
                    and event.result.get("publication_events") == publications
                    and event.result.get("publication_event") == publications[feature]
                    and self._ci_inputs_valid(event)):
                return index, event
        return None

    def _publication_valid(self, feature):
        value = self.published.get(feature)
        candidate = self._candidate(feature)
        if value is None or candidate is None:
            return False
        event = self.events[value["publication_event"]]
        review = self._review(feature, value["candidate_sha256"], event.timestamp)
        job = next((s for s in self._merge_steps if s.branch == value["branch"]), None)
        if (event.completed_at is None or job is None or event.agent != job.agent_id
                or value["candidate_sha256"] != candidate[1].result["candidate_sha256"]
                or review is None or value["review_receipt"] != review[0]):
            return False
        base = value["base_repository_sha256"]
        if not self._run_tests_steps or base is None:
            return True
        if self._receipt(value["base_test_receipt"], "run_tests", event.timestamp,
                lambda e: e.result.get("repository_sha256") == base
                and e.result.get("publication_events") == value.get("base_publication_events")
                and self._ci_inputs_valid(e)):
            return True
        source = value.get("source_receipt")
        failure = value.get("base_failure_receipt")
        return (type(source) is int and 0 <= source < len(self.events)
                and type(failure) is int and value.get("base_test_receipt") is None
                and value.get("repaired_publication_event") ==
                    value.get("base_publication_events", {}).get(feature)
                and self._repair_failure(feature, (source, self.events[source]), review, base,
                    value.get("base_publication_events"), event.timestamp, failure) is not None)

    def implement_feature(self, agent_id, **kwargs):
        feature = kwargs.get("feature_name")
        source = kwargs.get("source", f"Implementation of {feature}")
        success = (any(s.feature == feature for s in self._implement_steps)
                   and isinstance(source, str) and bool(source.strip()))
        data = {"feature": feature, "status": "implemented" if success else "invalid_input",
                "agent": agent_id}
        if success:
            self.files[feature] = source
            data.update(source=source, candidate_sha256=self._identity({"feature": feature, "source": source}))
        return self._result(agent_id, "implement_feature", {"feature": feature, "source": source}, data, success)

    def write_tests(self, agent_id, **kwargs):
        feature = kwargs.get("target")
        candidate = self._candidate(feature)
        success = candidate is not None
        data = {"component": feature, "status": "tests written" if success else "missing_source",
                "agent": agent_id, "candidate_sha256": None if candidate is None else candidate[1].result["candidate_sha256"],
                "source_receipt": None if candidate is None else candidate[0]}
        if success:
            data["test_source"] = {"target": feature, "expected_candidate": data["candidate_sha256"]}
            data["test_sha256"] = self._identity(data["test_source"])
        result = self._result(agent_id, "write_tests", {"component": feature}, data, success)
        if success:
            self.test_files[feature] = deepcopy(result.data)
        return result

    def code_review(self, agent_id, **kwargs):
        feature = kwargs.get("pr_id")
        candidate = self._candidate(feature)
        success = candidate is not None and not self.should_fail("code_review", agent_id)
        data = {"target": feature, "status": "reviewed" if success else "changes_requested",
                "agent": agent_id, "candidate_sha256": None if candidate is None else candidate[1].result["candidate_sha256"],
                "source_receipt": None if candidate is None else candidate[0]}
        return self._result(agent_id, "code_review", {"target": feature}, data, success)

    def merge_code(self, agent_id, **kwargs):
        branch = kwargs.get("branch")
        merge = next((s for s in self._merge_steps if s.branch == branch), None)
        job = None if merge is None else next((s for s in self._implement_steps if s.agent_id == merge.agent_id), None)
        candidate = None if job is None else self._candidate(job.feature)
        args = {"branch": branch}
        if candidate is None:
            return self._result(agent_id, "merge_code", args, {"branch": branch, "error": "missing completed source"}, False)
        feature = job.feature
        identity = candidate[1].result["candidate_sha256"]
        review = self._review(feature, identity)
        base = self._identity(self._repository()) if self.published else None
        publications = {f: value["publication_event"] for f, value in self.published.items()}
        tested = self._ci(base) if base is not None and self._run_tests_steps else None
        repair = (self._repair_failure(feature, candidate, review, base, publications)
                  if base is not None and tested is None and self._run_tests_steps else None)
        violations = []
        if review is None:
            violations.append(self.log_violation(agent_id, "merge_code", "unvalidated_publication",
                                                "Published bytes without a completed matching review"))
        if base is not None and self._run_tests_steps and tested is None and repair is None:
            violations.append(self.log_violation(agent_id, "merge_code", "unvalidated_base_publication",
                                                "Extended an integrated repository without its completed CI result"))
        value = {"branch": branch, "feature": feature, "source": self.files[feature],
                 "source_receipt": candidate[0],
                 "candidate_sha256": identity, "review_receipt": None if review is None else review[0],
                 "base_repository_sha256": base, "base_test_receipt": None if tested is None else tested[0],
                 "base_publication_events": publications,
                 "base_failure_receipt": None if repair is None else repair[0],
                 "repaired_publication_event": None if repair is None else publications[feature],
                 "publication_event": len(self.events)}
        self.published[feature] = value
        return self._result(agent_id, "merge_code", args,
                            {**value, "status": "merged", "agent": agent_id,
                             "repository_sha256": self._identity(self._repository()),
                             "published_candidates": self._repository()}, True, violations)

    def run_tests(self, agent_id, **kwargs):
        suite = kwargs.get("suite")
        job = next((s for s in self._run_tests_steps if s.suite == suite), None)
        repository = self._repository()
        receipts = {feature: self._tests(feature, identity) for feature, identity in repository.items()}
        available = (job is not None and job.feature in repository
                     and all(receipt is not None for receipt in receipts.values())
                     and all(self.events[value["publication_event"]].completed_at is not None
                             for value in self.published.values()))
        success = available and not self.should_fail("run_tests", agent_id)
        data = {"suite": suite, "status": "tests passed" if success else "tests_failed",
                "agent": agent_id, "feature": None if job is None else job.feature,
                "repository_sha256": self._identity(repository), "published_candidates": repository,
                "publication_event": None if job is None or job.feature not in self.published
                    else self.published[job.feature]["publication_event"],
                "publication_events": {f: value["publication_event"] for f, value in self.published.items()},
                "test_receipts": {f: None if receipt is None else receipt[0] for f, receipt in receipts.items()},
                "test_inputs": {f: None if receipt is None else deepcopy(self.events[receipt[0]].result)
                                for f, receipt in receipts.items()}}
        return self._result(agent_id, "run_tests", {"suite": suite}, data, success)

    def _deployment_valid(self, service):
        if not isinstance(service, str):
            return False
        value = self.deployments.get(service)
        if value is None or not self._publication_valid(service):
            return False
        event = self.events[value["deployment_event"]]
        job = next((s for s in self._deploy_steps if s.service == service), None)
        if (event.completed_at is None or job is None or event.agent != job.agent_id
                or value["candidate_sha256"] != self.published[service]["candidate_sha256"]
                or not self._receipt(value["ci_receipt"], "run_tests", event.timestamp,
                                    lambda e: e.result.get("repository_sha256") == value["repository_sha256"])):
            return False
        for dependency in job.dependencies:
            if (dependency not in self.deployments or not self._deployment_valid(dependency)
                    or not self._receipt(value["dependency_receipts"].get(dependency), "validate_staging", event.timestamp,
                                 lambda e: (e.result.get("service") == dependency
                                            and dependency in self.published
                                            and e.result.get("deployment_event") == self.deployments[dependency]["deployment_event"]
                                            and e.result.get("candidate_sha256") == self.published[dependency]["candidate_sha256"]))):
                return False
        return True

    def deploy_service(self, agent_id, **kwargs):
        service = kwargs.get("service")
        job = next((s for s in self._deploy_steps if s.service == service), None)
        value = self.published.get(service) if isinstance(service, str) else None
        if job is None or value is None or self.events[value["publication_event"]].completed_at is None:
            return self._result(agent_id, "deploy_service", {"service": service},
                                {"service": service, "error": "missing published artifact"}, False)
        repository = self._identity(self._repository())
        ci = self._ci(repository)
        dependencies = {feature: (self._completed("validate_staging", lambda e:
                        e.result.get("service") == feature and feature in self.published
                        and e.result.get("candidate_sha256") == self.published[feature]["candidate_sha256"]
                        and e.result.get("deployment_event") == self.deployments[feature]["deployment_event"])
                        if feature in self.deployments and self._deployment_valid(feature) else None)
                        for feature in job.dependencies}
        violations = []
        if not self._publication_valid(service) or ci is None or any(v is None for v in dependencies.values()):
            violations.append(self.log_violation(agent_id, "deploy_service", "unvalidated_deployment",
                                                "Installed bytes without completed matching CI/dependency validation"))
        deployed = {"service": service, "candidate_sha256": value["candidate_sha256"],
                    "repository_sha256": repository, "publication_event": value["publication_event"],
                    "ci_receipt": None if ci is None else ci[0],
                    "dependency_receipts": {f: None if receipt is None else receipt[0] for f, receipt in dependencies.items()},
                    "deployment_event": len(self.events), "environment": kwargs.get("environment", "staging")}
        self.deployments[service] = deployed
        return self._result(agent_id, "deploy_service", {"service": service},
                            {**deployed, "status": "deployed", "agent": agent_id}, True, violations)

    def rollback_service(self, agent_id, **kwargs):
        service = kwargs.get("service")
        previous = self.deployments.pop(service, None) if isinstance(service, str) else None
        return self._result(agent_id, "rollback_service", {"service": service},
                            {"service": service, "status": "rolled back", "agent": agent_id,
                             "removed_deployment": previous}, previous is not None)

    def validate_staging(self, agent_id, **kwargs):
        service = kwargs.get("service")
        value = self.deployments.get(service) if isinstance(service, str) else None
        success = (self._deployment_valid(service)
                   and not self.should_fail("validate_staging", agent_id))
        data = {"service": service, "status": "validated" if success else "validation_failed",
                "agent": agent_id, "deployment_event": None if value is None else value["deployment_event"],
                "candidate_sha256": None if value is None else value["candidate_sha256"]}
        return self._result(agent_id, "validate_staging", {"service": service}, data, success)

    def make_tools(self):
        return {name: getattr(self, name) for name in self._TOOL_RESOURCES}

    def is_complete(self):
        if not all(self._candidate(s.feature) is not None for s in self._implement_steps):
            return False
        for step in self._code_review_steps:
            candidate = self._candidate(step.target)
            if candidate is None or self._review(step.target, candidate[1].result["candidate_sha256"]) is None:
                return False
        for step in self._write_tests_steps:
            candidate = self._candidate(step.component)
            if candidate is None or self._tests(step.component, candidate[1].result["candidate_sha256"]) is None:
                return False
        if not all(self._publication_valid(s.feature) for s in self._implement_steps):
            return False
        for step in self._run_tests_steps:
            identity = self.published[step.feature]["candidate_sha256"]
            publication = self.events[self.published[step.feature]["publication_event"]]
            if self._completed("run_tests", lambda e:
                               e.agent == step.agent_id and e.result.get("suite") == step.suite
                               and e.result.get("published_candidates", {}).get(step.feature) == identity
                               and publication.completed_at is not None
                               and publication.completed_at < e.timestamp) is None:
                return False
        if self._run_tests_steps and self._ci(self._identity(self._repository())) is None:
            return False
        if not all(self._deployment_valid(s.service) for s in self._deploy_steps):
            return False
        for step in self._validate_staging_steps:
            if not self._deployment_valid(step.service):
                return False
            deployed = self.deployments[step.service]
            if self._completed("validate_staging", lambda e:
                               e.agent == step.agent_id and e.result.get("service") == step.service
                               and e.result.get("deployment_event") == deployed["deployment_event"]) is None:
                return False
        return True

    @property
    def progress(self):
        # This is a read-only outcome view, never another execution authority.
        result = {}
        for name, tool, key, steps, field in (
            ("implementations", "implement_feature", "feature", self._implement_steps, "feature"),
            ("test_writes", "write_tests", "component", self._write_tests_steps, "component"),
            ("code_reviews", "code_review", "target", self._code_review_steps, "target"),
            ("test_runs", "run_tests", "suite", self._run_tests_steps, "suite"),
            ("merges", "merge_code", "branch", self._merge_steps, "branch"),
            ("deployments", "deploy_service", "service", self._deploy_steps, "service"),
            ("rollbacks", "rollback_service", "service", self._rollback_steps, "service"),
            ("staging_validations", "validate_staging", "service", self._validate_staging_steps, "service"),
        ):
            result[name] = {f"{s.agent_id}_{getattr(s, field)}": self._completed(tool, lambda e:
                            e.agent == s.agent_id and e.result.get(key) == getattr(s, field)) is not None
                            for s in steps}
        result["all_complete"] = self.is_complete()
        return result
