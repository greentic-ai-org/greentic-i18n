# I18nId v1 Specification

## Purpose
- `I18nId` v1 is a deterministic digest of a resolved `I18nProfile`. It lets downstream systems cache or transmit profile references without re-running the resolver.

## Canonical CBOR encoding
- The canonical encoding is a definite-length CBOR map whose keys are sorted by their text names. The supported keys cover the full `I18nProfile` v1: `calendar`, `decimal_separator`, `direction`, `first_day`, `hour_cycle`, `numbering_system`, `tag`, `timezone`, plus optional keys `currency`, `collation`, `case_first`, and `units`.
- `currency` is only emitted when the profile contains a currency override; `None` values are omitted entirely (no `null` is written).
- `decimal_separator` is emitted as a single-character UTF-8 string (`.` or `,` today). No floating-point values are allowed anywhere in `I18nProfile` v1—any numeric quantities must be represented as integers or decimal strings.
- `tag` is the canonicalized locale string (e.g., `en-US-u-ca-gregory-cu-usd`), and the derived fields (`direction`, `first_day`, `hour_cycle`, `numbering_system`, `timezone`) are deterministic functions of that canonical tag plus deterministic fallback defaults.
- Example encoding for `en-GB-u-ca-gregory-cu-gbp` → `a36863757272656e637963...?` (see fixtures for the exact hex).

## Digest pipeline
- `canonical_cbor(profile)` → BLAKE3 hash → take the first 128 bits (`digest[0..16]`).
- Prefix the base32 (RFC 4648, no padding) encoding of those 128 bits with `i18n:v1:` to produce the stable identifier.
- Example: canonical bytes `a3...` → BLAKE3 → `i18n:v1:4RSRQGIR...`.

## API guarantees
- `I18nId::parse` accepts only strings that start with `i18n:v1:` followed by 26 base32 characters.
- `I18nId::bytes` exposes the raw 16-byte digest; `I18nId::version()` currently returns `"v1"`.
- The canonical CBOR + hash pipeline is part of the ABI contract—any change to the profile structure or hash must bump the version string and update this spec.

## Future-proofing
- If future profiles introduce additional metadata (e.g., explicit directionality, measurement system hints, derived separators), the canonical CBOR map must still honor sorted keys and the float prohibition so that existing IDs remain stable.
