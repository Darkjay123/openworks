#!/usr/bin/env bash
set -uo pipefail
source "$(dirname "$0")/lib.sh"
H0=$(echo -n "M0 site report: 4.2km cleared and graded, 38 geotagged photos" | sha256sum | cut -c1-64)
inv "Contractor submits M0 evidence (site report hash)" ow-contractor $VAULT0 submit --index 0 --evidence_hash $H0 --evidence_uri "ipfs://demo/abraka-eku/m0-report.pdf"
inv "Contractor tries to certify own work" ow-contractor $VAULT0 attest --inspector ow-contractor --index 0 --approve true
inv "Inspector 1 approves M0" ow-insp1 $VAULT0 attest --inspector ow-insp1 --index 0 --approve true
inv "Inspector 1 votes twice" ow-insp1 $VAULT0 attest --inspector ow-insp1 --index 0 --approve true
inv "Inspector 2 approves M0 (2 of 3: certified)" ow-insp2 $VAULT0 attest --inspector ow-insp2 --index 0 --approve true
inv "Release during 120s public challenge window" ow-funder $VAULT0 release --index 0
sleep 122
inv "Anyone releases M0: 900 XLM paid, 100 XLM retained" ow-funder $VAULT0 release --index 0
inv "Release M0 a second time" ow-funder $VAULT0 release --index 0
