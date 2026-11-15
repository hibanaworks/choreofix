"""4E's shared repository and integration results, without a workflow gate."""

from copy import deepcopy
from hashlib import sha256
import json
from pathlib import Path
from types import ModuleType

from benchmark.tools._base import ToolResult
from benchmark.tools.sim_base import SimContext
from benchmark.tools.sim_code_collab import CodeCollabSim

_METADATA = json.loads(
    (Path(__file__).resolve().parent.parent.parent
     / "descriptions" / "4E" / "metadata.json").read_text()
)

_API = {"api_name": "feature_api_spec", "request": "integer",
        "response": "integer", "expression": "value * 2 + 1"}
_FEATURES = {"backend_endpoint": ("BACKEND_DEV", "backend.py"),
             "frontend_component": ("FRONTEND_DEV", "frontend.py")}
_TEST_SUITE = "integration_tests"


class FrontendBackendAPISim(SimContext):
    """Tools change ordinary files; tests execute the files they observed.

    There is no next-operation/ready table. Calls may be made in any order,
    including premature tests, reimplementation and independent schedules.
    Completion requires a finished passing test of the current repository.
    """

    def __init__(self) -> None:
        super().__init__()
        self.files: dict[str, str] = {}
        for resource in ("REPO", "TEST_ENV"):
            self.init_resource(resource)
        self.load_from_metadata(_METADATA)

    def tool_delay(self, tool_name: str, **kwargs) -> tuple[float, float]:
        return CodeCollabSim._TOOL_DELAYS.get(tool_name, (0.0, 0.0))

    @staticmethod
    def _identity(files: dict[str, str]) -> str:
        return sha256(json.dumps(files, sort_keys=True).encode()).hexdigest()

    def _api(self) -> tuple[str, str] | None:
        source = self.files.get("api.json")
        if source is None:
            return None
        # A result still in flight is not evidence available to another agent.
        for event in reversed(self.events):
            if (event.tool == "define_api" and event.agent == "BACKEND_DEV" and event.success
                    and event.completed_at is not None
                    and event.result.get("source") == source):
                return source, sha256(source.encode()).hexdigest()
        return None

    def define_api(self, agent_id: str, **kwargs) -> ToolResult:
        name = kwargs.get("api_name", "")
        success = name == _API["api_name"]
        source = json.dumps(_API, sort_keys=True) if success else None
        if source is not None:
            self.files["api.json"] = source
        data = {"api_name": name, "status": "defined" if success else "unknown API",
                "agent": agent_id, "source": source}
        self.log_event(agent_id, "define_api", {"api_name": name}, success, data)
        return ToolResult("define_api", success, data)

    def implement_feature(self, agent_id: str, **kwargs) -> ToolResult:
        feature = kwargs.get("feature_name", "")
        target = _FEATURES.get(feature)
        success = target is not None and target[0] == agent_id
        source = None
        api = self._api()
        if success:
            identity = api[1] if api is not None else None
            source = f"API_SHA256 = {identity!r}\n"
            if feature == "backend_endpoint":
                expression = json.loads(api[0])["expression"] if api else "None"
                source += f"def endpoint(value):\n    return {expression}\n"
            else:
                source += ("from backend import endpoint\n"
                           "def render(value):\n    return 'feature=' + str(endpoint(value))\n")
            self.files[target[1]] = source
        data = {"feature": feature, "status": "implemented" if success else "unknown target",
                "agent": agent_id, "source": source, "api_source": api[0] if api else None}
        self.log_event(agent_id, "implement_feature", {"feature": feature}, success, data)
        return ToolResult("implement_feature", success, data)

    @staticmethod
    def _test(files: dict[str, str]) -> dict:
        outputs = []
        try:
            api = json.loads(files["api.json"])
            identity = sha256(files["api.json"].encode()).hexdigest()
            backend = ModuleType("backend")
            backend.__dict__["__builtins__"] = {}
            exec(compile(files["backend.py"], "backend.py", "exec"), backend.__dict__)

            def load_module(name, *args, **kwargs):
                if name != "backend":
                    raise ImportError(name)
                return backend

            frontend = {"__builtins__": {"__import__": load_module, "str": str}}
            exec(compile(files["frontend.py"], "frontend.py", "exec"), frontend)
            same_contract = (backend.API_SHA256 == identity
                             and frontend["API_SHA256"] == identity)
            if api != _API:
                raise ValueError("API contract differs from the feature specification")
            for value in (-3, 0, 7):
                actual = backend.endpoint(value)
                rendered = frontend["render"](value)
                expected = value * 2 + 1
                outputs.append({"input": value, "actual": actual, "expected": expected,
                                "rendered": rendered, "expected_rendered": f"feature={expected}"})
            return {"success": same_contract and all(
                        item["actual"] == item["expected"]
                        and item["rendered"] == item["expected_rendered"] for item in outputs),
                    "same_contract": same_contract, "outputs": outputs}
        except Exception as error:
            return {"success": False, "outputs": outputs,
                    "error": f"{type(error).__name__}: {error}"}

    def run_tests(self, agent_id: str, **kwargs) -> ToolResult:
        suite = kwargs.get("suite", "")
        files = dict(self.files)
        checked = self._test(files)
        success = suite == _TEST_SUITE and checked["success"]
        data = {"suite": suite, "status": "tests passed" if success else "tests failed",
                "agent": agent_id, "repository_sha256": self._identity(files),
                "files": files, "validation": checked}
        self.log_event(agent_id, "run_tests", {"suite": suite}, success, data)
        return ToolResult("run_tests", success, data)

    def make_tools(self) -> dict:
        return {"define_api": self.define_api, "implement_feature": self.implement_feature,
                "run_tests": self.run_tests}

    def is_complete(self) -> bool:
        if any(event.completed_at is None for event in self.events):
            return False
        api = self._api()
        if api is None:
            return False
        for feature, (agent, path) in _FEATURES.items():
            if not any(event.tool == "implement_feature" and event.agent == agent
                       and event.success and event.args.get("feature") == feature
                       and event.result.get("source") == self.files.get(path)
                       and event.result.get("api_source") == api[0] for event in self.events):
                return False
        identity = self._identity(self.files)
        return any(event.tool == "run_tests" and event.agent == "TESTER"
                   and event.success and event.result.get("suite") == _TEST_SUITE
                   and event.result.get("repository_sha256") == identity
                   and event.result.get("validation", {}).get("success") is True
                   for event in self.events) and self._test(dict(self.files))["success"]

    @property
    def progress(self) -> dict:
        return {"files": dict(self.files), "all_complete": self.is_complete(),
                "test_results": [deepcopy(event.result) for event in self.events
                                 if event.tool == "run_tests" and event.completed_at is not None]}
