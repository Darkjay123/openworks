import "./style.css";
import { rpc, Keypair, Networks, Contract, TransactionBuilder, nativeToScVal, scValToNative, BASE_FEE, Account } from "@stellar/stellar-sdk";
import { ENV, LOG } from "./data.js";

const server = new rpc.Server("https://soroban-testnet.stellar.org");
const EXP = "https://stellar.expert/explorer/testnet";
const NAMES = {
  GBBWKXQA5X7TO3BRJGIBBDE3MRCMWK7KJDEDXQHKLOICXACGWQYMJD3T: "Programme Director",
  GB4B7J2VBHIYO4VHWEKI47UXZF4EKJVRGWFSEW3DEAPSJ7F437M3I3JK: "Finance Lead",
  GBT4NZF4FCFM7M4ZS4VMTMRKPOCXE6TTIPQZ6LZ2TL5S575XW4N3TWVI: "Independent Auditor",
  GDNT36DFG3SWSJAYONBKNU3ABF3LFL7PDUCORE724TEK63RJSJDUAKZR: "Recipient",
  GBL4KXTWNRU2DOBMZQTLJDWONTBDSKQQWF2UPMBLISW3VBQAOSXVSAZY: "Reviewer 1",
  GCKAPP3KM6PIPI6DXLNXHW2NEMYJVZWLRJ7KK53OTNEQUW5LYRAZXIUG: "Reviewer 2",
  GDEDLE7CFR5GXHMCQSDBK55B6HCSYRIEPPAN3SLBBH43WA4HL7ZL46PP: "Reviewer 3",
};
const VAULT_ERR = {11:"InsufficientFunds",13:"OutOfOrder: milestones go strictly in order",14:"WrongState: nothing pays twice",15:"NotInspector: the recipient can't review their own work",16:"AlreadyVoted: one vote per reviewer per round",17:"ChallengeWindowOpen: public challenge window still running",20:"DefectsPeriodOpen: retention is still held"};
const TREAS_ERR = {3:"NotCouncilMember: outsiders can't sign",8:"ThresholdNotMet: needs 2 of 3 council signatures",9:"TimelockActive: approved actions wait before running",11:"InsufficientBudget: can't commit money the treasury doesn't hold"};

async function read(id, method, ...args) {
  const src = new Account(Keypair.random().publicKey(), "0");
  const tx = new TransactionBuilder(src, { fee: BASE_FEE, networkPassphrase: Networks.TESTNET })
    .addOperation(new Contract(id).call(method, ...args)).setTimeout(30).build();
  const sim = await server.simulateTransaction(tx);
  if (rpc.Api.isSimulationError(sim)) throw new Error(sim.error);
  return scValToNative(sim.result.retval);
}
const u32 = (n) => nativeToScVal(n, { type: "u32" });
const xlm = (v) => (Number(v) / 1e7).toLocaleString(undefined, { maximumFractionDigits: 0 });
const tag = (v) => (Array.isArray(v) ? v[0] : typeof v === "object" && v ? Object.keys(v)[0] : String(v));
const short = (a) => (a ? a.slice(0, 5) + "…" + a.slice(-5) : "");
const who = (a) => NAMES[a] || short(a);
const esc = (s) => String(s).replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));
const hex = (b) => (b instanceof Uint8Array || Array.isArray(b) ? Array.from(b).map((x) => x.toString(16).padStart(2, "0")).join("") : String(b));
const dur = (s) => { s = Number(s); return s >= 86400 ? `${s / 86400} days` : s >= 3600 ? `${s / 3600} h` : `${s}s`; };
const $ = (id) => document.getElementById(id);

function describe(action) {
  const k = tag(action);
  const v = Array.isArray(action) ? action.slice(1) : action[k];
  const a = Array.isArray(v) && v.length === 1 ? v[0] : v;
  switch (k) {
    case "CreateProject": {
      const tot = a.milestones.reduce((s, x) => s + Number(x), 0);
      return `Create project “${esc(a.title)}” · ${xlm(tot)} XLM over ${a.milestones.length} milestone${a.milestones.length === 1 ? "" : "s"}`;
    }
    case "CancelProject": return `Cancel project #${a}`;
    case "VoidMilestone": return `Void certification of milestone ${Number(v[1]) + 1} on project #${v[0]}`;
    case "ForfeitRetention": return `Forfeit retention on project #${a}`;
    default: return `${k} ${esc(JSON.stringify(a))}`;
  }
}

async function loadTreasury() {
  const T = ENV.TREASURY;
  const [cfg, council, thr, budget, pc] = await Promise.all([read(T, "config"), read(T, "council"), read(T, "threshold"), read(T, "budget"), read(T, "proposal_count")]);
  $("t-name").textContent = "Demo programme: " + cfg.name.replace(" (testnet demo)", "");
  $("t-thr").textContent = `${thr} of ${council.length} to approve`;
  $("t-tl").textContent = dur(cfg.timelock_secs);
  $("t-ttl").textContent = dur(cfg.proposal_ttl_secs);
  $("t-council").innerHTML = council.map((a) => `<div class="row"><span class="av">${who(a)[0]}</span><span>${who(a)}</span><a class="mono" href="${EXP}/account/${a}" target="_blank" rel="noopener">${short(a)}</a></div>`).join("");
  $("t-link").textContent = T; $("t-link").href = `${EXP}/contract/${T}`;
  $("k-dep").textContent = xlm(budget.total_deposited);
  $("k-com").textContent = xlm(budget.total_committed);
  $("k-bal").textContent = xlm(budget.balance);
  $("t-pc").textContent = `${pc} total`;
  const props = await Promise.all([...Array(Number(pc)).keys()].map((i) => read(T, "proposal", u32(i))));
  $("t-props").innerHTML = props.map((p) => {
    const st = tag(p.status);
    const k = tag(p.action), a = Array.isArray(p.action) ? p.action[1] : p.action[k];
    const over = k === "CreateProject" && a.milestones.reduce((s, x) => s + Number(x), 0) > Number(budget.balance);
    const label = st === "Executed" ? "Executed" : over ? "Refused: over budget" : Number(p.approved_at) ? "Approved, not run" : "Open";
    const cls = st === "Executed" ? "ok" : over ? "bad" : Number(p.approved_at) ? "warn" : "muted";
    return `<div class="prop"><div class="pid">#${p.id}</div><div class="pbody"><div>${describe(p.action)}</div><div class="small">Signed by ${p.approvals.map(who).join(" + ")}</div></div><span class="badge ${cls}">${label}</span></div>`;
  }).join("");
  return Number(budget.project_count);
}

const STEPS = ["Pending", "Submitted", "Certified", "Released"];
async function loadProject(i) {
  const V = await read(ENV.TREASURY, "project", u32(i));
  const [cfg, sum, ms] = await Promise.all([read(V, "config"), read(V, "summary"), read(V, "milestones")]);
  const st = tag(sum.status);
  const paidPct = (Number(sum.paid) / Number(sum.total)) * 100;
  const retPct = (Number(sum.retained) / Number(sum.total)) * 100;
  const rows = ms.map((m, j) => {
    const s = tag(m.state);
    const idx = STEPS.indexOf(s);
    const steps = STEPS.map((x, k) => `<span class="step ${k <= idx ? "on" : ""} ${k === idx ? "cur" : ""}">${x}</span>`).join("");
    const ev = Number(m.round) ? `<div class="ev mono" title="SHA-256 of the evidence the recipient submitted">evidence ${hex(m.evidence_hash).slice(0, 16)}… · ${esc(m.evidence_uri)}</div>` : `<div class="ev small">No evidence submitted yet</div>`;
    return `<div class="ms"><div class="msh"><b>Milestone ${j + 1}</b><span>${xlm(m.amount)} XLM</span><span class="small">round ${m.round} · ${m.approvals} approve / ${m.rejections} reject</span></div><div class="steps">${steps}</div>${ev}</div>`;
  }).join("");
  const cls = st === "Completed" ? "ok" : st === "Cancelled" ? "bad" : "warn";
  return { paid: Number(sum.paid), html: `<article class="proj">
    <div class="projh"><div><div class="small">Project #${i}</div><h3>${esc(cfg.title)}</h3></div><span class="badge ${cls}">${st}</span></div>
    <div class="facts"><span>Budget <b>${xlm(cfg.total)} XLM</b></span><span>Recipient <b>${who(cfg.contractor)}</b></span><span>Reviewers <b>${cfg.inspector_threshold} of ${cfg.inspectors.length}</b></span><span>Retention <b>${cfg.retention_bps / 100}%</b></span><span>Challenge <b>${dur(cfg.challenge_secs)}</b></span><span>Defects period <b>${dur(cfg.defects_secs)}</b></span></div>
    <div class="bar"><i class="paid" style="width:${paidPct}%"></i><i class="ret" style="width:${retPct}%"></i></div>
    <div class="legend small"><span><i class="paid"></i>Paid ${xlm(sum.paid)}</span><span><i class="ret"></i>${st === "Completed" && Number(sum.retained) === 0 ? `Retention released after defects period: ${xlm(Number(cfg.total) - Number(sum.paid))}` : `Retained ${xlm(sum.retained)}`}</span><span>Held in vault ${xlm(sum.balance)}</span>${st === "Cancelled" ? "<span>Unspent budget returned to treasury</span>" : ""}</div>
    <div class="mss">${rows}</div>
    <div class="small mono addr">Vault <a href="${EXP}/contract/${V}" target="_blank" rel="noopener">${V}</a></div>
  </article>` };
}

function renderLog() {
  $("log").innerHTML = LOG.map((r) => {
    const code = (r.err.match(/#(\d+)/) || [])[1];
    const isTreasury = /execute|approve|propose|Non-member|Council|council|Execute|budget/i.test(r.step) && !/release/i.test(r.step);
    const why = code ? ((isTreasury ? TREAS_ERR : VAULT_ERR)[code] || (VAULT_ERR[code] || TREAS_ERR[code]) || r.err) : "";
    return `<div class="lrow ${r.ok ? "ok" : "bad"}"><span class="ico">${r.ok ? "✓" : "✕"}</span><span class="lstep">${esc(r.step)}</span>${r.ok && r.tx ? `<a class="mono" href="${r.tx}" target="_blank" rel="noopener">tx ${r.tx.split("/").pop().slice(0, 8)}… ↗</a>` : `<span class="why">${esc(why)}</span>`}</div>`;
  }).join("");
}

async function main() {
  renderLog();
  try {
    const n = await loadTreasury();
    const ps = await Promise.all([...Array(n).keys()].map(loadProject));
    $("projects-list").innerHTML = ps.map((p) => p.html).join("");
    $("k-paid").textContent = xlm(ps.reduce((s, p) => s + p.paid, 0) + 0);
  } catch (e) {
    $("t-name").textContent = "Couldn't reach Stellar testnet right now. Refresh in a moment.";
    console.error(e);
  }
}
main();
