use anchor_lang::prelude::*;
use anchor_spl::token::spl_token::instruction::AuthorityType;
use anchor_spl::token::{self, SetAuthority, Token, TokenAccount, Transfer};

// Declaring the program ID for the bridge contract
declare_id!("1TetRib49XZYuKBypgVao4JoTSKJYgtmnNCp4P132pp");

#[program]
pub mod escrow_contract {
    use super::*;

    // Function to initialize the escrow account with an operator
    pub fn initialize(ctx: Context<Initialize>, operator: Pubkey) -> Result<()> {
        let escrow_account = &mut ctx.accounts.escrow_account;
        // Setting the operator for the escrow account
        escrow_account.operator = operator;
        msg!("Initialized EscrowAccount with operator: {:?}", operator);
        Ok(())
    }

    // Function to distribute SOL from the escrow account to a recipient
    pub fn distribute_sol(ctx: Context<DistributeSol>, amount: u64) -> Result<()> {
        // Ensuring the amount is greater than zero
        require!(amount > 0, CustomError::InvalidAmount);

        let escrow_account = &ctx.accounts.escrow_account;
        // Checking if the operator is authorized
        require_keys_eq!(
            escrow_account.operator,
            ctx.accounts.operator.key(),
            CustomError::UnauthorizedOperator
        );

        let recipient = &ctx.accounts.recipient;
        let mut adjusted_amount = amount;
        if recipient.lamports() == 0 {
            // If the recipient has no lamports
            // Adjusting the amount to cover the rent-exempt minimum balance
            adjusted_amount =
                amount + Rent::get()?.minimum_balance(recipient.to_account_info().data_len());
        }

        // Transferring the adjusted amount from the escrow account to the recipient
        **escrow_account.to_account_info().try_borrow_mut_lamports()? -= adjusted_amount;
        **recipient.to_account_info().try_borrow_mut_lamports()? += adjusted_amount;

        Ok(())
    }

    // Function to authorize the operator for the sender's token account
    pub fn authorize_operator_once(ctx: Context<AuthorizeToken>) -> Result<()> {
        msg!("Authorize tokens...");
        msg!(
            "Mint: {}",
            &ctx.accounts.token_program.to_account_info().key()
        );
        msg!(
            "Sender Token Account Address: {}",
            &ctx.accounts.sender_token_account.key()
        );
        msg!(
            "Sender Authority Address: {}",
            &ctx.accounts.sender_token_account_authority.key()
        );

        // Setting the operator as the authority of the sender's token account
        let cpi_accounts = SetAuthority {
            account_or_mint: ctx.accounts.sender_token_account.to_account_info(), // The token account to set authority on
            current_authority: ctx
                .accounts
                .sender_token_account_authority
                .to_account_info(), // Current authority of the token account
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts); // Creating a CPI context
        token::set_authority(
            cpi_ctx,
            AuthorityType::AccountOwner,
            Some(ctx.accounts.operator.key()), // New authority (operator)
        )?;

        Ok(())
    }

    // Function to distribute SPL tokens to the recipient
    pub fn distribute_token(ctx: Context<DistributeToken>, amount: u64) -> Result<()> {
        msg!("Transferring tokens..."); // Logging the token transfer process
        msg!(
            "Mint: {}",
            &ctx.accounts.token_program.to_account_info().key()
        );
        msg!(
            "From Token Address: {}",
            &ctx.accounts.sender_token_account.key()
        );
        msg!("To Token Address: {}", &ctx.accounts.recipient.key());

        // Transferring the tokens from the sender's account to the recipient
        let cpi_accounts = Transfer {
            from: ctx.accounts.sender_token_account.to_account_info(), // Source token account
            to: ctx.accounts.recipient.to_account_info(),              // Destination token account
            authority: ctx.accounts.operator.to_account_info(), // Operator performing the transfer
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts); // Creating a CPI context
        token::transfer(cpi_ctx, amount)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = payer, space = 8 + 32)]
    pub escrow_account: Account<'info, EscrowAccount>, // Escrow account to be initialized
    #[account(mut)]
    pub payer: Signer<'info>, // Signer who pays for the account creation
    pub system_program: Program<'info, System>, // System program for account management
}

#[derive(Accounts)]
pub struct AuthorizeToken<'info> {
    #[account(mut, has_one = operator)]
    pub escrow_account: Account<'info, EscrowAccount>,
    #[account(mut)]
    pub sender_token_account: AccountInfo<'info>, // Sender's token account
    pub sender_token_account_authority: Signer<'info>, // Authority of the sender's token account
    pub operator: AccountInfo<'info>,                  // Operator who is being authorized
    pub token_program: Program<'info, Token>,          // Token program for token operations
}

#[derive(Accounts)]
pub struct DistributeSol<'info> {
    #[account(mut, has_one = operator)]
    pub escrow_account: Account<'info, EscrowAccount>,
    pub operator: Signer<'info>, // Operator performing the distribution
    #[account(mut)]
    pub recipient: AccountInfo<'info>, // Recipient account for SOL
    pub system_program: Program<'info, System>, // System program for account management
}

#[derive(Accounts)]
pub struct DistributeToken<'info> {
    #[account(mut, has_one = operator)]
    pub escrow_account: Account<'info, EscrowAccount>,
    pub operator: Signer<'info>, // Operator performing the distribution
    #[account(mut, owner = token::ID)]
    pub sender_token_account: Account<'info, TokenAccount>, // Sender's token account
    #[account(mut, owner = token::ID)]
    pub recipient: Account<'info, TokenAccount>, // Recipient token account
    pub token_program: Program<'info, Token>, // Token program for token operations
}

#[account]
pub struct EscrowAccount {
    pub operator: Pubkey, // Public key of the operator managing the escrow account
}

#[error_code]
pub enum CustomError {
    #[msg("Unauthorized operator.")]
    UnauthorizedOperator,
    #[msg("Invalid amount.")]
    InvalidAmount,
    #[msg("InvalidTokenProgram.")]
    InvalidTokenProgram,
}
