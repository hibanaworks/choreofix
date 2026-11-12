"""Document scoring checks actual inputs, not a prescribed choreography."""
import importlib
import asyncio
import itertools
import unittest

from benchmark.tools import load_tools
from benchmark.tools.sim_base import FailureSpec


class DocumentCausality(unittest.IsolatedAsyncioTestCase):
    def setUp(self):
        self.sim = importlib.import_module(
            "benchmark.environments.7E.sim"
        ).TwoAuthorDocumentSim()
        self.sim._delay_multiplier = 0
        self.registry = load_tools("7E", sim=self.sim)

    async def test_future_writing_cannot_supply_an_earlier_review(self):
        for section in ["CHAPTER_1", "CHAPTER_2"]:
            result = await self.registry.call(
                "review_sections", agent_id="EDITOR", scope=section
            )
            self.assertFalse(result.success, "the reviewed document did not exist")
        for actor, section in [("WRITER_A", "CHAPTER_1"), ("WRITER_B", "CHAPTER_2")]:
            await self.registry.call("write_section", agent_id=actor, section_name=section)
            await self.registry.call("update_references", agent_id=actor, section_name=section)
        await self.registry.call(
            "combine_sections", agent_id="EDITOR", sections=["CHAPTER_1", "CHAPTER_2"]
        )
        self.assertFalse(self.sim.is_complete())

    async def test_later_reviews_cannot_authorize_an_earlier_combination(self):
        for actor, section in [("WRITER_A", "CHAPTER_1"), ("WRITER_B", "CHAPTER_2")]:
            await self.registry.call("write_section", agent_id=actor, section_name=section)
            await self.registry.call("update_references", agent_id=actor, section_name=section)
        combined = await self.registry.call(
            "combine_sections", agent_id="EDITOR", sections=["CHAPTER_1", "CHAPTER_2"]
        )
        self.assertTrue(combined.success, "unsafe effects must remain observable")
        for section in ["CHAPTER_2", "CHAPTER_1"]:
            await self.registry.call("review_sections", agent_id="EDITOR", scope=section)
        self.assertFalse(self.sim.is_complete())
        self.assertTrue(self.sim.violations)
        self.assertTrue(any(event.tool == "combine_sections" for event in self.sim.events))

    async def test_cross_chapter_edit_requires_fresh_review_and_combination(self):
        for actor, section in [("WRITER_B", "CHAPTER_2"), ("WRITER_A", "CHAPTER_1")]:
            await self.registry.call("write_section", agent_id=actor, section_name=section)
            await self.registry.call("update_references", agent_id=actor, section_name=section)
        for section in ["CHAPTER_2", "CHAPTER_1"]:
            await self.registry.call("review_sections", agent_id="EDITOR", scope=section)
        await self.registry.call(
            "combine_sections", agent_id="EDITOR", sections=["CHAPTER_1", "CHAPTER_2"]
        )
        self.assertTrue(self.sim.is_complete())
        changed = await self.registry.call(
            "revise_section", agent_id="WRITER_B", section_name="CHAPTER_1"
        )
        self.assertTrue(changed.success, "either writer may edit the other chapter")
        self.assertFalse(self.sim.is_complete(), "the approved document has changed")
        for section in ["CHAPTER_1", "CHAPTER_2"]:
            await self.registry.call("review_sections", agent_id="EDITOR", scope=section)
        await self.registry.call(
            "combine_sections", agent_id="EDITOR", sections=["CHAPTER_2", "CHAPTER_1"]
        )
        self.assertTrue(self.sim.is_complete())
        self.assertEqual(self.sim.violations, [])

    async def test_all_six_operation_orders_preserve_independent_preparation(self):
        operations = [
            ("write_section", "WRITER_A", {"section_name": "CHAPTER_1"}),
            ("write_section", "WRITER_B", {"section_name": "CHAPTER_2"}),
            ("update_references", "WRITER_A", {"section_name": "CHAPTER_1"}),
            ("update_references", "WRITER_B", {"section_name": "CHAPTER_2"}),
            ("review_sections", "EDITOR", {"scope": "CHAPTER_1"}),
            ("review_sections", "EDITOR", {"scope": "CHAPTER_2"}),
        ]
        accepted = 0
        for order in itertools.permutations(range(len(operations))):
            self.setUp()
            for index in order:
                tool, actor, arguments = operations[index]
                await self.registry.call(tool, agent_id=actor, **arguments)
            await self.registry.call(
                "combine_sections", agent_id="EDITOR", sections=["CHAPTER_1", "CHAPTER_2"]
            )
            prepared_before_reviews = max(order.index(i) for i in range(4)) < min(
                order.index(i) for i in [4, 5]
            )
            self.assertEqual(self.sim.is_complete(), prepared_before_reviews, order)
            if prepared_before_reviews:
                accepted += 1
                self.assertEqual(self.sim.violations, [], order)
        self.assertEqual(accepted, 48, "both writer, reference and review orders remain free")

    async def test_pending_review_does_not_authorize_combination(self):
        for actor, section in [("WRITER_A", "CHAPTER_1"), ("WRITER_B", "CHAPTER_2")]:
            await self.registry.call("write_section", agent_id=actor, section_name=section)
            await self.registry.call("update_references", agent_id=actor, section_name=section)
        await self.registry.call("review_sections", agent_id="EDITOR", scope="CHAPTER_1")
        self.sim._delay_multiplier = 1
        self.sim.tool_delay = lambda tool, **kwargs: (0.02, 0.02) if tool == "review_sections" else (0, 0)
        pending = asyncio.create_task(self.registry.call(
            "review_sections", agent_id="EDITOR", scope="CHAPTER_2"
        ))
        await asyncio.sleep(0)
        await self.registry.call(
            "combine_sections", agent_id="EDITOR", sections=["CHAPTER_1", "CHAPTER_2"]
        )
        await pending
        self.assertFalse(self.sim.is_complete())
        self.assertEqual(len(self.sim.violations), 1)
        await self.registry.call(
            "combine_sections", agent_id="EDITOR", sections=["CHAPTER_2", "CHAPTER_1"]
        )
        self.assertTrue(self.sim.is_complete())
        self.assertEqual(len(self.sim.violations), 1, "repair must not erase the unsafe effect")

    async def test_pending_replacement_is_not_a_completed_review_input(self):
        for actor, section in [("WRITER_A", "CHAPTER_1"), ("WRITER_B", "CHAPTER_2")]:
            await self.registry.call("write_section", agent_id=actor, section_name=section)
            await self.registry.call("update_references", agent_id=actor, section_name=section)
        self.sim._delay_multiplier = 1
        self.sim.tool_delay = lambda tool, **kwargs: (0.02, 0.02) if tool == "write_section" else (0, 0)
        pending = asyncio.create_task(self.registry.call(
            "write_section", agent_id="WRITER_A", section_name="CHAPTER_1", content_notes="replacement"
        ))
        await asyncio.sleep(0)
        premature = await self.registry.call("review_sections", agent_id="EDITOR", scope="CHAPTER_1")
        self.assertFalse(premature.success)
        await pending
        for section in ["CHAPTER_2", "CHAPTER_1"]:
            self.assertTrue((await self.registry.call(
                "review_sections", agent_id="EDITOR", scope=section
            )).success)
        await self.registry.call(
            "combine_sections", agent_id="EDITOR", sections=["CHAPTER_1", "CHAPTER_2"]
        )
        self.assertTrue(self.sim.is_complete())
        self.assertEqual(self.sim.violations, [])

    async def test_a_negative_review_needs_changed_material_before_approval(self):
        self.sim._failure_specs = [FailureSpec("review_sections", "EDITOR", 0)]
        for actor, section in [("WRITER_A", "CHAPTER_1"), ("WRITER_B", "CHAPTER_2")]:
            await self.registry.call("write_section", agent_id=actor, section_name=section)
            await self.registry.call("update_references", agent_id=actor, section_name=section)
        for _ in range(2):
            self.assertFalse((await self.registry.call(
                "review_sections", agent_id="EDITOR", scope="CHAPTER_1"
            )).success)
        revised = await self.registry.call(
            "revise_section", agent_id="WRITER_B", section_name="CHAPTER_1",
            feedback="Resolve the conflicting claims across chapters",
        )
        self.assertEqual(revised.data["artifact"]["feedback"],
                         "Resolve the conflicting claims across chapters")
        for section in ["CHAPTER_1", "CHAPTER_2"]:
            self.assertTrue((await self.registry.call(
                "review_sections", agent_id="EDITOR", scope=section
            )).success)
        await self.registry.call(
            "combine_sections", agent_id="EDITOR", sections=["CHAPTER_1", "CHAPTER_2"]
        )
        self.assertTrue(self.sim.is_complete())
        self.assertEqual(self.sim.violations, [])

    async def test_returned_values_cannot_rewrite_material_or_review_journal(self):
        for actor, section in [("WRITER_A", "CHAPTER_1"), ("WRITER_B", "CHAPTER_2")]:
            notes = f"Actual outline for {section}"
            written = await self.registry.call(
                "write_section", agent_id=actor, section_name=section, content_notes=notes
            )
            self.assertEqual(written.data["artifact"]["content"], notes)
            written.data["artifact"]["content"] = "changed outside the world"
            await self.registry.call("update_references", agent_id=actor, section_name=section)
        for section in ["CHAPTER_1", "CHAPTER_2"]:
            reviewed = await self.registry.call("review_sections", agent_id="EDITOR", scope=section)
            reviewed.data["input_snapshot"]["sections"]["CHAPTER_1"]["content"] = "forged approval"
        combined = await self.registry.call(
            "combine_sections", agent_id="EDITOR", sections=["CHAPTER_1", "CHAPTER_2"]
        )
        self.assertNotEqual(combined.data["input_snapshot"]["sections"]["CHAPTER_1"]["content"],
                            "changed outside the world")
        self.assertEqual(combined.data["input_snapshot"]["sections"]["CHAPTER_1"]["content"],
                         "Actual outline for CHAPTER_1")
        self.assertTrue(self.sim.is_complete())
        self.assertEqual(self.sim.violations, [])
