#!/usr/bin/env bash
set -uo pipefail
source "$(dirname "$0")/lib.sh"
HC=$(echo -n "M1 report v3: 14 culverts, independent lab test attached" | sha256sum | cut -c1-64)
H2=$(echo -n "M2 report: asphalt surfacing 4.2km, road markings, handover certificate" | sha256sum | cut -c1-64)
inv "Contractor resubmits M1 (round 3, lab test attached)" ow-contractor $VAULT0 submit --index 1 --evidence_hash $HC --evidence_uri "ipfs://demo/abraka-eku/m1-v3.pdf"
inv "Inspector 1 approves M1" ow-insp1 $VAULT0 attest --inspector ow-insp1 --index 1 --approve true
inv "Inspector 2 approves M1: certified" ow-insp2 $VAULT0 attest --inspector ow-insp2 --index 1 --approve true
inv "Contractor tries to skip ahead to M2" ow-contractor $VAULT0 submit --index 2 --evidence_hash $H2 --evidence_uri "ipfs://demo/abraka-eku/m2.pdf"
sleep 122
inv "Anyone releases M1: 1,350 XLM paid, 150 XLM retained" ow-funder $VAULT0 release --index 1
inv "Contractor submits M2 evidence" ow-contractor $VAULT0 submit --index 2 --evidence_hash $H2 --evidence_uri "ipfs://demo/abraka-eku/m2.pdf"
inv "Inspector 1 approves M2" ow-insp1 $VAULT0 attest --inspector ow-insp1 --index 2 --approve true
inv "Inspector 3 approves M2: certified" ow-insp3 $VAULT0 attest --inspector ow-insp3 --index 2 --approve true
