#!/usr/bin/env bash
set -uo pipefail
source "$(dirname "$0")/lib.sh"
HA=$(echo -n "M1 report v1: drainage culverts installed (photos reused from M0)" | sha256sum | cut -c1-64)
HB=$(echo -n "M1 report v2: 14 culverts installed, new geotagged photos" | sha256sum | cut -c1-64)
inv "Contractor submits M1 evidence (round 1)" ow-contractor $VAULT0 submit --index 1 --evidence_hash $HA --evidence_uri "ipfs://demo/abraka-eku/m1-v1.pdf"
inv "Inspector 1 rejects M1 (reused photos)" ow-insp1 $VAULT0 attest --inspector ow-insp1 --index 1 --approve false
inv "Inspector 3 rejects M1: sent back to contractor" ow-insp3 $VAULT0 attest --inspector ow-insp3 --index 1 --approve false
inv "Contractor resubmits M1 (round 2)" ow-contractor $VAULT0 submit --index 1 --evidence_hash $HB --evidence_uri "ipfs://demo/abraka-eku/m1-v2.pdf"
inv "Inspector 2 approves M1" ow-insp2 $VAULT0 attest --inspector ow-insp2 --index 1 --approve true
inv "Inspector 3 approves M1: certified" ow-insp3 $VAULT0 attest --inspector ow-insp3 --index 1 --approve true
inv "Auditor-General proposes voiding M1 (citizen complaint)" ow-auditor $TREASURY propose --proposer ow-auditor --action '{"VoidMilestone":[0,1]}'
P=$LAST
inv "Commissioner co-signs the void" ow-commissioner $TREASURY approve --signer ow-commissioner --id $P
sleep 32
inv "Void executes inside challenge window: M1 back to Pending" ow-funder $TREASURY execute --id $P
inv "Release voided M1" ow-funder $VAULT0 release --index 1
