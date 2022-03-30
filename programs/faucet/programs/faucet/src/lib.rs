use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Transfer, Token, TokenAccount };
use anchor_spl::associated_token::AssociatedToken;

declare_id!("7G5gNHT2T2fdb9EHY5K6FEPc6Z1mAF8M5uhURWDbsBjE");

#[program]
pub mod faucet {
    use super::*;
    pub fn initialize(
        ctx: Context<Initialize>,
        faucet_name: String,
        bumps: FaucetBumps
    ) -> Result<()> {
        msg!("INITIALIZE Auction");

        let state_account = &mut ctx.accounts.state_account;

        let name_bytes = faucet_name.as_bytes();
        let mut name_data = [b' '; 10];
        name_data[..name_bytes.len()].copy_from_slice(name_bytes);

        state_account.faucet_name = name_data;
        state_account.bumps = bumps;
        state_account.tbtc_mint = ctx.accounts.tbtc_mint.key();
        state_account.tusdc_mint = ctx.accounts.tusdc_mint.key();
        state_account.tmsol_mint = ctx.accounts.tmsol_mint.key();
        state_account.pool_tbtc = ctx.accounts.pool_tbtc.key();
        state_account.pool_tusdc = ctx.accounts.pool_tusdc.key();
        state_account.pool_tmsol = ctx.accounts.pool_tmsol.key();
        state_account.tbtc_amount = 200000000;
        state_account.tusdc_amount = 1000000000000;
        state_account.tmsol_amount = 10000000000;

        state_account.owner = ctx.accounts.authority.key();
        Ok(())
    }

    pub fn request_token(
        ctx: Context<RequestToken>
    ) -> Result<()> {
        let transfer_amount;

        if ctx.accounts.token_mint.to_account_info().key() == ctx.accounts.state_account.tbtc_mint {
            transfer_amount = ctx.accounts.state_account.tbtc_amount;
        } else if ctx.accounts.token_mint.to_account_info().key() == ctx.accounts.state_account.tusdc_mint {
            transfer_amount = ctx.accounts.state_account.tusdc_amount;
        } else if ctx.accounts.token_mint.to_account_info().key() == ctx.accounts.state_account.tmsol_mint {
            transfer_amount = ctx.accounts.state_account.tmsol_amount;
        } else {
            return Err(ErrorCode::InvalidToken.into());
        }

        if ctx.accounts.pool_token.amount < transfer_amount {
            return Err(ErrorCode::InsufficientPoolAmount.into());
        }
        let seeds = &[
            ctx.accounts.state_account.faucet_name.as_ref(),
            &[ctx.accounts.state_account.bumps.state_account],
        ];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.pool_token.to_account_info(),
            to: ctx.accounts.user_token.to_account_info(),
            authority: ctx.accounts.state_account.to_account_info()
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

        token::transfer(cpi_ctx, transfer_amount)?;

        Ok(())
    }

    pub fn withdraw_token(ctx: Context<WithdrawToken>) -> Result<()> {
        if ctx.accounts.state_account.owner != ctx.accounts.user_authority.to_account_info().key() {
            return Err(ErrorCode::NotAllowed.into());
        }
        if ctx.accounts.pool_token.amount < 1 {
            return Err(ErrorCode::InsufficientPoolAmount.into());
        }

        let seeds = &[
            ctx.accounts.state_account.faucet_name.as_ref(),
            &[ctx.accounts.state_account.bumps.state_account],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.pool_token.to_account_info(),
            to: ctx.accounts.user_token.to_account_info(),
            authority: ctx.accounts.state_account.to_account_info()
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

        token::transfer(cpi_ctx, ctx.accounts.pool_token.amount)?;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(faucet_name: String, bumps: FaucetBumps)]
pub struct Initialize <'info>{
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(init,
        seeds = [faucet_name.as_bytes()],
        bump,
        payer = authority
    )]
    pub state_account: Box<Account<'info, StateAccount>>,
    pub tusdc_mint: Box<Account<'info, Mint>>,
    pub tbtc_mint: Box<Account<'info, Mint>>,
    pub tmsol_mint: Box<Account<'info, Mint>>,
    // tUSDC POOL
    #[account(
        init,
        token::mint = tusdc_mint,
        token::authority = state_account,
        seeds = [faucet_name.as_bytes(), b"pool_tusdc".as_ref()],
        bump,
        payer = authority
    )]
    pub pool_tusdc: Account<'info, TokenAccount>,
    // tBTC POOL
    #[account(
        init,
        token::mint = tbtc_mint,
        token::authority = state_account,
        seeds = [faucet_name.as_bytes(), b"pool_tbtc".as_ref()],
        bump,
        payer = authority
    )]
    pub pool_tbtc: Account<'info, TokenAccount>,
    // tBTC POOL
    #[account(
        init,
        token::mint = tmsol_mint,
        token::authority = state_account,
        seeds = [faucet_name.as_bytes(), b"pool_tmsol".as_ref()],
        bump,
        payer = authority
    )]
    pub pool_tmsol: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct RequestToken<'info> {
    #[account(mut)]
    pub user_authority: Signer<'info>,
    #[account(
        init_if_needed,
        payer = user_authority,
        associated_token::mint = token_mint,
        associated_token::authority = user_authority
    )]
    pub user_token: Box<Account<'info, TokenAccount>>,
    pub token_mint: Account<'info, Mint>,
    #[account(mut,
        seeds = [state_account.faucet_name.as_ref()],
        bump = state_account.bumps.state_account
    )]
    pub state_account: Box<Account<'info, StateAccount>>,
    #[account(mut)]
    pub pool_token: Box<Account<'info, TokenAccount>>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct WithdrawToken<'info> {
    #[account(mut)]
    pub user_authority: Signer<'info>,
    #[account(
        init_if_needed,
        payer = user_authority,
        associated_token::mint = token_mint,
        associated_token::authority = user_authority
    )]
    pub user_token: Box<Account<'info, TokenAccount>>,
    pub token_mint: Account<'info, Mint>,
    #[account(mut,
        seeds = [state_account.faucet_name.as_ref()],
        bump = state_account.bumps.state_account
    )]
    pub state_account: Box<Account<'info, StateAccount>>,
    #[account(mut)]
    pub pool_token: Box<Account<'info, TokenAccount>>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>
}

#[account]
#[derive(Default)]
pub struct StateAccount {
    pub faucet_name: [u8; 10],
    pub bumps: FaucetBumps,
    pub owner: Pubkey,
    pub tbtc_mint: Pubkey,
    pub tusdc_mint: Pubkey,
    pub tmsol_mint: Pubkey,    
    pub pool_tbtc: Pubkey,
    pub pool_tusdc: Pubkey,
    pub pool_tmsol: Pubkey,
    pub tbtc_amount: u64,
    pub tusdc_amount: u64, 
    pub tmsol_amount: u64
}

#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone)]
pub struct FaucetBumps{
    pub state_account: u8,
    pub pool_tusdc: u8,
    pub pool_tbtc: u8,
    pub pool_tmsol: u8
}

#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient Pool's Amount")]
    InsufficientPoolAmount,
    #[msg("Wrong Token Address")]
    InvalidToken,
    #[msg("Wrong Owner")]
    NotAllowed
}