use soroban_sdk::{contracttype, Address, Env, Vec};

use crate::types::{Claim, MultiplierTable, Policy, VoteOption};

pub const PERSISTENT_TTL_THRESHOLD: u32 = 100_000;
pub const PERSISTENT_TTL_EXTEND_TO: u32 = 6_000_000;

// ── DataKey ───────────────────────────────────────────────────────────────────
#[contracttype]
pub enum DataKey {
    // Instance tier
    Admin,
    PendingAdmin,
    Token,
    PremiumTable,
    CalcAddress,
    AllowedAsset(Address),
    Voters,
    ClaimCounter,
    Paused,
    ActivePolicyCount(Address),
    // Persistent tier
    Policy(Address, u32),
    PolicyCounter(Address),
    Claim(u64),
    Vote(u64, Address),
    /// Snapshot of eligible voters captured at claim-filing time.
    ClaimVoters(u64),
    /// Last ledger at which `holder` filed a claim (rate-limit anchor).
    LastClaimLedger(Address),
}

// ── Instance bump ─────────────────────────────────────────────────────────────
pub fn bump_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

// ── Admin ─────────────────────────────────────────────────────────────────────
pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_admin(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .expect("contract not initialised: admin missing")
}

pub fn set_pending_admin(env: &Env, pending: &Address) {
    env.storage().instance().set(&DataKey::PendingAdmin, pending);
}

pub fn get_pending_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::PendingAdmin)
}

pub fn clear_pending_admin(env: &Env) {
    env.storage().instance().remove(&DataKey::PendingAdmin);
}

// ── Token ─────────────────────────────────────────────────────────────────────
pub fn set_token(env: &Env, token: &Address) {
    env.storage().instance().set(&DataKey::Token, token);
}

pub fn get_token(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::Token)
        .expect("contract not initialised: token missing")
}

// ── External calculator address ───────────────────────────────────────────────
pub fn set_calc_address(env: &Env, addr: &Address) {
    env.storage().instance().set(&DataKey::CalcAddress, addr);
}

pub fn get_calc_address(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::CalcAddress)
}

// ── Multiplier table ──────────────────────────────────────────────────────────
pub fn set_multiplier_table(env: &Env, table: &MultiplierTable) {
    env.storage().instance().set(&DataKey::PremiumTable, table);
}

pub fn get_multiplier_table(env: &Env) -> MultiplierTable {
    env.storage().instance().get(&DataKey::PremiumTable).unwrap()
}

// ── Allowed assets ────────────────────────────────────────────────────────────
pub fn set_allowed_asset(env: &Env, asset: &Address, allowed: bool) {
    env.storage()
        .instance()
        .set(&DataKey::AllowedAsset(asset.clone()), &allowed);
}

pub fn is_allowed_asset(env: &Env, asset: &Address) -> bool {
    env.storage()
        .instance()
        .get(&DataKey::AllowedAsset(asset.clone()))
        .unwrap_or(false)
}

// ── Claim (persistent) ────────────────────────────────────────────────────────
pub fn set_claim(env: &Env, claim: &Claim) {
    env.storage()
        .persistent()
        .set(&DataKey::Claim(claim.claim_id), claim);
    env.storage().persistent().extend_ttl(
        &DataKey::Claim(claim.claim_id),
        PERSISTENT_TTL_THRESHOLD,
        PERSISTENT_TTL_EXTEND_TO,
    );
}

pub fn get_claim(env: &Env, claim_id: u64) -> Option<Claim> {
    env.storage().persistent().get(&DataKey::Claim(claim_id))
}

pub fn next_claim_id(env: &Env) -> u64 {
    let current: u64 = env
        .storage()
        .instance()
        .get(&DataKey::ClaimCounter)
        .unwrap_or(0u64);
    let next = current
        .checked_add(1)
        .unwrap_or_else(|| panic!("claim_id overflow"));
    env.storage().instance().set(&DataKey::ClaimCounter, &next);
    next
}

pub fn get_claim_counter(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::ClaimCounter)
        .unwrap_or(0u64)
}

// ── Vote (persistent) ─────────────────────────────────────────────────────────
pub fn set_vote(env: &Env, claim_id: u64, voter: &Address, vote: &VoteOption) {
    let key = DataKey::Vote(claim_id, voter.clone());
    env.storage().persistent().set(&key, vote);
    env.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

pub fn get_vote(env: &Env, claim_id: u64, voter: &Address) -> Option<VoteOption> {
    env.storage()
        .persistent()
        .get(&DataKey::Vote(claim_id, voter.clone()))
}

// ── Claim voter snapshot ──────────────────────────────────────────────────────

/// Capture the current live voter set as the immutable electorate for `claim_id`.
pub fn snapshot_claim_voters(env: &Env, claim_id: u64) {
    let voters = get_voters(env);
    let key = DataKey::ClaimVoters(claim_id);
    env.storage().persistent().set(&key, &voters);
    env.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

pub fn get_claim_voters(env: &Env, claim_id: u64) -> Vec<Address> {
    env.storage()
        .persistent()
        .get(&DataKey::ClaimVoters(claim_id))
        .unwrap_or_else(|| Vec::new(env))
}

// ── Rate-limit anchor ─────────────────────────────────────────────────────────

pub fn set_last_claim_ledger(env: &Env, holder: &Address, ledger: u32) {
    env.storage()
        .persistent()
        .set(&DataKey::LastClaimLedger(holder.clone()), &ledger);
}

pub fn get_last_claim_ledger(env: &Env, holder: &Address) -> Option<u32> {
    env.storage()
        .persistent()
        .get(&DataKey::LastClaimLedger(holder.clone()))
}

// ── Policy counter (persistent) ───────────────────────────────────────────────
pub fn get_policy_counter(env: &Env, holder: &Address) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::PolicyCounter(holder.clone()))
        .unwrap_or(0u32)
}

pub fn next_policy_id(env: &Env, holder: &Address) -> u32 {
    let key = DataKey::PolicyCounter(holder.clone());
    let next: u32 = env.storage().persistent().get(&key).unwrap_or(0u32) + 1;
    env.storage().persistent().set(&key, &next);
    env.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
    next
}

// ── Policy (persistent) ───────────────────────────────────────────────────────
pub fn has_policy(env: &Env, holder: &Address, policy_id: u32) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::Policy(holder.clone(), policy_id))
}

/// Store a policy.  Key is derived from `policy.holder` and `policy.policy_id`.
pub fn set_policy(env: &Env, policy: &Policy) {
    let key = DataKey::Policy(policy.holder.clone(), policy.policy_id);
    env.storage().persistent().set(&key, policy);
    env.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

pub fn get_policy(env: &Env, holder: &Address, policy_id: u32) -> Option<Policy> {
    env.storage()
        .persistent()
        .get(&DataKey::Policy(holder.clone(), policy_id))
}

// ── Pause flag ────────────────────────────────────────────────────────────────
pub fn set_paused(env: &Env, paused: bool) {
    env.storage().instance().set(&DataKey::Paused, &paused);
}

pub fn is_paused(env: &Env) -> bool {
    env.storage()
        .instance()
        .get(&DataKey::Paused)
        .unwrap_or(false)
}

// ── Voter registry ────────────────────────────────────────────────────────────
pub fn get_voters(env: &Env) -> Vec<Address> {
    env.storage()
        .instance()
        .get(&DataKey::Voters)
        .unwrap_or_else(|| Vec::new(env))
}

pub fn set_voters(env: &Env, voters: &Vec<Address>) {
    env.storage().instance().set(&DataKey::Voters, voters);
}

pub fn add_voter(env: &Env, holder: &Address) {
    let mut voters = get_voters(env);
    let mut found = false;
    for v in voters.iter() {
        if v == *holder {
            found = true;
            break;
        }
    }
    if !found {
        voters.push_back(holder.clone());
    }
    set_voters(env, &voters);

    let key = DataKey::ActivePolicyCount(holder.clone());
    let count: u32 = env.storage().instance().get(&key).unwrap_or(0);
    env.storage().instance().set(&key, &(count + 1));
}

pub fn remove_voter(env: &Env, holder: &Address) {
    let voters = get_voters(env);
    let mut updated: Vec<Address> = Vec::new(env);
    for v in voters.iter() {
        if v != *holder {
            updated.push_back(v);
        }
    }
    set_voters(env, &updated);
}

pub fn get_active_policy_count(env: &Env, holder: &Address) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::ActivePolicyCount(holder.clone()))
        .unwrap_or(0)
}
