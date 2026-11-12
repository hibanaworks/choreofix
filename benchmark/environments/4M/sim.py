"""Code Collaboration simulation config for task 4M."""

import json
from pathlib import Path

from benchmark.tools.sim_code_collab import (
    CodeCollabSim,
    CodeReviewStep,
    DesignProposalStep,
    MergeStep,
    RefactorStep,
    RunTestsStep,
)

_METADATA = json.loads(
    (Path(__file__).resolve().parent.parent.parent
     / "descriptions" / "4M" / "metadata.json").read_text()
)

_RESOURCES = ["REPO", "TEST_ENV"]

_DESIGN_PROPOSAL_STEPS = [
    DesignProposalStep(agent_id="ARCHITECT", proposal="module_a_refactor"),
    DesignProposalStep(agent_id="ARCHITECT", proposal="module_b_refactor"),
]

_REFACTOR_STEPS = [
    RefactorStep(agent_id="DEVELOPER_A", target="module_a"),
    RefactorStep(agent_id="DEVELOPER_B", target="module_b"),
    RefactorStep(agent_id="DEVELOPER_A", target="SHARED_LIBRARY"),
    RefactorStep(agent_id="DEVELOPER_B", target="SHARED_LIBRARY"),
]

_CODE_REVIEW_STEPS = [
    CodeReviewStep(agent_id="REVIEWER", target="module_a"),
    CodeReviewStep(agent_id="REVIEWER", target="module_b"),
]

_RUN_TESTS_STEPS = [
    RunTestsStep(agent_id="TESTER", suite="module_a_tests"),
    RunTestsStep(agent_id="TESTER", suite="module_b_tests"),
]

_MERGE_STEPS = [
    MergeStep(agent_id="DEVELOPER_A", branch="module_a"),
    MergeStep(agent_id="DEVELOPER_B", branch="module_b"),
]


from hashlib import sha256
from types import ModuleType
from benchmark.tools._base import ToolResult
from benchmark.tools.sim_base import SimContext

_INITIAL = {
    "common": "def twice(value):\n    return value + value\n",
    "module_a": "from common import twice\ndef compute(value):\n    result = twice(value)\n    return result + 1\n",
    "module_b": "from common import twice\ndef compute(value):\n    result = twice(value)\n    return result + 2\n",
}
_REFACTORED = {
    "common": "def twice(value):\n    return value * 2\n",
    "module_a": "from common import twice\ndef compute(value):\n    return twice(value) + 1\n",
    "module_b": "from common import twice\ndef compute(value):\n    return twice(value) + 2\n",
}


class SharedLibraryRefactorSim(SimContext):
    """A repository and real validation results, with no expected execution order.

    Mechanical calls remain available. Publication records the completed review
    and test receipts that existed for those exact bytes at publication time.
    Later validation cannot authorize an earlier effect. Independent review/test
    orders remain free. Physical resource use is reported separately.
    """
    _DECISION_TOOLS: dict = {}

    def __init__(self) -> None:
        super().__init__()
        for resource in _RESOURCES:
            self.init_resource(resource)
        self.load_from_metadata(_METADATA)
        self.files = dict(_INITIAL)
        self.published: dict[str, str] = {}
        self.designs: dict[str, str] = {}

    @property
    def reviews(self) -> list[dict]:
        return [event.result for event in self.events
                if event.tool == "code_review" and event.completed_at is not None]

    @property
    def test_results(self) -> list[dict]:
        return [event.result for event in self.events
                if event.tool == "run_tests" and event.completed_at is not None]

    def tool_delay(self, tool_name: str, **kwargs) -> tuple[float, float]:
        return CodeCollabSim._TOOL_DELAYS.get(tool_name, (0.0, 0.0))

    @staticmethod
    def _identity(files: dict[str, str], module: str) -> str | None:
        if module not in files or "common" not in files:
            return None
        data = {module: files[module], "common": files["common"]}
        return sha256(json.dumps(data, sort_keys=True).encode()).hexdigest()

    @staticmethod
    def _validate(files: dict[str, str], module: str) -> dict:
        """Compile and execute the repository's actual in-memory Python sources."""
        outputs = []
        try:
            common = ModuleType("common")
            common.__dict__["__builtins__"] = {}
            exec(compile(files["common"], "common.py", "exec"), common.__dict__)

            def import_module(name, *args, **kwargs):
                if name != "common":
                    raise ImportError(name)
                return common

            namespace = {"__builtins__": {"__import__": import_module}}
            exec(compile(files[module], module + ".py", "exec"), namespace)
            offset = {"module_a": 1, "module_b": 2}[module]
            for value in (-3, 0, 7):
                actual = namespace["compute"](value)
                expected = value * 2 + offset
                outputs.append({"input": value, "actual": actual, "expected": expected})
            return {"success": all(v["actual"] == v["expected"] for v in outputs),
                    "outputs": outputs}
        except Exception as error:
            return {"success": False, "outputs": outputs,
                    "error": f"{type(error).__name__}: {error}"}

    def design_proposal(self, agent_id: str, **kwargs) -> ToolResult:
        title = kwargs.get("title", "")
        module = title.removesuffix("_refactor")
        success = module in ("module_a", "module_b")
        if success:
            self.designs[title] = (f"Simplify {module} and the shared twice implementation "
                                   "while preserving compute(value) = 2 * value + "
                                   + str(1 if module == "module_a" else 2))
        data = {"proposal": title, "document": self.designs.get(title),
                "error": None if success else "unknown design target"}
        self.log_event(agent_id, "design_proposal", {"proposal": title}, success, data)
        return ToolResult("design_proposal", success, data)

    def refactor_code(self, agent_id: str, **kwargs) -> ToolResult:
        target = kwargs.get("target", "")
        file = "common" if target == "SHARED_LIBRARY" else target
        success = file in _REFACTORED
        if success:
            self.files[file] = _REFACTORED[file]
        data = {"target": target, "source": self.files.get(file),
                "content_sha256": sha256(self.files[file].encode()).hexdigest() if success else None}
        self.log_event(agent_id, "refactor_code", {"target": target}, success, data)
        return ToolResult("refactor_code", success, data)

    def code_review(self, agent_id: str, **kwargs) -> ToolResult:
        module = kwargs.get("pr_id", "")
        checked = self._validate(dict(self.files), module)
        receipt = {"target": module, "candidate_sha256": self._identity(self.files, module),
                   **checked}
        self.log_event(agent_id, "code_review", {"target": module}, checked["success"], receipt)
        return ToolResult("code_review", checked["success"], receipt)

    def run_tests(self, agent_id: str, **kwargs) -> ToolResult:
        suite = kwargs.get("suite", "")
        module = suite.removesuffix("_tests")
        checked = self._validate(dict(self.files), module)
        receipt = {"suite": suite, "candidate_sha256": self._identity(self.files, module),
                   **checked}
        self.log_event(agent_id, "run_tests", {"suite": suite}, checked["success"], receipt)
        return ToolResult("run_tests", checked["success"], receipt)

    def merge_code(self, agent_id: str, **kwargs) -> ToolResult:
        module = kwargs.get("branch", "")
        success = module in ("module_a", "module_b")
        evidence = {}
        candidates = {}
        violations = []
        if success:
            published = {**self.published, module: self.files[module],
                         "common": self.files["common"]}
            for target in ("module_a", "module_b"):
                if target not in published:
                    continue
                identity = self._identity(published, target)
                if target != module and identity == self._identity(self.published, target):
                    continue
                candidates[target] = identity
                evidence[target] = {
                    "review": self._receipt("code_review", target, identity),
                    "test": self._receipt("run_tests", target, identity),
                }
            if any(receipt is None for pair in evidence.values() for receipt in pair.values()):
                violations.append(self.log_violation(
                    agent_id, "merge_code", "unvalidated_publication",
                    "Published bytes without a completed matching review and test"))
            # The effect is observable even when unsafe; the world does not
            # silently enforce a method's coordination or erase that violation.
            self.published = published
        data = {"branch": module, "candidate_sha256": self._identity(self.published, module),
                "published_candidates": candidates, "validation_receipts": evidence,
                "error": None if success else "unknown branch"}
        self.log_event(agent_id, "merge_code", {"branch": module}, success, data, violations)
        return ToolResult("merge_code", success, data)

    def _receipt(self, tool: str, module: str, identity: str) -> int | None:
        actor = "REVIEWER" if tool == "code_review" else "TESTER"
        key = "target" if tool == "code_review" else "suite"
        value = module if tool == "code_review" else module + "_tests"
        return next((index for index in range(len(self.events)-1, -1, -1)
                     if (event := self.events[index]).agent == actor
                     and event.tool == tool and event.success
                     and event.completed_at is not None
                     and event.result.get(key) == value
                     and event.result.get("candidate_sha256") == identity), None)

    def _publication_is_valid(self, event, module: str, identity: str) -> bool:
        pair = event.result.get("validation_receipts", {}).get(module, {})
        for name, tool, actor, key, value in [
            ("review", "code_review", "REVIEWER", "target", module),
            ("test", "run_tests", "TESTER", "suite", module + "_tests"),
        ]:
            index = pair.get(name)
            if not isinstance(index, int) or not 0 <= index < len(self.events):
                return False
            receipt = self.events[index]
            if (receipt.tool != tool or receipt.agent != actor or not receipt.success
                    or receipt.completed_at is None or receipt.completed_at >= event.timestamp
                    or receipt.result.get(key) != value
                    or receipt.result.get("candidate_sha256") != identity):
                return False
        return True

    def make_tools(self) -> dict:
        return {"design_proposal": self.design_proposal, "refactor_code": self.refactor_code,
                "code_review": self.code_review, "run_tests": self.run_tests,
                "merge_code": self.merge_code}

    def is_complete(self) -> bool:
        # Participation is a task outcome, never an operation-permission table.
        completed = {(e.agent, e.tool, tuple(sorted(e.args.items()))) for e in self.events
                     if e.success and e.completed_at is not None}
        required = [*( (s.agent_id, "design_proposal", (("proposal", s.proposal),))
                      for s in _DESIGN_PROPOSAL_STEPS),
                    *((s.agent_id, "refactor_code", (("target", s.target),))
                      for s in _REFACTOR_STEPS),
                    *((s.agent_id, "code_review", (("target", s.target),))
                      for s in _CODE_REVIEW_STEPS),
                    *((s.agent_id, "run_tests", (("suite", s.suite),))
                      for s in _RUN_TESTS_STEPS),
                    *((s.agent_id, "merge_code", (("branch", s.branch),))
                      for s in _MERGE_STEPS)]
        if not set(required).issubset(completed):
            return False
        if not all(step.proposal in self.designs for step in _DESIGN_PROPOSAL_STEPS):
            return False
        if any(self.published.get(name, source) == source for name, source in _INITIAL.items()):
            return False
        for module in ("module_a", "module_b"):
            identity = self._identity(self.published, module)
            if identity is None or not self._validate(self.published, module)["success"]:
                return False
            publication = next((event for event in reversed(self.events)
                                if event.tool == "merge_code" and event.success
                                and event.completed_at is not None
                                and module in event.result["published_candidates"]), None)
            if (publication is None or publication.result["published_candidates"][module] != identity
                    or not self._publication_is_valid(publication, module, identity)):
                return False
        return True

    @property
    def progress(self) -> dict:
        return {"designs": dict(self.designs),
                "working_candidates": {m: self._identity(self.files, m)
                                       for m in ("module_a", "module_b")},
                "published_candidates": {m: self._identity(self.published, m)
                                         for m in ("module_a", "module_b")},
                "reviews": list(self.reviews), "test_results": list(self.test_results),
                "all_complete": self.is_complete()}
