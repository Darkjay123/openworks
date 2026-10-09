#!/usr/bin/env bash
set -uo pipefail
source "$(dirname "$0")/lib.sh"
CO=$(addr ow-contractor); I1=$(addr ow-insp1); I2=$(addr ow-insp2); I3=$(addr ow-insp3)
SPEC="{\"CreateProject\":{\"challenge_secs\":120,\"contractor\":\"$CO\",\"defects_secs\":90,\"inspector_threshold\":2,\"inspectors\":[\"$I1\",\"$I2\",\"$I3\"],\"milestones\":[\"8000000000\",\"4000000000\"],\"retention_bps\":1000,\"title\":\"Obiaruku Primary Health Centre Borehole\"}}"
inv "Commissioner proposes project #1 (1,200 XLM)" ow-commissioner $TREASURY propose --proposer ow-commissioner --action "$SPEC"
P=$LAST
inv "Auditor-General approves" ow-auditor $TREASURY approve --signer ow-auditor --id $P
inv "Council proposes too-big project (5,000 XLM)" ow-accountant $TREASURY propose --proposer ow-accountant --action "$(echo $SPEC | sed 's/"8000000000","4000000000"/"50000000000"/')"
PB=$LAST
inv "Commissioner approves the oversized project" ow-commissioner $TREASURY approve --signer ow-commissioner --id $PB
sleep 32
inv "Execute project #1: vault deployed + funded" ow-funder $TREASURY execute --id $P
inv "Execute oversized project: budget check" ow-funder $TREASURY execute --id $PB
echo "VAULT1=$(view $TREASURY project --id 1 | tr -d '\"')" >> deploy.env
inv "Commissioner proposes cancelling project #1 (site dispute)" ow-commissioner $TREASURY propose --proposer ow-commissioner --action '{"CancelProject":1}'
P2=$LAST
inv "Accountant-General approves cancellation" ow-accountant $TREASURY approve --signer ow-accountant --id $P2
sleep 32
inv "Cancel executes: 1,200 XLM returned to treasury" ow-funder $TREASURY execute --id $P2
view $TREASURY budget
