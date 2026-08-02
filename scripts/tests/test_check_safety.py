"""Self-tests for scripts/check-safety.py."""

from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).parents[1] / "check-safety.py"
SPEC = importlib.util.spec_from_file_location("check_safety", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
CHECKER = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = CHECKER
SPEC.loader.exec_module(CHECKER)


class SafetyCheckerTests(unittest.TestCase):
    def workspace(self) -> tempfile.TemporaryDirectory[str]:
        temporary = tempfile.TemporaryDirectory()
        root = Path(temporary.name)
        (root / "src").mkdir()
        (root / "tests").mkdir()
        ffi = root / "crates" / "metal-rust-ffi" / "src"
        ffi.mkdir(parents=True)
        (root / "src" / "lib.rs").write_text("#![forbid(unsafe_code)]\n")
        return temporary

    def test_private_ffi_objc_declaration_is_not_a_public_signature_leak(self) -> None:
        with self.workspace() as temporary:
            root = Path(temporary)
            ffi = root / "crates" / "metal-rust-ffi" / "src"
            (ffi / "lib.rs").write_text("mod raw;\n")
            (ffi / "raw.rs").write_text(
                "pub struct Raw;\n"
                "impl Raw {\n"
                "    pub unsafe fn pointer(&self) -> *const objc2::runtime::AnyObject {\n"
                "        // SAFETY: test fixture only.\n"
                "        unsafe { core::ptr::null() }\n"
                "    }\n"
                "}\n"
            )
            self.assertEqual(CHECKER.run_audit(root), [])

    def test_exported_ffi_method_rejects_objc_and_raw_pointer(self) -> None:
        with self.workspace() as temporary:
            root = Path(temporary)
            ffi = root / "crates" / "metal-rust-ffi" / "src"
            (ffi / "lib.rs").write_text("mod wrapper;\npub use wrapper::Wrapper;\n")
            (ffi / "wrapper.rs").write_text(
                "pub struct Wrapper;\n"
                "impl Wrapper {\n"
                "    pub fn leaked(&self) -> *const objc2::runtime::AnyObject { loop {} }\n"
                "}\n"
            )
            findings = CHECKER.run_audit(root)
            self.assertTrue(any(item.category == "ffi-signature" for item in findings))
            self.assertTrue(any("Wrapper::leaked" in item.message for item in findings))

    def test_restricted_ffi_method_is_not_part_of_the_public_boundary(self) -> None:
        with self.workspace() as temporary:
            root = Path(temporary)
            ffi = root / "crates" / "metal-rust-ffi" / "src"
            (ffi / "lib.rs").write_text("mod wrapper;\npub use wrapper::Wrapper;\n")
            (ffi / "wrapper.rs").write_text(
                "pub struct Wrapper;\n"
                "impl Wrapper {\n"
                "    pub(crate) fn inner(&self) -> *const objc2::runtime::AnyObject { loop {} }\n"
                "}\n"
            )
            self.assertEqual(CHECKER.run_audit(root), [])

    def test_exported_public_unsafe_function_is_rejected(self) -> None:
        with self.workspace() as temporary:
            root = Path(temporary)
            ffi = root / "crates" / "metal-rust-ffi" / "src"
            (ffi / "lib.rs").write_text("mod wrapper;\npub use wrapper::danger;\n")
            (ffi / "wrapper.rs").write_text("pub unsafe fn danger() {}\n")
            findings = CHECKER.run_audit(root)
            self.assertTrue(any(item.category == "ffi-public-unsafe" for item in findings))

    def test_root_unsafe_trait_and_unsafe_extern_are_rejected(self) -> None:
        with self.workspace() as temporary:
            root = Path(temporary)
            ffi = root / "crates" / "metal-rust-ffi" / "src"
            (ffi / "lib.rs").write_text(
                "pub unsafe trait Boundary {}\n"
                "unsafe extern \"C\" { pub fn exposed(); }\n"
            )
            findings = CHECKER.run_audit(root)
            public_unsafe = [item for item in findings if item.category == "ffi-public-unsafe"]
            self.assertTrue(any("Boundary" in item.message for item in public_unsafe))
            self.assertTrue(any("exposed" in item.message for item in public_unsafe))

    def test_exported_field_enum_payload_and_trait_method_are_checked(self) -> None:
        with self.workspace() as temporary:
            root = Path(temporary)
            ffi = root / "crates" / "metal-rust-ffi" / "src"
            (ffi / "lib.rs").write_text(
                "pub struct Fields { pub raw: *mut u8, private: *mut u8 }\n"
                "pub enum Payload { Raw(*const u8) }\n"
                "pub trait Callback { fn selected(&self, value: Sel); }\n"
            )
            findings = CHECKER.run_audit(root)
            messages = [item.message for item in findings if item.category == "ffi-signature"]
            self.assertTrue(any("Fields" in message for message in messages))
            self.assertTrue(any("Payload" in message for message in messages))
            self.assertTrue(any("Callback" in message for message in messages))

    def test_generated_lifecycle_shell_and_module_are_rejected(self) -> None:
        with self.workspace() as temporary:
            root = Path(temporary)
            ffi = root / "crates" / "metal-rust-ffi" / "src"
            (root / "src" / "helper.rs").write_text("pub type SharedPtr<T> = std::sync::Arc<T>;\n")
            (ffi / "lib.rs").write_text("pub use metal::generated_object_types;\n")
            findings = CHECKER.run_audit(root)
            categories = {item.category for item in findings}
            self.assertIn("generated-lifecycle", categories)
            self.assertIn("generated-module", categories)

    def test_unsafe_block_still_requires_a_local_proof(self) -> None:
        with self.workspace() as temporary:
            root = Path(temporary)
            ffi = root / "crates" / "metal-rust-ffi" / "src"
            (ffi / "lib.rs").write_text("mod implementation;\n")
            (ffi / "implementation.rs").write_text("fn private() { unsafe {} }\n")
            findings = CHECKER.run_audit(root)
            self.assertTrue(any(item.category == "ffi-unsafe-proof" for item in findings))

    def test_send_sync_policy_missing_and_valid_states_are_distinct(self) -> None:
        with self.workspace() as temporary:
            root = Path(temporary)
            ffi = root / "crates" / "metal-rust-ffi" / "src"
            (ffi / "lib.rs").write_text("")
            policy = root / "api" / "send-sync-policy.json"
            missing = CHECKER.run_audit(root, policy)
            self.assertTrue(any(item.category == "send-sync-policy" for item in missing))
            policy.parent.mkdir()
            policy.write_text(
                json.dumps(
                    {
                        "schema": 1,
                        "types": [
                            {
                                "type": "metal_rust::Device",
                                "send": False,
                                "sync": False,
                                "evidence": "compile-time assertion plus Apple threading contract",
                            }
                        ],
                    }
                )
            )
            self.assertEqual(CHECKER.run_audit(root, policy), [])

    def test_objc_owner_requires_thread_bound_marker(self) -> None:
        with self.workspace() as temporary:
            root = Path(temporary)
            ffi = root / "crates" / "metal-rust-ffi" / "src"
            (ffi / "lib.rs").write_text("mod wrapper;\npub use wrapper::Wrapper;\n")
            (ffi / "wrapper.rs").write_text(
                "pub struct Wrapper { inner: Retained<AnyObject> }\n"
            )
            findings = CHECKER.run_audit(root)
            self.assertTrue(any(item.category == "send-sync-marker" for item in findings))
            (ffi / "lib.rs").write_text(
                "use std::marker::PhantomData;\n"
                "use std::rc::Rc;\n"
                "pub(crate) struct ThreadBound(PhantomData<Rc<()>>);\n"
                "mod wrapper;\n"
                "pub use wrapper::Wrapper;\n"
            )
            (ffi / "wrapper.rs").write_text(
                "pub struct Wrapper {\n"
                "    inner: Retained<AnyObject>,\n"
                "    _thread_bound: crate::ThreadBound,\n"
                "}\n"
            )
            self.assertFalse(
                any(item.category == "send-sync-marker" for item in CHECKER.run_audit(root))
            )


if __name__ == "__main__":
    unittest.main()
