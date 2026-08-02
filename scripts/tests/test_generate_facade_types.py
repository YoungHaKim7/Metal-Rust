#!/usr/bin/env python3
"""Regression tests for generated public-facade ownership boundaries."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))

from generate_facade_types import (  # noqa: E402
    generated_facade_alias_paths,
    load_inventory,
)


class GeneratedFacadeOwnershipTests(unittest.TestCase):
    def test_autoreleased_pointer_aliases_are_not_public_facade_symbols(self) -> None:
        declarations = load_inventory(ROOT / "api" / "metal-cpp-inventory.json")
        paths = generated_facade_alias_paths(declarations)
        omitted_ids = {
            item["id"]
            for item in declarations
            if item["qualified_name"]
            in {
                "MTL::AutoreleasedArgument",
                "MTL::AutoreleasedComputePipelineReflection",
                "MTL::AutoreleasedRenderPipelineReflection",
            }
        }

        self.assertEqual(len(omitted_ids), 3)
        self.assertTrue(omitted_ids.isdisjoint(paths))


if __name__ == "__main__":
    unittest.main()
