use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, MintTo, Burn, Token, TokenAccount }
};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

const PREFIX: &str = "lendtokens";
const TOKEN_DECIMALS: u8 = 9;

#[program]
pub mod lending_tokens {
    use super::*;
    pub fn initialize(
        ctx: Context<Initialize>
    ) -> Result<()> {
        msg!("INITIALIZE TOKEN PROGRAM");

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

        config.state_account = ctx.accounts.state_account.key();
        config.last_mint_timestamp = 0;

        // INITIAL SUPPLY
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
            mint: ctx.accounts.lpdao_mint.to_account_info(),
            to: ctx.accounts.user_daotoken.to_account_info(),
            authority: ctx.accounts.state_account.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

        token::mint_to(cpi_ctx, INITIAL_SUPPLY * 1000000000)?;

        Ok(())
    }
    
    pub fn burn_token(
        ctx: Context<BurnToken>,
        amount: u64
    ) -> Result<()> {
        if amount == 0 {
            return Err(ErrorCode::InvalidAmount.into());
        }

        let cpi_accounts = Burn {
            mint: ctx.accounts.token_mint.to_account_info(),
            to: ctx.accounts.user_token.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        token::burn(cpi_ctx, amount)?;
        Ok(())
    }

    pub fn mint_token(
        ctx: Context<MintToken>,
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
            mint: ctx.accounts.token_mint.to_account_info(),
            to: ctx.accounts.user_token.to_account_info(),
            authority: ctx.accounts.state_account.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

        token::mint_to(cpi_ctx, amount)?;
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
        payer = authority
    )]
    pub state_account: Box<Account<'info, TokenStateAccount>>,

    // State Accounts
    #[account(init,
        payer = authority
    )]
    pub config: Box<Account<'info, Config>>,

    #[account(init,
        mint::decimals = TOKEN_DECIMALS,
        mint::authority = state_account,
        seeds = [PREFIX.as_bytes(), b"ust_mint".as_ref()],
        bump,
        payer = authority
    )]
    pub ust_mint: Box<Account<'info, Mint>>,  

    #[account(init,
        mint::decimals = TOKEN_DECIMALS,
        mint::authority = state_account,
        seeds = [PREFIX.as_bytes(), b"usdc_mint".as_ref()],
        bump,
        payer = authority
    )]
    pub usdc_mint: Box<Account<'info, Mint>>,

    #[account(init,
        mint::decimals = TOKEN_DECIMALS,
        mint::authority = state_account,
        seeds = [PREFIX.as_bytes(), b"msol_mint".as_ref()],
        bump,
        payer = authority
    )]
    pub msol_mint: Box<Account<'info, Mint>>,
    #[account(init,
        mint::decimals = TOKEN_DECIMALS,
        mint::authority = state_account,
        seeds = [PREFIX.as_bytes(), b"srm_mint".as_ref()],
        bump,
        payer = authority
    )]
    pub srm_mint: Box<Account<'info, Mint>>,

    #[account(init,
        mint::decimals = TOKEN_DECIMALS,
        mint::authority = state_account,
        seeds = [PREFIX.as_bytes(), b"scnsol_mint".as_ref()],
        bump,
        payer = authority
    )]
    pub scnsol_mint: Box<Account<'info, Mint>>,

    #[account(init,
        mint::decimals = TOKEN_DECIMALS,
        mint::authority = state_account,
        seeds = [PREFIX.as_bytes(), b"stsol_mint".as_ref()],
        bump,
        payer = authority
    )]
    pub stsol_mint: Box<Account<'info, Mint>>,
    
    #[account(init,
        mint::decimals = TOKEN_DECIMALS,
        mint::authority = state_account,
        seeds = [PREFIX.as_bytes(), b"btc_mint".as_ref()],
        bump,
        payer = authority
    )]
    pub btc_mint: Box<Account<'info, Mint>>,

    #[account(init,
        mint::decimals = TOKEN_DECIMALS,
        mint::authority = state_account,
        seeds = [PREFIX.as_bytes(), b"usdt_mint".as_ref()],
        bump,
        payer = authority
    )]
    pub usdt_mint: Box<Account<'info, Mint>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct BurnToken<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut)]
    pub state_account: Box<Account<'info, TokenStateAccount>>,
    #[account(mut)]
    pub config: Box<Account<'info, Config>>,
    #[account(
        mut,
        constraint = user_token.mint == token_mint.key(),
        constraint = user_token.owner == owner.key()
    )]
    pub user_token: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub token_mint: Account<'info, Mint>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct MintToken<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut)]
    pub state_account: Box<Account<'info, TokenStateAccount>>,
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = token_mint,
        associated_token::authority = owner
    )]
    pub user_token: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub token_mint: Account<'info, Mint>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>
}


#[derive(Accounts)]
pub struct UpdateConfigAccount<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut, has_one = owner)]
    pub state_account: Box<Account<'info, TokenStateAccount>>,
    #[account(mut, has_one = state_account)]
    pub config: Box<Account<'info, Config>>
}

#[account]
#[derive(Default)]
pub struct TokenStateAccount {
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
