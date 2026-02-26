#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

AUTH_MODE="${AUTH_MODE:-auto}"
LOCALE="${LOCALE:-en}"

usage() {
  cat <<'EOF'
Usage: tools/i18n.sh [translate|validate|status|all]

Environment overrides:
  EN_PATH=...        English source file path (if set, only this catalog is processed)
  AUTH_MODE=...      Translator auth mode for translate (default: auto)
  LOCALE=...         CLI locale used for output (default: en)
  CODEX_HOME=...     Optional codex home path

Examples:
  tools/i18n.sh all
  AUTH_MODE=api-key tools/i18n.sh translate
  EN_PATH=crates/greentic-i18n/i18n/en.json tools/i18n.sh validate
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

MODE="${1:-all}"

default_en_paths=(
  "crates/greentic-i18n-translator/i18n/en.json"
  "crates/greentic-i18n/i18n/en.json"
)

resolve_en_paths() {
  if [[ -n "${EN_PATH:-}" ]]; then
    printf '%s\n' "$EN_PATH"
    return 0
  fi
  printf '%s\n' "${default_en_paths[@]}"
}

run_translate_for() {
  local en_path="$1"
  echo "==> translate: $en_path"
  cargo run -p greentic-i18n-translator -- \
    --locale "$LOCALE" \
    translate --langs all --en "$en_path" --auth-mode "$AUTH_MODE"
}

run_validate_for() {
  local en_path="$1"
  echo "==> validate: $en_path"
  cargo run -p greentic-i18n-translator -- \
    --locale "$LOCALE" \
    validate --langs all --en "$en_path"
}

run_status_for() {
  local en_path="$1"
  echo "==> status: $en_path"
  cargo run -p greentic-i18n-translator -- \
    --locale "$LOCALE" \
    status --langs all --en "$en_path"
}

case "$MODE" in
  translate)
    while IFS= read -r en_path; do
      run_translate_for "$en_path"
    done < <(resolve_en_paths)
    ;;
  validate)
    while IFS= read -r en_path; do
      run_validate_for "$en_path"
    done < <(resolve_en_paths)
    ;;
  status)
    while IFS= read -r en_path; do
      run_status_for "$en_path"
    done < <(resolve_en_paths)
    ;;
  all)
    while IFS= read -r en_path; do
      run_translate_for "$en_path"
      run_validate_for "$en_path"
      run_status_for "$en_path"
    done < <(resolve_en_paths)
    ;;
  *)
    echo "Unknown mode: $MODE" >&2
    usage
    exit 2
    ;;
esac
