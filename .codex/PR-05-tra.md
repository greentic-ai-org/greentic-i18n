PR-05: Translation cache + “don’t clobber manual edits”
Goal

Make repeated runs cheap and safe.

Add cache

Default: OS user cache dir (not repo)

Linux: ~/.cache/greentic/i18n-translator/

macOS: ~/Library/Caches/.../greentic/i18n-translator/

Windows: %LOCALAPPDATA%/.../greentic/i18n-translator/

Allow override with --cache-dir

cache key: hash(lang + english_text + glossary_version + rules_version)

store translated string

Add state tracking

.i18n/translator-state.json:

tracked in git (commit this file)

per key:

last english hash translated

last produced translation hash

engine/model tag

timestamp

Manual override policy

If L.json[key] differs from last bot translation while English hasn’t changed:

keep the existing translation

report “manual override preserved” (optional --overwrite-manual to force)

Applies to all runs, including bot PR runs (manual wins by default).

Tests

Cache hit skips provider call (mock provider)

Manual override preserved logic

Cache path behavior (repo-local cache is not required)
