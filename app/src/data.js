export const ENV={"TOKEN": "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC", "TREASURY": "CBRVOEUWRBXFMMKIII5XHMSYB5BAFIZIIGT6I3XX7O7P6Z5FQXJZLZDL", "VAULT0": "CBTPAFFSXIALUH3RFEB2HGUZM6APGXMFLONAWEPVGFTEDWO6SDEHQ4ZP", "VAULT1": "CBS2C6QQS3GEMIQLAI725LTFBRRYAPYZA3ERMPZ2PSLAVO5TCJAVSA3A", "VAULT_HASH": "9899789b983236c6e4ac988223a535c3d06026c923896bd08fc5d342cb6cfd3b"};
export const LOG=[
{
"step": "Funder deposits 5,000 XLM",
"ok": true,
"err": "",
"tx": "https://stellar.expert/explorer/testnet/tx/d83e20ebc65352e8fa5c31b30454e867ef4305a0365a3fd7fd28d190fead48cd"
},
{
"step": "Programme Director proposes project #0 (3,000 XLM, 3 milestones)",
"ok": true,
"err": "",
"tx": "https://stellar.expert/explorer/testnet/tx/43b0034d42ea47631f19a879b3e9126c8899963b96937098820c268daa9226d4"
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
"step": "Finance Lead approves (2 of 3)",
"ok": true,
"err": "",
"tx": "https://stellar.expert/explorer/testnet/tx/2544c1a29c6416e1f904e7ed63213bac390c01754ced7030a6664090abd82dad"
},
{
"step": "Execute during 30s timelock",
"ok": false,
"err": "Error(Contract, #9)",
"tx": ""
},
{
"step": "Anyone executes after timelock: vault deployed + funded",
"ok": true,
"err": "",
"tx": "https://stellar.expert/explorer/testnet/tx/6cadfd0afbf0e2d890b8c89d70b911ca9a2532276736bd159cc7202b695d56f7"
},
{
"step": "Recipient submits M0 evidence (evidence hash)",
"ok": true,
"err": "",
"tx": "https://stellar.expert/explorer/testnet/tx/93bcc1efaf179d17e21da30bbe518a434fb186e7e2f9ef8b61fbf4e78f8a8e87"
},
{
"step": "Recipient tries to certify own work",
"ok": false,
"err": "Error(Contract, #15)",
"tx": ""
},
{
"step": "Reviewer 1 approves M0",
"ok": true,
"err": "",
"tx": "https://stellar.expert/explorer/testnet/tx/13d1e12ba203a81fc54f9b6920e4f458c501b65b7559ad933c8217e829d65a4d"
},
{
"step": "Reviewer 1 votes twice",
"ok": false,
"err": "Error(Contract, #16)",
"tx": ""
},
{
"step": "Reviewer 2 approves M0 (2 of 3: certified)",
"ok": true,
"err": "",
"tx": "https://stellar.expert/explorer/testnet/tx/827b59054607fa70d03af3ae9a00dbb86366ec206f30e2fd86b8937458fb7d03"
},
{
"step": "Release during 120s public challenge window",
"ok": false,
"err": "Error(Contract, #17)",
"tx": ""
},
{
"step": "Anyone releases M0: 900 XLM paid, 100 XLM retained",
"ok": true,
"err": "",
"tx": "https://stellar.expert/explorer/testnet/tx/d0dfa6951853389aad83a4a2b29510f6f7439fd4f43eb9f09bea7bef95bf169e"
},
{
"step": "Release M0 a second time",
"ok": false,
"err": "Error(Contract, #14)",
"tx": ""
},
{
"step": "Recipient submits M1 evidence (round 1)",
"ok": true,
"err": "",
"tx": "https://stellar.expert/explorer/testnet/tx/747e2649881cbc420bb228ce631ac524d325cf8b7765eff7d6f48b3263053def"
},
{
"step": "Reviewer 1 rejects M1 (reused photos)",
"ok": true,
"err": "",
"tx": "https://stellar.expert/explorer/testnet/tx/c0288b06567e1d95a42a66d90880f7374a3736f6448a3906cba0b07bee5e2250"
},
{
"step": "Reviewer 3 rejects M1: sent back to recipient",
"ok": true,
"err": "",
"tx": "https://stellar.expert/explorer/testnet/tx/df46a7bd317aba8df014e806fadbc862ff8873344fc27eda171eb3e18c445b56"
},
{
"step": "Recipient resubmits M1 (round 2)",
"ok": true,
"err": "",
"tx": "https://stellar.expert/explorer/testnet/tx/319c51bb93d66ac3e72d49e04d78cb0084a247c8e91f0c30a6c55dc7f7d5c0a1"
},
{
"step": "Reviewer 2 approves M1",
"ok": true,
"err": "",
"tx": "https://stellar.expert/explorer/testnet/tx/077c2ea5bdaf14e03afe734111e9885689e87928370b659914ec39c83a86342a"
},
{
"step": "Reviewer 3 approves M1: certified",
"ok": true,
"err": "",
"tx": "https://stellar.expert/explorer/testnet/tx/5521376de1ceec8625a12202b0475835d617dcc230f585ff0216a1f82c02531b"
},
{
"step": "Independent Auditor proposes voiding M1 (public complaint)",
"ok": true,
"err": "",
"tx": "https://stellar.expert/explorer/testnet/tx/2174dbd289c9ddf839dd0ed93c52424d38980ad72179cd072856c71e15584fc0"
},
{
"step": "Programme Director co-signs the void",
"ok": true,
"err": "",
"tx": "https://stellar.expert/explorer/testnet/tx/b95ca4ff5deb779cef4f73d7c1822ad5ddf4759feb3d611877c509019965c107"
},
{
"step": "Void executes inside challenge window: M1 back to Pending",
"ok": true,
"err": "",
"tx": "https://stellar.expert/explorer/testnet/tx/d99f8d53271c2d5c6e53b0098b11ecdf82d3b6fdca57fc15cc9d5248fcd4fd78"
},
{
"step": "Release voided M1",
"ok": false,
"err": "Error(Contract, #14)",
"tx": ""
},
{
"step": "Recipient resubmits M1 (round 3, extra evidence attached)",
"ok": true,
"err": "",
"tx": "https://stellar.expert/explorer/testnet/tx/fbfc49631d281e62e11e23fda9fa39100a46b254967c05b290443c6e8ce03d22"
},
{
"step": "Reviewer 1 approves M1",
"ok": true,
"err": "",
"tx": "https://stellar.expert/explorer/testnet/tx/d2859d531e9e647160bd27d69eb0f96cff05494283eaa166f47a9df97993aab3"
},
{
"step": "Reviewer 2 approves M1: certified",
"ok": true,
"err": "",
"tx": "https://stellar.expert/explorer/testnet/tx/7a5237543e4ab4ba79e6ae19361c929f37dcd368a540ad57e44a2882b04eca9b"
},
{
"step": "Recipient tries to skip ahead to M2",
"ok": false,
"err": "Error(Contract, #13)",
"tx": ""
},
{
"step": "Anyone releases M1: 1,350 XLM paid, 150 XLM retained",
"ok": true,
"err": "",
"tx": "https://stellar.expert/explorer/testnet/tx/551bc31628009aae7d9886497aef227ea7642afe8f5d4b873d57633df92d4d07"
},
{
"step": "Recipient submits M2 evidence",
"ok": true,
"err": "",
"tx": "https://stellar.expert/explorer/testnet/tx/e78c070221bf2dff3253c1a1a0eb9c165c080def96a5915422ad9e17149b9c25"
},
{
"step": "Reviewer 1 approves M2",
"ok": true,
"err": "",
"tx": "https://stellar.expert/explorer/testnet/tx/c39a31d129d0f8ebf9877f0c60f0f0c3f664da0cd86c8aaec7e93406b8de960d"
},
{
"step": "Reviewer 3 approves M2: certified",
"ok": true,
"err": "",
"tx": "https://stellar.expert/explorer/testnet/tx/967f626e4ce97f75427ca56eece8234cebb11eff0de97d8fdf3e2adf576726aa"
},
{
"step": "Anyone releases M2: project Completed (450 paid, 50 retained)",
"ok": true,
"err": "",
"tx": "https://stellar.expert/explorer/testnet/tx/e41832213ad944e0dbaf5c338a0af77e55dc0a24914d5bec895108fe7751ef9a"
},
{
"step": "Release retention during 90s defects-liability period",
"ok": false,
"err": "Error(Contract, #20)",
"tx": ""
},
{
"step": "Defects period over: 300 XLM retention released to recipient",
"ok": true,
"err": "",
"tx": "https://stellar.expert/explorer/testnet/tx/bda21a11e21eca38833b3624e4cbd0cca575af5a2726697c6cc2b5d7e39d1003"
},
{
"step": "Programme Director proposes project #1 (1,200 XLM)",
"ok": true,
"err": "",
"tx": "https://stellar.expert/explorer/testnet/tx/d320552d38ab4f4803edab5b58363eb1aca79066796c0738b7c1cc5594a28441"
},
{
"step": "Independent Auditor approves",
"ok": true,
"err": "",
"tx": "https://stellar.expert/explorer/testnet/tx/4d498a9005cfbc89066dfc13aa8d2c38f6277b767ff94f1deaa1ce0cf1bdaedd"
},
{
"step": "Council proposes too-big project (5,000 XLM)",
"ok": true,
"err": "",
"tx": "https://stellar.expert/explorer/testnet/tx/d1780e9af20148f0725e05cc37b142ceaf399f54f4c2dfec0f76b92566298d05"
},
{
"step": "Programme Director approves the oversized project",
"ok": true,
"err": "",
"tx": "https://stellar.expert/explorer/testnet/tx/708d63cb989253c6023ca33b3de4685a0570e84e4281c7cf14b076c9d343b025"
},
{
"step": "Execute project #1: vault deployed + funded",
"ok": true,
"err": "",
"tx": "https://stellar.expert/explorer/testnet/tx/139c73cfdf84049741cef8053a7d5f80e9a549d358b2b2655cb631fdf8d3e64c"
},
{
"step": "Execute oversized project: budget check",
"ok": false,
"err": "Error(Contract, #11)",
"tx": ""
},
{
"step": "Programme Director proposes cancelling project #1 (dispute)",
"ok": true,
"err": "",
"tx": "https://stellar.expert/explorer/testnet/tx/4726dc7b326699017b412fdeb2a5ffa1c89d3c8e6d607ac5e2fc76c6176a409a"
},
{
"step": "Finance Lead approves cancellation",
"ok": true,
"err": "",
"tx": "https://stellar.expert/explorer/testnet/tx/2fb9577d864e6edab6bfcd31e5cca8f50fa9219097ad87c7dd59f97d683af6e9"
},
{
"step": "Cancel executes: 1,200 XLM returned to treasury",
"ok": true,
"err": "",
"tx": "https://stellar.expert/explorer/testnet/tx/ace188f3cbd5a5d5e7ecdf322865b5552ed6742c556714a88f4ef6e5d4d38004"
}
];
