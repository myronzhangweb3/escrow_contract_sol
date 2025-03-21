use anchor_lang::prelude::*;

#[error_code]
pub enum CustomError {
    #[msg("Unauthorized operator.")]
    UnauthorizedOperator,
    #[msg("Invalid amount.")]
    InvalidAmount,
    #[msg("Unauthorized program.")]
    UnauthorizedProgram,
    #[msg("InvalidTokenProgram.")]
    InvalidTokenProgram,
    #[msg("InsufficientFunds.")]
    InsufficientFunds,
    #[msg("Overflow.")]
    Overflow,
}

#[account]
pub struct EscrowAccount {
    pub operator: Pubkey, // Public key of the operator managing the escrow account
    pub allowed_program_id: Pubkey, // Public key of the allowed program ID
}
