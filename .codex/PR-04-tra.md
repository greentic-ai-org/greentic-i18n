PR-04: Codex CLI provider + auth flow (browser login, API key fallback)
Goal

Implement translation using Codex CLI with the exact UX you asked for:

If already logged in → run

Else → open browser login

If fails and API key provided → login via API key then run

Provider structure

crates/greentic-i18n-translator/src/provider/mod.rs

trait TranslatorProvider { translate_batch(lang, items) -> map }

crates/greentic-i18n-translator/src/provider/codex_cli.rs

Auth helpers

codex login status (check)

codex login (browser)

optional: codex login --device-auth when --device-auth passed

codex login --with-api-key reading from stdin when --api-key-stdin or OPENAI_API_KEY set

Translation call approach

For each batch:

invoke codex with a prompt requiring JSON-only output

parse JSON

validate each translated string (PR-03 validator)

retry up to 2 times on validation failure with a “you broke placeholders/backticks/newlines” error message

CLI additions

translate --langs all|fr,de,...

flags:

--auth-mode auto|browser|device|api-key

--codex-home <path> (sets CODEX_HOME)

--batch-size <n>

--max-retries <n>

--glossary i18n/glossary.json (optional in PR-05)

Tests

Unit test for prompt-builder (ensures it includes placeholder rules)

Unit test for parsing provider JSON response

Integration test (ignored): runs codex if available and logged-in; otherwise skips