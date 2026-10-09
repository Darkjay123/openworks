#!/usr/bin/env bash
set -uo pipefail
source "$(dirname "$0")/lib.sh"
C1=$(addr ow-commissioner); CO=$(addr ow-contractor); I1=$(addr ow-insp1); I2=$(addr ow-insp2); I3=$(addr ow-insp3); F=$(addr ow-funder)
echo "| Step | Result | Return | Transaction |" > "$LOG"; echo "|---|---|---|---|" >> "$LOG"
inv "Funder deposits 5,000 XLM" ow-funder $TREASURY deposit --from $F --amount 50000000000
SPEC="{\"CreateProject\":{\"challenge_secs\":120,\"contractor\":\"$CO\",\"defects_secs\":90,\"inspector_threshold\":2,\"inspectors\":[\"$I1\",\"$I2\",\"$I3\"],\"milestones\":[\"10000000000\",\"15000000000\",\"5000000000\"],\"retention_bps\":1000,\"title\":\"Abraka-Eku Road Rehabilitation, Phase 1\"}}"
inv "Commissioner proposes project #0 (3,000 XLM, 3 milestones)" ow-commissioner $TREASURY propose --proposer ow-commissioner --action "$SPEC"
P=$LAST
inv "Non-member tries to approve" ow-contractor $TREASURY approve --signer ow-contractor --id $P
inv "Execute before 2nd signature" ow-funder $TREASURY execute --id $P
inv "Accountant-General approves (2 of 3)" ow-accountant $TREASURY approve --signer ow-accountant --id $P
inv "Execute during 30s timelock" ow-funder $TREASURY execute --id $P
sleep 32
inv "Anyone executes after timelock: vault deployed + funded" ow-funder $TREASURY execute --id $P
echo "VAULT0=$(view $TREASURY project --id 0 | tr -d '\"')" >> deploy.env
tail -1 deploy.env
