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
use anchor_lang::solana_program::sysvar;
use anchor_spl::token::{Mint, Token, TokenAccount};
use sha2_const::Sha256;
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
                    / u128::from(
                        ctx.accounts
                            .token_to
                            .delegated_amount
                            .checked_add(amount)
                            .unwrap(),
                    ),
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
        let mint_bytes = ctx.accounts.token_to.mint.to_bytes();
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
        // shares * total pool amount / total shares
        let token_amount = u64::try_from(
            u128::from(amount) * u128::from(ctx.accounts.token_from.amount)
                / u128::from(ctx.accounts.pool_share_mint.supply),
        )
        .unwrap();

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
                .checked_sub(amount)
                .unwrap(),
        )?;

        Ok(())
    }

    /// Borrow funds from a lending pool
    pub fn borrow(ctx: Context<Borrow>, amount: u64) -> Result<()> {
        let instructions_sysvar = ctx.accounts.instructions_sysvar.to_account_info();

        // make sure this isn't a cpi call
        let current_idx =
            sysvar::instructions::load_current_index_checked(&instructions_sysvar)? as usize;
        let current_ixn =
            sysvar::instructions::load_instruction_at_checked(current_idx, &instructions_sysvar)?;
        require_keys_eq!(current_ixn.program_id, *ctx.program_id);

        // get expected repay amount
        let expected_repayment = u64::try_from(
            u128::from(amount) * u128::from(LOAN_FEE + ADMIN_FEE)
                / u128::from(LOAN_FEE_DENOMINATOR),
        )
        .unwrap();
        // get the ix identifier
        let repay_ix_identifier =
            u64::from_be_bytes(Repay::INSTRUCTION_HASH[..8].try_into().unwrap());

        // loop through instructions, looking for an equivalent repay to this borrow
        let mut ix_index = current_idx + 1;
        loop {
            // get the next instruction, die if theres no more
            if let Ok(ixn) = sysvar::instructions::load_instruction_at_checked(ix_index, &instructions_sysvar) {
                let ixn_identifier = u64::from_be_bytes(ixn.data[..8].try_into().unwrap());

                // check if we have a top level repay ix toward the same token account
                // if so, confirm the amount, otherwise next instruction
                if ixn.program_id == *ctx.program_id
                    && ixn_identifier == repay_ix_identifier
                    && ixn.accounts[2].pubkey == ctx.accounts.token_from.key()
                {
                    if u64::from_le_bytes(ixn.data[8..16].try_into().unwrap()) == expected_repayment
                    {
                        break;
                    } else {
                        return Err(error!(FlashLoanError::IncorrectRepay));
                    }
                } else {
                    ix_index += 1;
                }
            } else {
                return Err(error!(FlashLoanError::NoRepay));
            }
        }

        // get signer seeds
        let mint_bytes = ctx.accounts.token_from.mint.to_bytes();
        let pool_authority_seeds = [
            POOL_SEED,
            mint_bytes.as_ref(),
            &[*ctx.bumps.get("pool_authority").unwrap()],
        ];

        // transfer from pool to borrower
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
            amount,
        )?;

        Ok(())
    }

    /// Repay funds to a lending pool
    pub fn repay(ctx: Context<Repay>, amount: u64) -> Result<()> {
        let instructions_sysvar = ctx.accounts.instructions_sysvar.to_account_info();

        // make sure this isn't a cpi call
        let current_idx =
            sysvar::instructions::load_current_index_checked(&instructions_sysvar)? as usize;
        let current_ixn =
            sysvar::instructions::load_instruction_at_checked(current_idx, &instructions_sysvar)?;
        require_keys_eq!(current_ixn.program_id, *ctx.program_id);

        // get admin fee
        let admin_fee = u64::try_from(
            u128::from(amount)
                * (u128::from(ADMIN_FEE) / u128::from(LOAN_FEE_DENOMINATOR))
                * u128::from(LOAN_FEE_DENOMINATOR)
                / u128::from(LOAN_FEE_DENOMINATOR + LOAN_FEE + ADMIN_FEE),
        )
        .unwrap();

        // transfer into pool (borrowed amount + loan fee)
        anchor_spl::token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: ctx.accounts.token_from.to_account_info(),
                    to: ctx.accounts.token_to.to_account_info(),
                    authority: ctx.accounts.repayer.to_account_info(),
                },
            ),
            amount.checked_sub(admin_fee).unwrap(),
        )?;
        // transfer to admin (just admin fee)
        anchor_spl::token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: ctx.accounts.token_from.to_account_info(),
                    to: ctx.accounts.admin_token_to.to_account_info(),
                    authority: ctx.accounts.repayer.to_account_info(),
                },
            ),
            admin_fee,
        )?;

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
    /// The entity depositing funds into the pool
    pub depositor: Signer<'info>,

    /// The token to deposit into the pool
    /// CHECK: checked in token program
    #[account(mut)]
    pub token_from: UncheckedAccount<'info>,

    /// The token to receive tokens deposited into the pool
    #[account(
        mut,
        constraint = token_to.owner == *pool_authority.key,
    )]
    pub token_to: Account<'info, TokenAccount>,

    /// The token account for receiving shares in the pool
    /// CHECK: checked in token program
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
            token_to.mint.key().as_ref(),
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
    /// The entity withdrawing funds into the pool
    pub withdrawer: Signer<'info>,

    /// The token to withdraw from the pool
    #[account(mut)]
    pub token_from: Account<'info, TokenAccount>,

    /// The token to receive tokens withdrawn from the pool
    /// CHECK: checked in token program
    #[account(mut)]
    pub token_to: UncheckedAccount<'info>,

    /// The token account for redeeming shares of the pool
    /// CHECK: checked in token program
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
            token_from.mint.key().as_ref(),
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
pub struct Borrow<'info> {
    /// The entity borrowing funds from the pool
    pub borrower: Signer<'info>,

    /// The token to borrow from the pool
    #[account(mut)]
    pub token_from: Account<'info, TokenAccount>,

    /// The token to receive tokens borrowed from the pool
    /// CHECK: checked in token program
    #[account(mut)]
    pub token_to: UncheckedAccount<'info>,

    /// The pool authority
    /// CHECK: checked with seeds & in token program
    #[account(
        seeds = [
            POOL_SEED,
            token_from.mint.key().as_ref(),
        ],
        bump,
    )]
    pub pool_authority: UncheckedAccount<'info>,

    /// Solana Instructions Sysvar
    /// CHECK: Checked using address
    #[account(address = sysvar::instructions::ID)]
    pub instructions_sysvar: UncheckedAccount<'info>,

    /// The [Token] program
    pub token_program: Program<'info, Token>,
}

/// Accounts for `Repay`
// `Mint` and `Token` don't implement `Debug`...
#[allow(missing_debug_implementations)]
#[derive(Accounts)]
pub struct Repay<'info> {
    /// The entity repaying funds from the pool
    pub repayer: Signer<'info>,

    /// The token to repay back to the pool
    /// CHECK: checked in token program
    #[account(mut)]
    pub token_from: UncheckedAccount<'info>,

    /// The token to receive tokens repaid into the pool
    #[account(
        mut,
        constraint = token_to.owner == *pool_authority.key,
    )]
    pub token_to: Account<'info, TokenAccount>,

    /// The token to receive tokens repaid into the pool
    #[account(
        mut,
        constraint = admin_token_to.owner == ADMIN_KEY,
    )]
    pub admin_token_to: Account<'info, TokenAccount>,

    /// The pool authority
    /// CHECK: checked with seeds & in token program
    #[account(
        seeds = [
            POOL_SEED,
            token_to.mint.key().as_ref(),
        ],
        bump,
    )]
    pub pool_authority: UncheckedAccount<'info>,

    /// Solana Instructions Sysvar
    /// CHECK: Checked using address
    #[account(address = sysvar::instructions::ID)]
    pub instructions_sysvar: UncheckedAccount<'info>,

    /// The [Token] program
    pub token_program: Program<'info, Token>,
}

impl Repay<'_> {
    /// Get the Anchor instruction identifier
    /// https://github.com/project-serum/anchor/blob/9e070870f4815849e99f19700d675638d3443b8f/lang/syn/src/codegen/program/dispatch.rs#L119
    ///
    /// Sha256("global:<rust-identifier>")[..8],
    const INSTRUCTION_HASH: [u8; 32] = Sha256::new().update(b"global:repay").finalize();
}

/// Errors for this program
#[error_code]
pub enum FlashLoanError {
    #[msg("There is no repayment instruction")]
    NoRepay,
    #[msg("The repayment amount is incorrect")]
    IncorrectRepay,
}
