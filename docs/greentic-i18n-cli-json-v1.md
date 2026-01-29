# `greentic-i18n` CLI JSON schema (v1)

The `resolve --json` output for `greentic-i18n` is a deterministic contract that downstream tools can parse. It is versioned so future releases can add fields, rename sections, or introduce new payload variants without ambiguity.

## Payload structure

```json
{
  "schema_version": "v1",
  "tag": "<canonical tag>",
  "id": "<i18n:v1:...>",
  "fallback_chain": ["<tag>", "..."],
  "profile": {
    "language": "en",
    "script": null,
    "region": "US",
    "variants": [],
    "calendar": "gregory",
    "number_system": "latn",
    "currency_tag": "usd",
    "currency": "USD",
    "timezone": "asia/muscat",
    "first_day": "sat",
    "hour_cycle": "h23",
    "collation": null,
    "case_first": null,
    "units": null,
    "direction": "rtl",
    "decimal_separator": "."
  },
  "samples": {
    "number": "1234.56",
    "currency": "USD 42.00",
    "date": "29/01/2026",
    "time": "12:35",
    "datetime": "1769689962.832 UTC"
  }
}
```

### Field guidance

- `schema_version` must equal `"v1"` in this release.
- `tag`, `id`, and `fallback_chain` mirror the resolver output and identify the canonical profile.
- The `profile` object mirrors the CLI’s decoded fields (language/script/region/extensions, calendar, numbering system, timezone, first-day/hour-cycle, direction, decimal separator, optional `currency_tag`/`currency`, collation, case-first, and `units`).
- `samples` bundles the CLI samples for number, currency, date, time, and datetime so consumers can display representative output.

When the CLI adds new metadata or sample types, the schema version should be bumped to `v2` unless backwards-compatible fields are appended in a non-breaking manner. Regression tests should continue verifying the `v1` schema until that bump occurs.
