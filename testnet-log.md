| Step | Result | Return | Transaction |
|---|---|---|---|
| Funder deposits 5,000 XLM | REJECTED | Error(Contract, #10) | |
| Commissioner proposes project #0 (3,000 XLM, 3 milestones) | ok | 5 | https://stellar.expert/explorer/testnet/tx/27da219fb4bcea837b08e340b356a6ed785d636dc836dfb820c06552afc904d5 |
| Non-member tries to approve | REJECTED | Error(Contract, #3) | |
| Execute before 2nd signature | REJECTED | Error(Contract, #8) | |
| Accountant-General approves (2 of 3) | ok |  | https://stellar.expert/explorer/testnet/tx/62fe15362fb0795d844537a85ab856545ae1d358225816699f3c653b0353c87d |
| Execute during 30s timelock | REJECTED | Error(Contract, #9) | |
| Anyone executes after timelock: vault deployed + funded | REJECTED | Error(Contract, #11) | |
