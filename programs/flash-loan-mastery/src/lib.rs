#![warn(missing_debug_implementations, clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::wildcard_imports,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc
)]
//! Simple and best flash loan program :)

use anchor_lang::prelude::*;
use anchor_lang::solana_program::hash::hashv;
use anchor_lang::solana_program::sysvar;
use anchor_lang::solana_program::sysvar::instructions::{
    load_current_index_checked, load_instruction_at_checked,
};
use anchor_spl::token::{Mint, Token, TokenAccount};
use solana_security_txt::security_txt;
use spl_associated_token_account::get_associated_token_address;

#[cfg(not(feature = "no-entrypoint"))]
security_txt! {
    name: "Flash Loan Mastery",
    project_url: "https://github.com/moshthepitt/flash-loan-mastery",
    contacts: "link:https://twitter.com/moshthepitt",
    policy: "https://github.com/moshthepitt/flash-loan-mastery/blob/master/SECURITY.md",
    preferred_languages: "en",
    source_code: "https://github.com/moshthepitt/flash-loan-mastery"
}

declare_id!("1oanfPPN8r1i4UbugXHDxWMbWVJ5qLSN5qzNFZkz6Fg");

// with these numbers total loan fee is 0.09% or 0.095% if there is a referral
pub static LOAN_FEE: u128 = 900;
pub static REFERRAL_FEE: u128 = 50;
pub static LOAN_FEE_DENOMINATOR: u128 = 10000;
pub static ONE_HUNDRED: u128 = 100;

pub static POOL_SEED: &[u8] = b"flash_loan";

#[must_use]
/// Get the Anchor instruction identifier
/// This is documented [here](https://github.com/project-serum/anchor/blob/9e070870f4815849e99f19700d675638d3443b8f/lang/syn/src/codegen/program/dispatch.rs#L119)
pub fn get_instruction_discriminator(namespace: &[&[u8]]) -> u64 {
    let mut discriminator = [0u8; 8];
    discriminator.copy_from_slice(&hashv(namespace).to_bytes()[..8]);
    u64::from_be_bytes(discriminator)
}

#[program]
#[allow(clippy::needless_pass_by_value)]
pub mod flash_loan_mastery {
    use super::*;

    /// Initialize a lending pool
    pub fn init_pool(ctx: Context<InitPool>) -> Result<()> {
        let mut pool_authority = ctx.accounts.pool_authority.load_init()?;
        *pool_authority = PoolAuthority {
            mint: ctx.accounts.mint.key(),
            pool_share_mint: ctx.accounts.pool_share_mint.key(),
            bump: *ctx.bumps.get("pool_authority").unwrap(),
        };

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

        if ctx.accounts.pool_share_mint.freeze_authority.is_some() {
            anchor_spl::token::set_authority(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    anchor_spl::token::SetAuthority {
                        current_authority: ctx.accounts.pool_share_mint_authority.to_account_info(),
                        account_or_mint: ctx.accounts.pool_share_mint.to_account_info(),
                    },
                ),
                spl_token::instruction::AuthorityType::FreezeAccount,
                Option::None,
            )?;
        }

        Ok(())
    }

    /// Deposit funds into a lending pool
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        // calculate share amount
        // amount * total shares / total pool amount
        let share_amount = if ctx.accounts.token_to.amount == 0 {
            amount
        } else {
            u64::try_from(
                u128::from(amount) * u128::from(ctx.accounts.pool_share_mint.supply)
                    / u128::from(ctx.accounts.token_to.amount),
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
            &[ctx.accounts.pool_authority.load()?.bump],
        ];

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
            &[ctx.accounts.pool_authority.load()?.bump],
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

        Ok(())
    }

    /// Borrow funds from a lending pool
    pub fn borrow(ctx: Context<Borrow>, amount: u64) -> Result<()> {
        let instructions_sysvar = ctx.accounts.instructions_sysvar.to_account_info();

        // make sure this isn't a cpi call
        let current_idx = load_current_index_checked(&instructions_sysvar)? as usize;
        let current_ixn = load_instruction_at_checked(current_idx, &instructions_sysvar)?;
        require_keys_eq!(
            current_ixn.program_id,
            crate::ID,
            FlashLoanError::ProgramMismatch
        );

        // get expected repay amount
        let fee = u64::try_from(
            u128::from(amount) * (LOAN_FEE + REFERRAL_FEE) / (LOAN_FEE_DENOMINATOR * ONE_HUNDRED),
        )
        .unwrap();
        let expected_repayment = amount.checked_add(fee).unwrap();

        // get the ix identifier
        let borrow_ix_identifier = get_instruction_discriminator(&[b"global:borrow"]);
        let repay_ix_identifier = get_instruction_discriminator(&[b"global:repay"]);

        let mut ix_index = current_idx;
        loop {
            ix_index += 1;
            if let Ok(ixn) = load_instruction_at_checked(ix_index, &instructions_sysvar) {
                if ixn.program_id == crate::ID {
                    let ixn_identifier = u64::from_be_bytes(ixn.data[..8].try_into().unwrap());
                    // deal with repay instruction
                    if ixn_identifier == repay_ix_identifier {
                        require_keys_eq!(
                            ixn.accounts[2].pubkey,
                            ctx.accounts.token_from.key(),
                            FlashLoanError::AddressMismatch
                        );
                        require_keys_eq!(
                            ixn.accounts[3].pubkey,
                            ctx.accounts.pool_authority.key(),
                            FlashLoanError::PoolMismatch
                        );
                        let repay_ix_amount =
                            u64::from_le_bytes(ixn.data[8..16].try_into().unwrap());
                        require_gte!(
                            repay_ix_amount,
                            expected_repayment,
                            FlashLoanError::IncorrectRepaymentAmount
                        );
                        // ALL is good :)
                        break;
                    } else if ixn_identifier == borrow_ix_identifier {
                        return Err(error!(FlashLoanError::CannotBorrowBeforeRepay));
                    }
                }
            } else {
                return Err(error!(FlashLoanError::NoRepaymentInstructionFound));
            }
        }

        // get signer seeds
        let mint_bytes = ctx.accounts.token_from.mint.to_bytes();
        let pool_authority_seeds = [
            POOL_SEED,
            mint_bytes.as_ref(),
            &[ctx.accounts.pool_authority.load()?.bump],
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
    pub fn repay<'info>(ctx: Context<'_, '_, '_, 'info, Repay<'info>>, amount: u64) -> Result<()> {
        let instructions_sysvar = ctx.accounts.instructions_sysvar.to_account_info();

        // make sure this isn't a cpi call
        let current_idx =
            sysvar::instructions::load_current_index_checked(&instructions_sysvar)? as usize;
        let current_ixn =
            sysvar::instructions::load_instruction_at_checked(current_idx, &instructions_sysvar)?;
        require_keys_eq!(
            current_ixn.program_id,
            crate::ID,
            FlashLoanError::ProgramMismatch
        );

        // get referral fee
        let original_amt = LOAN_FEE_DENOMINATOR * ONE_HUNDRED * u128::from(amount)
            / ((LOAN_FEE_DENOMINATOR * ONE_HUNDRED) + LOAN_FEE + REFERRAL_FEE);
        let referral_fee =
            u64::try_from(original_amt * REFERRAL_FEE / (LOAN_FEE_DENOMINATOR * ONE_HUNDRED))
                .unwrap();

        // should we pay a referral fee?
        let mut pay_referral_fee = false;
        if let Some(referral_info) = ctx.remaining_accounts.get(0) {
            let referral_token_info = Account::<TokenAccount>::try_from(referral_info);
            if referral_token_info.is_ok() {
                pay_referral_fee = true;
            }
        }

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
            amount.checked_sub(referral_fee).unwrap(),
        )?;
        // transfer referral fee
        if pay_referral_fee {
            anchor_spl::token::transfer(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    anchor_spl::token::Transfer {
                        from: ctx.accounts.token_from.to_account_info(),
                        to: ctx.remaining_accounts.get(0).unwrap().to_account_info(),
                        authority: ctx.accounts.repayer.to_account_info(),
                    },
                ),
                referral_fee,
            )?;
        }

        Ok(())
    }
}

/// `PoolAuthority` account
#[account(zero_copy)]
#[repr(packed)]
#[derive(Debug)]
pub struct PoolAuthority {
    /// The token mint
    pub mint: Pubkey,
    /// The `pool_share_mint`
    pub pool_share_mint: Pubkey,
    /// The PDA bump
    pub bump: u8,
}

impl PoolAuthority {
    const LEN: usize = 8 + 1 + 32 + 32;
}

/// Accounts for `InitPool`
// `Mint` and `Token` don't implement `Debug`...
#[allow(missing_debug_implementations)]
#[derive(Accounts)]
pub struct InitPool<'info> {
    /// The funder for the `pool_authority` account
    #[account(mut)]
    pub funder: Signer<'info>,

    /// The mint representing the token that will be borrowed via flash loans
    pub mint: Account<'info, Mint>,

    /// The mint of the token that will represent shares in the new pool
    #[account(
        mut,
        constraint = pool_share_mint.decimals == mint.decimals @FlashLoanError::InvalidMintDecimals,
        constraint = pool_share_mint.supply == 0 @FlashLoanError::InvalidMintSupply,
    )]
    pub pool_share_mint: Account<'info, Mint>,

    /// The current mint authority of `pool_share_mint`
    pub pool_share_mint_authority: Signer<'info>,

    /// The pool authority
    #[account(
        init,
        payer = funder,
        space = PoolAuthority::LEN,
        seeds = [
            POOL_SEED,
            mint.key().as_ref(),
        ],
        bump,
    )]
    pub pool_authority: AccountLoader<'info, PoolAuthority>,

    /// The [Token] program
    pub token_program: Program<'info, Token>,

    /// The Solana System program
    pub system_program: Program<'info, System>,
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
        constraint = token_to.owner == pool_authority.key() @FlashLoanError::OwnerMismatch,
        address = get_associated_token_address(pool_authority.as_ref().key, &pool_authority.load()?.mint) @FlashLoanError::AddressMismatch,
    )]
    pub token_to: Account<'info, TokenAccount>,

    /// The token account for receiving shares in the pool
    /// CHECK: checked in token program
    #[account(mut)]
    pub pool_share_token_to: UncheckedAccount<'info>,

    /// The mint of the token representing shares in the pool
    #[account(mut, address = pool_authority.load()?.pool_share_mint @FlashLoanError::AddressMismatch)]
    pub pool_share_mint: Account<'info, Mint>,

    /// The pool authority
    /// CHECK: checked with seeds & constraints
    #[account(
        address = pool_share_mint.mint_authority.unwrap() @FlashLoanError::AddressMismatch,
        seeds = [
            POOL_SEED,
            token_to.mint.key().as_ref(),
        ],
        bump = pool_authority.load()?.bump,
    )]
    pub pool_authority: AccountLoader<'info, PoolAuthority>,

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
    #[account(mut, address = pool_authority.load()?.pool_share_mint @FlashLoanError::AddressMismatch)]
    pub pool_share_mint: Account<'info, Mint>,

    /// The pool authority
    /// CHECK: checked with seeds & constraints
    #[account(
        address = pool_share_mint.mint_authority.unwrap() @FlashLoanError::AddressMismatch,
        seeds = [
            POOL_SEED,
            token_from.mint.key().as_ref(),
        ],
        bump = pool_authority.load()?.bump,
    )]
    pub pool_authority: AccountLoader<'info, PoolAuthority>,

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
        bump = pool_authority.load()?.bump,
    )]
    pub pool_authority: AccountLoader<'info, PoolAuthority>,

    /// Solana Instructions Sysvar
    /// CHECK: Checked using address
    #[account(address = sysvar::instructions::ID @FlashLoanError::AddressMismatch)]
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
        constraint = token_to.owner == pool_authority.key() @FlashLoanError::OwnerMismatch,
    )]
    pub token_to: Account<'info, TokenAccount>,

    /// The pool authority
    /// CHECK: checked with seeds & in token program
    #[account(
        seeds = [
            POOL_SEED,
            token_to.mint.key().as_ref(),
        ],
        bump = pool_authority.load()?.bump,
    )]
    pub pool_authority: AccountLoader<'info, PoolAuthority>,

    /// Solana Instructions Sysvar
    /// CHECK: Checked using address
    #[account(address = sysvar::instructions::ID @FlashLoanError::AddressMismatch)]
    pub instructions_sysvar: UncheckedAccount<'info>,

    /// The [Token] program
    pub token_program: Program<'info, Token>,
}

/// Errors for this program
#[error_code]
pub enum FlashLoanError {
    #[msg("Address Mismatch")]
    AddressMismatch,
    #[msg("Owner Mismatch")]
    OwnerMismatch,
    #[msg("Pool Mismatch")]
    PoolMismatch,
    #[msg("Program Mismatch")]
    ProgramMismatch,
    #[msg("Invalid Mint Supply")]
    InvalidMintSupply,
    #[msg("Invalid Mint Decimals")]
    InvalidMintDecimals,
    #[msg("Cannot Borrow Before Repay")]
    CannotBorrowBeforeRepay,
    #[msg("There is no repayment instruction")]
    NoRepaymentInstructionFound,
    #[msg("The repayment amount is incorrect")]
    IncorrectRepaymentAmount,
}
