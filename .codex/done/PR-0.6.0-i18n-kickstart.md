# PR: greentic-i18n – Kickstart (Resolver + I18nId + Formatting Facade)

**Base branch:** `release/0.6.0` (new repo can start on this branch from day 1)  
Goal is to make i18n **convenient** for every other Greentic repo.

## Goal

Create a new crate/repo `greentic-i18n` that provides:

- canonicalization of locale tags (`I18nTag`)
- `I18nId` generation (`i18n:v1:...`) aligned with `greentic-types`
- resolution: (tenant default + user preference + overrides) -> `I18nProfile`
- formatting facade APIs for:
  - numbers
  - currency
  - date/time
  - units (optional in v1)

Keep the public API stable; allow swapping implementation later (e.g., ICU4X).

## Repo layout

- `crates/i18n-core`
  - tag canonicalization
  - id generation helpers
  - profile struct re-export (or use from greentic-types)
  - resolver trait + default resolver
- `crates/i18n-format`
  - formatter traits and implementations
  - initial “basic” formatter (non-ICU) if necessary
- `crates/i18n-cli` (optional but recommended)
  - `greentic-i18n normalize <tag>`
  - `greentic-i18n id <tag>`
  - `greentic-i18n resolve <tag> [--currency GBP] ...`

## Public API (minimum)

- `pub fn normalize_tag(input: &str) -> Result<I18nTag>`
- `pub fn id_for_tag(tag: &I18nTag) -> I18nId`
- `pub trait I18nResolver { fn resolve(&self, req: I18nRequest) -> Result<I18nProfile>; }`
- `pub struct I18n { resolver: Arc<dyn I18nResolver>, cache: ... }`
  - `fn profile(&self, id: &I18nId) -> Option<I18nProfile>`
  - `fn resolve_and_cache(&self, req: I18nRequest) -> (I18nId, I18nProfile)`

- Formatting:
  - `profile.format_number(...)`
  - `profile.format_currency(...)`
  - `profile.format_datetime(...)`

## Tests

- normalize tests for common tags:
  - `en-gb` -> `en-GB`
  - `zh-hant-tw` -> `zh-Hant-TW`
  - preserves `-u-` extensions ordering (document rules)
- stable id tests for:
  - `en-GB-u-ca-gregory-cu-gbp`
- formatting smoke tests:
  - decimal separator respects profile
  - currency symbol/placement basic correctness (can be minimal if no ICU yet)

## Acceptance criteria

- Other repos can depend on `greentic-i18n` to get:
  - a stable `I18nId` generator/validator
  - a resolver they can call from session initialization
  - formatting helpers for UI/card/template layers
