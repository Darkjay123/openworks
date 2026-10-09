//! # OpenWorks Project Vault
//!
//! One vault per public project (a road, a borehole, a clinic). It is deployed
//! and funded by an OpenWorks Treasury, and it holds the full project budget
//! until each milestone is independently certified.
//!
//! Lifecycle of a milestone:
//! `Pending -> Submitted -> Certified -> (challenge window) -> Released`
//!
//! * The contractor submits evidence (a SHA-256 hash of the site report and
//!   photos, plus a URI where the files live). Milestones go strictly in order.
//! * Independent inspectors vote. `k` approvals certify the milestone. Enough
//!   rejections that `k` can no longer be reached send it back to `Pending`.
//! * After certification there is a public challenge window. During it the
//!   treasury council can void the certification. After it, anyone can
//!   trigger the release, and the money can only go to the contractor.
//! * A retention percentage (standard in public works contracts) is held back
//!   from every payment until a defects-liability period after completion.
//!   The council can forfeit it for defects inside that period; otherwise
//!   anyone can release it to the contractor.
//! * The treasury council can cancel the project. Unspent funds return to the
//!   treasury, never to any individual.
//!
//! Security properties (each covered by tests):
//! * Atomic set-up through `__constructor`: no front-runnable `init`.
//! * Role checks with `require_auth()` on every state change.
//! * Strict per-milestone state machine: nothing is paid twice.
//! * Per-round vote records: an inspector votes once per submission.
//! * Effects before interactions; checked arithmetic; overflow checks in release.
//! * Activation verifies the vault's real token balance.
//! * Bounded milestone and inspector counts; per-milestone persistent storage.
//! * No admin key and no sweep: funds only ever reach the contractor or the
//!   treasury that funded the project.
#![no_std]

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, panic_with_error, token,
    Address, BytesN, Env, String, Vec,
};

pub const MAX_MILESTONES: u32 = 24;
pub const MAX_INSPECTORS: u32 = 9;
pub const MAX_RETENTION_BPS: u32 = 2_000; // 20%
const BPS: i128 = 10_000;

const DAY_IN_LEDGERS: u32 = 17_280;
const TTL_THRESHOLD: u32 = 30 * DAY_IN_LEDGERS;
const TTL_EXTEND_TO: u32 = 180 * DAY_IN_LEDGERS;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    InvalidParties = 1,
    NoMilestones = 2,
    TooManyMilestones = 3,
    InvalidAmount = 4,
    InvalidInspectors = 5,
    InvalidThreshold = 6,
    InvalidRetention = 7,
    Overflow = 8,
    NotActive = 9,
    AlreadyActivated = 10,
    InsufficientFunds = 11,
    MilestoneNotFound = 12,
    OutOfOrder = 13,
    WrongState = 14,
    NotInspector = 15,
    AlreadyVoted = 16,
    ChallengeWindowOpen = 17,
    ChallengeWindowClosed = 18,
    NothingRetained = 19,
    DefectsPeriodOpen = 20,
    DefectsPeriodClosed = 21,
    NotFinished = 22,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Config,
    Status,
    Current,
    Paid,
    Retained,
    FinishedAt,
    Milestone(u32),
    Vote(u32, u32, Address),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub treasury: Address,
    pub token: Address,
    pub contractor: Address,
    pub title: String,
    pub inspectors: Vec<Address>,
    pub inspector_threshold: u32,
    pub retention_bps: u32,
    pub challenge_secs: u64,
    pub defects_secs: u64,
    pub total: i128,
    pub milestone_count: u32,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectStatus {
    AwaitingFunds,
    Active,
    Completed,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MilestoneState {
    Pending,
    Submitted,
    Certified,
    Released,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Milestone {
    pub amount: i128,
    pub state: MilestoneState,
    /// Submission round; bumps on every (re)submission so votes reset.
    pub round: u32,
    pub approvals: u32,
    pub rejections: u32,
    pub evidence_hash: BytesN<32>,
    pub evidence_uri: String,
    pub certified_at: u64,
    pub released_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Summary {
    pub status: ProjectStatus,
    pub total: i128,
    pub paid: i128,
    pub retained: i128,
    pub balance: i128,
    pub current: u32,
    pub milestone_count: u32,
    pub finished_at: u64,
}

// ---------------- events ----------------
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Activated {
    pub total: i128,
}
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Submitted {
    #[topic]
    pub index: u32,
    pub round: u32,
    pub evidence_hash: BytesN<32>,
}
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Attested {
    #[topic]
    pub index: u32,
    #[topic]
    pub inspector: Address,
    pub approve: bool,
}
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Certified {
    #[topic]
    pub index: u32,
    pub releasable_at: u64,
}
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rejected {
    #[topic]
    pub index: u32,
    pub round: u32,
}
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Voided {
    #[topic]
    pub index: u32,
}
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Released {
    #[topic]
    pub index: u32,
    pub paid: i128,
    pub retained: i128,
}
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetentionReleased {
    pub amount: i128,
}
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetentionForfeited {
    pub amount: i128,
}
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Cancelled {
    pub refunded: i128,
}

#[contract]
pub struct ProjectVault;

fn bump(env: &Env) {
    env.storage().instance().extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
}
fn cfg(env: &Env) -> Config {
    env.storage().instance().get(&DataKey::Config).unwrap()
}
fn status(env: &Env) -> ProjectStatus {
    env.storage().instance().get(&DataKey::Status).unwrap()
}
fn set_status(env: &Env, s: ProjectStatus) {
    env.storage().instance().set(&DataKey::Status, &s);
}
fn get_i128(env: &Env, k: &DataKey) -> i128 {
    env.storage().instance().get(k).unwrap_or(0)
}
fn load_ms(env: &Env, i: u32) -> Milestone {
    let key = DataKey::Milestone(i);
    let m: Milestone = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| panic_with_error!(env, Error::MilestoneNotFound));
    env.storage().persistent().extend_ttl(&key, TTL_THRESHOLD, TTL_EXTEND_TO);
    m
}
fn save_ms(env: &Env, i: u32, m: &Milestone) {
    let key = DataKey::Milestone(i);
    env.storage().persistent().set(&key, m);
    env.storage().persistent().extend_ttl(&key, TTL_THRESHOLD, TTL_EXTEND_TO);
}
fn require_active(env: &Env) {
    if status(env) != ProjectStatus::Active {
        panic_with_error!(env, Error::NotActive);
    }
}
fn balance(env: &Env, c: &Config) -> i128 {
    token::Client::new(env, &c.token).balance(&env.current_contract_address())
}

#[contractimpl]
impl ProjectVault {
    #[allow(clippy::too_many_arguments)]
    pub fn __constructor(
        env: Env,
        treasury: Address,
        token: Address,
        contractor: Address,
        title: String,
        milestones: Vec<i128>,
        inspectors: Vec<Address>,
        inspector_threshold: u32,
        retention_bps: u32,
        challenge_secs: u64,
        defects_secs: u64,
    ) {
        if contractor == treasury || token == treasury || token == contractor {
            panic_with_error!(&env, Error::InvalidParties);
        }
        let n = milestones.len();
        if n == 0 {
            panic_with_error!(&env, Error::NoMilestones);
        }
        if n > MAX_MILESTONES {
            panic_with_error!(&env, Error::TooManyMilestones);
        }
        let k = inspectors.len();
        if k == 0 || k > MAX_INSPECTORS {
            panic_with_error!(&env, Error::InvalidInspectors);
        }
        for i in 0..k {
            let a = inspectors.get_unchecked(i);
            if a == contractor || a == treasury {
                panic_with_error!(&env, Error::InvalidInspectors);
            }
            for j in (i + 1)..k {
                if a == inspectors.get_unchecked(j) {
                    panic_with_error!(&env, Error::InvalidInspectors);
                }
            }
        }
        if inspector_threshold == 0 || inspector_threshold > k {
            panic_with_error!(&env, Error::InvalidThreshold);
        }
        if retention_bps > MAX_RETENTION_BPS {
            panic_with_error!(&env, Error::InvalidRetention);
        }
        let empty_hash = BytesN::from_array(&env, &[0u8; 32]);
        let empty_uri = String::from_str(&env, "");
        let mut total: i128 = 0;
        for i in 0..n {
            let amount = milestones.get_unchecked(i);
            if amount <= 0 {
                panic_with_error!(&env, Error::InvalidAmount);
            }
            total = total
                .checked_add(amount)
                .unwrap_or_else(|| panic_with_error!(&env, Error::Overflow));
            save_ms(
                &env,
                i,
                &Milestone {
                    amount,
                    state: MilestoneState::Pending,
                    round: 0,
                    approvals: 0,
                    rejections: 0,
                    evidence_hash: empty_hash.clone(),
                    evidence_uri: empty_uri.clone(),
                    certified_at: 0,
                    released_at: 0,
                },
            );
        }
        let c = Config {
            treasury,
            token,
            contractor,
            title,
            inspectors,
            inspector_threshold,
            retention_bps,
            challenge_secs,
            defects_secs,
            total,
            milestone_count: n,
        };
        let s = env.storage().instance();
        s.set(&DataKey::Config, &c);
        s.set(&DataKey::Status, &ProjectStatus::AwaitingFunds);
        s.set(&DataKey::Current, &0u32);
        s.set(&DataKey::Paid, &0i128);
        s.set(&DataKey::Retained, &0i128);
        s.set(&DataKey::FinishedAt, &0u64);
        bump(&env);
    }

    /// Called by the treasury right after it transfers the budget in.
    pub fn activate(env: Env) {
        let c = cfg(&env);
        c.treasury.require_auth();
        if status(&env) != ProjectStatus::AwaitingFunds {
            panic_with_error!(&env, Error::AlreadyActivated);
        }
        if balance(&env, &c) < c.total {
            panic_with_error!(&env, Error::InsufficientFunds);
        }
        set_status(&env, ProjectStatus::Active);
        bump(&env);
        Activated { total: c.total }.publish(&env);
    }

    pub fn submit(env: Env, index: u32, evidence_hash: BytesN<32>, evidence_uri: String) {
        let c = cfg(&env);
        c.contractor.require_auth();
        require_active(&env);
        let current: u32 = env.storage().instance().get(&DataKey::Current).unwrap();
        if index >= c.milestone_count {
            panic_with_error!(&env, Error::MilestoneNotFound);
        }
        if index != current {
            panic_with_error!(&env, Error::OutOfOrder);
        }
        let mut m = load_ms(&env, index);
        if m.state != MilestoneState::Pending {
            panic_with_error!(&env, Error::WrongState);
        }
        m.state = MilestoneState::Submitted;
        m.round = m.round.checked_add(1).unwrap_or_else(|| panic_with_error!(&env, Error::Overflow));
        m.approvals = 0;
        m.rejections = 0;
        m.evidence_hash = evidence_hash.clone();
        m.evidence_uri = evidence_uri;
        save_ms(&env, index, &m);
        bump(&env);
        Submitted { index, round: m.round, evidence_hash }.publish(&env);
    }

    pub fn attest(env: Env, inspector: Address, index: u32, approve: bool) {
        inspector.require_auth();
        let c = cfg(&env);
        require_active(&env);
        if !c.inspectors.contains(&inspector) {
            panic_with_error!(&env, Error::NotInspector);
        }
        let mut m = load_ms(&env, index);
        if m.state != MilestoneState::Submitted {
            panic_with_error!(&env, Error::WrongState);
        }
        let vote_key = DataKey::Vote(index, m.round, inspector.clone());
        if env.storage().persistent().has(&vote_key) {
            panic_with_error!(&env, Error::AlreadyVoted);
        }
        env.storage().persistent().set(&vote_key, &approve);
        env.storage().persistent().extend_ttl(&vote_key, TTL_THRESHOLD, TTL_EXTEND_TO);
        Attested { index, inspector, approve }.publish(&env);

        let k = c.inspectors.len();
        if approve {
            m.approvals += 1;
            if m.approvals >= c.inspector_threshold {
                m.state = MilestoneState::Certified;
                m.certified_at = env.ledger().timestamp();
                let releasable_at = m
                    .certified_at
                    .checked_add(c.challenge_secs)
                    .unwrap_or_else(|| panic_with_error!(&env, Error::Overflow));
                Certified { index, releasable_at }.publish(&env);
            }
        } else {
            m.rejections += 1;
            // Threshold can no longer be reached: send back to the contractor.
            if m.rejections > k - c.inspector_threshold {
                m.state = MilestoneState::Pending;
                Rejected { index, round: m.round }.publish(&env);
            }
        }
        save_ms(&env, index, &m);
        bump(&env);
    }

    /// Council challenge: undo a certification during the challenge window.
    pub fn void_certification(env: Env, index: u32) {
        let c = cfg(&env);
        c.treasury.require_auth();
        require_active(&env);
        let mut m = load_ms(&env, index);
        if m.state != MilestoneState::Certified {
            panic_with_error!(&env, Error::WrongState);
        }
        let now = env.ledger().timestamp();
        if now >= m.certified_at.saturating_add(c.challenge_secs) {
            panic_with_error!(&env, Error::ChallengeWindowClosed);
        }
        m.state = MilestoneState::Pending;
        m.certified_at = 0;
        save_ms(&env, index, &m);
        bump(&env);
        Voided { index }.publish(&env);
    }

    /// Permissionless: anyone can trigger, but money only goes to the contractor.
    pub fn release(env: Env, index: u32) {
        let c = cfg(&env);
        require_active(&env);
        let mut m = load_ms(&env, index);
        if m.state != MilestoneState::Certified {
            panic_with_error!(&env, Error::WrongState);
        }
        let now = env.ledger().timestamp();
        if now < m.certified_at.saturating_add(c.challenge_secs) {
            panic_with_error!(&env, Error::ChallengeWindowOpen);
        }
        let retention = m
            .amount
            .checked_mul(c.retention_bps as i128)
            .unwrap_or_else(|| panic_with_error!(&env, Error::Overflow))
            / BPS;
        let payout = m.amount - retention;

        // effects
        m.state = MilestoneState::Released;
        m.released_at = now;
        save_ms(&env, index, &m);
        let s = env.storage().instance();
        let paid = get_i128(&env, &DataKey::Paid)
            .checked_add(payout)
            .unwrap_or_else(|| panic_with_error!(&env, Error::Overflow));
        let retained = get_i128(&env, &DataKey::Retained)
            .checked_add(retention)
            .unwrap_or_else(|| panic_with_error!(&env, Error::Overflow));
        s.set(&DataKey::Paid, &paid);
        s.set(&DataKey::Retained, &retained);
        let next = index + 1;
        s.set(&DataKey::Current, &next);
        if next == c.milestone_count {
            set_status(&env, ProjectStatus::Completed);
            s.set(&DataKey::FinishedAt, &now);
        }
        bump(&env);

        // interaction
        if payout > 0 {
            token::Client::new(&env, &c.token).transfer(
                &env.current_contract_address(),
                &c.contractor,
                &payout,
            );
        }
        Released { index, paid: payout, retained: retention }.publish(&env);
    }

    /// Permissionless once the defects-liability period has passed.
    pub fn release_retention(env: Env) {
        let c = cfg(&env);
        let st = status(&env);
        if st != ProjectStatus::Completed && st != ProjectStatus::Cancelled {
            panic_with_error!(&env, Error::NotFinished);
        }
        let finished: u64 = env.storage().instance().get(&DataKey::FinishedAt).unwrap();
        if env.ledger().timestamp() < finished.saturating_add(c.defects_secs) {
            panic_with_error!(&env, Error::DefectsPeriodOpen);
        }
        let retained = get_i128(&env, &DataKey::Retained);
        if retained <= 0 {
            panic_with_error!(&env, Error::NothingRetained);
        }
        env.storage().instance().set(&DataKey::Retained, &0i128);
        bump(&env);
        token::Client::new(&env, &c.token).transfer(
            &env.current_contract_address(),
            &c.contractor,
            &retained,
        );
        RetentionReleased { amount: retained }.publish(&env);
    }

    /// Council only, inside the defects period: retention goes back to the treasury.
    pub fn forfeit_retention(env: Env) {
        let c = cfg(&env);
        c.treasury.require_auth();
        let st = status(&env);
        if st != ProjectStatus::Completed && st != ProjectStatus::Cancelled {
            panic_with_error!(&env, Error::NotFinished);
        }
        let finished: u64 = env.storage().instance().get(&DataKey::FinishedAt).unwrap();
        if env.ledger().timestamp() >= finished.saturating_add(c.defects_secs) {
            panic_with_error!(&env, Error::DefectsPeriodClosed);
        }
        let retained = get_i128(&env, &DataKey::Retained);
        if retained <= 0 {
            panic_with_error!(&env, Error::NothingRetained);
        }
        env.storage().instance().set(&DataKey::Retained, &0i128);
        bump(&env);
        token::Client::new(&env, &c.token).transfer(
            &env.current_contract_address(),
            &c.treasury,
            &retained,
        );
        RetentionForfeited { amount: retained }.publish(&env);
    }

    /// Council only. Everything not yet paid or retained goes back to the treasury.
    pub fn cancel(env: Env) {
        let c = cfg(&env);
        c.treasury.require_auth();
        let st = status(&env);
        if st != ProjectStatus::Active && st != ProjectStatus::AwaitingFunds {
            panic_with_error!(&env, Error::NotActive);
        }
        let now = env.ledger().timestamp();
        set_status(&env, ProjectStatus::Cancelled);
        env.storage().instance().set(&DataKey::FinishedAt, &now);
        bump(&env);
        let retained = get_i128(&env, &DataKey::Retained);
        let refund = balance(&env, &c) - retained;
        if refund > 0 {
            token::Client::new(&env, &c.token).transfer(
                &env.current_contract_address(),
                &c.treasury,
                &refund,
            );
        }
        Cancelled { refunded: refund }.publish(&env);
    }

    // ---------------- views ----------------
    pub fn config(env: Env) -> Config {
        cfg(&env)
    }
    pub fn milestone(env: Env, index: u32) -> Milestone {
        load_ms(&env, index)
    }
    pub fn milestones(env: Env) -> Vec<Milestone> {
        let c = cfg(&env);
        let mut out = Vec::new(&env);
        for i in 0..c.milestone_count {
            out.push_back(load_ms(&env, i));
        }
        out
    }
    pub fn has_voted(env: Env, index: u32, inspector: Address) -> bool {
        let m = load_ms(&env, index);
        env.storage().persistent().has(&DataKey::Vote(index, m.round, inspector))
    }
    pub fn summary(env: Env) -> Summary {
        let c = cfg(&env);
        Summary {
            status: status(&env),
            total: c.total,
            paid: get_i128(&env, &DataKey::Paid),
            retained: get_i128(&env, &DataKey::Retained),
            balance: balance(&env, &c),
            current: env.storage().instance().get(&DataKey::Current).unwrap(),
            milestone_count: c.milestone_count,
            finished_at: env.storage().instance().get(&DataKey::FinishedAt).unwrap(),
        }
    }
}

#[cfg(test)]
mod test;
