#!/usr/bin/env zsh

set -euo pipefail

dir="$HOME/Documents/lr2ir-backup/raw-data/searchcgi"

if [[ ! -d "$dir" ]]; then
  echo "Directory not found: $dir"
  exit 1
fi

count=0
for f in "$dir"/*/*.html; do
  [[ -e "$f" ]] || { echo "No .html files found."; exit 0; }
  mv "$f" "${f}.gz"
  (( count++, 1 ))
done

echo "Done. Renamed $count file(s)."
