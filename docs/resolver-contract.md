# Resolver Contract v1

## Resolve modes and precedence
- `ResolveMode::Lenient` is the default and produces deterministic metadata via small fallback rules and the tenant default.
- `ResolveMode::Strict` requires explicit timezone/calendar/numbering-system inputs; any missing field in strict mode returns `I18nError::MissingField` so callers know exactly which data was absent.
- The resolver honors an explicit precedence order when evaluating inputs:
  1. content tag
  2. session override
  3. user preference
  4. tenant default (`DefaultResolver::tenant_default`).
  The first available tag becomes the resolved profile/tag and the fallback chain begins with that tag.

## Lenient defaults
- **Calendar** defaults to `gregory` and **numbering system** to `latn` when the tag lacks `-u-ca` or `-u-nu` extensions.
- **Timezone** is taken from the `-u-tz` extension if present, otherwise from the request payload (e.g., CLI `--timezone`/session hint). When both are missing, it deterministically falls back to `UTC`.
- **Direction** is derived through `direction_for_language` (languages such as `ar`, `he`, `fa`, `ur`, `ps`, `dv`, `yi` yield `rtl`; everything else is `ltr`). This derivation is stable and runs even if the tag omits any `-u-` direction hints.
- **First day of week / hour cycle** use a minimal rule set keyed to a few common regions (US, GB/EU, SA) and otherwise default to Monday + `h23`:
  | Region | First day | Hour cycle |
  |--------|-----------|------------|
  | `US`   | `sun`     | `h12`      |
  | `GB`/`EU` | `mon`  | `h23`      |
  | `SA`   | `sat`     | `h23`      |
  | default | `mon`   | `h23`      |
- Other metadata such as units, collation, and case-first are read straight from the `-u-` extensions when present and omitted otherwise.

## Fallback chain
- `build_fallback_chain` enumerates the parent tags of the resolved tag (dropping subtags one by one) and appends the tenant default unless it already appears in the chain. Duplicates are removed while preserving the derived order.
- The fallback chain is therefore deterministic: it always starts with the chosen tag, then its parents, and lastly the tenant default if needed.

## Cache and registry
- `I18n` now maintains an `I18nCache` that keeps entries as `Arc<I18nProfile>` plus the associated fallback chain and evicts the least recently used entries once `I18nCacheConfig::max_entries` is exceeded.
- `resolve_and_cache` inserts the resolved profile so downstream consumers can call `get(id)`/`get_with_fallback(id)` to rehydrate the profile from its `I18nId` without rerunning the resolver.
- `insert(profile, fallback_chain)` recomputes the canonical CBOR digest and registers the profile explicitly if other systems already have the metadata.
- Eviction remains internal; cache hits simply extend the life of an entry and misses should trigger `resolve_and_cache`.

## Testing the contract
- `tests::resolver_precedence_table_is_deterministic` locks the explicit precedence order.
- `tests::lenient_mode_derives_deterministic_defaults` ensures the derived calendar/numbering/timezone/direction/first-day/hour-cycle values stay fixed for the same input.
- `tests::fallback_chain_reuses_tenant_parent` confirms the fallback list stays unique and ordered.
- `tests::strict_mode_requires_*` cover the deterministic error responses required in strict mode.
