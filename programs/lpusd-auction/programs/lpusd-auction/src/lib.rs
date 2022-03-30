use anchor_lang::prelude::*;
// use pyth_client;
use anchor_spl::token::{self, Mint, Transfer, Token, TokenAccount };

use cbs_protocol::cpi::accounts::LiquidateCollateral;
use cbs_protocol::program::CbsProtocol;
use cbs_protocol::{self, UserAccount, StateAccount};

use lpfinance_swap::cpi::accounts::LiquidateToken;
use lpfinance_swap::program::LpfinanceSwap;
use lpfinance_swap::{self};

declare_id!("6KS4ho2CDvr7MGofHU6F6WJfQ5j6DL8nhBWJtkhMTzqt");

const DENOMINATOR:u64 = 100;
const LTV_PERMISSION:u64 = 94;

pub fn get_price(pyth_account: AccountInfo) -> Result<u64> {
    let pyth_price_info = &pyth_account;
    let pyth_price_data = &pyth_price_info.try_borrow_data()?;
    let pyth_price = pyth_client::cast::<pyth_client::Price>(pyth_price_data);
    let price = pyth_price.agg.price as u64;
    Ok(price)
}

#[program]
pub mod lpusd_auction {
    use super::*;
    pub fn initialize(
        ctx: Context<Initialize>,
        auction_name: String,
        bumps: AuctionBumps
    ) -> Result<()> {
        msg!("INITIALIZE Auction");

        let state_account = &mut ctx.accounts.state_account;

        let name_bytes = auction_name.as_bytes();
        let mut name_data = [b' '; 10];
        name_data[..name_bytes.len()].copy_from_slice(name_bytes);

        state_account.auction_name = name_data;
        state_account.bumps = bumps;
        state_account.btc_mint = ctx.accounts.btc_mint.key();
        state_account.usdc_mint = ctx.accounts.usdc_mint.key();
        state_account.lpusd_mint = ctx.accounts.lpusd_mint.key();
        state_account.lpsol_mint = ctx.accounts.lpsol_mint.key();
        state_account.msol_mint = ctx.accounts.msol_mint.key();
        state_account.pool_btc = ctx.accounts.pool_btc.key();
        state_account.pool_usdc = ctx.accounts.pool_usdc.key();
        state_account.pool_lpsol = ctx.accounts.pool_lpsol.key();
        state_account.pool_lpusd = ctx.accounts.pool_lpusd.key();
        state_account.pool_msol = ctx.accounts.pool_msol.key();

        state_account.total_percent = 100; // 10000000
        state_account.total_lpusd = 0;
        state_account.epoch_duration = 0;
        state_account.total_deposited_lpusd = 0;
        state_account.last_epoch_percent = 0;
        state_account.last_epoch_profit = 0;

        state_account.owner = ctx.accounts.authority.key();

        Ok(())
    }

    // Init user account
    pub fn init_user_account(
        ctx: Context<InitUserAccount>, 
        bump: u8
    ) -> Result<()> {
        // Make as 1 string for pubkey
        let user_account = &mut ctx.accounts.user_account;
        user_account.owner = ctx.accounts.user_authority.key();
        user_account.bump = bump;

        user_account.lpusd_amount = 0;
        user_account.temp_amount = 0;
        Ok(())
    }

    pub fn deposit_lpusd(
        ctx: Context<DepositLpUSD>,
        amount: u64
    ) -> Result<()> {
        msg!("UserLpUSD Balance: !!{:?}!!", ctx.accounts.user_lpusd.amount);
        if amount < 1 {
            return Err(ErrorCode::InvalidAmount.into());
        }
        if ctx.accounts.user_lpusd.amount < amount {
            return Err(ErrorCode::InsufficientAmount.into());
        }

        let cpi_accounts = Transfer {
            from: ctx.accounts.user_lpusd.to_account_info(),
            to: ctx.accounts.pool_lpusd.to_account_info(),
            authority: ctx.accounts.user_authority.to_account_info()
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        let user_account = &mut ctx.accounts.user_account;
        user_account.temp_amount = user_account.temp_amount + amount;
        user_account.lpusd_amount = (user_account.lpusd_amount * ctx.accounts.state_account.total_percent + amount * DENOMINATOR) / ctx.accounts.state_account.total_percent;

        let state_account = &mut ctx.accounts.state_account;
        state_account.total_lpusd = state_account.total_lpusd + amount;
        state_account.total_deposited_lpusd = state_account.total_deposited_lpusd + amount;

        Ok(())
    }

    pub fn liquidate (
        ctx: Context<Liquidate>
    ) -> Result<()> {
        msg!("Started liquidate");

        let liquidator = &mut ctx.accounts.liquidator;

        let borrowed_lpusd = liquidator.borrowed_lpusd;       
        let borrowed_lpsol = liquidator.borrowed_lpsol;
        let btc_amount = liquidator.btc_amount;
        let sol_amount = liquidator.sol_amount;
        let usdc_amount = liquidator.usdc_amount;
        let lpsol_amount = liquidator.lpsol_amount;
        let lpusd_amount = liquidator.lpusd_amount;
        let msol_amount = liquidator.msol_amount;

        // Stop all diposit and withdraw in cbs For liquidator
        ctx.accounts.cbs_account.liquidation_run = true;
        // Check if withdraw/deposit started or not

        if borrowed_lpusd == 0 && borrowed_lpsol == 0 {
            return Err(ErrorCode::InvalidAmount.into());
        }

        // Fetch the price
        let sol_price: u128 = get_price(ctx.accounts.pyth_sol_account.to_account_info())? as u128;
        let usdc_price: u128 = get_price(ctx.accounts.pyth_usdc_account.to_account_info())? as u128;
        let btc_price: u128 = get_price(ctx.accounts.pyth_btc_account.to_account_info())? as u128;
        let msol_price: u128 = get_price(ctx.accounts.pyth_msol_account.to_account_info())? as u128;

        // Total Deposited Price
        let mut total_price: u128 = 0;
        total_price += sol_price * sol_amount as u128;
        total_price += sol_price * lpsol_amount as u128;
        total_price += btc_price * btc_amount as u128;
        total_price += msol_price * msol_amount as u128;
        total_price += usdc_price * lpusd_amount as u128;
        total_price += usdc_price * usdc_amount as u128;
        // Total Borrowed Price 
        let total_borrowed_price:u128 = borrowed_lpusd as u128 * usdc_price + borrowed_lpsol as u128 * sol_price;

        // LTV should be > 94
        // Formula: LTV = (total_borrowed_price / total_price) * 100 > 94
        if total_price * LTV_PERMISSION as u128 >= total_borrowed_price * 100{
            return Err(ErrorCode::NotEnoughLTV.into());
        }

        let seeds = &[
            ctx.accounts.auction_account.auction_name.as_ref(),
            &[ctx.accounts.auction_account.bumps.state_account],
        ];
        let signer = &[&seeds[..]];

        if borrowed_lpusd > 0 {
            if borrowed_lpusd > ctx.accounts.auction_lpusd.amount {
                return Err(ErrorCode::InsufficientPoolAmount.into());            
            }
            // Transfer lpusd from auction to cbs
            let cpi_accounts = Transfer {
                from: ctx.accounts.auction_lpusd.to_account_info(),
                to: ctx.accounts.cbs_lpusd.to_account_info(),
                authority: ctx.accounts.auction_account.to_account_info()
            };
    
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, borrowed_lpusd)?;
        }

        // Transfer all collaterals from cbs to auction
        {
            let cpi_program = ctx.accounts.cbs_program.to_account_info();
            let cpi_accounts = LiquidateCollateral {
                user_account: ctx.accounts.liquidator.to_account_info(),
                state_account: ctx.accounts.cbs_account.to_account_info(),
                auction_account: ctx.accounts.auction_account.to_account_info(),
                auction_lpusd: ctx.accounts.auction_lpusd.to_account_info(),
                auction_lpsol: ctx.accounts.auction_lpsol.to_account_info(),
                auction_msol: ctx.accounts.auction_msol.to_account_info(),
                auction_btc: ctx.accounts.auction_btc.to_account_info(),
                auction_usdc: ctx.accounts.auction_usdc.to_account_info(),
                cbs_lpusd: ctx.accounts.cbs_lpusd.to_account_info(),
                cbs_lpsol: ctx.accounts.cbs_lpsol.to_account_info(),
                cbs_usdc: ctx.accounts.cbs_usdc.to_account_info(),
                cbs_btc: ctx.accounts.cbs_btc.to_account_info(),
                cbs_msol: ctx.accounts.cbs_msol.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info()
            };
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
            cbs_protocol::cpi::liquidate_collateral(cpi_ctx)?;
        }
 

        // Liquidate LpSOL (Swap LpUSD to LpSOL and transfer LpSOL to CBS)
        if borrowed_lpsol > 0 {            
            let transfer_amount = (sol_price * borrowed_lpsol as u128 / usdc_price) as u64;

            let cpi_program = ctx.accounts.swap_program.to_account_info();
            let cpi_accounts = LiquidateToken {
                state_account: ctx.accounts.auction_account.to_account_info(),
                auction_pool: ctx.accounts.cbs_lpsol.to_account_info(),
                swap_pool: ctx.accounts.swap_lpsol.to_account_info(),
                dest_mint: ctx.accounts.lpsol_mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info()
            };
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
            lpfinance_swap::cpi::liquidate_token(cpi_ctx, borrowed_lpsol)?;

            let cpi_accounts = Transfer {
                from: ctx.accounts.auction_lpusd.to_account_info(),
                to: ctx.accounts.swap_lpusd.to_account_info(),
                authority: ctx.accounts.auction_account.to_account_info()
            };
    
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, transfer_amount)?;
        }
  
        // BTC
        if btc_amount > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.auction_btc.to_account_info(),
                to: ctx.accounts.swap_btc.to_account_info(),
                authority: ctx.accounts.auction_account.to_account_info()
            };
    
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, btc_amount)?;
        }

        // LpSOL
        if lpsol_amount > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.auction_lpsol.to_account_info(),
                to: ctx.accounts.swap_lpsol.to_account_info(),
                authority: ctx.accounts.auction_account.to_account_info()
            };
    
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, lpsol_amount)?;
        }
        
        // mSOL
        if msol_amount > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.auction_msol.to_account_info(),
                to: ctx.accounts.swap_msol.to_account_info(),
                authority: ctx.accounts.auction_account.to_account_info()
            };
    
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, msol_amount)?;
        }
        
        // USDC 
        if usdc_amount > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.auction_usdc.to_account_info(),
                to: ctx.accounts.swap_usdc.to_account_info(),
                authority: ctx.accounts.auction_account.to_account_info()
            };
    
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, usdc_amount)?;
        }

        // Request LpUSD amount from SWAP to AUCTION
        let total_request_lpusd = ((total_price - usdc_price * lpusd_amount as u128) / usdc_price) as u64;
        
        if total_request_lpusd == 0 {
            return Err(ErrorCode::InvalidAmount.into());
        } else {

            let transfer_amount = total_request_lpusd;

            let cpi_program = ctx.accounts.swap_program.to_account_info();
            let cpi_accounts = LiquidateToken {
                state_account: ctx.accounts.swap_account.to_account_info(),
                auction_pool: ctx.accounts.auction_lpusd.to_account_info(),
                swap_pool: ctx.accounts.swap_lpusd.to_account_info(),
                dest_mint: ctx.accounts.lpusd_mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info()
            };
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
            lpfinance_swap::cpi::liquidate_token(cpi_ctx, transfer_amount)?;

        }

        // SOL transfer
        if sol_amount > 0 {
            **ctx.accounts.auction_account.to_account_info().try_borrow_mut_lamports()? -= sol_amount;
            **ctx.accounts.swap_account.to_account_info().try_borrow_mut_lamports()? += sol_amount;
        }

        let reward = (total_price - total_borrowed_price) / usdc_price;
        
        let auction_account = &mut ctx.accounts.auction_account;
        let total_amount = auction_account.total_lpusd + reward as u64;
        let auction_percent = auction_account.total_percent as u128 * total_amount as u128 / auction_account.total_lpusd as u128;

        auction_account.last_epoch_percent = total_amount * 100 / auction_account.total_lpusd;
        auction_account.last_epoch_profit = reward as u64;
        auction_account.total_lpusd = total_amount;
        auction_account.total_percent = auction_percent as u64;
        
        // Make CBS working again
        ctx.accounts.cbs_account.liquidation_run = false;

        Ok(())
    }

    pub fn withdraw_lpusd(        
        ctx: Context<WithdrawLpUSD>,
        amount: u64
    ) -> Result<()> {
        // NOTE: check if able to withdraw
        if amount < 1 {
            return Err(ErrorCode::InvalidAmount.into());
        }

        let user_account = &mut ctx.accounts.user_account;
        let state_account = &mut ctx.accounts.state_account;

        let total_withdrawable_amount = user_account.lpusd_amount * state_account.total_percent / DENOMINATOR;
        msg!("Total withdraw amount: !!{:?}!!", total_withdrawable_amount.to_string());
        msg!("pool_lpusd amount: !!{:?}!!", ctx.accounts.pool_lpusd.amount.to_string());
        if ctx.accounts.pool_lpusd.amount < amount {
            return Err(ErrorCode::InsufficientPoolAmount.into());
        }

        if amount > total_withdrawable_amount {
            return Err(ErrorCode::ExceedAmount.into());
        }

        let seeds = &[
            ctx.accounts.state_account.auction_name.as_ref(),
            &[ctx.accounts.state_account.bumps.state_account],
        ];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.pool_lpusd.to_account_info(),
            to: ctx.accounts.user_lpusd.to_account_info(),
            authority: ctx.accounts.state_account.to_account_info()
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, amount)?;
        
        let state_account = &mut ctx.accounts.state_account;

        state_account.total_lpusd = state_account.total_lpusd - user_account.lpusd_amount;

        // Init user account
        user_account.lpusd_amount = (user_account.lpusd_amount * ctx.accounts.state_account.total_percent - amount * DENOMINATOR) / ctx.accounts.state_account.total_percent;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(auction_name: String, bumps: AuctionBumps)]
pub struct Initialize <'info>{
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(init,
        seeds = [auction_name.as_bytes()],
        bump,
        payer = authority
    )]
    pub state_account: Box<Account<'info, AuctionStateAccount>>,
    pub usdc_mint: Box<Account<'info, Mint>>,
    pub btc_mint: Box<Account<'info, Mint>>,
    pub msol_mint: Box<Account<'info, Mint>>,
    pub lpusd_mint: Box<Account<'info,Mint>>,
    pub lpsol_mint: Box<Account<'info,Mint>>,
    // USDC POOL
    #[account(
        init,
        token::mint = usdc_mint,
        token::authority = state_account,
        seeds = [auction_name.as_bytes(), b"pool_usdc".as_ref()],
        bump,
        payer = authority
    )]
    pub pool_usdc: Box<Account<'info, TokenAccount>>,
    // BTC POOL
    #[account(
        init,
        token::mint = btc_mint,
        token::authority = state_account,
        seeds = [auction_name.as_bytes(), b"pool_btc".as_ref()],
        bump,
        payer = authority
    )]
    pub pool_btc: Box<Account<'info, TokenAccount>>,
    // LpUSD POOL
    #[account(
        init,
        token::mint = lpusd_mint,
        token::authority = state_account,
        seeds = [auction_name.as_bytes(), b"pool_lpusd".as_ref()],
        bump,
        payer = authority
    )]
    pub pool_lpusd: Box<Account<'info, TokenAccount>>,
    // LpSOL POOL
    #[account(
        init,
        token::mint = lpsol_mint,
        token::authority = state_account,
        seeds = [auction_name.as_bytes(), b"pool_lpsol".as_ref()],
        bump,
        payer = authority
    )]
    pub pool_lpsol: Box<Account<'info, TokenAccount>>,
    // mSOL POOL
    #[account(
        init,
        token::mint = msol_mint,
        token::authority = state_account,
        seeds = [auction_name.as_bytes(), b"pool_msol".as_ref()],
        bump,
        payer = authority
    )]
    pub pool_msol: Box<Account<'info, TokenAccount>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct InitUserAccount<'info> {
    // State account for each user/wallet
    #[account(
        init,
        seeds = [state_account.auction_name.as_ref(), user_authority.key().as_ref()],
        bump,
        payer = user_authority
    )]
    pub user_account: Account<'info, UserStateAccount>,
    #[account(mut)]
    pub state_account: Box<Account<'info, AuctionStateAccount>>,
    // Contract Authority accounts
    #[account(mut)]
    pub user_authority: Signer<'info>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct DepositLpUSD<'info> {
    #[account(mut)]
    pub user_authority: Signer<'info>,
    #[account(
        mut,
        constraint = user_lpusd.owner == user_authority.key(),
        constraint = user_lpusd.mint == lpusd_mint.key()
    )]
    pub user_lpusd: Box<Account<'info, TokenAccount>>,
    pub lpusd_mint: Account<'info, Mint>,
    #[account(mut,
        seeds = [state_account.auction_name.as_ref()],
        bump = state_account.bumps.state_account
    )]
    pub state_account: Box<Account<'info, AuctionStateAccount>>,
    #[account(mut,
        seeds = [state_account.auction_name.as_ref(), b"pool_lpusd".as_ref()],
        bump = state_account.bumps.pool_lpusd
    )]
    pub pool_lpusd: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = user_account.owner == user_authority.key()
    )]
    pub user_account: Box<Account<'info, UserStateAccount>>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct Liquidate<'info> {
    #[account(mut)]
    pub user_authority: Signer<'info>,
    #[account(mut,
        seeds = [auction_account.auction_name.as_ref()],
        bump = auction_account.bumps.state_account)]
    pub auction_account: Box<Account<'info, AuctionStateAccount>>,
    // UserAccount from CBS protocol
    #[account(mut)]
    pub liquidator: Box<Account<'info, UserAccount>>,
    #[account(mut)]
    pub cbs_account: Box<Account<'info, StateAccount>>,
    #[account(mut)]
    pub swap_account: Box<Account<'info, lpfinance_swap::StateAccount>>,
    pub cbs_program: Program<'info, CbsProtocol>,
    pub swap_program: Program<'info, LpfinanceSwap>,
    #[account(mut)]
    pub swap_lpusd: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub swap_lpsol: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub swap_btc: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub swap_usdc: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub swap_msol: Box<Account<'info, TokenAccount>>,

    // #[account(mut)]
    // pub btc_mint: Box<Account<'info,Mint>>,
    // #[account(mut)]
    // pub usdc_mint: Box<Account<'info,Mint>>,
    #[account(mut)]
    pub lpsol_mint: Box<Account<'info,Mint>>,
    #[account(mut)]
    pub lpusd_mint: Box<Account<'info,Mint>>,

    #[account(mut)]
    pub auction_lpusd: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub auction_lpsol: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub auction_btc: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub auction_usdc: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub auction_msol: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub cbs_lpusd: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub cbs_lpsol: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub cbs_msol: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub cbs_usdc: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub cbs_btc: Box<Account<'info, TokenAccount>>,
    // pyth
    pub pyth_btc_account: AccountInfo<'info>,
    pub pyth_usdc_account: AccountInfo<'info>,
    pub pyth_sol_account: AccountInfo<'info>,
    pub pyth_msol_account: AccountInfo<'info>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct WithdrawLpUSD<'info> {
    #[account(mut)]
    pub user_authority: Signer<'info>,
    #[account(
        mut,
        constraint = user_lpusd.owner == user_authority.key(),
        constraint = user_lpusd.mint == lpusd_mint.key()
    )]
    pub user_lpusd: Box<Account<'info, TokenAccount>>,
    pub lpusd_mint: Account<'info, Mint>,
    #[account(mut,
        seeds = [state_account.auction_name.as_ref()],
        bump = state_account.bumps.state_account
    )]
    pub state_account: Box<Account<'info, AuctionStateAccount>>,
    #[account(mut,
        seeds = [state_account.auction_name.as_ref(), b"pool_lpusd".as_ref()],
        bump = state_account.bumps.pool_lpusd
    )]
    pub pool_lpusd: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = user_account.owner == user_authority.key()
    )]
    pub user_account: Box<Account<'info, UserStateAccount>>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}

#[account]
#[derive(Default)]
pub struct UserStateAccount {
    // deposited lpusd
    // NOTE: only lpusd is able to be deposited
    pub lpusd_amount: u64,
    pub owner: Pubkey,
    pub bump: u8,
    pub temp_amount: u64
}

#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone)]
pub struct AuctionBumps{
    pub state_account: u8,
    pub pool_usdc: u8,
    pub pool_btc: u8,
    pub pool_lpusd: u8,
    pub pool_lpsol: u8,
    pub pool_msol: u8,
}

#[account]
#[derive(Default)]
pub struct AuctionStateAccount {
    pub auction_name: [u8; 10],
    pub bumps: AuctionBumps,
    pub owner: Pubkey,
    pub lpsol_mint: Pubkey,
    pub lpusd_mint: Pubkey,
    pub msol_mint: Pubkey,
    pub btc_mint: Pubkey,
    pub usdc_mint: Pubkey,
    pub pool_btc: Pubkey,
    pub pool_usdc: Pubkey,
    pub pool_lpsol: Pubkey,
    pub pool_lpusd: Pubkey,
    pub pool_msol: Pubkey,

    pub total_deposited_lpusd: u64,
    pub total_lpusd: u64,
    pub total_percent: u64,
    pub epoch_duration: u64,
    pub last_epoch_percent: u64,
    pub last_epoch_profit: u64
}

#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient User's Amount")]
    InsufficientAmount,
    #[msg("Insufficient Pool's Amount")]
    InsufficientPoolAmount,
    #[msg("Invalid Amount")]
    InvalidAmount,
    #[msg("Exceed Amount")]
    ExceedAmount,
    #[msg("Not Enough For LTV")]
    NotEnoughLTV
}