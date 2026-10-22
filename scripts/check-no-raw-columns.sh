#!/usr/bin/env bash
# Anti-recurrence gate (design §5.6, §1.4 L2/L6): columns are built in exactly one place.
#
# `Ui::columns` divides whatever width it is handed by the column count — no minimum, no guard.
# At the width a squeezed window hands the tab body it asks for a negative column and trips a
# `debug_assert`; that was the V4 launch panic, and hand-rolled `columns` calls in tabs are what
# brought back the overlapping cards at 1024×640. So `theme::grid` owns the call, tabs reach a
# grid through `grid()`/`split()`, and this script fails if `columns` appears anywhere else.
#
# usage: scripts/check-no-raw-columns.sh        # exit 0 clean, 1 on a stray call
set -u

REPO=$(cd "$(dirname "$0")/.." && pwd)
ALLOWED="src/theme/grid.rs"

# Every spelling of the column API, so a rename cannot slip past the gate.
PATTERN='\.columns(_dyn|_const)?[[:space:]]*\(|ui\.columns|columns_dyn|columns_const'

cd "$REPO" || exit 1

hits=$(grep -rnE "$PATTERN" --include='*.rs' src/ | grep -v "^${ALLOWED}:" || true)

if [ -n "$hits" ]; then
  echo "FAIL: raw column layout outside ${ALLOWED} — use theme::grid::grid or theme::grid::split:" >&2
  echo "$hits" >&2
  exit 1
fi

echo "OK: no raw column layout outside ${ALLOWED}"
