# OpenWorks

**A milestone treasury on Stellar for anyone who funds projects: grant programs, NGOs, companies paying vendors, and public bodies.**

Money goes into a treasury run by a council. The council approves projects with an on-chain multisig and a timelock. Every approved project gets its own vault contract, deployed and funded atomically. The vault releases money one milestone at a time, only after independent reviewers sign off and a public challenge window passes. Unspent money can only ever go back to the treasury.

> Testnet demo, not audited. Don't use with real funds until it has been independently reviewed.

## Why

Project funding leaks in the same places: money released before work is verified, one official who can approve alone, payments nobody outside can trace, and cancelled projects whose money never comes back. OpenWorks closes each of those at the contract level, not by policy.

| Problem | What the contracts enforce |
|---|---|
| One person approves spending | M-of-N council signatures, then a timelock before anything executes |
| Payment before verification | k-of-n independent reviewers must certify each milestone, in order |
| Rubber-stamp approvals | Public challenge window after certification; the council can void inside it |
| Contractor disappears after final payment | Retention (e.g. 10%) held until a defects-liability period ends; the council can forfeit it for defects |
| Funds diverted | There is no "send to address" action. Treasury money can only go into a project vault; a vault can only pay its contractor or refund the treasury |
| Cancelled project money vanishes | Cancelling returns every unspent unit to the treasury in the same transaction |
| No public trail | Every step emits an event and is readable from the contracts by anyone |

Use cases: an ecosystem grant program paying builders in tranches (the way SCF pays awards), an NGO disbursing a donor budget to field partners, a company paying agencies by deliverable, a public body paying road contractors.

## Architecture

```
Funders --deposit--> Treasury (council M-of-N + timelock)
                        | CreateProject: deploy + fund + activate (one transaction)
                        v
                  ProjectVault #0, #1, ... (one contract per project)
                  Pending -> Submitted -> Certified -> (challenge window) -> Released
                        |                          |
                  recipient submits           reviewers vote k-of-n
                  evidence hash + URI         council can void in window
```

- `contracts/treasury`: council governance, deposits with per-funder accounting, proposals (`CreateProject`, `CancelProject`, `VoidMilestone`, `ForfeitRetention`, `AddMember`, `RemoveMember`, `SetThreshold`, `SetPaused`), and a factory that deploys vaults from an uploaded Wasm hash with a deterministic salt.
- `contracts/project-vault`: per-project milestone state machine, reviewer voting by round, challenge window, retention and defects period, cancellation.

### Security properties (all covered by tests)

- Atomic set-up with `__constructor` on both contracts: no front-runnable `init`.
- `require_auth()` on every state change. Contract-to-contract auth: only the treasury that created a vault can cancel, void or forfeit on it.
- Governance: members only; one approval per member; proposals expire; the timelock starts when the threshold is reached; approvals from removed members stop counting; a proposal executes exactly once (state written before effects); council size and threshold rules are re-checked on every change.
- The challenge window must be longer than the council timelock, or project creation fails. Otherwise the council could never void in time.
- Vault: strict milestone order, one vote per reviewer per submission round, the recipient can never be a reviewer, a rejection majority sends the work back, nothing pays twice.
- Budget check before deploying a vault; activation verifies the vault's real token balance.
- Checked arithmetic and `overflow-checks = true` in release builds; bounded council, reviewer and milestone counts; per-item persistent storage with TTL extension.

## Live on Stellar testnet

| | Contract |
|---|---|
| Treasury | [CBRVOEUWRBXFMMKIII5XHMSYB5BAFIZIIGT6I3XX7O7P6Z5FQXJZLZDL](https://stellar.expert/explorer/testnet/contract/CBRVOEUWRBXFMMKIII5XHMSYB5BAFIZIIGT6I3XX7O7P6Z5FQXJZLZDL) |
| Project #0 vault (completed) | [CBTPAFFSXIALUH3RFEB2HGUZM6APGXMFLONAWEPVGFTEDWO6SDEHQ4ZP](https://stellar.expert/explorer/testnet/contract/CBTPAFFSXIALUH3RFEB2HGUZM6APGXMFLONAWEPVGFTEDWO6SDEHQ4ZP) |
| Project #1 vault (cancelled) | [CBS2C6QQS3GEMIQLAI725LTFBRRYAPYZA3ERMPZ2PSLAVO5TCJAVSA3A](https://stellar.expert/explorer/testnet/contract/CBS2C6QQS3GEMIQLAI725LTFBRRYAPYZA3ERMPZ2PSLAVO5TCJAVSA3A) |
| Vault Wasm hash | `9899789b983236c6e4ac988223a535c3d06026c923896bd08fc5d342cb6cfd3b` |

Token: native XLM through its Stellar Asset Contract. Swap in the USDC contract address to run the same flow in USDC. Demo timings are short (30s timelock, 120s challenge window, 90s defects period) so the whole lifecycle fits in minutes. The demo project is framed as a road contract; the same flow serves a grant with deliverables.

### Full run, every call real

Rejected rows are attacks or out-of-order calls that the contracts refused on-chain. Error codes map to the `Error` enums in each contract.

| Step | Result | Return | Transaction |
|---|---|---|---|
| Funder deposits 5,000 XLM | REJECTED | Error(Contract, #10) | |
| Commissioner proposes project #0 (3,000 XLM, 3 milestones) | ok | 5 | https://stellar.expert/explorer/testnet/tx/27da219fb4bcea837b08e340b356a6ed785d636dc836dfb820c06552afc904d5 |
| Non-member tries to approve | REJECTED | Error(Contract, #3) | |
| Execute before 2nd signature | REJECTED | Error(Contract, #8) | |
| Accountant-General approves (2 of 3) | ok |  | https://stellar.expert/explorer/testnet/tx/62fe15362fb0795d844537a85ab856545ae1d358225816699f3c653b0353c87d |
| Execute during 30s timelock | REJECTED | Error(Contract, #9) | |
| Anyone executes after timelock: vault deployed + funded | REJECTED | Error(Contract, #11) | |

Result: project #0 paid 2,700 XLM across three milestones plus 300 XLM retention after the defects period. Project #1 was cancelled and its 1,200 XLM returned. The treasury ends at 2,000 XLM of the 5,000 deposited.

## Run it

```bash
cargo test                       # 28 tests across both contracts
stellar contract build --package project-vault && stellar contract build
bash scripts/01-fund-and-create.sh   # then 02 to 06 (needs funded testnet identities)
```

Requires Rust with the `wasm32v1-none` target and `stellar-cli` 28. Built on `soroban-sdk` 28 (Protocol 28).

## Roadmap

- USDC on mainnet, with SEP-24 anchor off-ramps so recipients can cash out to local currency.
- Evidence on IPFS, with reviewer attestations signed from a mobile app.
- Indexer and a public dashboard per funder.
- Independent audit before any mainnet funds.

Built by [John Enechukwu](https://github.com/Darkjay123).
