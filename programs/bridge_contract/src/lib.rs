use anchor_lang::prelude::*;
use anchor_spl::token::{self, SetAuthority, Token, TokenAccount, Transfer};
use anchor_spl::token::spl_token::instruction::AuthorityType;

declare_id!("1TetRib49XZYuKBypgVao4JoTSKJYgtmnNCp4P132pp");

#[program]
pub mod bridge_contract {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, operator: Pubkey) -> Result<()> {
        let escrow_account = &mut ctx.accounts.escrow_account;
        escrow_account.authority = *ctx.accounts.authority.key;
        escrow_account.operator = operator;
        msg!("Initialized EscrowAccount with operator: {:?}", operator);
        Ok(())
    }

    pub fn authorize_operator_once(ctx: Context<AuthorizeToken>) -> Result<()> {
        let escrow_account = &ctx.accounts.escrow_account;
        require_keys_eq!(escrow_account.operator, ctx.accounts.operator.key(), CustomError::UnauthorizedOperator);

        let cpi_accounts = SetAuthority {
            account_or_mint: ctx.accounts.sender.to_account_info(),
            current_authority: ctx.accounts.sender_authority.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token::set_authority(
            cpi_ctx,
            AuthorityType::AccountOwner,
            Some(ctx.accounts.operator.key()),
        )?;

        Ok(())
    }

    pub fn distribute_sol(ctx: Context<DistributeSol>, amount: u64) -> Result<()> {
        let escrow_account = &ctx.accounts.escrow_account;
        require_keys_eq!(escrow_account.operator, ctx.accounts.operator.key(), CustomError::UnauthorizedOperator);

        let sender = &ctx.accounts.sender;
        let recipient = &ctx.accounts.recipient;

        if recipient.lamports() == 0 {
            let rent_exempt_amount = Rent::get()?.minimum_balance(recipient.to_account_info().data_len());
            let rent_ix = anchor_lang::solana_program::system_instruction::transfer(
                &sender.key(),
                &recipient.key(),
                rent_exempt_amount,
            );
            anchor_lang::solana_program::program::invoke(
                &rent_ix,
                &[
                    sender.to_account_info(),
                    recipient.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
            )?;
        }

        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &sender.key(),
            &recipient.key(),
            amount,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                sender.to_account_info(),
                recipient.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        Ok(())
    }

    pub fn distribute_token(ctx: Context<DistributeToken>, amount: u64) -> Result<()> {
        let escrow_account = &ctx.accounts.escrow_account;
        require_keys_eq!(escrow_account.operator, ctx.accounts.operator.key(), CustomError::UnauthorizedOperator);

        let cpi_accounts = Transfer {
            from: ctx.accounts.sender.to_account_info(),
            to: ctx.accounts.recipient.to_account_info(),
            authority: ctx.accounts.operator.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = authority, space = 8 + 32 + 32)]
    pub escrow_account: Account<'info, EscrowAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AuthorizeToken<'info> {
    #[account(mut)]
    pub sender: AccountInfo<'info>,
    pub sender_authority: Signer<'info>,
    #[account(mut, has_one = operator)]
    pub escrow_account: Account<'info, EscrowAccount>,
    pub operator: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct DistributeSol<'info> {
    #[account(mut)]
    pub sender: Signer<'info>,
    #[account(mut)]
    pub recipient: AccountInfo<'info>,
    #[account(mut, has_one = operator)]
    pub escrow_account: Account<'info, EscrowAccount>,
    pub operator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DistributeToken<'info> {
    #[account(mut, owner = token::ID)]
    pub sender: Account<'info, TokenAccount>,
    #[account(mut, owner = token::ID)]
    pub recipient: Account<'info, TokenAccount>,
    #[account(mut, has_one = operator)]
    pub escrow_account: Account<'info, EscrowAccount>,
    pub operator: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct EscrowAccount {
    pub authority: Pubkey,
    pub operator: Pubkey,
}

#[error_code]
pub enum CustomError {
    #[msg("Unauthorized operator.")]
    UnauthorizedOperator,
}
