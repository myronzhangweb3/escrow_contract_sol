use anchor_lang::prelude::*;
use anchor_spl::token::{self, SetAuthority, Token, TokenAccount, Transfer};
use anchor_lang::system_program;

declare_id!("1TetRib49XZYuKBypgVao4JoTSKJYgtmnNCp4P132pp");

#[program]
pub mod bridge_contract {
    use anchor_spl::token::spl_token::instruction::AuthorityType;

    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        // Store the USDT or other token in this contract for future distribution
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
            &ctx.accounts.sender.key()
        );
        msg!(
            "sender_authority Address: {}",
            &ctx.accounts.sender_authority.key()
        );

        // Set the operator as the authority of the sender's token account
        let cpi_accounts = SetAuthority {
            account_or_mint: ctx.accounts.sender.to_account_info(),
            current_authority: ctx.accounts.sender_authority.to_account_info(), // Operator is the current authority
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token::set_authority(
            cpi_ctx,
            AuthorityType::AccountOwner,
            Some(ctx.accounts.operator.key()),
        )?;

        Ok(())
    }

    // TODO operator sign tx
    // Function to authorize and distribute SOL by the operator
    pub fn distribute_sol(ctx: Context<DistributeSol>, amount: u64) -> Result<()> {
        msg!("Authorizing and transferring sol...");
        msg!("Sender Address: {}", &ctx.accounts.sender.key());
        msg!("Recipient Address: {}", &ctx.accounts.recipient.key());
        msg!("Sol amount: {}", amount);

        // Ensure the operator is authorized by the sender
        let sender = &ctx.accounts.sender;
        let recipient = &ctx.accounts.recipient;

        // Check if the recipient account has a zero balance
        if recipient.lamports() == 0 {
            // Calculate the rent-exempt amount for the recipient account
            let rent_exempt_amount = Rent::get()?.minimum_balance(recipient.to_account_info().data_len());
            // Transfer the rent-exempt amount from the sender to the recipient
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

        // Use a system program transfer to move SOL from the sender to the recipient
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &sender.key(),
            &recipient.key(),
            amount,
        );
        let result = anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                sender.to_account_info(),
                recipient.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        );

        Ok(())
    }

    // Function to distribute SPL token to the recipient
    pub fn distribute_token(ctx: Context<DistributeToken>, amount: u64) -> Result<()> {
        msg!("Transferring tokens...");
        msg!(
            "Mint: {}",
            &ctx.accounts.token_program.to_account_info().key()
        );
        msg!("From Token Address: {}", &ctx.accounts.sender.key());
        msg!("To Token Address: {}", &ctx.accounts.recipient.key());

        // Now transfer the tokens from the sender's account to the recipient
        let cpi_accounts = Transfer {
            from: ctx.accounts.sender.to_account_info(),
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
    // Account that will store the USDT or other tokens for distribution
    pub token_account: Account<'info, TokenAccount>,
}

#[derive(Accounts)]
pub struct AuthorizeToken<'info> {
    // Sender's account holding the token to distribute
    #[account(mut)] // Ensure the token account is owned by the Token program
    pub sender: AccountInfo<'info>, // Sender's token account
    pub sender_authority: Signer<'info>,
    // Operator authorized to carry out the distribution
    pub operator: AccountInfo<'info>, // Operator is the signer for the transaction
    // The token program responsible for the transfer
    pub token_program: Program<'info, Token>, // Token program
}

#[derive(Accounts)]
pub struct DistributeSol<'info> {
    // Sender's account holding the token to distribute
    #[account(mut)]
    pub sender: Signer<'info>,
    // Recipient who will receive SOL
    #[account(mut)]
    pub recipient: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DistributeToken<'info> {
    // Sender's account holding the token to distribute
    #[account(mut, owner = token::ID)] // Ensure the token account is owned by the Token program
    pub sender: Account<'info, TokenAccount>, // Sender's token account
    // Recipient's account to receive the distributed token
    #[account(mut, owner = token::ID)] // Ensure the token account is owned by the Token program
    pub recipient: Account<'info, TokenAccount>, // Recipient's token account
    // Operator authorized to carry out the distribution
    pub operator: Signer<'info>, // Operator is the signer for the transaction
    // The token program responsible for the transfer
    pub token_program: Program<'info, Token>, // Token program
}
