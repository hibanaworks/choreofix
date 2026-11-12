"""Common code-development outcomes must describe real, completed input data."""
import importlib
import asyncio
import itertools
import unittest

from benchmark.tools import load_tools


class CodebaseDevelopment(unittest.IsolatedAsyncioTestCase):
    def setUp(self):
        self.sim = importlib.import_module(
            "benchmark.environments.6M.sim"
        ).MultiFileFeatureDevSim()
        self.sim._delay_multiplier = 0
        self.registry = load_tools("6M", sim=self.sim)

    async def test_reverse_work_without_any_merge_is_not_complete(self):
        for feature in ("user_authentication", "payment_processing"):
            await self.registry.call("code_review", agent_id="REVIEWER", pr_id=feature)
            await self.registry.call("write_tests", agent_id="TEST_WRITER", target=feature)
        for suite in ("full_test_suite_after_auth", "full_test_suite_after_payment"):
            await self.registry.call("run_tests", agent_id="CI_RUNNER", suite=suite)
        for actor, feature in (
            ("DEVELOPER_A", "user_authentication"),
            ("DEVELOPER_B", "payment_processing"),
        ):
            await self.registry.call("implement_feature", agent_id=actor, feature_name=feature)
        self.assertFalse(self.sim.is_complete(), "No source was merged and validation preceded its input")

    async def test_ci_reports_its_actual_test_inputs_without_aliasing_the_live_files(self):
        for actor, feature, suite in (
            ("DEVELOPER_A", "user_authentication", "full_test_suite_after_auth"),
            ("DEVELOPER_B", "payment_processing", "full_test_suite_after_payment"),
        ):
            self.setUp()
            source = await self.registry.call("implement_feature", agent_id=actor, feature_name=feature)
            tests = await self.registry.call("write_tests", agent_id="TEST_WRITER", target=feature)
            await self.registry.call("code_review", agent_id="REVIEWER", pr_id=feature)
            merged = await self.registry.call("merge_code", agent_id=actor, branch=feature)
            ci = await self.registry.call("run_tests", agent_id="CI_RUNNER", suite=suite)
            self.assertTrue(ci.success)
            self.assertEqual(merged.data["published_candidates"], {feature: source.data["candidate_sha256"]})
            self.assertEqual(ci.data["test_inputs"][feature], tests.data)
            self.assertEqual(ci.data["test_receipts"][feature], int(tests.data["artifact_id"].split(":")[1]))
            # A returned CI result is a snapshot of the completed call, not a
            # borrowed reference into the world's mutable current test file.
            self.sim.test_files[feature]["test_source"]["target"] = "later edited test contents"
            self.assertEqual(ci.data["test_inputs"][feature]["test_source"]["target"], feature)

    async def test_identical_republication_still_requires_ci_after_that_publication(self):
        for actor, feature, suite in (
            ("DEVELOPER_A", "user_authentication", "full_test_suite_after_auth"),
            ("DEVELOPER_B", "payment_processing", "full_test_suite_after_payment"),
        ):
            await self.registry.call("implement_feature", agent_id=actor, feature_name=feature)
            await self.registry.call("write_tests", agent_id="TEST_WRITER", target=feature)
            await self.registry.call("code_review", agent_id="REVIEWER", pr_id=feature)
            await self.registry.call("merge_code", agent_id=actor, branch=feature)
            await self.registry.call("run_tests", agent_id="CI_RUNNER", suite=suite)
        self.assertTrue(self.sim.is_complete())
        await self.registry.call("merge_code", agent_id="DEVELOPER_A", branch="user_authentication")
        self.assertFalse(self.sim.is_complete(), "an earlier CI cannot certify a later identical publication")
        await self.registry.call("run_tests", agent_id="CI_RUNNER", suite="full_test_suite_after_payment")
        self.assertFalse(self.sim.is_complete(), "the required suite must follow its feature's latest merge")
        await self.registry.call("run_tests", agent_id="CI_RUNNER", suite="full_test_suite_after_auth")
        self.assertTrue(self.sim.is_complete())
        self.assertEqual(self.sim.violations, [])

    async def test_ci_failure_can_be_repaired_without_unrelated_publication(self):
        from benchmark.tools.sim_base import FailureSpec
        self.sim._failure_specs = [FailureSpec("run_tests", "CI_RUNNER", 0)]
        for tool, args in (
            ("implement_feature", {"feature_name": "user_authentication", "source": "auth initial"}),
            ("write_tests", {"target": "user_authentication"}),
            ("code_review", {"pr_id": "user_authentication"}),
            ("merge_code", {"branch": "user_authentication"}),
        ):
            actor = "TEST_WRITER" if tool == "write_tests" else "REVIEWER" if tool == "code_review" else "DEVELOPER_A"
            self.assertTrue((await self.registry.call(tool, agent_id=actor, **args)).success)
        failed = await self.registry.call("run_tests", agent_id="CI_RUNNER", suite="full_test_suite_after_auth")
        self.assertFalse(failed.success)
        for tool, args in (
            ("implement_feature", {"feature_name": "user_authentication", "source": "auth repaired"}),
            ("write_tests", {"target": "user_authentication"}),
            ("code_review", {"pr_id": "user_authentication"}),
            ("merge_code", {"branch": "user_authentication"}),
        ):
            actor = "TEST_WRITER" if tool == "write_tests" else "REVIEWER" if tool == "code_review" else "DEVELOPER_A"
            self.assertTrue((await self.registry.call(tool, agent_id=actor, **args)).success)
        self.assertEqual(self.sim.violations, [], "a fresh repair is not an unrelated next publication")
        self.assertTrue((await self.registry.call("run_tests", agent_id="CI_RUNNER", suite="full_test_suite_after_auth")).success)
        for actor, tool, args in (
            ("DEVELOPER_B", "implement_feature", {"feature_name": "payment_processing"}),
            ("TEST_WRITER", "write_tests", {"target": "payment_processing"}),
            ("REVIEWER", "code_review", {"pr_id": "payment_processing"}),
            ("DEVELOPER_B", "merge_code", {"branch": "payment_processing"}),
            ("CI_RUNNER", "run_tests", {"suite": "full_test_suite_after_payment"}),
        ):
            self.assertTrue((await self.registry.call(tool, agent_id=actor, **args)).success)
        self.assertTrue(self.sim.is_complete())
        self.assertEqual(self.sim.violations, [])

    async def test_repair_can_be_prepared_while_the_failed_ci_is_still_running(self):
        from benchmark.tools.sim_base import FailureSpec
        self.sim._failure_specs = [FailureSpec("run_tests", "CI_RUNNER", 0)]
        for actor, tool, args in (
            ("DEVELOPER_A", "implement_feature", {"feature_name": "user_authentication", "source": "old"}),
            ("TEST_WRITER", "write_tests", {"target": "user_authentication"}),
            ("REVIEWER", "code_review", {"pr_id": "user_authentication"}),
            ("DEVELOPER_A", "merge_code", {"branch": "user_authentication"}),
        ):
            self.assertTrue((await self.registry.call(tool, agent_id=actor, **args)).success)
        self.sim._delay_multiplier = 1
        self.sim.tool_delay = lambda tool, **kwargs: (0.03, 0.03) if tool == "run_tests" else (0, 0)
        pending = asyncio.create_task(self.registry.call("run_tests", agent_id="CI_RUNNER", suite="full_test_suite_after_auth"))
        await asyncio.sleep(0)
        for actor, tool, args in (
            ("DEVELOPER_A", "implement_feature", {"feature_name": "user_authentication", "source": "prepared repair"}),
            ("TEST_WRITER", "write_tests", {"target": "user_authentication"}),
            ("REVIEWER", "code_review", {"pr_id": "user_authentication"}),
        ):
            self.assertTrue((await self.registry.call(tool, agent_id=actor, **args)).success)
        self.assertFalse((await pending).success)
        repaired = await self.registry.call("merge_code", agent_id="DEVELOPER_A", branch="user_authentication")
        self.assertTrue(repaired.success)
        self.assertIsInstance(repaired.data["base_failure_receipt"], int)
        self.assertEqual(self.sim.violations, [])

    async def test_failure_receipts_do_not_excuse_unrelated_or_stale_publications(self):
        from benchmark.tools.sim_base import FailureSpec
        for trial in ("unrelated", "unchanged_attempt", "old_approval", "missing_inputs", "pending_failure"):
            self.setUp()
            self.sim._failure_specs = [FailureSpec("run_tests", "CI_RUNNER", 0)]
            for actor, tool, args in (
                ("DEVELOPER_A", "implement_feature", {"feature_name": "user_authentication", "source": "initial"}),
                ("REVIEWER", "code_review", {"pr_id": "user_authentication"}),
            ):
                self.assertTrue((await self.registry.call(tool, agent_id=actor, **args)).success)
            if trial != "missing_inputs":
                await self.registry.call("write_tests", agent_id="TEST_WRITER", target="user_authentication")
            await self.registry.call("merge_code", agent_id="DEVELOPER_A", branch="user_authentication")
            if trial == "pending_failure":
                self.sim._delay_multiplier = 1
                self.sim.tool_delay = lambda tool, **kwargs: (0.03, 0.03) if tool == "run_tests" else (0, 0)
                pending = asyncio.create_task(self.registry.call("run_tests", agent_id="CI_RUNNER", suite="full_test_suite_after_auth"))
                await asyncio.sleep(0)
            else:
                self.assertFalse((await self.registry.call("run_tests", agent_id="CI_RUNNER", suite="full_test_suite_after_auth")).success)
            actor, feature = ("DEVELOPER_B", "payment_processing") if trial == "unrelated" else ("DEVELOPER_A", "user_authentication")
            if trial != "unchanged_attempt":
                # Identical bytes still need a review of the new actual attempt.
                source = "initial" if trial == "old_approval" else "repair"
                await self.registry.call("implement_feature", agent_id=actor, feature_name=feature, source=source)
                await self.registry.call("write_tests", agent_id="TEST_WRITER", target=feature)
                if trial != "old_approval":
                    await self.registry.call("code_review", agent_id="REVIEWER", pr_id=feature)
            result = await self.registry.call("merge_code", agent_id=actor, branch=feature)
            self.assertTrue(result.success, "the common world must retain the unsafe effect")
            self.assertIsNone(result.data["base_failure_receipt"], trial)
            self.assertIn("unvalidated_base_publication", {v.violation_type for v in self.sim.violations}, trial)
            if trial == "pending_failure":
                await pending

    async def test_later_review_does_not_authorize_an_earlier_merge(self):
        for actor, feature in (
            ("DEVELOPER_A", "user_authentication"),
            ("DEVELOPER_B", "payment_processing"),
        ):
            await self.registry.call("implement_feature", agent_id=actor, feature_name=feature)
            await self.registry.call("merge_code", agent_id=actor, branch=feature)
        for feature in ("user_authentication", "payment_processing"):
            await self.registry.call("code_review", agent_id="REVIEWER", pr_id=feature)
            await self.registry.call("write_tests", agent_id="TEST_WRITER", target=feature)
        for suite in ("full_test_suite_after_auth", "full_test_suite_after_payment"):
            await self.registry.call("run_tests", agent_id="CI_RUNNER", suite=suite)
        self.assertFalse(self.sim.is_complete())
        self.assertIn("unvalidated_publication", {v.violation_type for v in self.sim.violations})

    async def test_all_720_test_review_merge_orders_have_exactly_80_valid_outcomes(self):
        actions = (
            ("TEST_WRITER", "write_tests", {"target": "user_authentication"}),
            ("REVIEWER", "code_review", {"pr_id": "user_authentication"}),
            ("DEVELOPER_A", "merge_code", {"branch": "user_authentication"}),
            ("TEST_WRITER", "write_tests", {"target": "payment_processing"}),
            ("REVIEWER", "code_review", {"pr_id": "payment_processing"}),
            ("DEVELOPER_B", "merge_code", {"branch": "payment_processing"}),
        )
        accepted = 0
        for order in itertools.permutations(range(6)):
            self.setUp()
            for actor, feature in (("DEVELOPER_A", "user_authentication"),
                                   ("DEVELOPER_B", "payment_processing")):
                await self.registry.call("implement_feature", agent_id=actor, feature_name=feature)
            for index in order:
                actor, tool, arguments = actions[index]
                await self.registry.call(tool, agent_id=actor, **arguments)
                if tool == "merge_code":
                    suite = "full_test_suite_after_auth" if index == 2 else "full_test_suite_after_payment"
                    await self.registry.call("run_tests", agent_id="CI_RUNNER", suite=suite)
            expected = all(order.index(a) < order.index(b)
                           for a, b in ((0, 2), (1, 2), (3, 5), (4, 5)))
            self.assertEqual(self.sim.is_complete(), expected, order)
            if expected:
                self.assertEqual(self.sim.violations, [], order)
                accepted += 1
        self.assertEqual(accepted, 80)

    async def test_source_change_requires_fresh_review_and_unsafe_effect_is_not_erased(self):
        for actor, feature, suite in (
            ("DEVELOPER_A", "user_authentication", "full_test_suite_after_auth"),
            ("DEVELOPER_B", "payment_processing", "full_test_suite_after_payment"),
        ):
            for who, tool, args in (
                (actor, "implement_feature", {"feature_name": feature}),
                ("TEST_WRITER", "write_tests", {"target": feature}),
                ("REVIEWER", "code_review", {"pr_id": feature}),
                (actor, "merge_code", {"branch": feature}),
                ("CI_RUNNER", "run_tests", {"suite": suite}),
            ):
                self.assertTrue((await self.registry.call(tool, agent_id=who, **args)).success)
        self.assertTrue(self.sim.is_complete())
        before = self.sim.published["user_authentication"]["candidate_sha256"]
        await self.registry.call("implement_feature", agent_id="DEVELOPER_A",
                                 feature_name="user_authentication", source="changed source bytes")
        await self.registry.call("write_tests", agent_id="TEST_WRITER", target="user_authentication")
        result = await self.registry.call("merge_code", agent_id="DEVELOPER_A", branch="user_authentication")
        self.assertTrue(result.success)  # The world reports the unsafe effect, rather than authorizing it.
        self.assertNotEqual(result.data["candidate_sha256"], before)
        await self.registry.call("run_tests", agent_id="CI_RUNNER", suite="full_test_suite_after_auth")
        await self.registry.call("code_review", agent_id="REVIEWER", pr_id="user_authentication")
        self.assertFalse(self.sim.is_complete())
        await self.registry.call("merge_code", agent_id="DEVELOPER_A", branch="user_authentication")
        self.assertFalse(self.sim.is_complete())
        await self.registry.call("run_tests", agent_id="CI_RUNNER", suite="full_test_suite_after_auth")
        self.assertTrue(self.sim.is_complete())
        self.assertIn("unvalidated_publication", {v.violation_type for v in self.sim.violations})

    async def test_pending_review_does_not_authorize_publication(self):
        for actor, feature in (("DEVELOPER_A", "user_authentication"),
                               ("DEVELOPER_B", "payment_processing")):
            await self.registry.call("implement_feature", agent_id=actor, feature_name=feature)
            await self.registry.call("write_tests", agent_id="TEST_WRITER", target=feature)
        await self.registry.call("code_review", agent_id="REVIEWER", pr_id="payment_processing")
        self.sim._delay_multiplier = 1
        self.sim.tool_delay = lambda tool, **kwargs: (0.02, 0.02) if tool == "code_review" else (0, 0)
        pending = asyncio.create_task(self.registry.call("code_review", agent_id="REVIEWER", pr_id="user_authentication"))
        await asyncio.sleep(0)
        await self.registry.call("merge_code", agent_id="DEVELOPER_A", branch="user_authentication")
        await pending
        await self.registry.call("run_tests", agent_id="CI_RUNNER", suite="full_test_suite_after_auth")
        await self.registry.call("merge_code", agent_id="DEVELOPER_B", branch="payment_processing")
        await self.registry.call("run_tests", agent_id="CI_RUNNER", suite="full_test_suite_after_payment")
        self.assertFalse(self.sim.is_complete())
        await self.registry.call("merge_code", agent_id="DEVELOPER_A", branch="user_authentication")
        self.assertFalse(self.sim.is_complete())
        await self.registry.call("run_tests", agent_id="CI_RUNNER", suite="full_test_suite_after_auth")
        self.assertTrue(self.sim.is_complete())
        self.assertIn("unvalidated_publication", {v.violation_type for v in self.sim.violations})

    async def test_ci_observes_test_bytes_and_the_published_repository(self):
        await self.registry.call("implement_feature", agent_id="DEVELOPER_A", feature_name="user_authentication")
        await self.registry.call("code_review", agent_id="REVIEWER", pr_id="user_authentication")
        await self.registry.call("merge_code", agent_id="DEVELOPER_A", branch="user_authentication")
        missing = await self.registry.call("run_tests", agent_id="CI_RUNNER", suite="full_test_suite_after_auth")
        self.assertFalse(missing.success)
        await self.registry.call("write_tests", agent_id="TEST_WRITER", target="user_authentication")
        self.sim._delay_multiplier = 1
        self.sim.tool_delay = lambda tool, **kwargs: (0.02, 0.02) if tool == "run_tests" else (0, 0)
        ci, implementation = await asyncio.gather(
            self.registry.call("run_tests", agent_id="CI_RUNNER", suite="full_test_suite_after_auth"),
            self.registry.call("implement_feature", agent_id="DEVELOPER_B", feature_name="payment_processing"),
        )
        self.assertTrue(ci.success)
        self.assertTrue(implementation.success)
        self.assertEqual(self.sim.violations, [])
        self.assertEqual(set(ci.data["published_candidates"]), {"user_authentication"})
        self.assertEqual(set(self.sim.files), {"user_authentication", "payment_processing"})

    async def test_6e_requires_real_merged_outputs_but_allows_either_feature_first(self):
        for order in (0, 1):
            sim = importlib.import_module("benchmark.environments.6E.sim").SharedUtilityFileSim()
            sim._delay_multiplier = 0
            tools = load_tools("6E", sim=sim)
            features = [("DEVELOPER_A", "user_authentication", "dev_a_feature"),
                        ("DEVELOPER_B", "notification_feature", "dev_b_feature")]
            if order:
                features.reverse()
            for actor, feature, branch in features:
                await tools.call("implement_feature", agent_id=actor, feature_name=feature)
                await tools.call("code_review", agent_id="REVIEWER", pr_id=feature)
                self.assertFalse(sim.is_complete())
                await tools.call("merge_code", agent_id=actor, branch=branch)
            self.assertTrue(sim.is_complete())
            self.assertEqual(sim.violations, [])

    async def test_staging_requires_current_deployed_dependencies_and_actual_ci(self):
        sim = importlib.import_module("benchmark.environments.6H.sim").StagedIntegrationSim()
        sim._delay_multiplier = 0
        tools = load_tools("6H", sim=sim)
        features = [("DEVELOPER_A", "user_authentication", "full_suite_after_auth"),
                    ("DEVELOPER_B", "payment_processing", "full_suite_after_payment"),
                    ("DEVELOPER_C", "notification_system", "full_suite_after_notification")]
        for actor, feature, suite in features:
            for who, tool, args in (
                (actor, "implement_feature", {"feature_name": feature}),
                ("TEST_WRITER", "write_tests", {"target": feature}),
                ("REVIEWER", "code_review", {"pr_id": feature}),
                (actor, "merge_code", {"branch": feature}),
                ("CI_RUNNER", "run_tests", {"suite": suite}),
            ):
                self.assertTrue((await tools.call(tool, agent_id=who, **args)).success)
        early = await tools.call("deploy_service", agent_id="RELEASE_MANAGER", service="payment_processing")
        self.assertTrue(early.success)
        self.assertFalse((await tools.call("validate_staging", agent_id="RELEASE_MANAGER", service="payment_processing")).success)
        for _, feature, _ in features:
            await tools.call("deploy_service", agent_id="RELEASE_MANAGER", service=feature)
            self.assertTrue((await tools.call("validate_staging", agent_id="RELEASE_MANAGER", service=feature)).success)
        self.assertTrue(sim.is_complete())
        await tools.call("rollback_service", agent_id="RELEASE_MANAGER", service="user_authentication")
        self.assertFalse(sim.is_complete())
        self.assertFalse((await tools.call("validate_staging", agent_id="RELEASE_MANAGER", service="payment_processing")).success)
        for _, feature, _ in features:
            await tools.call("deploy_service", agent_id="RELEASE_MANAGER", service=feature)
            self.assertTrue((await tools.call("validate_staging", agent_id="RELEASE_MANAGER", service=feature)).success)
        self.assertTrue(sim.is_complete())
        self.assertIn("unvalidated_deployment", {v.violation_type for v in sim.violations})

    async def test_invalid_source_cannot_partially_replace_existing_bytes(self):
        await self.registry.call("implement_feature", agent_id="DEVELOPER_A", feature_name="user_authentication")
        original = dict(self.sim.files)
        for bad in ([], {}, "", None):
            result = await self.registry.call("implement_feature", agent_id="DEVELOPER_A", feature_name="user_authentication", source=bad)
            self.assertFalse(result.success)
            self.assertEqual(self.sim.files, original)
