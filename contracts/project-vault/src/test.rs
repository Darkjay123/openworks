#![cfg(test)]
extern crate std;

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::{StellarAssetClient, TokenClient},
    vec, Address, BytesN, Env, String,
};

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
    treasury: Address,
    contractor: Address,
    insp: std::vec::Vec<Address>,
    token: TokenClient<'static>,
    v: ProjectVaultClient<'static>,
}

const CHALLENGE: u64 = 3_600;
const DEFECTS: u64 = 86_400;

fn deploy(env: &Env, treasury: &Address, token: &Address, contractor: &Address, amounts: Vec<i128>, insp: Vec<Address>, k: u32, ret: u32) -> Address {
    env.register(
        ProjectVault,
        (
            treasury.clone(),
            token.clone(),
            contractor.clone(),
            String::from_str(env, "Abraka-Eku Road, Phase 1"),
            amounts,
            insp,
            k,
            ret,
            CHALLENGE,
            DEFECTS,
        ),
    )
}

fn setup(amounts: &[i128], fund: i128) -> S {
    let env = Env::default();
    env.ledger().set_timestamp(1_000);
    env.mock_all_auths();
    let treasury = Address::generate(&env);
    let contractor = Address::generate(&env);
    let insp: std::vec::Vec<Address> = (0..3).map(|_| Address::generate(&env)).collect();
    let admin = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(admin);
    let token = TokenClient::new(&env, &sac.address());
    StellarAssetClient::new(&env, &sac.address()).mint(&treasury, &10_000_000);
    let mut a = Vec::new(&env);
    for x in amounts { a.push_back(*x); }
    let iv = vec![&env, insp[0].clone(), insp[1].clone(), insp[2].clone()];
    let id = deploy(&env, &treasury, &sac.address(), &contractor, a, iv, 2, 1_000);
    let v = ProjectVaultClient::new(&env, &id);
    if fund > 0 {
        token.transfer(&treasury, &id, &fund);
        v.activate();
    }
    S { env, treasury, contractor, insp, token, v }
}

fn h(env: &Env, b: u8) -> BytesN<32> { BytesN::from_array(env, &[b; 32]) }
fn uri(env: &Env) -> String { String::from_str(env, "ipfs://bafy-site-report") }

fn certify(s: &S, i: u32) {
    s.v.submit(&i, &h(&s.env, i as u8 + 1), &uri(&s.env));
    s.v.attest(&s.insp[0], &i, &true);
    s.v.attest(&s.insp[1], &i, &true);
}
fn advance(s: &S, secs: u64) {
    let t = s.env.ledger().timestamp();
    s.env.ledger().set_timestamp(t + secs);
}

#[test]
fn happy_path_with_retention_and_defects_period() {
    let s = setup(&[100_000, 300_000], 400_000);
    certify(&s, 0);
    assert_eq!(s.v.milestone(&0).state, MilestoneState::Certified);
    advance(&s, CHALLENGE);
    s.v.release(&0);
    assert_eq!(s.token.balance(&s.contractor), 90_000); // 10% retained
    certify(&s, 1);
    advance(&s, CHALLENGE);
    s.v.release(&1);
    assert_eq!(s.token.balance(&s.contractor), 360_000);
    let sum = s.v.summary();
    assert_eq!(sum.status, ProjectStatus::Completed);
    assert_eq!(sum.retained, 40_000);
    assert_err(s.v.try_release_retention(), Error::DefectsPeriodOpen);
    advance(&s, DEFECTS);
    s.v.release_retention();
    assert_eq!(s.token.balance(&s.contractor), 400_000);
    assert_eq!(s.token.balance(&s.v.address), 0);
}

#[test]
fn constructor_rejects_bad_configs() {
    let env = Env::default();
    let t = Address::generate(&env);
    let c = Address::generate(&env);
    let tok = Address::generate(&env);
    let i1 = Address::generate(&env);
    let i2 = Address::generate(&env);
    let ok_ms = vec![&env, 10i128];
    let cases: std::vec::Vec<(Vec<i128>, Vec<Address>, u32, u32)> = std::vec![
        (Vec::new(&env), vec![&env, i1.clone()], 1, 0),                 // no milestones
        (vec![&env, 0i128], vec![&env, i1.clone()], 1, 0),               // zero amount
        (ok_ms.clone(), Vec::new(&env), 1, 0),                           // no inspectors
        (ok_ms.clone(), vec![&env, i1.clone(), i1.clone()], 1, 0),       // duplicate inspector
        (ok_ms.clone(), vec![&env, c.clone()], 1, 0),                    // contractor inspects self
        (ok_ms.clone(), vec![&env, i1.clone(), i2.clone()], 3, 0),       // threshold > inspectors
        (ok_ms.clone(), vec![&env, i1.clone(), i2.clone()], 0, 0),       // zero threshold
        (ok_ms.clone(), vec![&env, i1.clone()], 1, 2_001),               // retention > 20%
    ];
    for (ms, iv, k, r) in cases {
        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            deploy(&env, &t, &tok, &c, ms.clone(), iv.clone(), k, r);
        }));
        assert!(res.is_err());
    }
}

#[test]
fn activation_requires_full_budget_and_happens_once() {
    let s = setup(&[100_000, 300_000], 0);
    s.token.transfer(&s.treasury, &s.v.address, &399_999);
    assert_err(s.v.try_activate(), Error::InsufficientFunds);
    assert_err(s.v.try_submit(&0, &h(&s.env, 1), &uri(&s.env)), Error::NotActive);
    s.token.transfer(&s.treasury, &s.v.address, &1);
    s.v.activate();
    assert_err(s.v.try_activate(), Error::AlreadyActivated);
}

#[test]
fn milestones_must_go_in_order() {
    let s = setup(&[100, 200, 300], 600);
    assert_err(s.v.try_submit(&1, &h(&s.env, 1), &uri(&s.env)), Error::OutOfOrder);
    assert_err(s.v.try_submit(&9, &h(&s.env, 1), &uri(&s.env)), Error::MilestoneNotFound);
    s.v.submit(&0, &h(&s.env, 1), &uri(&s.env));
    assert_err(s.v.try_submit(&0, &h(&s.env, 1), &uri(&s.env)), Error::WrongState);
}

#[test]
fn only_registered_inspectors_vote_once_per_round() {
    let s = setup(&[100], 100);
    s.v.submit(&0, &h(&s.env, 1), &uri(&s.env));
    let stranger = Address::generate(&s.env);
    assert_err(s.v.try_attest(&stranger, &0, &true), Error::NotInspector);
    assert_err(s.v.try_attest(&s.contractor, &0, &true), Error::NotInspector);
    s.v.attest(&s.insp[0], &0, &true);
    assert_err(s.v.try_attest(&s.insp[0], &0, &true), Error::AlreadyVoted);
    assert!(s.v.has_voted(&0, &s.insp[0]));
}

#[test]
fn rejection_majority_sends_milestone_back_and_new_round_resets_votes() {
    let s = setup(&[100], 100);
    s.v.submit(&0, &h(&s.env, 1), &uri(&s.env));
    s.v.attest(&s.insp[0], &0, &true);
    s.v.attest(&s.insp[1], &0, &false);
    assert_eq!(s.v.milestone(&0).state, MilestoneState::Submitted);
    s.v.attest(&s.insp[2], &0, &false); // 2 rejections of 3 with k=2 -> unreachable
    let m = s.v.milestone(&0);
    assert_eq!(m.state, MilestoneState::Pending);
    assert_eq!(m.round, 1);
    // contractor fixes the work and resubmits; same inspectors can vote again
    s.v.submit(&0, &h(&s.env, 2), &uri(&s.env));
    assert!(!s.v.has_voted(&0, &s.insp[0]));
    s.v.attest(&s.insp[0], &0, &true);
    s.v.attest(&s.insp[2], &0, &true);
    assert_eq!(s.v.milestone(&0).state, MilestoneState::Certified);
    assert_eq!(s.v.milestone(&0).evidence_hash, h(&s.env, 2));
}

#[test]
fn release_waits_for_challenge_window_and_pays_once() {
    let s = setup(&[1_000], 1_000);
    assert_err(s.v.try_release(&0), Error::WrongState);
    certify(&s, 0);
    assert_err(s.v.try_release(&0), Error::ChallengeWindowOpen);
    advance(&s, CHALLENGE - 1);
    assert_err(s.v.try_release(&0), Error::ChallengeWindowOpen);
    advance(&s, 1);
    s.v.release(&0);
    assert_err(s.v.try_release(&0), Error::NotActive); // project completed
    assert_eq!(s.token.balance(&s.contractor), 900);
}

#[test]
fn council_can_void_only_inside_challenge_window() {
    let s = setup(&[1_000, 1_000], 2_000);
    certify(&s, 0);
    s.v.void_certification(&0);
    assert_eq!(s.v.milestone(&0).state, MilestoneState::Pending);
    certify(&s, 0);
    advance(&s, CHALLENGE);
    assert_err(s.v.try_void_certification(&0), Error::ChallengeWindowClosed);
}

#[test]
fn cancel_refunds_treasury_and_keeps_earned_retention() {
    let s = setup(&[1_000, 2_000, 3_000], 6_000);
    let before = s.token.balance(&s.treasury);
    certify(&s, 0);
    advance(&s, CHALLENGE);
    s.v.release(&0); // 900 paid, 100 retained
    s.v.cancel();
    assert_eq!(s.token.balance(&s.treasury), before + 5_000);
    assert_eq!(s.token.balance(&s.v.address), 100);
    assert_err(s.v.try_submit(&1, &h(&s.env, 1), &uri(&s.env)), Error::NotActive);
    assert_err(s.v.try_cancel(), Error::NotActive);
    advance(&s, DEFECTS);
    s.v.release_retention();
    assert_eq!(s.token.balance(&s.contractor), 1_000);
}

#[test]
fn council_can_forfeit_retention_for_defects_inside_period_only() {
    let s = setup(&[1_000], 1_000);
    assert_err(s.v.try_forfeit_retention(), Error::NotFinished);
    certify(&s, 0);
    advance(&s, CHALLENGE);
    s.v.release(&0);
    let before = s.token.balance(&s.treasury);
    s.v.forfeit_retention();
    assert_eq!(s.token.balance(&s.treasury), before + 100);
    assert_err(s.v.try_release_retention(), Error::DefectsPeriodOpen);
    advance(&s, DEFECTS);
    assert_err(s.v.try_release_retention(), Error::NothingRetained);
}

#[test]
fn forfeit_blocked_after_defects_period() {
    let s = setup(&[1_000], 1_000);
    certify(&s, 0);
    advance(&s, CHALLENGE);
    s.v.release(&0);
    advance(&s, DEFECTS);
    assert_err(s.v.try_forfeit_retention(), Error::DefectsPeriodClosed);
}

#[test]
#[should_panic]
fn submit_requires_contractor_signature() {
    let s = setup(&[100], 100);
    s.env.set_auths(&[]);
    s.v.submit(&0, &h(&s.env, 1), &uri(&s.env));
}

#[test]
#[should_panic]
fn cancel_requires_treasury_signature() {
    let s = setup(&[100], 100);
    s.env.set_auths(&[]);
    s.v.cancel();
}

#[test]
#[should_panic]
fn attest_requires_inspector_signature() {
    let s = setup(&[100], 100);
    s.v.submit(&0, &h(&s.env, 1), &uri(&s.env));
    s.env.set_auths(&[]);
    s.v.attest(&s.insp[0], &0, &true);
}
