"""16M build artifacts, workspace snapshots, and deployment-time evidence."""

from copy import deepcopy
import json
from pathlib import Path

from benchmark.tools._base import ToolResult
from benchmark.tools.sim_base import FailureScenario, FailureSpec, SimContext
from benchmark.tools.sim_cicd import CICDSim

_METADATA = json.loads((Path(__file__).resolve().parents[2]
                        / "descriptions" / "16M" / "metadata.json").read_text())
_COMPONENTS = {"FRONTEND_DEV": "frontend", "BACKEND_DEV": "backend"}
_QA = "QA_ENGINEER"
_RELEASE = "RELEASE_ENGINEER"
_SUITES = {"run_integration_tests": "api_integration", "run_e2e_tests": "user_flows"}


class CICDPipelineSim(SimContext):
    """The journal owns the world; each operation reads completed inputs."""

    _DECISION_TOOLS = {
        "compile_code": 0.2,
        "run_unit_tests": 0.3,
        "run_integration_tests": 0.3,
        "run_e2e_tests": 0.3,
        "deploy_staging": 0.2,
        "run_smoke_tests": 0.3,
    }
    _FAILURE_SCENARIOS = {
        "build_fail": FailureScenario(
            name="build_fail",
            failures=[FailureSpec("compile_code", "BACKEND_DEV", call_index=0)],
            description="Backend compilation fails, forcing rebuild",
        ),
        "qa_fail": FailureScenario(
            name="qa_fail",
            failures=[FailureSpec("run_integration_tests", _QA, call_index=0)],
            description="Integration tests fail, forcing fix cycle",
        ),
        "staging_fail": FailureScenario(
            name="staging_fail",
            failures=[FailureSpec("run_smoke_tests", _RELEASE, call_index=0)],
            description="Staging smoke test fails, triggering abort",
        ),
    }

    def __init__(self, scenario: str | None = None) -> None:
        super().__init__()
        for resource in _METADATA["resources"]:
            self.init_resource(resource)
        self.load_from_metadata(_METADATA)
        if scenario:
            self.configure_scenario(scenario)

    def tool_delay(self, tool_name: str, **kwargs) -> tuple[float, float]:
        return CICDSim._TOOL_DELAYS[tool_name]

    def _latest(self, tool, /, **fields):
        """Find the last completed effect or check with these inputs."""
        return max(
            (event for event in self.events
             if event.tool == tool and event.completed_at is not None
             and all(event.result.get(key) == value for key, value in fields.items())),
            key=lambda event: (event.completed_at, event.result["receipt"]),
            default=None,
        )

    def _record(self, agent, tool, args, success, status, data, violations=()):
        result = {**args, **data, "status": status, "receipt": len(self.events) + 1}
        self.log_event(agent, tool, args, success, result, list(violations))
        return ToolResult(tool, success, result, status.replace("_", " "))

    def compile_code(self, agent_id: str, component: str) -> ToolResult:
        tool = "compile_code"
        valid = agent_id in _COMPONENTS and _COMPONENTS[agent_id] == component
        success = valid and not self.should_fail(tool, agent_id)
        status = "compiled" if success else "compilation_failed"
        return self._record(agent_id, tool, {"component": component}, success, status,
                            {"developer": agent_id})

    def run_unit_tests(self, agent_id: str, component: str) -> ToolResult:
        tool = "run_unit_tests"
        build = self._latest("compile_code", developer=agent_id, component=component)
        success = build is not None and build.success and not self.should_fail(tool, agent_id)
        status = "tests passed" if success else "tests_failed"
        return self._record(agent_id, tool, {"component": component}, success, status,
                            {"developer": agent_id,
                             "build": build.result["receipt"] if build and build.success else None})

    def publish_artifact(self, agent_id: str, artifact: str) -> ToolResult:
        tool = "publish_artifact"
        component = _COMPONENTS.get(agent_id)
        args = {"artifact": artifact}
        data = {"developer": agent_id}
        if component is None or artifact != f"{component}_image":
            return self._record(agent_id, tool, args, False, "invalid_artifact", data)
        build = self._latest("compile_code", developer=agent_id, component=component)
        if build is None or not build.success:
            return self._record(agent_id, tool, args, False, "missing_build", data)

        version = build.result["receipt"]
        tested = self._latest("run_unit_tests", developer=agent_id, build=version)
        data["build"] = version
        data["unit_test"] = tested.result["receipt"] if tested and tested.success else None
        violations = []
        if data["unit_test"] is None:
            violations.append(self.log_violation(agent_id, tool, "unvalidated_artifact",
                                                 "Published a build without passing unit tests"))
        return self._record(agent_id, tool, args, True, "published", data, violations)

    def _registry(self):
        artifacts = {}
        for developer, component in _COMPONENTS.items():
            artifact = f"{component}_image"
            published = self._latest("publish_artifact", developer=developer,
                                     artifact=artifact, status="published")
            if published is not None:
                artifacts[artifact] = published.result
        return artifacts

    def pull_artifacts(self, agent_id: str, target: str) -> ToolResult:
        tool = "pull_artifacts"
        args = {"target": target}
        data = {"agent": agent_id}
        valid = ((agent_id == _QA and target == "test_harness")
                 or (agent_id == _RELEASE and target in ("staging_deploy", "production_deploy")))
        if not valid:
            return self._record(agent_id, tool, args, False, "invalid_workspace", data)

        published = self._registry()
        if len(published) != len(_COMPONENTS):
            return self._record(agent_id, tool, args, False, "missing_artifacts", data)
        data["artifacts"] = {name: record["build"] for name, record in published.items()}
        data["unit_tests"] = {name: record["unit_test"] for name, record in published.items()}
        return self._record(agent_id, tool, args, True, "pulled", data)

    def _test_harness(self, tool, agent_id, suite):
        pull = self._latest("pull_artifacts", agent=_QA, target="test_harness", status="pulled")
        valid = agent_id == _QA and suite == _SUITES[tool] and pull is not None
        success = valid and not self.should_fail(tool, agent_id)
        status = "tests passed" if success else "tests_failed"
        return self._record(agent_id, tool, {"suite": suite}, success, status,
                            {"engineer": agent_id,
                             "artifacts": pull.result["artifacts"] if valid else None})

    def run_integration_tests(self, agent_id: str, suite: str) -> ToolResult:
        return self._test_harness("run_integration_tests", agent_id, suite)

    def run_e2e_tests(self, agent_id: str, suite: str) -> ToolResult:
        return self._test_harness("run_e2e_tests", agent_id, suite)

    def deploy_staging(self, agent_id: str, component: str) -> ToolResult:
        tool = "deploy_staging"
        pull = self._latest("pull_artifacts", agent=_RELEASE,
                            target="staging_deploy", status="pulled")
        valid = agent_id == _RELEASE and component == "full_stack" and pull is not None
        success = valid and not self.should_fail(tool, agent_id)
        status = "deployed to staging" if success else "deploy_failed"
        return self._record(agent_id, tool, {"component": component}, success, status,
                            {"engineer": agent_id,
                             "artifacts": pull.result["artifacts"] if valid else None})

    def deploy_production(self, agent_id: str, component: str) -> ToolResult:
        tool = "deploy_production"
        args = {"component": component}
        data = {"engineer": agent_id}
        pull = self._latest("pull_artifacts", agent=_RELEASE, status="pulled")
        if agent_id != _RELEASE or component != "full_stack" or pull is None:
            return self._record(agent_id, tool, args, False, "deploy_failed", data)

        artifacts = pull.result["artifacts"]
        data["artifacts"] = artifacts
        data["unit_tests"] = pull.result["unit_tests"]
        data["qa"] = {}
        for check, suite in _SUITES.items():
            result = self._latest(check, engineer=_QA, suite=suite, artifacts=artifacts)
            data["qa"][check] = result.result["receipt"] if result and result.success else None

        staging = self._latest("deploy_staging", status="deployed to staging")
        smoke = None
        if staging is not None and staging.result["artifacts"] == artifacts:
            smoke = self._latest("run_smoke_tests", target="staging",
                                 deployment=staging.result["receipt"])
        data["staging_smoke"] = smoke.result["receipt"] if smoke and smoke.success else None

        # Production can receive any available image pair. Record the evidence
        # available at deployment so later QA or smoke tests cannot certify it.
        violations = []
        if (None in data["unit_tests"].values() or None in data["qa"].values()
                or data["staging_smoke"] is None):
            violations.append(self.log_violation(
                agent_id, tool, "unvalidated_publication",
                "Deployed without completed unit, QA, and staging smoke tests for these images",
            ))
        return self._record(agent_id, tool, args, True, "deployed to production", data, violations)

    def run_smoke_tests(self, agent_id: str, target: str) -> ToolResult:
        tool = "run_smoke_tests"
        deployment = None
        if agent_id == _RELEASE and target in ("staging", "production"):
            deployment = self._latest(f"deploy_{target}", status=f"deployed to {target}")
        success = deployment is not None and not self.should_fail(tool, agent_id)
        status = "smoke tests passed" if success else "smoke_tests_failed"
        return self._record(agent_id, tool, {"target": target}, success, status,
                            {"engineer": agent_id,
                             "deployment": deployment.result["receipt"] if deployment else None})

    def make_tools(self) -> dict:
        return {
            "compile_code": self.compile_code,
            "run_unit_tests": self.run_unit_tests,
            "publish_artifact": self.publish_artifact,
            "pull_artifacts": self.pull_artifacts,
            "run_integration_tests": self.run_integration_tests,
            "run_e2e_tests": self.run_e2e_tests,
            "deploy_staging": self.deploy_staging,
            "deploy_production": self.deploy_production,
            "run_smoke_tests": self.run_smoke_tests,
        }

    def is_complete(self) -> bool:
        if any(event.completed_at is None for event in self.events):
            return False
        deployment = self._latest("deploy_production", status="deployed to production")
        if deployment is None or deployment.violations:
            return False
        published = self._registry()
        if deployment.result["artifacts"] != {name: row["build"] for name, row in published.items()}:
            return False
        smoke = self._latest("run_smoke_tests", target="production",
                             deployment=deployment.result["receipt"])
        return smoke is not None and smoke.success

    @property
    def progress(self) -> dict:
        production = self._latest("deploy_production", status="deployed to production")
        return {"published": deepcopy(self._registry()),
                "production": deepcopy(production.result) if production else None,
                "all_complete": self.is_complete()}
