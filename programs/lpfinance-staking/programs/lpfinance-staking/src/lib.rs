use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Transfer, Token, TokenAccount }
};

use lpfinance_tokens::cpi::accounts::MintLpToken;
use lpfinance_tokens::program::LpfinanceTokens;
use lpfinance_tokens::{self};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

const PREFIX: &str = "lpfi-staking";
const DAY_IN_SECONDS: i64 = 86400; 
// Reward Rate => 0.00809%
// so need to divide with 10000
const DAILY_REWARD_RATE: u64 = 10000809;
const DENOMINATOR: u64 =       10000000;

#[program]
pub mod lpfinance_staking {
    use super::*;
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.owner = ctx.accounts.authority.key();
        config.lpfi_mint = ctx.accounts.lpfi_mint.key();
        config.last_mint_timestamp = 0;
        config.reward_rate = DENOMINATOR;
        config.total_staked = 0;
        
        Ok(())
    }

    pub fn deposit_token(
        ctx: Context<DepositToken>,
        amount: u64
    ) -> Result<()> {

        if amount == 0 {
            return Err(ErrorCode::InvalidAmount.into());
        }

        let user_account = &mut ctx.accounts.user_account;
        let config = &mut ctx.accounts.config;

        let cpi_accounts = Transfer {
            from: ctx.accounts.user_lpfi.to_account_info(),
            to: ctx.accounts.pool_lpfi.to_account_info(),
            authority: ctx.accounts.authority.to_account_info()
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        let sum = user_account.lpfi_amount * config.reward_rate / DENOMINATOR + amount;
        let cur_amount = sum * DENOMINATOR/ config.reward_rate;
        
        user_account.lpfi_amount = cur_amount;
        config.total_staked = config.total_staked + amount;
        Ok(())
    }

    pub fn withdraw_token(
        ctx: Context<WithdrawToken>,
        amount: u64
    ) -> Result<()> {
        if amount == 0 {
            return Err(ErrorCode::InvalidAmount.into());
        }
        
        let user_account = &mut ctx.accounts.user_account;
        let config = &mut ctx.accounts.config;

        let withdrawable_amount = user_account.lpfi_amount * config.reward_rate / DENOMINATOR;
        if amount > withdrawable_amount {
            return Err(ErrorCode::ExceedAmount.into());
        }

        let remain_amount = (withdrawable_amount - amount) * DENOMINATOR / config.reward_rate;
        config.total_staked = config.total_staked - amount;
        user_account.lpfi_amount = remain_amount;
        
        let (token_authority, token_authority_bump) = 
            Pubkey::find_program_address(&[PREFIX.as_bytes()], ctx.program_id);
        
        if token_authority != ctx.accounts.config.to_account_info().key() {
            return Err(ErrorCode::InvalidOwner.into());
        }

        let seeds = &[
            PREFIX.as_bytes(),
            &[token_authority_bump]
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.pool_lpfi.to_account_info(),
            to: ctx.accounts.user_lpfi.to_account_info(),
            authority: ctx.accounts.config.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

        token::transfer(cpi_ctx, amount)?;

        Ok(())
    }

    pub fn init_user_account(
        ctx: Context<InitUserAccount>
    ) -> Result<()> {
        let user_account = &mut ctx.accounts.user_account;
        user_account.owner = ctx.accounts.authority.key();
        user_account.lpfi_amount = 0;

        Ok(())
    }

    pub fn daily_reward(
        ctx: Context<DailyReward>
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;

        if config.second_owner != ctx.accounts.second_owner.key() {
            return Err(ErrorCode::InvalidOwner.into());
        }

        let total_supply = ctx.accounts.lpfi_mint.supply;
        
        // MINT TOkENS
        let mint_amount = (DAILY_REWARD_RATE - DENOMINATOR) * total_supply/ DENOMINATOR;

        let cpi_program = ctx.accounts.lptoken_program.to_account_info();
        let cpi_accounts = MintLpToken {
            signer: ctx.accounts.config.to_account_info(),
            state_account: ctx.accounts.lptoken_state.to_account_info(),
            config: ctx.accounts.lptoken_config.to_account_info(),
            user_lptoken: ctx.accounts.pool_lpfi.to_account_info(),
            lptoken_mint: ctx.accounts.lpfi_mint.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info()
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        lpfinance_tokens::cpi::mint_lptoken(cpi_ctx, mint_amount)?;
        // END MINT
        

        let config = &mut ctx.accounts.config;
        let total_staked = config.total_staked;

        if total_staked == 0 {
            return Err(ErrorCode::InvalidAmount.into());
        }

        let clock = Clock::get()?; // Returns real-world time in second uint
        let dur_seconds = clock.unix_timestamp  - config.last_mint_timestamp ;
        if dur_seconds < DAY_IN_SECONDS {
            return Err(ErrorCode::TooOftenMint.into());
        }
        config.last_mint_timestamp = clock.unix_timestamp;


        // (current_rate / denominator) * ((daily_reward / denominator) * (total_supply / total_staked))
        let numerator : u128 = config.reward_rate as u128 *  total_supply as u128 * (DAILY_REWARD_RATE - DENOMINATOR) as u128;
        let denominator: u128 = DENOMINATOR as u128 * DENOMINATOR as u128 * total_staked as u128;
        let new_rate = numerator / denominator;

        config.total_staked = (config.total_staked as u128 * new_rate  / config.reward_rate as u128) as u64;
        config.reward_rate = new_rate as u64;

        Ok(())
    }

    pub fn update_owner(
        ctx: Context<UpdateConfig>,
        new_owner: Pubkey
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;
        if config.owner != ctx.accounts.owner.key() {
            return Err(ErrorCode::InvalidOwner.into());
        }

        config.owner = new_owner;

        Ok(())
    }

    pub fn update_second_owner(
        ctx: Context<UpdateConfig>,
        new_owner: Pubkey
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;
        if config.owner != ctx.accounts.owner.key() || config.second_owner == new_owner {
            return Err(ErrorCode::InvalidOwner.into());
        }

        config.second_owner = new_owner;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    // State Accounts
    #[account(init,
        payer = authority,
        seeds = [PREFIX.as_bytes()],
        bump
    )]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub lpfi_mint: Account<'info, Mint>,
    #[account(init,
        token::mint = lpfi_mint,
        token::authority = config,
        payer= authority,
        seeds = [PREFIX.as_bytes(), b"pool_lpfi".as_ref()],
        bump
    )]
    pub pool_lpfi: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct DepositToken<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut,
        constraint = user_lpfi.mint == lpfi_mint.key(),
        constraint = user_lpfi.owner == authority.key()
    )]
    pub user_lpfi: Account<'info, TokenAccount>,
    #[account(mut)]
    pub lpfi_mint: Account<'info, Mint>,
    #[account(mut)]
    pub pool_lpfi: Account<'info, TokenAccount>,
    // State Accounts
    #[account(mut)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub user_account: Box<Account<'info, UserAccount>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct WithdrawToken<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut,
        constraint = user_lpfi.mint == lpfi_mint.key(),
        constraint = user_lpfi.owner == authority.key()
    )]
    pub user_lpfi: Account<'info, TokenAccount>,
    #[account(mut)]
    pub lpfi_mint: Account<'info, Mint>,
    #[account(mut)]
    pub pool_lpfi: Account<'info, TokenAccount>,
    // State Accounts
    #[account(mut)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub user_account: Box<Account<'info, UserAccount>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct InitUserAccount<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        seeds = [PREFIX.as_bytes(), authority.key().as_ref()],
        bump,
        payer = authority
    )]
    pub user_account: Account<'info, UserAccount>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct DailyReward<'info> {
    #[account(mut)]
    pub second_owner: Signer<'info>,
    
    #[account(mut,
        constraint = pool_lpfi.owner == config.key(),
        constraint = pool_lpfi.mint == lpfi_mint.key()
    )]
    pub pool_lpfi: Box<Account<'info, TokenAccount>>,
    
    #[account(mut, has_one = second_owner)]
    pub config: Box<Account<'info, Config>>,

    #[account(mut)]
    pub lpfi_mint: Account<'info, Mint>,

    #[account(mut)]
    pub lptoken_state: Account<'info, lpfinance_tokens::TokenStateAccount>,
    #[account(mut)]
    pub lptoken_config: Account<'info, lpfinance_tokens::Config>,
    // Programs and Sysvars
    pub lptoken_program: Program<'info, LpfinanceTokens>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut, has_one = owner)]
    pub config: Box<Account<'info, Config>>
}

#[account]
#[derive(Default)]
pub struct Config {
    // Program Owner
    pub owner: Pubkey,
    pub lpfi_mint: Pubkey,
    pub last_mint_timestamp: i64,
    pub reward_rate: u64, // compounded
    pub total_staked: u64,
    // Daily Reward Owner
    pub second_owner: Pubkey
}

#[account]
#[derive(Default)]
pub struct UserAccount {
    // amount to be able to withdraw
    pub lpfi_amount: u64,
    pub owner: Pubkey
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid Amount")]
    InvalidAmount,
    #[msg("Invalid Owner")]
    InvalidOwner,
    #[msg("Too often mint")]
    TooOftenMint,
    #[msg("Exceed Amount")]
    ExceedAmount,
    #[msg("Insufficient Amount")]
    InsufficientAmount
}