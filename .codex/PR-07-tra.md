PR-07: Docs + contributor UX
Goal

Make it obvious how to use.

Add docs/i18n-translator.md:

One-time local setup:

codex login (browser)

Run locally:

greentic-i18n-translator translate --langs all

Validate:

greentic-i18n-translator validate --langs all

If login doesn’t work:

export OPENAI_API_KEY=...

greentic-i18n-translator translate --auth-mode api-key ...

How to add a new language file (just add empty i18n/<lang>.json or let tool create)

“Staleness check” policy

Add a command:

greentic-i18n-translator status --langs ...

compares each lang file against translator-state.json

reports missing keys or keys that need update (English changed)
Then CI can fail if en.json changed but translations weren’t regenerated.

Status command is in scope for this PR (not deferred).

Policy matrix (final defaults)

Validation

{} placeholder count: exact

newline count: exact

backtick spans: exact inner text preserved and still wrapped in backticks

State

.i18n/translator-state.json: committed

cache dir: local-only, not committed

Overrides

manual edits preserved by default, including bot runs

--overwrite-manual overrides this behavior explicitly

CI

always run validate when i18n/*.json files change

run staleness/status checks only when i18n/en.json or .i18n/translator-state.json changes

Bot

new PR per run

no-op when there is no diff

Repo layout after PR-01..PR-04 (target)
greentic-i18n/
  i18n/
    en.json
    fr.json
    ...
  crates/
    greentic-i18n/                # existing (if present)
    greentic-i18n-translator/
      src/
        main.rs
        cli.rs
        json_map.rs
        git_diff.rs
        validate.rs
        provider/
          mod.rs
          codex_cli.rs
