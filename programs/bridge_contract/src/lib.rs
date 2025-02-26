use anchor_lang::prelude::*;
use anchor_spl::token::spl_token::instruction::AuthorityType;
use anchor_spl::token::{self, SetAuthority, Token, TokenAccount, Transfer};

declare_id!("1TetRib49XZYuKBypgVao4JoTSKJYgtmnNCp4P132pp");

#[program]
pub mod bridge_contract {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, operator: Pubkey) -> Result<()> {
        let escrow_account = &mut ctx.accounts.escrow_account;
        escrow_account.operator = operator;
        msg!("Initialized EscrowAccount with operator: {:?}", operator);
        Ok(())
    }

    pub fn distribute_sol(ctx: Context<DistributeSol>, amount: u64) -> Result<()> {
        require!(amount > 0, CustomError::InvalidAmount);

        let escrow_account = &ctx.accounts.escrow_account;
        require_keys_eq!(
            escrow_account.operator,
            ctx.accounts.operator.key(),
            CustomError::UnauthorizedOperator
        );

        let recipient = &ctx.accounts.recipient;
        let mut adjusted_amount = amount;
        if recipient.lamports() == 0 {
            adjusted_amount =
                amount + Rent::get()?.minimum_balance(recipient.to_account_info().data_len());
        }

        **escrow_account.to_account_info().try_borrow_mut_lamports()? -= adjusted_amount;
        **recipient.to_account_info().try_borrow_mut_lamports()? += adjusted_amount;

        Ok(())
    }

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
            "sender_authority Address: {}",
            &ctx.accounts.sender_token_account_authority.key()
        );

        // Set the operator as the authority of the sender's token account
        let cpi_accounts = SetAuthority {
            account_or_mint: ctx.accounts.sender_token_account.to_account_info(),
            current_authority: ctx
                .accounts
                .sender_token_account_authority
                .to_account_info(), // Operator is the current authority
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token::set_authority(
            cpi_ctx,
            AuthorityType::AccountOwner,
            Some(ctx.accounts.operator.key()),
        )?;

        Ok(())
    }

    // Function to distribute SPL token to the recipient
    pub fn distribute_token(ctx: Context<DistributeToken>, amount: u64) -> Result<()> {
        msg!("Transferring tokens...");
        msg!(
            "Mint: {}",
            &ctx.accounts.token_program.to_account_info().key()
        );
        msg!(
            "From Token Address: {}",
            &ctx.accounts.sender_token_account.key()
        );
        msg!("To Token Address: {}", &ctx.accounts.recipient.key());

        // Now transfer the tokens from the sender's account to the recipient
        let cpi_accounts = Transfer {
            from: ctx.accounts.sender_token_account.to_account_info(),
            to: ctx.accounts.recipient.to_account_info(),
            authority: ctx.accounts.operator.to_account_info(), // Operator performs the transfer
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = payer, space = 8 + 32)]
    pub escrow_account: Account<'info, EscrowAccount>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AuthorizeToken<'info> {
    #[account(mut, has_one = operator)]
    pub escrow_account: Account<'info, EscrowAccount>,
    #[account(mut)]
    pub sender_token_account: AccountInfo<'info>,
    pub sender_token_account_authority: Signer<'info>,
    pub operator: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct DistributeSol<'info> {
    #[account(mut, has_one = operator)]
    pub escrow_account: Account<'info, EscrowAccount>,
    pub operator: Signer<'info>,
    #[account(mut)]
    pub recipient: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DistributeToken<'info> {
    #[account(mut, has_one = operator)]
    pub escrow_account: Account<'info, EscrowAccount>,
    pub operator: Signer<'info>,
    #[account(mut, owner = token::ID)]
    pub sender_token_account: Account<'info, TokenAccount>,
    #[account(mut, owner = token::ID)]
    pub recipient: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct EscrowAccount {
    pub operator: Pubkey,
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
