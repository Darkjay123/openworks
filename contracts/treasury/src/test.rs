#![cfg(test)]
extern crate std;

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::{StellarAssetClient, TokenClient},
    vec, Address, BytesN, Env, String,
};

const TIMELOCK: u64 = 86_400;
const TTL: u64 = 14 * 86_400;

fn assert_err<T: core::fmt::Debug>(
    res: Result<T, Result<soroban_sdk::Error, soroban_sdk::InvokeError>>,
    e: Error,
) {
    match res {
        Err(Ok(got)) => assert_eq!(got, soroban_sdk::Error::from_contract_error(e as u32)),
        other => panic!("expected contract error {:?}, got {:?}", e, other),
    }
}

struct S {
    env: Env,
    council: std::vec::Vec<Address>,
    funder: Address,
    contractor: Address,
    insp: std::vec::Vec<Address>,
    token: TokenClient<'static>,
    t: TreasuryClient<'static>,
}

fn setup() -> S {
    let env = Env::default();
    env.ledger().set_timestamp(10_000);
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();
    let council: std::vec::Vec<Address> = (0..3).map(|_| Address::generate(&env)).collect();
    let funder = Address::generate(&env);
    let contractor = Address::generate(&env);
    let insp: std::vec::Vec<Address> = (0..3).map(|_| Address::generate(&env)).collect();
    let sac = env.register_stellar_asset_contract_v2(Address::generate(&env));
    let token = TokenClient::new(&env, &sac.address());
    StellarAssetClient::new(&env, &sac.address()).mint(&funder, &100_000_000);
    let wasm = env.deployer().upload_contract_wasm(vault::WASM);
    let cv = vec![&env, council[0].clone(), council[1].clone(), council[2].clone()];
    let id = env.register(
        Treasury,
        (String::from_str(&env, "Delta State Rural Roads 2026"), cv, 2u32, sac.address(), wasm, TIMELOCK, TTL),
    );
    let t = TreasuryClient::new(&env, &id);
    S { env, council, funder, contractor, insp, token, t }
}

fn spec(s: &S, amounts: &[i128]) -> ProjectSpec {
    let mut m = Vec::new(&s.env);
    for a in amounts { m.push_back(*a); }
    ProjectSpec {
        contractor: s.contractor.clone(),
        title: String::from_str(&s.env, "Abraka-Eku Road, Phase 1"),
        milestones: m,
        inspectors: vec![&s.env, s.insp[0].clone(), s.insp[1].clone(), s.insp[2].clone()],
        inspector_threshold: 2,
        retention_bps: 1_000,
        challenge_secs: 2 * 86_400,
        defects_secs: 30 * 86_400,
    }
}

fn advance(s: &S, secs: u64) {
    let t = s.env.ledger().timestamp();
    s.env.ledger().set_timestamp(t + secs);
}

/// propose + second signature + timelock + execute
fn pass(s: &S, a: Action) -> u32 {
    let id = s.t.propose(&s.council[0], &a);
    s.t.approve(&s.council[1], &id);
    advance(s, TIMELOCK);
    s.t.execute(&id);
    id
}

#[test]
fn deposit_is_tracked_per_funder() {
    let s = setup();
    s.t.deposit(&s.funder, &5_000_000);
    assert_eq!(s.t.budget().balance, 5_000_000);
    assert_eq!(s.t.budget().total_deposited, 5_000_000);
    assert_eq!(s.t.funded_by(&s.funder), 5_000_000);
    assert_err(s.t.try_deposit(&s.funder, &0), Error::InvalidAmount);
}

#[test]
fn constructor_rejects_bad_council() {
    let env = Env::default();
    let a = Address::generate(&env);
    let tok = Address::generate(&env);
    let w = BytesN::from_array(&env, &[1u8; 32]);
    for (c, thr) in [
        (Vec::<Address>::new(&env), 1u32),
        (vec![&env, a.clone(), a.clone()], 1),
        (vec![&env, a.clone()], 2),
        (vec![&env, a.clone()], 0),
    ] {
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            env.register(Treasury, (String::from_str(&env, "x"), c.clone(), thr, tok.clone(), w.clone(), TIMELOCK, TTL));
        }));
        assert!(r.is_err());
    }
}

#[test]
fn full_flow_council_creates_funded_project_vault() {
    let s = setup();
    s.t.deposit(&s.funder, &5_000_000);
    pass(&s, Action::CreateProject(spec(&s, &[1_000_000, 2_000_000])));
    let vault_addr = s.t.project(&0);
    let v = vault::Client::new(&s.env, &vault_addr);
    let sum = v.summary();
    assert_eq!(sum.status, vault::ProjectStatus::Active);
    assert_eq!(sum.total, 3_000_000);
    assert_eq!(s.token.balance(&vault_addr), 3_000_000);
    assert_eq!(s.t.budget().balance, 2_000_000);
    assert_eq!(s.t.budget().total_committed, 3_000_000);
    assert_eq!(v.config().treasury, s.t.address);

    // contractor + inspectors drive milestone 0 to payment
    v.submit(&0, &BytesN::from_array(&s.env, &[7u8; 32]), &String::from_str(&s.env, "ipfs://report-0"));
    v.attest(&s.insp[0], &0, &true);
    v.attest(&s.insp[2], &0, &true);
    advance(&s, 2 * 86_400);
    v.release(&0);
    assert_eq!(s.token.balance(&s.contractor), 900_000);
}

#[test]
fn non_member_cannot_propose_or_approve() {
    let s = setup();
    let x = Address::generate(&s.env);
    assert_err(s.t.try_propose(&x, &Action::SetPaused(true)), Error::NotCouncilMember);
    let id = s.t.propose(&s.council[0], &Action::SetPaused(true));
    assert_err(s.t.try_approve(&x, &id), Error::NotCouncilMember);
}

#[test]
fn threshold_and_timelock_are_enforced() {
    let s = setup();
    let id = s.t.propose(&s.council[0], &Action::SetPaused(true));
    assert_err(s.t.try_execute(&id), Error::ThresholdNotMet);
    assert_err(s.t.try_approve(&s.council[0], &id), Error::AlreadyApproved);
    s.t.approve(&s.council[1], &id);
    assert_err(s.t.try_execute(&id), Error::TimelockActive);
    advance(&s, TIMELOCK - 1);
    assert_err(s.t.try_execute(&id), Error::TimelockActive);
    advance(&s, 1);
    s.t.execute(&id);
    assert!(s.t.paused());
    assert_err(s.t.try_execute(&id), Error::ProposalClosed);
    assert_err(s.t.try_approve(&s.council[2], &id), Error::ProposalClosed);
}

#[test]
fn proposals_expire() {
    let s = setup();
    let id = s.t.propose(&s.council[0], &Action::SetPaused(true));
    advance(&s, TTL);
    assert_err(s.t.try_approve(&s.council[1], &id), Error::ProposalExpired);
    assert_err(s.t.try_execute(&id), Error::ProposalExpired);
}

#[test]
fn project_cannot_exceed_available_budget() {
    let s = setup();
    s.t.deposit(&s.funder, &1_000);
    let id = s.t.propose(&s.council[0], &Action::CreateProject(spec(&s, &[600, 600])));
    s.t.approve(&s.council[1], &id);
    advance(&s, TIMELOCK);
    assert_err(s.t.try_execute(&id), Error::InsufficientBudget);
    // the failed execution reverted, so the proposal is still open
    assert_eq!(s.t.proposal(&id).status, ProposalStatus::Open);
    s.t.deposit(&s.funder, &200);
    s.t.execute(&id);
    assert_eq!(s.t.budget().project_count, 1);
}

#[test]
fn pause_blocks_new_projects() {
    let s = setup();
    s.t.deposit(&s.funder, &10_000);
    pass(&s, Action::SetPaused(true));
    let id = s.t.propose(&s.council[0], &Action::CreateProject(spec(&s, &[100])));
    s.t.approve(&s.council[1], &id);
    advance(&s, TIMELOCK);
    assert_err(s.t.try_execute(&id), Error::Paused);
}

#[test]
fn removed_members_approvals_stop_counting() {
    let s = setup();
    let id = s.t.propose(&s.council[0], &Action::SetPaused(true));
    s.t.approve(&s.council[1], &id);
    // council removes member 1 before the paused proposal runs
    let rm = s.t.propose(&s.council[0], &Action::RemoveMember(s.council[1].clone()));
    s.t.approve(&s.council[2], &rm);
    advance(&s, TIMELOCK);
    s.t.execute(&rm);
    assert_eq!(s.t.council().len(), 2);
    assert_err(s.t.try_execute(&id), Error::ThresholdNotMet);
}

#[test]
fn council_size_rules_hold() {
    let s = setup();
    pass(&s, Action::SetThreshold(3));
    assert_eq!(s.t.threshold(), 3);
    // with threshold 3 of 3, removing a member would break the quorum
    let id = s.t.propose(&s.council[0], &Action::RemoveMember(s.council[2].clone()));
    s.t.approve(&s.council[1], &id);
    s.t.approve(&s.council[2], &id);
    advance(&s, TIMELOCK);
    assert_err(s.t.try_execute(&id), Error::InvalidThreshold);
    let id2 = s.t.propose(&s.council[0], &Action::AddMember(s.council[1].clone()));
    s.t.approve(&s.council[1], &id2);
    s.t.approve(&s.council[2], &id2);
    advance(&s, TIMELOCK);
    assert_err(s.t.try_execute(&id2), Error::AlreadyMember);
}

#[test]
fn council_can_void_cancel_and_recover_funds() {
    let s = setup();
    s.t.deposit(&s.funder, &10_000);
    pass(&s, Action::CreateProject(spec(&s, &[2_000, 3_000])));
    let v = vault::Client::new(&s.env, &s.t.project(&0));
    v.submit(&0, &BytesN::from_array(&s.env, &[1u8; 32]), &String::from_str(&s.env, "ipfs://r"));
    v.attest(&s.insp[0], &0, &true);
    v.attest(&s.insp[1], &0, &true);
    // a citizen flags doctored site photos; the council voids inside the 2-day window
    let id = s.t.propose(&s.council[0], &Action::VoidMilestone(0, 0));
    s.t.approve(&s.council[1], &id);
    advance(&s, TIMELOCK);
    s.t.execute(&id);
    assert_eq!(v.milestone(&0).state, vault::MilestoneState::Pending);
    assert_eq!(s.token.balance(&s.contractor), 0);

    // cancel the project: unspent budget returns to the treasury
    pass(&s, Action::CancelProject(0));
    assert_eq!(v.summary().status, vault::ProjectStatus::Cancelled);
    assert_eq!(s.t.budget().balance, 10_000);
}

#[test]
fn unknown_project_is_rejected() {
    let s = setup();
    let id = s.t.propose(&s.council[0], &Action::CancelProject(7));
    s.t.approve(&s.council[1], &id);
    advance(&s, TIMELOCK);
    assert_err(s.t.try_execute(&id), Error::ProjectNotFound);
}

#[test]
#[should_panic]
fn propose_requires_member_signature() {
    let s = setup();
    s.env.set_auths(&[]);
    s.t.propose(&s.council[0], &Action::SetPaused(true));
}

#[test]
fn challenge_window_must_outlast_council_timelock() {
    let s = setup();
    s.t.deposit(&s.funder, &10_000);
    let mut sp = spec(&s, &[100]);
    sp.challenge_secs = TIMELOCK;
    let id = s.t.propose(&s.council[0], &Action::CreateProject(sp));
    s.t.approve(&s.council[1], &id);
    advance(&s, TIMELOCK);
    assert_err(s.t.try_execute(&id), Error::ChallengeTooShort);
}
