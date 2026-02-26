PR-01: Add translator crate + CLI skeleton (no LLM yet)
Goal

Introduce crates/greentic-i18n-translator as a workspace member with a stable CLI surface and JSON read/write helpers.

Changes

Cargo.toml (workspace): add member crates/greentic-i18n-translator

crates/greentic-i18n-translator/

src/main.rs – CLI entry

src/lib.rs – reusable logic

src/json_map.rs – read/write stable JSON map (sorted keys, deterministic formatting)

src/paths.rs – locate i18n/en.json, target lang files, etc.

src/cli.rs – clap command tree

CLI (initial)

greentic-i18n-translator diff --base <ref> --head <ref> --en i18n/en.json

greentic-i18n-translator validate --langs <...> --en i18n/en.json

greentic-i18n-translator translate --langs <...> --en i18n/en.json (stubbed: prints “not implemented”)

Tests

Unit test: round-trip JSON read/write keeps same key/value content and stable ordering.

Unit test: validates the sample file parses and keys/values preserved.

Notes

No Codex integration yet; keep this PR purely structural.