#!/usr/bin/env bash
set -uo pipefail
source "$(dirname "$0")/lib.sh"
sleep 1
inv "Anyone releases M2: project Completed (450 paid, 50 retained)" ow-funder $VAULT0 release --index 2
inv "Release retention during 90s defects-liability period" ow-funder $VAULT0 release_retention
sleep 92
inv "Defects period over: 300 XLM retention released to contractor" ow-funder $VAULT0 release_retention
view $VAULT0 summary
