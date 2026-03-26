---
name: test-naming-convention
description: dxf-engine test naming prefixes to avoid multi-agent conflicts
type: feedback
---

Prefix ALL test functions with agent role to avoid naming conflicts when multiple agents edit same files.
- This agent (dxf-engine test specialist): `test_reader_` for reader.rs, `test_index_ext_` for index.rs
- Always `grep -r 'fn test_.*name' crates/` before adding any test
- Never add a test with a name that already exists

**Why:** Multiple agents adding tests to same files caused duplicate function name compile errors.
**How to apply:** Before every test addition, grep for the proposed name first.
