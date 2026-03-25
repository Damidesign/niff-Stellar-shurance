use crate::{
    calculator,
    ledger,
    premium,
    storage,
    token,
<<<<<<< HEAD
    types::{Policy, PolicyType, PremiumQuote, RegionTier, RiskInput},
=======
    types::{AgeBand, CoverageType, Policy, PolicyType, PremiumQuote, RegionTier, RiskInput},
>>>>>>> f31c36f7aaafe0e6592326e70bf1e4291a0fcd67
    validate::{self, Error},
};
use soroban_sdk::{contractevent, contracterror, contracttype, Address, Env, String};

pub use ledger::QUOTE_TTL_LEDGERS;

/// Current event schema version for PolicyInitiated.
pub const POLICY_EVENT_VERSION: u32 = 1;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum QuoteError {
    InvalidAge = 1,
    InvalidRiskScore = 2,
    InvalidQuoteTtl = 3,
    ArithmeticOverflow = 4,
}

/// Errors specific to policy initiation and lifecycle.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum PolicyError {
    /// Contract is paused by admin.
    ContractPaused = 100,
    /// A policy with this (holder, policy_id) already exists.
    DuplicatePolicyId = 101,
    /// Coverage must be > 0.
    InvalidCoverage = 102,
    /// Computed premium is zero or negative (should not happen with valid inputs).
    InvalidPremium = 103,
    /// Premium computation overflowed.
    PremiumOverflow = 104,
    /// Policy duration would overflow ledger sequence.
    LedgerOverflow = 105,
    /// Policy struct failed internal validation.
    PolicyValidation = 106,
    /// Caller is not authorized (require_auth failed or wrong signer).
    Unauthorized = 107,
    /// Age out of range (1..=120).
    InvalidAge = 108,
    /// Risk score out of range (1..=10).
    InvalidRiskScore = 109,
    /// Supplied asset is not on the admin-controlled allowlist.
    AssetNotAllowed = 110,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuoteFailure {
    pub code: u32,
    pub message: String,
}

/// Versioned event emitted by `initiate_policy`.
///
/// NestJS indexers subscribe to this event to render dashboards without
/// scanning entire storage.  The `version` field allows the indexer consumer
/// to be versioned alongside contract releases.
///
/// Topic fields (`holder`) are indexed for efficient subscription filtering.
/// Data fields are serialised as a map in the event body.
#[contractevent]
#[derive(Clone, Debug)]
pub struct PolicyInitiated {
    /// Schema version; currently 1.
    #[topic]
    pub holder: Address,
    pub version: u32,
    pub policy_id: u32,
    pub premium: i128,
    pub asset: Address,
    pub policy_type: PolicyType,
    pub region: RegionTier,
    pub coverage: i128,
    pub start_ledger: u32,
    pub end_ledger: u32,
}

pub fn generate_premium(
    env: &Env,
    input: RiskInput,
    base_amount: i128,
    include_breakdown: bool,
) -> Result<PremiumQuote, Error> {
    validate::check_risk_input(&input)?;
    if base_amount <= 0 {
        return Err(Error::InvalidBaseAmount);
    }
    if QUOTE_TTL_LEDGERS == 0 {
        return Err(Error::InvalidQuoteTtl);
    }

    let table = crate::storage::get_multiplier_table(env);
    let computation = premium::compute_premium(&input, base_amount, &table)?;
    let line_items = if include_breakdown {
        Some(premium::build_line_items(env, &computation))
    } else {
        None
    };

    let current_ledger = env.ledger().sequence();
    let valid_until_ledger = current_ledger
        .checked_add(QUOTE_TTL_LEDGERS)
        .ok_or(Error::Overflow)?;

    Ok(PremiumQuote {
        total_premium: computation.total_premium,
        line_items,
        valid_until_ledger,
        config_version: computation.config_version,
    })
}

pub fn map_quote_error(env: &Env, err: Error) -> QuoteFailure {
    let message = match err {
        Error::InvalidBaseAmount => "invalid base amount: expected > 0",
        Error::SafetyScoreOutOfRange => "invalid safety_score: expected 0..=100",
        Error::InvalidConfigVersion => "invalid premium table version: expected a strictly newer version",
        Error::MissingRegionMultiplier => "premium table missing one or more region multipliers",
        Error::MissingAgeMultiplier => "premium table missing one or more age-band multipliers",
        Error::MissingCoverageMultiplier => "premium table missing one or more coverage multipliers",
        Error::RegionMultiplierOutOfBounds => "region multiplier out of bounds: expected 0.5000x..=5.0000x",
        Error::AgeMultiplierOutOfBounds => "age-band multiplier out of bounds: expected 0.5000x..=5.0000x",
        Error::CoverageMultiplierOutOfBounds => {
            "coverage multiplier out of bounds: expected 0.5000x..=5.0000x"
        }
        Error::SafetyDiscountOutOfBounds => {
            "safety discount out of bounds: expected 0.0000x..=0.5000x"
        }
        Error::Overflow => "pricing arithmetic overflow: reduce base amount or multiplier values",
        Error::DivideByZero => "pricing divide by zero: check configured scaling factors",
        Error::InvalidQuoteTtl => "quote ttl misconfigured: contact support",
        Error::NegativePremiumNotSupported => "negative premium inputs are not supported",
        Error::ClaimNotFound => "claim not found",
        Error::InvalidAsset => "claim asset is not allowlisted for payout",
        Error::InsufficientTreasury => "treasury balance is insufficient for the approved payout",
        Error::AlreadyPaid => "claim payout already executed",
        Error::ClaimNotApproved => "claim must be approved before payout",
        Error::ZeroCoverage => "policy coverage must be greater than zero",
        Error::ZeroPremium => "policy premium must be greater than zero",
        Error::InvalidLedgerWindow => "invalid ledger window: end_ledger must be greater than start_ledger",
        Error::PolicyExpired => "policy is expired",
        Error::PolicyInactive => "policy is inactive",
        Error::ClaimAmountZero => "claim amount must be greater than zero",
        Error::ClaimExceedsCoverage => "claim amount exceeds policy coverage",
        Error::DetailsTooLong => "claim details exceed maximum length",
        Error::TooManyImageUrls => "too many image URLs supplied",
        Error::ImageUrlTooLong => "image URL exceeds maximum length",
        Error::ReasonTooLong => "termination reason exceeds maximum length",
        Error::ClaimAlreadyTerminal => "claim already reached a terminal status",
        Error::DuplicateVote => "duplicate vote detected",
        Error::CalculatorNotSet => "no external calculator configured",
        Error::CalculatorCallFailed => "cross-contract call to premium calculator failed",
        Error::CalculatorPaused => "premium calculator is paused; policy bind rejected",
        Error::VotingWindowClosed => "voting window has closed; use finalize_claim",
        Error::VotingWindowStillOpen => "voting window is still open; cannot finalize yet",
        Error::NotEligibleVoter => "caller is not in the claim voter snapshot",
        Error::RateLimitExceeded => "claim rate-limit: wait before filing another claim",
    };
    QuoteFailure {
        code: err as u32,
        message: String::from_str(env, message),
    }
}

/// Turns an accepted quote into an enforceable on-chain policy.
///
/// # Auth
/// `holder.require_auth()` — only the policyholder may initiate.
///
/// # Asset
/// `asset` must be on the admin-controlled allowlist at call time.
/// The asset is bound to the policy and used for both premium payment
/// and future claim payouts — no cross-asset settlement in MVP.
///
/// # Flow
/// 1. Check contract is not paused.
/// 2. Validate asset is allowlisted.
/// 3. Authenticate the holder.
/// 4. Validate inputs (age, risk_score, coverage).
/// 5. Compute premium via `premium::compute_premium_checked`.
/// 6. Allocate a unique per-holder `policy_id`.
/// 7. Transfer premium from holder → contract address using the policy asset.
/// 8. Persist the `Policy` struct with `is_active = true`.
/// 9. Update voter registry.
/// 10. Emit versioned `PolicyInitiated` event for NestJS indexers.
pub fn initiate_policy(
    env: &Env,
    holder: Address,
    policy_type: PolicyType,
    region: RegionTier,
    coverage: i128,
    age: u32,
    risk_score: u32,
    asset: Address,
) -> Result<Policy, PolicyError> {
    // 1. Pause guard
    if storage::is_paused(env) {
        return Err(PolicyError::ContractPaused);
    }

    // 2. Asset allowlist check — before auth so callers get a clear error
    if !storage::is_allowed_asset(env, &asset) {
        return Err(PolicyError::AssetNotAllowed);
    }

    // 3. Authenticate the holder
    holder.require_auth();

    // 4. Input validation
    if age == 0 || age > 120 {
        return Err(PolicyError::InvalidAge);
    }
    if risk_score == 0 || risk_score > 10 {
        return Err(PolicyError::InvalidRiskScore);
    }
    if coverage <= 0 {
        return Err(PolicyError::InvalidCoverage);
    }

<<<<<<< HEAD
    // 5. Compute premium (smallest units / stroops)
    let premium_amount = premium::compute_premium_checked(&policy_type, &region, age, risk_score)
        .ok_or(PolicyError::PremiumOverflow)?;
=======
    // 4. Compute premium via the calculator (external or local fallback).
    //    Map calculator errors to PolicyError so callers get a typed failure.
    let risk_input = crate::types::RiskInput {
        region: region.clone(),
        age_band: age_to_band(age),
        coverage: risk_score_to_coverage(risk_score),
        safety_score: 0,
    };
    let base_amount = coverage / 10; // 10% of coverage as base
    let quote = crate::calculator::compute_quote(env, &risk_input, base_amount, false, QUOTE_TTL_LEDGERS)
        .map_err(|e| match e {
            validate::Error::CalculatorPaused => PolicyError::ContractPaused,
            validate::Error::CalculatorCallFailed | validate::Error::CalculatorNotSet => PolicyError::PremiumOverflow,
            _ => PolicyError::PremiumOverflow,
        })?;
    let premium_amount = quote.total_premium;
>>>>>>> f31c36f7aaafe0e6592326e70bf1e4291a0fcd67
    if premium_amount <= 0 {
        return Err(PolicyError::InvalidPremium);
    }

    // 6. Allocate unique per-holder policy_id
    let policy_id = storage::next_policy_id(env, &holder);

    if storage::has_policy(env, &holder, policy_id) {
        return Err(PolicyError::DuplicatePolicyId);
    }

    // 7. Premium transfer: holder → contract address using the policy's asset
    let contract_addr = env.current_contract_address();
    token::transfer(env, &asset, &holder, &contract_addr, premium_amount);

    // 8. Build and validate policy struct
    let current_ledger = env.ledger().sequence();
    let end_ledger = current_ledger
        .checked_add(ledger::POLICY_DURATION_LEDGERS)
        .ok_or(PolicyError::LedgerOverflow)?;

    let policy = Policy {
        holder: holder.clone(),
        policy_id,
        policy_type: policy_type.clone(),
        region: region.clone(),
        premium: premium_amount,
        coverage,
        is_active: true,
        start_ledger: current_ledger,
        end_ledger,
        asset: asset.clone(),
    };

    validate::check_policy(&policy).map_err(|_| PolicyError::PolicyValidation)?;

<<<<<<< HEAD
    // 9. Persist policy
    storage::set_policy(env, &holder, policy_id, &policy);
=======
    // 8. Persist policy
    storage::set_policy(env, &policy);
>>>>>>> f31c36f7aaafe0e6592326e70bf1e4291a0fcd67

    // 10. Update voter registry
    storage::add_voter(env, &holder);

    // 11. Emit versioned PolicyInitiated event
    PolicyInitiated {
        version: POLICY_EVENT_VERSION,
        policy_id,
        holder: holder.clone(),
        premium: premium_amount,
        asset: asset.clone(),
        policy_type,
        region,
        coverage,
        start_ledger: current_ledger,
        end_ledger,
    }
    .publish(env);

    Ok(policy)
}

fn age_to_band(age: u32) -> crate::types::AgeBand {
    if age < 30 {
        crate::types::AgeBand::Young
    } else if age < 60 {
        crate::types::AgeBand::Adult
    } else {
        crate::types::AgeBand::Senior
    }
}

fn risk_score_to_coverage(risk_score: u32) -> crate::types::CoverageType {
    if risk_score <= 3 {
        crate::types::CoverageType::Basic
    } else if risk_score <= 7 {
        crate::types::CoverageType::Standard
    } else {
        crate::types::CoverageType::Premium
    }
}
