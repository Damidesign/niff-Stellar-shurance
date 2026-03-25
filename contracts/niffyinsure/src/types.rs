use soroban_sdk::{contracttype, Address, Map, String, Vec};

// ── Field size limits ─────────────────────────────────────────────────────────
pub const DETAILS_MAX_LEN: u32 = 256;
pub const IMAGE_URL_MAX_LEN: u32 = 128;
pub const IMAGE_URLS_MAX: u32 = 5;
pub const REASON_MAX_LEN: u32 = 128;
pub const SAFETY_SCORE_MAX: u32 = 100;

// ── Ledger window constants (re-exported from ledger.rs for ABI visibility) ───
//
// These are the canonical values used by on-chain checks.  The frontend and
// backend MUST import from here (or the generated contract spec) rather than
// hard-coding their own values.
//
// Conversion: 1 ledger ≈ 5 s on Stellar Mainnet (Protocol 20+).
// See: https://developers.stellar.org/docs/learn/fundamentals/stellar-consensus-protocol
pub use crate::ledger::{
    LEDGERS_PER_DAY, LEDGERS_PER_HOUR, LEDGERS_PER_MIN, LEDGERS_PER_WEEK,
    POLICY_DURATION_LEDGERS, QUOTE_TTL_LEDGERS, RATE_LIMIT_WINDOW_LEDGERS,
    RENEWAL_WINDOW_LEDGERS, SECS_PER_LEDGER, VOTE_WINDOW_LEDGERS,
};

// ── Enums ─────────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum PolicyType {
    Auto,
    Health,
    Property,
}

#[contracttype]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RegionTier {
    Low,
    Medium,
    High,
}

#[contracttype]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AgeBand {
    Young,
    Adult,
    Senior,
}

#[contracttype]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum CoverageType {
    Basic,
    Standard,
    Premium,
}

/// Claim lifecycle state machine.
///
/// Transitions:
///   Processing → Approved (majority approve vote or deadline plurality)
///   Processing → Rejected (majority reject vote or deadline plurality/tie)
///   Approved   → Paid     (admin calls process_claim)
#[contracttype]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ClaimStatus {
    Processing,
    Approved,
    Paid,
    Rejected,
}

impl ClaimStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, ClaimStatus::Approved | ClaimStatus::Paid | ClaimStatus::Rejected)
    }
}

#[contracttype]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum VoteOption {
    Approve,
    Reject,
}

// ── Premium engine structs ────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RiskInput {
    pub region: RegionTier,
    pub age_band: AgeBand,
    pub coverage: CoverageType,
    pub safety_score: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultiplierTable {
    pub region: Map<RegionTier, i128>,
    pub age: Map<AgeBand, i128>,
    pub coverage: Map<CoverageType, i128>,
    pub safety_discount: i128,
    pub version: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PremiumTableUpdated {
    pub version: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimProcessed {
    pub claim_id: u64,
    pub recipient: Address,
    pub amount: i128,
}

// ── Core structs ──────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone)]
pub struct Policy {
    pub holder: Address,
    pub policy_id: u32,
    pub policy_type: PolicyType,
    pub region: RegionTier,
    pub premium: i128,
    pub coverage: i128,
    pub is_active: bool,
    pub start_ledger: u32,
    pub end_ledger: u32,
}

/// On-chain claim record.
///
/// `filed_at` is the ledger sequence at which the claim was filed.  It anchors
/// the voting deadline: votes are accepted while `now < filed_at + VOTE_WINDOW_LEDGERS`.
#[contracttype]
#[derive(Clone)]
pub struct Claim {
    pub claim_id: u64,
    pub policy_id: u32,
    pub claimant: Address,
    pub amount: i128,
    pub details: String,
    pub image_urls: Vec<String>,
    pub status: ClaimStatus,
    pub approve_votes: u32,
    pub reject_votes: u32,
    /// Ledger sequence at which this claim was filed (voting window anchor).
    pub filed_at: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PremiumQuoteLineItem {
    pub component: String,
    pub factor: i128,
    pub amount: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PremiumQuote {
    pub total_premium: i128,
    pub line_items: Option<Vec<PremiumQuoteLineItem>>,
    pub valid_until_ledger: u32,
    pub config_version: u32,
}
