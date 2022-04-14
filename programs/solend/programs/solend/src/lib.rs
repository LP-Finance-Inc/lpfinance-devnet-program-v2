use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, Transfer, TokenAccount }
};

use lending_tokens::cpi::accounts::MintToken;
use lending_tokens::program::LendingTokens;
use lending_tokens::{self};

declare_id!("CwB4GeieH48q4p864wammdzFYBMNWopYRQPYkS8zbZwL");

const PREFIX: &str = "solend0";

const DAY_IN_SECONDS: i64 = 86400; 
const DENOMINATOR: u64 = 10000000;

#[program]
pub mod solend {
    use super::*;
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {

        let state_account = &mut ctx.accounts.state_account;
        let config = &mut ctx.accounts.config;

        state_account.owner = ctx.accounts.authority.key();

        config.ust_mint = ctx.accounts.ust_mint.key();
        config.usdc_mint = ctx.accounts.usdc_mint.key();
        config.msol_mint = ctx.accounts.msol_mint.key();
        config.srm_mint = ctx.accounts.srm_mint.key();
        config.scnsol_mint = ctx.accounts.scnsol_mint.key();
        config.stsol_mint = ctx.accounts.stsol_mint.key();
        config.btc_mint = ctx.accounts.btc_mint.key();
        config.usdt_mint = ctx.accounts.usdt_mint.key();
        config.eth_mint = ctx.accounts.eth_mint.key();

        config.state_account = ctx.accounts.state_account.key();

        Ok(())
    }

    // Init user account
    pub fn init_user_account(
        ctx: Context<InitUserAccount>
    ) -> Result<()> {
        // Make as 1 string for pubkey

        let user_account = &mut ctx.accounts.user_account;
        user_account.owner = ctx.accounts.user_authority.key();

        user_account.ust_amount = 0;
        user_account.usdc_amount = 0;
        user_account.msol_amount = 0;
        user_account.srm_amount = 0;
        user_account.scnsol_amount = 0;
        user_account.stsol_amount = 0;
        user_account.btc_amount = 0;
        user_account.usdt_amount = 0;
        user_account.eth_amount = 0;

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
            from: ctx.accounts.user_token.to_account_info(),
            to: ctx.accounts.pool_token.to_account_info(),
            authority: ctx.accounts.authority.to_account_info()
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        if config.ust_mint == ctx.accounts.token_mint.key() {
            let sum = user_account.ust_amount + amount * DENOMINATOR / config.ust_rate;
            
            user_account.ust_amount = sum;
            config.ust_amount = config.ust_amount + amount;
        } else if config.usdc_mint == ctx.accounts.token_mint.key() {
            let sum = user_account.usdc_amount + amount * DENOMINATOR / config.usdc_rate;
            
            user_account.usdc_amount = sum;
            config.usdc_amount = config.usdc_amount + amount;
        } else if config.msol_mint == ctx.accounts.token_mint.key() {
            let sum = user_account.msol_amount + amount * DENOMINATOR / config.msol_rate;
            
            user_account.msol_amount = sum;
            config.msol_amount = config.msol_amount + amount;
        } else if config.srm_mint == ctx.accounts.token_mint.key() {
            let sum = user_account.srm_amount + amount * DENOMINATOR / config.srm_rate;
            
            user_account.srm_amount = sum;
            config.srm_amount = config.srm_amount + amount;
        } else if config.scnsol_mint == ctx.accounts.token_mint.key() {
            let sum = user_account.scnsol_amount + amount * DENOMINATOR / config.scnsol_rate;
            
            user_account.scnsol_amount = sum;
            config.scnsol_amount = config.scnsol_amount + amount;
        } else if config.stsol_mint == ctx.accounts.token_mint.key() {
            let sum = user_account.stsol_amount + amount * DENOMINATOR / config.stsol_rate;
            
            user_account.stsol_amount = sum;
            config.stsol_amount = config.stsol_amount + amount;
        } else if config.btc_mint == ctx.accounts.token_mint.key() {
            let sum = user_account.btc_amount + amount * DENOMINATOR / config.btc_rate;
            
            user_account.btc_amount = sum;
            config.btc_amount = config.btc_amount + amount;
        } else if config.usdt_mint == ctx.accounts.token_mint.key() {
            let sum = user_account.usdt_amount + amount * DENOMINATOR / config.usdt_rate;
            
            user_account.usdt_amount = sum;
            config.usdt_amount = config.usdt_amount + amount;
        } else if config.eth_mint == ctx.accounts.token_mint.key() {
            let sum = user_account.eth_amount + amount * DENOMINATOR / config.eth_rate;
            
            user_account.eth_amount = sum;
            config.eth_amount = config.eth_amount + amount;
        }

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

        if config.ust_mint == ctx.accounts.token_mint.key() {
            let withdrawable_amount = user_account.ust_amount * config.ust_rate / DENOMINATOR;
        
            if amount > withdrawable_amount {
                return Err(ErrorCode::ExceedAmount.into());
            }

            let remain_amount = (withdrawable_amount - amount) * DENOMINATOR / config.ust_rate;
            config.ust_amount = config.ust_amount - amount;
            user_account.ust_amount = remain_amount;

        } else if config.usdc_mint == ctx.accounts.token_mint.key() {
            let withdrawable_amount = user_account.usdc_amount * config.usdc_rate / DENOMINATOR;
        
            if amount > withdrawable_amount {
                return Err(ErrorCode::ExceedAmount.into());
            }

            let remain_amount = (withdrawable_amount - amount) * DENOMINATOR / config.usdc_rate;
            config.usdc_amount = config.usdc_amount - amount;
            user_account.usdc_amount = remain_amount;
        } else if config.msol_mint == ctx.accounts.token_mint.key() {
            
            let withdrawable_amount = user_account.msol_amount * config.msol_rate / DENOMINATOR;
        
            if amount > withdrawable_amount {
                return Err(ErrorCode::ExceedAmount.into());
            }

            let remain_amount = (withdrawable_amount - amount) * DENOMINATOR / config.msol_rate;
            config.msol_amount = config.msol_amount - amount;
            user_account.msol_amount = remain_amount;
        } else if config.srm_mint == ctx.accounts.token_mint.key() {
            
            let withdrawable_amount = user_account.srm_amount * config.srm_rate / DENOMINATOR;
        
            if amount > withdrawable_amount {
                return Err(ErrorCode::ExceedAmount.into());
            }

            let remain_amount = (withdrawable_amount - amount) * DENOMINATOR / config.srm_rate;
            config.srm_amount = config.srm_amount - amount;
            user_account.srm_amount = remain_amount;
        } else if config.scnsol_mint == ctx.accounts.token_mint.key() {
            
            let withdrawable_amount = user_account.scnsol_amount * config.scnsol_rate / DENOMINATOR;
        
            if amount > withdrawable_amount {
                return Err(ErrorCode::ExceedAmount.into());
            }

            let remain_amount = (withdrawable_amount - amount) * DENOMINATOR / config.scnsol_rate;
            config.scnsol_amount = config.scnsol_amount - amount;
            user_account.scnsol_amount = remain_amount;
        } else if config.stsol_mint == ctx.accounts.token_mint.key() {
            
            let withdrawable_amount = user_account.stsol_amount * config.stsol_rate / DENOMINATOR;
        
            if amount > withdrawable_amount {
                return Err(ErrorCode::ExceedAmount.into());
            }

            let remain_amount = (withdrawable_amount - amount) * DENOMINATOR / config.stsol_rate;
            config.stsol_amount = config.stsol_amount - amount;
            user_account.stsol_amount = remain_amount;
        } else if config.btc_mint == ctx.accounts.token_mint.key() {
            
            let withdrawable_amount = user_account.btc_amount * config.btc_rate / DENOMINATOR;
        
            if amount > withdrawable_amount {
                return Err(ErrorCode::ExceedAmount.into());
            }

            let remain_amount = (withdrawable_amount - amount) * DENOMINATOR / config.btc_rate;
            config.btc_amount = config.btc_amount - amount;
            user_account.btc_amount = remain_amount;
        } else if config.usdt_mint == ctx.accounts.token_mint.key() {
            
            let withdrawable_amount = user_account.usdt_amount * config.usdt_rate / DENOMINATOR;
        
            if amount > withdrawable_amount {
                return Err(ErrorCode::ExceedAmount.into());
            }

            let remain_amount = (withdrawable_amount - amount) * DENOMINATOR / config.usdt_rate;
            config.usdt_amount = config.usdt_amount - amount;
            user_account.usdt_amount = remain_amount;
        } else if config.eth_mint == ctx.accounts.token_mint.key() {
            
            let withdrawable_amount = user_account.eth_amount * config.eth_rate / DENOMINATOR;
        
            if amount > withdrawable_amount {
                return Err(ErrorCode::ExceedAmount.into());
            }

            let remain_amount = (withdrawable_amount - amount) * DENOMINATOR / config.eth_rate;
            config.eth_amount = config.eth_amount - amount;
            user_account.eth_amount = remain_amount;
        } else {
            return Err(ErrorCode::InvalidToken.into())
        }
        
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
            from: ctx.accounts.pool_token.to_account_info(),
            to: ctx.accounts.user_token.to_account_info(),
            authority: ctx.accounts.state_account.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

        token::transfer(cpi_ctx, amount)?;

        Ok(())
    }

    pub fn daily_reward(
        ctx: Context<DailyReward>,
        rate: u64
    ) -> Result<()> {

        if ctx.accounts.state_account.second_owner != ctx.accounts.second_owner.key() {
            return Err(ErrorCode::InvalidOwner.into());
        }

        if rate < DENOMINATOR {
            return Err(ErrorCode::InvalidAmount.into());
        }

        let config = &mut ctx.accounts.config;

        let clock = Clock::get()?; // Returns real-world time in second uint
        let dur_seconds = clock.unix_timestamp  - config.last_mint_timestamp ;
        if dur_seconds < DAY_IN_SECONDS {
            return Err(ErrorCode::TooOftenMint.into());
        }

        let mut mint_amount = 0;
        if config.ust_mint == ctx.accounts.token_mint.key() {
            let reward_amount = config.ust_amount * (rate - DENOMINATOR) / DENOMINATOR;

            let rate_will = config.ust_rate * rate  / DENOMINATOR;

            config.ust_amount = config.ust_amount + reward_amount;
            config.ust_rate = rate_will;
            mint_amount = reward_amount;
        } else if config.usdc_mint == ctx.accounts.token_mint.key() {
            let reward_amount = config.usdc_amount * (rate - DENOMINATOR) / DENOMINATOR;

            let rate_will = config.usdc_rate * rate  / DENOMINATOR;

            config.usdc_amount = config.usdc_amount + reward_amount;
            config.usdc_rate = rate_will;
            mint_amount = reward_amount;
        } else if config.msol_mint == ctx.accounts.token_mint.key() {
            let reward_amount = config.msol_amount * (rate - DENOMINATOR) / DENOMINATOR;

            let rate_will = config.msol_rate * rate  / DENOMINATOR;

            config.msol_amount = config.msol_amount + reward_amount;
            config.msol_rate = rate_will;
            mint_amount = reward_amount;
        } else if config.srm_mint == ctx.accounts.token_mint.key() {
            let reward_amount = config.srm_amount * (rate - DENOMINATOR) / DENOMINATOR;

            let rate_will = config.srm_rate * rate  / DENOMINATOR;

            config.srm_amount = config.srm_amount + reward_amount;
            config.srm_rate = rate_will;
            mint_amount = reward_amount;
        } else if config.scnsol_mint == ctx.accounts.token_mint.key() {
            let reward_amount = config.scnsol_amount * (rate - DENOMINATOR) / DENOMINATOR;

            let rate_will = config.scnsol_rate * rate  / DENOMINATOR;

            config.scnsol_amount = config.scnsol_amount + reward_amount;
            config.scnsol_rate = rate_will;
            mint_amount = reward_amount;
        } else if config.stsol_mint == ctx.accounts.token_mint.key() {
            let reward_amount = config.stsol_amount * (rate - DENOMINATOR) / DENOMINATOR;

            let rate_will = config.stsol_rate * rate  / DENOMINATOR;

            config.stsol_amount = config.stsol_amount + reward_amount;
            config.stsol_rate = rate_will;
            mint_amount = reward_amount;
        } else if config.btc_mint == ctx.accounts.token_mint.key() {
            let reward_amount = config.btc_amount * (rate - DENOMINATOR) / DENOMINATOR;

            let rate_will = config.btc_rate * rate  / DENOMINATOR;

            config.btc_amount = config.btc_amount + reward_amount;
            config.btc_rate = rate_will;
            mint_amount = reward_amount;
        } else if config.usdt_mint == ctx.accounts.token_mint.key() {
            let reward_amount = config.usdt_amount * (rate - DENOMINATOR) / DENOMINATOR;

            let rate_will = config.usdt_rate * rate  / DENOMINATOR;

            config.usdt_amount = config.usdt_amount + reward_amount;
            config.usdt_rate = rate_will;
            mint_amount = reward_amount;
        } else if config.eth_mint == ctx.accounts.token_mint.key() {
            let reward_amount = config.eth_amount * (rate - DENOMINATOR) / DENOMINATOR;

            let rate_will = config.eth_rate * rate  / DENOMINATOR;

            config.eth_amount = config.eth_amount + reward_amount;
            config.eth_rate = rate_will;
            mint_amount = reward_amount;
        } else {
            return Err(ErrorCode::InvalidToken.into())
        }
        
        // MINT TOkENS
        let cpi_program = ctx.accounts.lending_program.to_account_info();
        let cpi_accounts = MintToken {
            owner: ctx.accounts.state_account.to_account_info(),
            state_account: ctx.accounts.token_state.to_account_info(),
            user_token: ctx.accounts.pool_token.to_account_info(),
            token_mint: ctx.accounts.token_mint.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info()
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        lending_tokens::cpi::mint_token(cpi_ctx, mint_amount)?;
        // END MINT
        
        config.last_mint_timestamp = clock.unix_timestamp;

        Ok(())
    }

    pub fn update_owner(
        ctx: Context<UpdateConfigAccount>,
        new_owner: Pubkey
    ) -> Result<()> {
        let state_account = &mut ctx.accounts.state_account;
        if state_account.owner != ctx.accounts.owner.key() || ctx.accounts.owner.key() == new_owner {
            return Err(ErrorCode::InvalidOwner.into());
        }

        state_account.owner = new_owner;

        Ok(())
    }

    pub fn update_second_owner(
        ctx: Context<UpdateConfigAccount>,
        new_owner: Pubkey
    ) -> Result<()> {
        let state_account = &mut ctx.accounts.state_account;
        if state_account.owner != ctx.accounts.owner.key() || state_account.second_owner == new_owner {
            return Err(ErrorCode::InvalidOwner.into());
        }

        state_account.second_owner = new_owner;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    // Token program owner
    #[account(mut)]
    pub authority: Signer<'info>,
    // State Accounts
    #[account(init,
        seeds = [PREFIX.as_bytes()],
        bump,
        payer = authority
    )]
    pub state_account: Box<Account<'info, StateAccount>>,
 
    // State Accounts
    #[account(init,
        payer = authority
    )]
    pub config: Box<Account<'info, Config>>,
    pub ust_mint: Box<Account<'info, Mint>>,  
    pub usdc_mint: Box<Account<'info, Mint>>,
    pub msol_mint: Box<Account<'info, Mint>>,
    pub srm_mint: Box<Account<'info, Mint>>,
    pub scnsol_mint: Box<Account<'info, Mint>>,
    pub stsol_mint: Box<Account<'info, Mint>>,
    pub btc_mint: Box<Account<'info, Mint>>,
    pub usdt_mint: Box<Account<'info, Mint>>,
    pub eth_mint: Box<Account<'info, Mint>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct InitUserAccount<'info> {
    // State account for each user/wallet
    #[account(
        init,
        seeds = [PREFIX.as_bytes(), user_authority.key().as_ref()],
        bump,
        payer = user_authority
    )]
    pub user_account: Account<'info, UserAccount>,
    // Contract Authority accounts
    #[account(mut)]
    pub user_authority: Signer<'info>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct DepositToken<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut,
        constraint = user_token.mint == token_mint.key(),
        constraint = user_token.owner == authority.key()
    )]
    pub user_token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_mint: Account<'info, Mint>,
    #[account(mut)]
    pub pool_token: Account<'info, TokenAccount>,
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
        constraint = user_token.mint == token_mint.key(),
        constraint = user_token.owner == authority.key()
    )]
    pub user_token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_mint: Account<'info, Mint>,
    #[account(mut)]
    pub pool_token: Account<'info, TokenAccount>,
    // State Accounts
    #[account(mut)]
    pub state_account: Box<Account<'info, Config>>,
    #[account(mut, has_one = state_account)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub user_account: Box<Account<'info, UserAccount>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}


#[derive(Accounts)]
pub struct DailyReward<'info> {
    #[account(mut)]
    pub second_owner: Signer<'info>,
    
    #[account(mut,
        constraint = pool_token.owner == config.key(),
        constraint = pool_token.mint == token_mint.key()
    )]
    pub pool_token: Box<Account<'info, TokenAccount>>,
    
    #[account(mut, has_one = state_account)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut, has_one = second_owner)]
    pub state_account: Box<Account<'info, StateAccount>>,

    #[account(mut)]
    pub token_mint: Account<'info, Mint>,
    #[account(mut)]
    pub token_state: Account<'info, lending_tokens::TokenStateAccount>,
    // Programs and Sysvars
    pub lending_program: Program<'info, LendingTokens>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct UpdateConfigAccount<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut, has_one = owner)]
    pub state_account: Box<Account<'info, StateAccount>>,
    #[account(mut, has_one = state_account)]
    pub config: Box<Account<'info, Config>>
}

#[account]
#[derive(Default)]
pub struct StateAccount {
    pub owner: Pubkey,
    pub second_owner: Pubkey
}

#[account]
#[derive(Default)]
pub struct Config {
    pub state_account: Pubkey,
    pub ust_mint: Pubkey,
    pub usdc_mint: Pubkey,
    pub msol_mint: Pubkey,
    pub srm_mint: Pubkey,
    pub scnsol_mint: Pubkey,
    pub stsol_mint: Pubkey,
    pub btc_mint: Pubkey,
    pub usdt_mint: Pubkey,
    pub eth_mint: Pubkey,
    //
    pub ust_amount: u64,
    pub usdc_amount: u64,
    pub msol_amount: u64,
    pub srm_amount: u64,
    pub scnsol_amount: u64,
    pub stsol_amount: u64,
    pub btc_amount: u64,
    pub usdt_amount: u64,
    pub eth_amount: u64,
    //
    pub ust_rate: u64,
    pub usdc_rate: u64,
    pub msol_rate: u64,
    pub srm_rate: u64,
    pub scnsol_rate: u64,
    pub stsol_rate: u64,
    pub btc_rate: u64,
    pub usdt_rate: u64,
    pub eth_rate: u64,
    // 
    pub last_mint_timestamp: i64
}

#[account]
#[derive(Default)]
pub struct UserAccount {
    pub owner: Pubkey,
    pub ust_amount: u64,
    pub usdc_amount: u64,
    pub msol_amount: u64,
    pub srm_amount: u64,
    pub scnsol_amount: u64,
    pub stsol_amount: u64,
    pub btc_amount: u64,
    pub usdt_amount: u64,
    pub eth_amount: u64
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
    #[msg("Invalid Token")]
    InvalidToken
}