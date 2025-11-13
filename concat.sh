set -euo pipefail

usage() { echo "Usage: $0 --input /path/to/directory"; exit 1; }

INPUT_DIR=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --input)
      shift
      [[ $# -gt 0 ]] || { echo "Error: --input requires a path"; usage; }
      INPUT_DIR="$1"
      shift
      ;;
    -h|--help) usage ;;
    *) echo "Unknown argument: $1"; usage ;;
  esac
done

[[ -n "$INPUT_DIR" ]] || usage
[[ -d "$INPUT_DIR" ]] || { echo "Error: not a directory: $INPUT_DIR"; exit 1; }

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"

OUTFILE="$(mktemp "$SCRIPT_DIR/concat_XXXXXXXX.txt")"

while IFS= read -r -d '' file; do
  if [[ -e "$file" ]] && [[ -e "$OUTFILE" ]] && [[ "$file" -ef "$OUTFILE" ]]; then
    continue
  fi

  printf '%s\n' "$file" >> "$OUTFILE"

  printf '```\n' >> "$OUTFILE"

  cat -- "$file" >> "$OUTFILE"

  if [[ -s "$file" ]]; then
    last_char="$(tail -c1 -- "$file" || true)"
    if [[ "$last_char" != $'\n' ]]; then
      printf '\n' >> "$OUTFILE"
    fi
  fi

  printf '```\n' >> "$OUTFILE"
done < <(find "$INPUT_DIR" -type f -readable -print0 2>/dev/null)

echo "Wrote: $OUTFILE"