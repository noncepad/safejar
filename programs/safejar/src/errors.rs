use anchor_lang::error_code;



#[error_code]
pub enum TreasuryError {
    #[msg("unknown error")]
    Unknown,
    #[msg("amount out of range")]
    AmountOutOfRange,
    #[msg("rule out of range")]
    RuleOutOfRange,
    #[msg("no space to store spend request")]
    SpendRequestNoSpace,
    #[msg("rule evaluates to false")]
    RuleEvalFalse,
    #[msg("rules address lookup table key does not match")]
    RulesKeyDoesNotMatch,
    #[msg("failed to serialize rule")]
    RulesFailedToSerialize,
    #[msg("unknown rule")]
    RuleUnknown,
    #[msg("bad hash")]
    BadHash,
    #[msg("failed to add rule")]
    RuleAddFail,
    #[msg("rate limiter max spend is too low")]
    RateLimiterMaxSpendCannotBeZero,
    #[msg("rate limiter delta slot is below 100")]
    RateLimiterDeltaSlotMustBeGreaterThan100,
    #[msg("rule authorizer does not match")]
    RuleAuthorizerConstraintMustMatch,
    #[msg("rule program owner does not match")]
    RuleProgramConstraintMustMatch,
    #[msg("rule rate limiter slot out of order")]
    RuleRateLimiterSlotOutOfOrder,
    #[msg("rule rate limiter spend exceeds limit")]
    RuleRateLimiterCannotExceedSpendLimit,
    #[msg("rule sweep insufficient funds")]
    RuleSweepNotEnoughFunds,
    #[msg("rule sweep wrong destination")]
    RuleSweepWrongDestination,
    #[msg("rule max constraint max exceeded")]
    RuleMaxBalanceExceeded,
    #[msg("insufficient amount")]
    BalanceInsufficient,
    
}
