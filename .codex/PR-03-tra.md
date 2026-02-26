PR-03: Validation rules for your {} placeholder format (+ backticks + newlines)
Goal

Add strict validators tailored to your message format to prevent broken translations.

Rules (final)

For each key:

count("{}") in translation must equal English

newline count must match English exactly

preserve backtick spans exactly: identical inner text and still wrapped in backticks

ensure JSON is valid, no empty translations unless English empty

Normalization detail

Treat both literal newlines and escaped \\n consistently by counting newline characters on parsed strings.

Changes

crates/greentic-i18n-translator/src/validate.rs

count_placeholders_positional(s: &str) -> usize

extract_backtick_spans(s: &str) -> Vec<String>

count_newlines_normalized(s: &str) -> usize

validate_translation(en, tr) -> Result<(), ValidationError>

validate CLI command:

validates all selected langs against en.json

prints a compact report + non-zero exit on failure

Tests

Placeholder mismatch cases

Backtick exact-preservation cases (content + wrappers)

Newline exact-match cases
