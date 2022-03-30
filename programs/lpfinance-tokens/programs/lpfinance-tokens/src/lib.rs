use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, MintTo, Burn, Token, TokenAccount }
};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

const LP_TOKEN_DECIMALS: u8 = 9;
const PREFIX: &str = "lpfinance";
const INITIAL_SUPPLY: u64 = 500000000;
const DAY_IN_SECONDS: i64 = 86400; 

// Reward Rate => 0.00809%
// so need to divide with 10000
const DAILY_REWARD_RATE: u64 = 809;
const DENOMINATOR: u64 = 100000 * 100;

#[program]
pub mod lpfinance_tokens {
    use super::*;
    pub fn initialize(
        ctx: Context<Initialize>
    ) -> Result<()> {
        msg!("INITIALIZE TOKEN PROGRAM");

        let state_account = &mut ctx.accounts.state_account;
        let config = &mut ctx.accounts.config;

        state_account.owner = ctx.accounts.authority.key();

        config.lpbtc_mint = ctx.accounts.lpbtc_mint.key();
        config.lpsol_mint = ctx.accounts.lpsol_mint.key();
        config.lpusd_mint = ctx.accounts.lpusd_mint.key();
        config.state_account = ctx.accounts.state_account.key();
        config.last_mint_timestamp = 0;

        Ok(())
    }

    pub fn mint_lptoken(
        ctx: Context<MintLpToken>,
        amount: u64
    ) -> Result<()> {
        if amount == 0 {
            return Err(ErrorCode::InvalidAmount.into());
        }

        let (mint_token_authority, mint_token_authority_bump) = 
            Pubkey::find_program_address(&[PREFIX.as_bytes()], ctx.program_id);
        
        if mint_token_authority != ctx.accounts.state_account.key() {
            return Err(ErrorCode::InvalidOwner.into());
        }

        // Mint
        let seeds = &[
            PREFIX.as_bytes(),
            &[mint_token_authority_bump]
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = MintTo {
            mint: ctx.accounts.lptoken_mint.to_account_info(),
            to: ctx.accounts.cbs_lptoken.to_account_info(),
            authority: ctx.accounts.state_account.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

        token::mint_to(cpi_ctx, amount)?;
        Ok(())
    }

    pub fn burn_lptoken(
        ctx: Context<BurnLpToken>,
        amount: u64
    ) -> Result<()> {
        if amount == 0 {
            return Err(ErrorCode::InvalidAmount.into());
        }

        let (mint_token_authority, mint_token_authority_bump) = 
            Pubkey::find_program_address(&[PREFIX.as_bytes()], ctx.program_id);
        
        if mint_token_authority != ctx.accounts.state_account.key() {
            return Err(ErrorCode::InvalidOwner.into());
        }

        // Mint
        let seeds = &[
            PREFIX.as_bytes(),
            &[mint_token_authority_bump]
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = Burn {
            mint: ctx.accounts.lptoken_mint.to_account_info(),
            to: ctx.accounts.cbs_lptoken.to_account_info(),
            authority: ctx.accounts.state_account.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

        token::burn(cpi_ctx, amount)?;
        Ok(())
    }

    pub fn owner_mint_lptoken(
        ctx: Context<OwnerLpToken>,
        amount: u64
    ) -> Result<()> {
        if amount == 0 {
            return Err(ErrorCode::InvalidAmount.into());
        }

        if ctx.accounts.state_account.owner != ctx.accounts.owner.key() {
            return Err(ErrorCode::InvalidOwner.into());
        }

        let (mint_token_authority, mint_token_authority_bump) = 
        Pubkey::find_program_address(&[PREFIX.as_bytes()], ctx.program_id);
    
        if mint_token_authority != ctx.accounts.state_account.key() {
            return Err(ErrorCode::InvalidOwner.into());
        }

        // Mint
        let seeds = &[
            PREFIX.as_bytes(),
            &[mint_token_authority_bump]
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = MintTo {
            mint: ctx.accounts.lptoken_mint.to_account_info(),
            to: ctx.accounts.user_lptoken.to_account_info(),
            authority: ctx.accounts.state_account.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

        token::mint_to(cpi_ctx, amount)?;
        Ok(())
    }

    pub fn owner_burn_lptoken(
        ctx: Context<OwnerLpToken>,
        amount: u64
    ) -> Result<()> {
        if amount == 0 {
            return Err(ErrorCode::InvalidAmount.into());
        }

        if ctx.accounts.state_account.owner != ctx.accounts.owner.key() {
            return Err(ErrorCode::InvalidOwner.into());
        }

        // Burn
        let cpi_accounts = Burn {
            mint: ctx.accounts.lptoken_mint.to_account_info(),
            to: ctx.accounts.user_lptoken.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        token::burn(cpi_ctx, amount)?;

        Ok(())
    }

    pub fn update_cbs_account(
        ctx: Context<UpdateStateAccount>,
        new_cbs: Pubkey
    ) -> Result<()> {
        let state_account = &mut ctx.accounts.state_account;
        
        if state_account.owner != ctx.accounts.owner.key() {
            return Err(ErrorCode::InvalidOwner.into());
        }

        state_account.cbs_account = new_cbs;
        Ok(())
    }

    pub fn update_owner(
        ctx: Context<UpdateStateAccount>,
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
        ctx: Context<UpdateStateAccount>,
        new_owner: Pubkey
    ) -> Result<()> {
        let state_account = &mut ctx.accounts.state_account;
        if state_account.owner != ctx.accounts.owner.key() || state_account.second_owner == new_owner {
            return Err(ErrorCode::InvalidOwner.into());
        }

        state_account.second_owner = new_owner;

        Ok(())
    }

    pub fn mint_dao_lptoken (
        ctx: Context<MintDaoLpToken>
    ) -> Result<()> {
        let total_supply = ctx.accounts.lptoken_mint.supply;
        let config = &mut ctx.accounts.config;

        if total_supply == 0 {
            return Err(ErrorCode::InvalidAmount.into());
        }

        if ctx.accounts.state_account.second_owner != ctx.accounts.owner.key() {
            return Err(ErrorCode::InvalidOwner.into());
        }

        let clock = Clock::get()?; // Returns real-world time in second uint
        let dur_seconds = clock.unix_timestamp  - config.last_mint_timestamp ;
        if dur_seconds < DAY_IN_SECONDS {
            return Err(ErrorCode::TooOftenMint.into());
        }
        config.last_mint_timestamp = clock.unix_timestamp;

        let mint_amount = total_supply * DAILY_REWARD_RATE / DENOMINATOR;


        let (mint_token_authority, mint_token_authority_bump) = 
        Pubkey::find_program_address(&[PREFIX.as_bytes()], ctx.program_id);
    
        if mint_token_authority != ctx.accounts.state_account.key() {
            return Err(ErrorCode::InvalidOwner.into());
        }

        // Mint
        let seeds = &[
            PREFIX.as_bytes(),
            &[mint_token_authority_bump]
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = MintTo {
            mint: ctx.accounts.lptoken_mint.to_account_info(),
            to: ctx.accounts.user_lptoken.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

        token::mint_to(cpi_ctx, mint_amount)?;
        Ok(())
    }
}


#[derive(Accounts)]
#[instruction(program_name: String)]
pub struct Initialize<'info> {
    // Token program owner
    #[account(mut)]
    pub authority: Signer<'info>,
    // State Accounts
    #[account(init,
        payer = authority
    )]
    pub state_account: Box<Account<'info, TokenStateAccount>>,

    // State Accounts
    #[account(init,
        payer = authority
    )]
    pub config: Box<Account<'info, Config>>,

    #[account(init,
        mint::decimals = LP_TOKEN_DECIMALS,
        mint::authority = state_account,
        seeds = [PREFIX.as_bytes(), b"lpsol_mint".as_ref()],
        bump,
        payer = authority
    )]
    pub lpsol_mint: Box<Account<'info, Mint>>,  

    #[account(init,
        mint::decimals = LP_TOKEN_DECIMALS,
        mint::authority = state_account,
        seeds = [PREFIX.as_bytes(), b"lpusd_mint".as_ref()],
        bump,
        payer = authority
    )]
    pub lpusd_mint: Box<Account<'info, Mint>>,

    #[account(init,
        mint::decimals = LP_TOKEN_DECIMALS,
        mint::authority = state_account,
        seeds = [PREFIX.as_bytes(), b"lpbtc_mint".as_ref()],
        bump,
        payer = authority
    )]
    pub lpbtc_mint: Box<Account<'info, Mint>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct MintLpToken<'info> {
    #[account(mut)]
    pub cbs_account: Signer<'info>,
    #[account(mut, has_one = cbs_account)]
    pub state_account: Box<Account<'info, TokenStateAccount>>,
    #[account(
        mut,
        constraint = cbs_lptoken.mint == lptoken_mint.key(),
        constraint = cbs_lptoken.owner == cbs_account.key()
    )]
    pub cbs_lptoken: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub lptoken_mint: Account<'info, Mint>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct BurnLpToken<'info> {
    #[account(mut)]
    pub cbs_account: Signer<'info>,
    #[account(mut, has_one = cbs_account)]
    pub state_account: Box<Account<'info, TokenStateAccount>>,
    #[account(
        mut,
        constraint = cbs_lptoken.mint == lptoken_mint.key(),
        constraint = cbs_lptoken.owner == cbs_account.key()
    )]
    pub cbs_lptoken: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub lptoken_mint: Account<'info, Mint>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct OwnerLpToken<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut, has_one = owner)]
    pub state_account: Box<Account<'info, TokenStateAccount>>,
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = lptoken_mint,
        associated_token::authority = owner
    )]
    pub user_lptoken: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub lptoken_mint: Account<'info, Mint>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct MintDaoLpToken<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut, has_one = owner)]
    pub state_account: Box<Account<'info, TokenStateAccount>>,
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = lptoken_mint,
        associated_token::authority = owner
    )]
    pub user_lptoken: Box<Account<'info, TokenAccount>>,
    #[account(mut, has_one = state_account)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub lptoken_mint: Account<'info, Mint>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct UpdateStateAccount<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut, has_one = owner)]
    pub state_account: Box<Account<'info, TokenStateAccount>>
}

#[account]
#[derive(Default)]
pub struct TokenStateAccount {
    pub owner: Pubkey,
    pub cbs_account: Pubkey,
    pub second_owner: Pubkey
}

#[account]
#[derive(Default)]
pub struct Config {
    pub state_account: Pubkey,
    pub lpbtc_mint: Pubkey,
    pub lpusd_mint: Pubkey,
    pub lpsol_mint: Pubkey,
    pub last_mint_timestamp: i64
}

#[derive(AnchorDeserialize, AnchorSerialize, Default, Clone)]
pub struct ProgramBumps {
    pub state_account: u8,
    pub lpbtc_mint: u8,
    pub lpsol_mint: u8,
    pub lpusd_mint: u8
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid Amount")]
    InvalidAmount,
    #[msg("Invalid Owner")]
    InvalidOwner,
    #[msg("Too often mint")]
    TooOftenMint
}