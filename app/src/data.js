export const ENV={"TOKEN": "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC", "VAULT_HASH": "9899789b983236c6e4ac988223a535c3d06026c923896bd08fc5d342cb6cfd3b", "TREASURY": "CBRVOEUWRBXFMMKIII5XHMSYB5BAFIZIIGT6I3XX7O7P6Z5FQXJZLZDL", "VAULT0": "CBTPAFFSXIALUH3RFEB2HGUZM6APGXMFLONAWEPVGFTEDWO6SDEHQ4ZP", "VAULT1": "CBS2C6QQS3GEMIQLAI725LTFBRRYAPYZA3ERMPZ2PSLAVO5TCJAVSA3A"};
export const LOG=[
{
"step": "Funder deposits 5,000 XLM",
"ok": false,
"err": "Error(Contract, #10)",
"tx": ""
},
{
"step": "Commissioner proposes project #0 (3,000 XLM, 3 milestones)",
"ok": true,
"err": "",
"tx": "https://stellar.expert/explorer/testnet/tx/27da219fb4bcea837b08e340b356a6ed785d636dc836dfb820c06552afc904d5"
},
{
"step": "Non-member tries to approve",
"ok": false,
"err": "Error(Contract, #3)",
"tx": ""
},
{
"step": "Execute before 2nd signature",
"ok": false,
"err": "Error(Contract, #8)",
"tx": ""
},
{
"step": "Accountant-General approves (2 of 3)",
"ok": true,
"err": "",
"tx": "https://stellar.expert/explorer/testnet/tx/62fe15362fb0795d844537a85ab856545ae1d358225816699f3c653b0353c87d"
},
{
"step": "Execute during 30s timelock",
"ok": false,
"err": "Error(Contract, #9)",
"tx": ""
},
{
"step": "Anyone executes after timelock: vault deployed + funded",
"ok": false,
"err": "Error(Contract, #11)",
"tx": ""
}
];
