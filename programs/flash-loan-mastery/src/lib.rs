#![warn(missing_debug_implementations, clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::wildcard_imports,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc
)]
//! Simple and best flash loan program :)

use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_spl::token::{Mint, Token, TokenAccount};
use static_pubkey::static_pubkey;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

pub static LOAN_FEE: u64 = 900;
pub static ADMIN_FEE: u64 = 100;
pub static LOAN_FEE_DENOMINATOR: u64 = 10000;
pub static POOL_SEED: &[u8] = b"flash_loan";
pub static ADMIN_KEY: Pubkey = static_pubkey!("44fVncfVm5fB8VsRBwVZW75FdR1nSVUKcf9nUa4ky6qN");

#[program]
#[allow(clippy::needless_pass_by_value)]
pub mod flash_loan_mastery {
    use super::*;

    /// Initialize a lending pool
    pub fn init_pool(ctx: Context<InitPool>) -> Result<()> {
        anchor_spl::token::set_authority(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::SetAuthority {
                    current_authority: ctx.accounts.pool_share_mint_authority.to_account_info(),
                    account_or_mint: ctx.accounts.pool_share_mint.to_account_info(),
                },
            ),
            spl_token::instruction::AuthorityType::MintTokens,
            Some(ctx.accounts.pool_authority.key()),
        )?;

        Ok(())
    }

    /// Deposit funds into a lending pool
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        // calculate share amount
        // amount * total shares / total deposits
        let share_amount = if ctx.accounts.token_to.delegated_amount == 0 {
            amount
        } else {
            u64::try_from(
                u128::from(amount) * u128::from(ctx.accounts.pool_share_mint.supply)
                    / u128::from(ctx.accounts.token_to.delegated_amount),
            )
            .unwrap()
        };

        // transfer to pool
        anchor_spl::token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: ctx.accounts.token_from.to_account_info(),
                    to: ctx.accounts.token_to.to_account_info(),
                    authority: ctx.accounts.depositor.to_account_info(),
                },
            ),
            amount,
        )?;

        // get signer seeds
        let mint_bytes = ctx.accounts.token_from.mint.to_bytes();
        let pool_authority_seeds = [
            POOL_SEED,
            mint_bytes.as_ref(),
            &[*ctx.bumps.get("pool_authority").unwrap()],
        ];

        // set `delegated_amount` (keeps track of total deposit amount)
        anchor_spl::token::approve(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Approve {
                    to: ctx.accounts.token_to.to_account_info(),
                    delegate: ctx.accounts.pool_authority.to_account_info(),
                    authority: ctx.accounts.pool_authority.to_account_info(),
                },
            )
            .with_signer(&[&pool_authority_seeds[..]]),
            ctx.accounts
                .token_to
                .delegated_amount
                .checked_add(amount)
                .unwrap(),
        )?;

        // mint new pool share tokens
        anchor_spl::token::mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::MintTo {
                    mint: ctx.accounts.pool_share_mint.to_account_info(),
                    to: ctx.accounts.pool_share_token_to.to_account_info(),
                    authority: ctx.accounts.pool_authority.to_account_info(),
                },
            )
            .with_signer(&[&pool_authority_seeds[..]]),
            share_amount,
        )?;

        Ok(())
    }

    /// Withdraw funds from a lending pool
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        // calculate token amount
        // shares * total deposits / total shares
        let token_amount = if ctx.accounts.token_to.delegated_amount == 0 {
            amount
        } else {
            u64::try_from(
                u128::from(amount) * u128::from(ctx.accounts.token_to.delegated_amount)
                    / u128::from(ctx.accounts.pool_share_mint.supply),
            )
            .unwrap()
        };

        // burn pool share tokens
        anchor_spl::token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Burn {
                    mint: ctx.accounts.pool_share_mint.to_account_info(),
                    from: ctx.accounts.pool_share_token_from.to_account_info(),
                    authority: ctx.accounts.withdrawer.to_account_info(),
                },
            ),
            amount,
        )?;

        // get signer seeds
        let mint_bytes = ctx.accounts.token_from.mint.to_bytes();
        let pool_authority_seeds = [
            POOL_SEED,
            mint_bytes.as_ref(),
            &[*ctx.bumps.get("pool_authority").unwrap()],
        ];

        // transfer from pool
        anchor_spl::token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: ctx.accounts.token_from.to_account_info(),
                    to: ctx.accounts.token_to.to_account_info(),
                    authority: ctx.accounts.pool_authority.to_account_info(),
                },
            )
            .with_signer(&[&pool_authority_seeds[..]]),
            token_amount,
        )?;

        // set `delegated_amount` (keeps track of total deposit amount)
        anchor_spl::token::approve(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Approve {
                    to: ctx.accounts.token_from.to_account_info(),
                    delegate: ctx.accounts.pool_authority.to_account_info(),
                    authority: ctx.accounts.pool_authority.to_account_info(),
                },
            )
            .with_signer(&[&pool_authority_seeds[..]]),
            ctx.accounts
                .token_from
                .delegated_amount
                .checked_sub(token_amount)
                .unwrap(),
        )?;

        Ok(())
    }

    /// Borrow funds from a lending pool
    pub fn borrow(ctx: Context<Borrow>, amount: u64) -> Result<()> {
        Ok(())
    }

    /// Repay funds to a lending pool
    pub fn repay(ctx: Context<Repay>, amount: u64) -> Result<()> {
        Ok(())
    }
}

/// Accounts for `InitPool`
// `Mint` and `Token` don't implement `Debug`...
#[allow(missing_debug_implementations)]
#[derive(Accounts)]
pub struct InitPool<'info> {
    /// The mint representing the token that will be borrowed via flash loans
    pub mint: Account<'info, Mint>,

    /// The mint of the token that will represent shares in a given pool
    #[account(
        mut,
        constraint = pool_share_mint.decimals == 0,
        constraint = pool_share_mint.supply == 0,
    )]
    pub pool_share_mint: Account<'info, Mint>,

    /// The current mint authority of `pool_share_mint`
    pub pool_share_mint_authority: Signer<'info>,

    /// The pool authority
    /// CHECK: checked with seeds
    #[account(
        seeds = [
            POOL_SEED,
            mint.key().as_ref(),
        ],
        bump,
    )]
    pub pool_authority: UncheckedAccount<'info>,

    /// The [Token] program
    pub token_program: Program<'info, Token>,
}

/// Accounts for `Deposit`
// `Mint` and `Token` don't implement `Debug`...
#[allow(missing_debug_implementations)]
#[derive(Accounts)]
pub struct Deposit<'info> {
    /// The entity depositing funds into the poll
    pub depositor: Signer<'info>,

    /// The token to deposit into the pool
    #[account(mut)]
    pub token_from: Account<'info, TokenAccount>,

    /// The token to receive tokens deposited into the pool
    #[account(mut)]
    pub token_to: Account<'info, TokenAccount>,

    /// The token account for receiving shares in the pool
    #[account(mut)]
    pub pool_share_token_to: UncheckedAccount<'info>,

    /// The mint of the token representing shares in the pool
    #[account(mut)]
    pub pool_share_mint: Account<'info, Mint>,

    /// The pool authority
    /// CHECK: checked with seeds & constraints
    #[account(
        address = pool_share_mint.mint_authority.unwrap(),
        seeds = [
            POOL_SEED,
            token_from.mint.key().as_ref(),
        ],
        bump,
    )]
    pub pool_authority: UncheckedAccount<'info>,

    /// The [Token] program
    pub token_program: Program<'info, Token>,
}

/// Accounts for `Withdraw`
// `Mint` and `Token` don't implement `Debug`...
#[allow(missing_debug_implementations)]
#[derive(Accounts)]
pub struct Withdraw<'info> {
    /// The entity withdrawing funds into the poll
    pub withdrawer: Signer<'info>,

    /// The token to withdraw from the pool
    #[account(mut)]
    pub token_from: Account<'info, TokenAccount>,

    /// The token to receive tokens withdrawn from the pool
    #[account(mut)]
    pub token_to: Account<'info, TokenAccount>,

    /// The token account for redeeming shares of the pool
    #[account(mut)]
    pub pool_share_token_from: UncheckedAccount<'info>,

    /// The mint of the token representing shares in the pool
    #[account(mut)]
    pub pool_share_mint: Account<'info, Mint>,

    /// The pool authority
    /// CHECK: checked with seeds & constraints
    #[account(
        address = pool_share_mint.mint_authority.unwrap(),
        seeds = [
            POOL_SEED,
            token_to.mint.key().as_ref(),
        ],
        bump,
    )]
    pub pool_authority: UncheckedAccount<'info>,

    /// The [Token] program
    pub token_program: Program<'info, Token>,
}

/// Accounts for `Borrow`
// `Mint` and `Token` don't implement `Debug`...
#[allow(missing_debug_implementations)]
#[derive(Accounts)]
pub struct Borrow {}

/// Accounts for `Repay`
// `Mint` and `Token` don't implement `Debug`...
#[allow(missing_debug_implementations)]
#[derive(Accounts)]
pub struct Repay {}
