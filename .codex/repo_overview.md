# Repository Overview

## 1. High-Level Purpose
- The workspace now exposes two crates and a CLI that power Greentic’s deterministic i18n tooling: `crates/greentic-i18n-lib` bundles locale resolution, canonical CBOR/I18nId v1 generation, and the formatting facade, and `crates/greentic-i18n` publishes a CLI that drives those helpers and documents the JSON contract for downstream consumers.
- Supporting doc sources (`docs/`) capture the canonical CBOR schema, the resolver contract/lenient fallsbacks, and the CLI JSON schema so integrators can rely on stable IDs, formatting samples, and schema versions even as the implementation evolves.

## 2. Main Components and Functionality
- **Path:** `crates/greentic-i18n-lib`
  - **Role:** Shared library that replaced the former separate `i18n-core`/`i18n-format` crates by providing canonicalization, caching, and formatting under a single publication.
  - **Key functionality:** `normalize_tag`, the resolver stack, and `I18nProfile`/`I18nId` hashing preserve the canonical CBOR bytes (sorted keys, string-only values, no floats) and feed the cache so `I18nId` stays deterministic; `format.rs` introduces `FormatBackend`, `BasicBackend`, and `FormatFacade` while keeping the `icu4x` feature toggle for future ICU integration; the library re-exports the helpers most consumers need (direction, parsing, formatting, serialization). The crate’s `Cargo.toml` now sets metadata via workspace defaults and publishes the combined functionality.
  - **Key dependencies / integration points:** `blake3` for digests, `data-encoding` for Base32, plus `serde_json` in dev builds for fixture validation; the `tag` module derives directionality/default first day/hour cycle, extension values, and parent chains.
- **Path:** `crates/greentic-i18n`
  - **Role:** CLI surface that exercises the library, normalizes tags, resolves profiles, emits human-readable output, and offers a structured JSON payload.
  - **Key functionality:** Provides `normalize`, `id`, and `resolve` commands (defaulting to `--help` when nothing is passed), prints tagged details (language/script/region/variants/calendar/number system/collation/case-first/units/timezone/first-day/hour-cycle/currency/direction/decimal separator), supports `--json` + `--currency`, and includes the complex example covering language/calendar/number/currency/datetime/unit/direction/script/variant/collation/timezone/first-day/hour-cycle so QA can verify the end-to-end contract.
  - **Key dependencies / integration points:** Depends on `greentic-i18n-lib` for canonicalization, resolution, and formatting; `tests/json_schema.rs` uses `assert_cmd` + `serde_json` to lock the `resolve --json` schema, matching the narrative in `docs/greentic-i18n-cli-json-v1.md`.
- **Path:** `docs/`
  - **Role:** Spec sources for the runtime and CLI contracts.
  - **Key functionality:** `docs/i18n-id-v1.md` describes the canonical CBOR layout (sorted keys, string-only values, no floats, BLAKE3 digest → `i18n:v1:`), `docs/resolver-contract.md` captures the strict/lenient modes plus deterministic lenient defaults (gregory/latn/calendar, timezone fallback to UTC/request, region-based first day/hour cycle), and `docs/greentic-i18n-cli-json-v1.md` lays out the JSON schema (versioned payload, `profile` fields, `samples`).
- **Path:** `ci/`
  - **Role:** Automation helpers that gate contributions and publishing.
  - **Key functionality:** `ci/local_check.sh` sequentially runs `cargo fmt`, `cargo clippy`, `cargo test`, and per-crate `cargo publish --dry-run --allow-dirty`; `publish.sh` installs Rust, caches the toolchain, runs fmt/clippy/test in parallel, and publishes `greentic-i18n-lib` plus `greentic-i18n` once `CARGO_REGISTRY_TOKEN` is available.

## 3. Work In Progress, TODOs, and Stubs
- **Location:** repository-wide search
  - **Status:** None
  - **Short description:** No `TODO`/`FIXME`/`todo!` markers or obvious stubs remain; each crate compiles and contains a finished implementation.

## 4. Broken, Failing, or Conflicting Areas
- **Location:** workspace automation
  - **Evidence:** `ci/local_check.sh` and `publish.sh` are set up but haven’t been run in this session; no known failing tests or unresolved merge conflicts exist.
  - **Likely cause / nature of issue:** None (build/test infrastructure is clean, though publishing still requires a crate token).

## 5. Notes for Future Work
- Maintain the canonical CBOR/digest and CLI JSON schema docs whenever profile fields or CLI outputs change so downstream consumers can pin to a stable `schema_version`.
- Keep the BasicBackend formatter as the deterministic golden implementation while the `icu4x` feature remains opt-in, and continue to keep lenient defaults small/deterministic (gregory/latn/timezone fallback/region-based first day/hour-cycle).
- Expand CLI regression coverage (e.g., `resolve --json` acceptance tests) and consider exposing cache stats/helpers once eviction semantics are locked down.
