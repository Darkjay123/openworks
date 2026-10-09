//! # OpenWorks Treasury
//!
//! A public-works budget run by a council (for example a state ministry's
//! commissioner, accountant-general and an independent auditor). Funders
//! deposit stablecoins. The council approves projects through an on-chain
//! multisig with a timelock, and every approved project gets its own
//! `ProjectVault` contract, deployed and funded atomically by this treasury.
//!
//! Governance model:
//! * `propose` -> `approve` (M-of-N council signatures) -> timelock -> `execute`.
//! * The timelock starts when the threshold is reached, so the public sees
//!   every approved action before it takes effect.
//! * Proposals expire. Approvals from members removed later stop counting.
//! * Anyone can trigger `execute` once the conditions hold.
//!
//! Actions: create a project (deploy + fund + activate its vault), cancel a
//! project, void a milestone certification, forfeit retention for defects,
//! add or remove council members, change the threshold, pause new projects.
//!
//! There is no "send money to address" action. Treasury funds can only move
//! into a project vault, and a vault can only pay its contractor or refund
//! this treasury.
#![no_std]

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, panic_with_error, token,
    vec, Address, BytesN, Env, IntoVal, String, Val, Vec,
};

mod vault {
    soroban_sdk::contractimport!(file = "../../target/wasm32v1-none/release/project_vault.wasm");
}

pub const MAX_COUNCIL: u32 = 15;
const DAY_IN_LEDGERS: u32 = 17_280;
const TTL_THRESHOLD: u32 = 30 * DAY_IN_LEDGERS;
const TTL_EXTEND_TO: u32 = 180 * DAY_IN_LEDGERS;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    InvalidCouncil = 1,
    InvalidThreshold = 2,
    NotCouncilMember = 3,
    ProposalNotFound = 4,
    ProposalClosed = 5,
    ProposalExpired = 6,
    AlreadyApproved = 7,
    ThresholdNotMet = 8,
    TimelockActive = 9,
    InvalidAmount = 10,
    InsufficientBudget = 11,
    Paused = 12,
    AlreadyMember = 13,
    ProjectNotFound = 14,
    Overflow = 15,
    BalanceMismatch = 16,
    ChallengeTooShort = 17,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectSpec {
    pub contractor: Address,
    pub title: String,
    pub milestones: Vec<i128>,
    pub inspectors: Vec<Address>,
    pub inspector_threshold: u32,
    pub retention_bps: u32,
    pub challenge_secs: u64,
    pub defects_secs: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Action {
    CreateProject(ProjectSpec),
    CancelProject(u32),
    VoidMilestone(u32, u32),
    ForfeitRetention(u32),
    AddMember(Address),
    RemoveMember(Address),
    SetThreshold(u32),
    SetPaused(bool),
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProposalStatus {
    Open,
    Executed,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    pub id: u32,
    pub proposer: Address,
    pub action: Action,
    pub approvals: Vec<Address>,
    pub created_at: u64,
    /// When approvals first reached the threshold (0 = not yet). Timelock runs from here.
    pub approved_at: u64,
    pub status: ProposalStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub name: String,
    pub token: Address,
    pub vault_wasm: BytesN<32>,
    pub timelock_secs: u64,
    pub proposal_ttl_secs: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Budget {
    pub balance: i128,
    pub total_deposited: i128,
    pub total_committed: i128,
    pub project_count: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Config,
    Council,
    Threshold,
    Paused,
    ProposalCount,
    Proposal(u32),
    ProjectCount,
    Project(u32),
    TotalDeposited,
    TotalCommitted,
    Funder(Address),
}

// ---------------- events ----------------
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Deposited {
    #[topic]
    pub from: Address,
    pub amount: i128,
}
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposed {
    #[topic]
    pub id: u32,
    pub proposer: Address,
}
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Approved {
    #[topic]
    pub id: u32,
    pub signer: Address,
    pub approvals: u32,
}
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Executed {
    #[topic]
    pub id: u32,
}
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectCreated {
    #[topic]
    pub project_id: u32,
    pub vault: Address,
    pub contractor: Address,
    pub budget: i128,
}

#[contract]
pub struct Treasury;

fn bump(env: &Env) {
    env.storage().instance().extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
}
fn cfg(env: &Env) -> Config {
    env.storage().instance().get(&DataKey::Config).unwrap()
}
fn council(env: &Env) -> Vec<Address> {
    env.storage().instance().get(&DataKey::Council).unwrap()
}
fn threshold(env: &Env) -> u32 {
    env.storage().instance().get(&DataKey::Threshold).unwrap()
}
fn get_i128(env: &Env, k: &DataKey) -> i128 {
    env.storage().instance().get(k).unwrap_or(0)
}
fn get_u32(env: &Env, k: &DataKey) -> u32 {
    env.storage().instance().get(k).unwrap_or(0)
}
fn require_member(env: &Env, who: &Address) {
    if !council(env).contains(who) {
        panic_with_error!(env, Error::NotCouncilMember);
    }
}
fn load_proposal(env: &Env, id: u32) -> Proposal {
    let key = DataKey::Proposal(id);
    let p: Proposal = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| panic_with_error!(env, Error::ProposalNotFound));
    env.storage().persistent().extend_ttl(&key, TTL_THRESHOLD, TTL_EXTEND_TO);
    p
}
fn save_proposal(env: &Env, p: &Proposal) {
    let key = DataKey::Proposal(p.id);
    env.storage().persistent().set(&key, p);
    env.storage().persistent().extend_ttl(&key, TTL_THRESHOLD, TTL_EXTEND_TO);
}
fn project_addr(env: &Env, id: u32) -> Address {
    let key = DataKey::Project(id);
    let a: Address = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| panic_with_error!(env, Error::ProjectNotFound));
    env.storage().persistent().extend_ttl(&key, TTL_THRESHOLD, TTL_EXTEND_TO);
    a
}
/// Approvals that still count: signers who are council members right now.
fn live_approvals(env: &Env, p: &Proposal) -> u32 {
    let c = council(env);
    let mut n = 0u32;
    for a in p.approvals.iter() {
        if c.contains(&a) {
            n += 1;
        }
    }
    n
}
fn validate_council(env: &Env, members: &Vec<Address>, thr: u32) {
    let n = members.len();
    if n == 0 || n > MAX_COUNCIL {
        panic_with_error!(env, Error::InvalidCouncil);
    }
    for i in 0..n {
        for j in (i + 1)..n {
            if members.get_unchecked(i) == members.get_unchecked(j) {
                panic_with_error!(env, Error::InvalidCouncil);
            }
        }
    }
    if thr == 0 || thr > n {
        panic_with_error!(env, Error::InvalidThreshold);
    }
}

#[contractimpl]
impl Treasury {
    pub fn __constructor(
        env: Env,
        name: String,
        council: Vec<Address>,
        threshold: u32,
        token: Address,
        vault_wasm: BytesN<32>,
        timelock_secs: u64,
        proposal_ttl_secs: u64,
    ) {
        validate_council(&env, &council, threshold);
        if proposal_ttl_secs <= timelock_secs {
            panic_with_error!(&env, Error::InvalidThreshold);
        }
        let s = env.storage().instance();
        s.set(&DataKey::Config, &Config { name, token, vault_wasm, timelock_secs, proposal_ttl_secs });
        s.set(&DataKey::Council, &council);
        s.set(&DataKey::Threshold, &threshold);
        s.set(&DataKey::Paused, &false);
        bump(&env);
    }

    /// Anyone (a federal allocation, a donor, a bond) can fund the treasury.
    pub fn deposit(env: Env, from: Address, amount: i128) {
        from.require_auth();
        if amount <= 0 {
            panic_with_error!(&env, Error::InvalidAmount);
        }
        let c = cfg(&env);
        let tok = token::Client::new(&env, &c.token);
        let me = env.current_contract_address();
        let before = tok.balance(&me);
        tok.transfer(&from, &me, &amount);
        if tok.balance(&me) - before != amount {
            panic_with_error!(&env, Error::BalanceMismatch);
        }
        let total = get_i128(&env, &DataKey::TotalDeposited)
            .checked_add(amount)
            .unwrap_or_else(|| panic_with_error!(&env, Error::Overflow));
        env.storage().instance().set(&DataKey::TotalDeposited, &total);
        let fk = DataKey::Funder(from.clone());
        let f: i128 = env.storage().persistent().get(&fk).unwrap_or(0);
        env.storage().persistent().set(&fk, &(f + amount));
        env.storage().persistent().extend_ttl(&fk, TTL_THRESHOLD, TTL_EXTEND_TO);
        bump(&env);
        Deposited { from, amount }.publish(&env);
    }

    pub fn propose(env: Env, proposer: Address, action: Action) -> u32 {
        proposer.require_auth();
        require_member(&env, &proposer);
        let id = get_u32(&env, &DataKey::ProposalCount);
        env.storage().instance().set(&DataKey::ProposalCount, &(id + 1));
        let now = env.ledger().timestamp();
        let mut p = Proposal {
            id,
            proposer: proposer.clone(),
            action,
            approvals: vec![&env, proposer.clone()],
            created_at: now,
            approved_at: 0,
            status: ProposalStatus::Open,
        };
        if live_approvals(&env, &p) >= threshold(&env) {
            p.approved_at = now;
        }
        save_proposal(&env, &p);
        bump(&env);
        Proposed { id, proposer }.publish(&env);
        id
    }

    pub fn approve(env: Env, signer: Address, id: u32) {
        signer.require_auth();
        require_member(&env, &signer);
        let mut p = load_proposal(&env, id);
        if p.status != ProposalStatus::Open {
            panic_with_error!(&env, Error::ProposalClosed);
        }
        let now = env.ledger().timestamp();
        if now >= p.created_at.saturating_add(cfg(&env).proposal_ttl_secs) {
            panic_with_error!(&env, Error::ProposalExpired);
        }
        if p.approvals.contains(&signer) {
            panic_with_error!(&env, Error::AlreadyApproved);
        }
        p.approvals.push_back(signer.clone());
        let live = live_approvals(&env, &p);
        if p.approved_at == 0 && live >= threshold(&env) {
            p.approved_at = now;
        }
        save_proposal(&env, &p);
        bump(&env);
        Approved { id, signer, approvals: live }.publish(&env);
    }

    /// Permissionless once the threshold is met and the timelock has passed.
    pub fn execute(env: Env, id: u32) {
        let c = cfg(&env);
        let mut p = load_proposal(&env, id);
        if p.status != ProposalStatus::Open {
            panic_with_error!(&env, Error::ProposalClosed);
        }
        let now = env.ledger().timestamp();
        if now >= p.created_at.saturating_add(c.proposal_ttl_secs) {
            panic_with_error!(&env, Error::ProposalExpired);
        }
        if p.approved_at == 0 || live_approvals(&env, &p) < threshold(&env) {
            panic_with_error!(&env, Error::ThresholdNotMet);
        }
        if now < p.approved_at.saturating_add(c.timelock_secs) {
            panic_with_error!(&env, Error::TimelockActive);
        }
        // effects first: a proposal can only ever run once
        p.status = ProposalStatus::Executed;
        save_proposal(&env, &p);
        bump(&env);

        match p.action.clone() {
            Action::CreateProject(spec) => Self::create_project(&env, &c, spec),
            Action::CancelProject(pid) => {
                vault::Client::new(&env, &project_addr(&env, pid)).cancel();
            }
            Action::VoidMilestone(pid, idx) => {
                vault::Client::new(&env, &project_addr(&env, pid)).void_certification(&idx);
            }
            Action::ForfeitRetention(pid) => {
                vault::Client::new(&env, &project_addr(&env, pid)).forfeit_retention();
            }
            Action::AddMember(a) => {
                let mut m = council(&env);
                if m.contains(&a) {
                    panic_with_error!(&env, Error::AlreadyMember);
                }
                m.push_back(a);
                validate_council(&env, &m, threshold(&env));
                env.storage().instance().set(&DataKey::Council, &m);
            }
            Action::RemoveMember(a) => {
                let m = council(&env);
                let idx = m
                    .first_index_of(&a)
                    .unwrap_or_else(|| panic_with_error!(&env, Error::NotCouncilMember));
                let mut m2 = m.clone();
                m2.remove(idx);
                validate_council(&env, &m2, threshold(&env));
                env.storage().instance().set(&DataKey::Council, &m2);
            }
            Action::SetThreshold(t) => {
                validate_council(&env, &council(&env), t);
                env.storage().instance().set(&DataKey::Threshold, &t);
            }
            Action::SetPaused(b) => env.storage().instance().set(&DataKey::Paused, &b),
        }
        Executed { id }.publish(&env);
    }

    fn create_project(env: &Env, c: &Config, spec: ProjectSpec) {
        let paused: bool = env.storage().instance().get(&DataKey::Paused).unwrap_or(false);
        if paused {
            panic_with_error!(env, Error::Paused);
        }
        // The council must be able to void a bad certification: its own
        // propose -> approve -> timelock cycle has to fit inside the window.
        if spec.challenge_secs <= c.timelock_secs {
            panic_with_error!(env, Error::ChallengeTooShort);
        }
        let mut total: i128 = 0;
        for a in spec.milestones.iter() {
            if a <= 0 {
                panic_with_error!(env, Error::InvalidAmount);
            }
            total = total
                .checked_add(a)
                .unwrap_or_else(|| panic_with_error!(env, Error::Overflow));
        }
        let tok = token::Client::new(env, &c.token);
        let me = env.current_contract_address();
        if tok.balance(&me) < total {
            panic_with_error!(env, Error::InsufficientBudget);
        }
        let pid = get_u32(env, &DataKey::ProjectCount);
        let mut salt = [0u8; 32];
        salt[28..].copy_from_slice(&pid.to_be_bytes());
        let args: Vec<Val> = vec![
            env,
            me.into_val(env),
            c.token.into_val(env),
            spec.contractor.into_val(env),
            spec.title.into_val(env),
            spec.milestones.into_val(env),
            spec.inspectors.into_val(env),
            spec.inspector_threshold.into_val(env),
            spec.retention_bps.into_val(env),
            spec.challenge_secs.into_val(env),
            spec.defects_secs.into_val(env),
        ];
        let vault_addr = env
            .deployer()
            .with_current_contract(BytesN::from_array(env, &salt))
            .deploy_v2(c.vault_wasm.clone(), args);

        env.storage().instance().set(&DataKey::ProjectCount, &(pid + 1));
        let key = DataKey::Project(pid);
        env.storage().persistent().set(&key, &vault_addr);
        env.storage().persistent().extend_ttl(&key, TTL_THRESHOLD, TTL_EXTEND_TO);
        let committed = get_i128(env, &DataKey::TotalCommitted)
            .checked_add(total)
            .unwrap_or_else(|| panic_with_error!(env, Error::Overflow));
        env.storage().instance().set(&DataKey::TotalCommitted, &committed);

        tok.transfer(&me, &vault_addr, &total);
        vault::Client::new(env, &vault_addr).activate();
        ProjectCreated { project_id: pid, vault: vault_addr, contractor: spec.contractor, budget: total }
            .publish(env);
    }

    // ---------------- views ----------------
    pub fn config(env: Env) -> Config {
        cfg(&env)
    }
    pub fn council(env: Env) -> Vec<Address> {
        council(&env)
    }
    pub fn threshold(env: Env) -> u32 {
        threshold(&env)
    }
    pub fn paused(env: Env) -> bool {
        env.storage().instance().get(&DataKey::Paused).unwrap_or(false)
    }
    pub fn proposal(env: Env, id: u32) -> Proposal {
        load_proposal(&env, id)
    }
    pub fn proposal_count(env: Env) -> u32 {
        get_u32(&env, &DataKey::ProposalCount)
    }
    pub fn project(env: Env, id: u32) -> Address {
        project_addr(&env, id)
    }
    pub fn projects(env: Env) -> Vec<Address> {
        let n = get_u32(&env, &DataKey::ProjectCount);
        let mut out = Vec::new(&env);
        for i in 0..n {
            out.push_back(project_addr(&env, i));
        }
        out
    }
    pub fn funded_by(env: Env, funder: Address) -> i128 {
        env.storage().persistent().get(&DataKey::Funder(funder)).unwrap_or(0)
    }
    pub fn budget(env: Env) -> Budget {
        let c = cfg(&env);
        Budget {
            balance: token::Client::new(&env, &c.token).balance(&env.current_contract_address()),
            total_deposited: get_i128(&env, &DataKey::TotalDeposited),
            total_committed: get_i128(&env, &DataKey::TotalCommitted),
            project_count: get_u32(&env, &DataKey::ProjectCount),
        }
    }
}

#[cfg(test)]
mod test;
