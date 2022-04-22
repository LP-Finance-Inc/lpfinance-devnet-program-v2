use anchor_lang::prelude::*;
// use pyth_client;
use anchor_spl::token::{self, Mint, Transfer, Token, TokenAccount };

use cbs_protocol::cpi::accounts::{LiquidateCollateral, LiquidateLpTokenCollateral};
use cbs_protocol::program::CbsProtocol;
use cbs_protocol::{self, UserAccount, StateAccount};

use lpfinance_swap::cpi::accounts::LiquidateToken;
use lpfinance_swap::program::LpfinanceSwap;
use lpfinance_swap::{self};

declare_id!("E3tXtRu4xvVCxUHiM9cEMpjhuSUXkNBd3gxr5RdKzSRw");

const DENOMINATOR:u64 = 100;
const LTV_PERMISSION:u64 = 94;

const PREFIX: &str = "lpauction1";

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
    ) -> Result<()> {
        msg!("INITIALIZE Auction");

        let state_account = &mut ctx.accounts.state_account;
        let config = &mut ctx.accounts.config;

        config.lpusd_mint = ctx.accounts.lpusd_mint.key();
        config.lpsol_mint = ctx.accounts.lpsol_mint.key();
        config.lpbtc_mint = ctx.accounts.lpbtc_mint.key();
        config.lpeth_mint = ctx.accounts.lpeth_mint.key();

        config.pool_lpsol = ctx.accounts.pool_lpsol.key();
        config.pool_lpusd = ctx.accounts.pool_lpusd.key();
        config.pool_lpbtc = ctx.accounts.pool_lpbtc.key();
        config.pool_lpeth = ctx.accounts.pool_lpeth.key();

        state_account.owner = ctx.accounts.authority.key();
        config.state_account = ctx.accounts.state_account.key();

        Ok(())
    }

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
    ) -> Result<()> {
        msg!("INITIALIZE Pool");

        let state_account = &mut ctx.accounts.state_account;
        let config = &mut ctx.accounts.config;

        if state_account.owner != ctx.accounts.authority.key() {
            return Err(ErrorCode::InvalidOwner.into());
        }

        config.btc_mint = ctx.accounts.btc_mint.key();
        config.usdc_mint = ctx.accounts.usdc_mint.key();
        config.msol_mint = ctx.accounts.msol_mint.key();
        config.eth_mint = ctx.accounts.eth_mint.key();
        config.ust_mint = ctx.accounts.ust_mint.key();
        config.srm_mint = ctx.accounts.srm_mint.key();
        config.scnsol_mint = ctx.accounts.scnsol_mint.key();
        config.stsol_mint = ctx.accounts.stsol_mint.key();
        config.usdt_mint = ctx.accounts.usdt_mint.key();

        config.pool_btc = ctx.accounts.pool_btc.key();
        config.pool_usdc = ctx.accounts.pool_usdc.key();
        config.pool_msol = ctx.accounts.pool_msol.key();
        config.pool_eth = ctx.accounts.pool_eth.key();
        config.pool_ust = ctx.accounts.pool_ust.key();
        config.pool_srm = ctx.accounts.pool_srm.key();
        config.pool_scnsol = ctx.accounts.pool_scnsol.key();
        config.pool_stsol = ctx.accounts.pool_stsol.key();
        config.pool_usdt = ctx.accounts.pool_usdt.key();

        config.total_percent = 100; // 10000000
        config.total_lpusd = 0;
        config.epoch_duration = 0;
        config.total_deposited_lpusd = 0;
        config.last_epoch_percent = 0;
        config.last_epoch_profit = 0;

        Ok(())
    }

    // Init user account
    pub fn init_user_account(
        ctx: Context<InitUserAccount>
    ) -> Result<()> {
        // Make as 1 string for pubkey
        let user_account = &mut ctx.accounts.user_account;
        user_account.owner = ctx.accounts.user_authority.key();

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
        user_account.lpusd_amount = (user_account.lpusd_amount * ctx.accounts.config.total_percent + amount * DENOMINATOR) / ctx.accounts.config.total_percent;

        let config = &mut ctx.accounts.config;
        config.total_lpusd = config.total_lpusd + amount;
        config.total_deposited_lpusd = config.total_deposited_lpusd + amount;

        Ok(())
    }

    pub fn liquidate_from_cbs(
        ctx: Context<LiquidateFromCBS>
    ) -> Result<()> {

        // Transfer all collaterals from cbs to auction
        msg!("Start LiquidateFromCBS");
        let cpi_program = ctx.accounts.cbs_program.to_account_info();
        let cpi_accounts = LiquidateCollateral {
            user_account: ctx.accounts.liquidator.to_account_info(),
            state_account: ctx.accounts.cbs_account.to_account_info(),
            auction_account: ctx.accounts.state_account.to_account_info(),

            auction_usdc: ctx.accounts.auction_usdc.to_account_info(),
            auction_eth: ctx.accounts.auction_eth.to_account_info(),
            auction_ust: ctx.accounts.auction_ust.to_account_info(),
            auction_srm: ctx.accounts.auction_srm.to_account_info(),
            auction_scnsol: ctx.accounts.auction_srm.to_account_info(),
            auction_stsol: ctx.accounts.auction_srm.to_account_info(),
            auction_usdt: ctx.accounts.auction_srm.to_account_info(),

            // cbs_usdc: ctx.accounts.cbs_usdc.to_account_info(),
            // cbs_eth: ctx.accounts.cbs_eth.to_account_info(),
            // cbs_ust: ctx.accounts.cbs_ust.to_account_info(),
            // cbs_srm: ctx.accounts.cbs_srm.to_account_info(),
            // cbs_scnsol: ctx.accounts.cbs_scnsol.to_account_info(),
            // cbs_stsol: ctx.accounts.cbs_stsol.to_account_info(),
            // cbs_usdt: ctx.accounts.cbs_usdt.to_account_info(),

            system_program: ctx.accounts.system_program.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info()
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        cbs_protocol::cpi::liquidate_collateral(cpi_ctx)?;

        Ok(())
    }

    pub fn liquidate_lptoken_from_cbs(
        ctx: Context<LiquidateLpTokenFromCBS>
    ) -> Result<()> {

        // Transfer all collaterals from cbs to auction
        
        let cpi_program = ctx.accounts.cbs_program.to_account_info();
        let cpi_accounts = LiquidateLpTokenCollateral {
            user_account: ctx.accounts.liquidator.to_account_info(),
            state_account: ctx.accounts.cbs_account.to_account_info(),

            auction_msol: ctx.accounts.auction_msol.to_account_info(),
            auction_btc: ctx.accounts.auction_btc.to_account_info(),
            auction_lpusd: ctx.accounts.auction_lpusd.to_account_info(),
            auction_lpsol: ctx.accounts.auction_lpsol.to_account_info(),
            auction_lpbtc: ctx.accounts.auction_lpbtc.to_account_info(),
            auction_lpeth: ctx.accounts.auction_lpeth.to_account_info(),

            cbs_btc: ctx.accounts.cbs_btc.to_account_info(),
            cbs_msol: ctx.accounts.cbs_msol.to_account_info(),
            cbs_lpusd: ctx.accounts.cbs_lpusd.to_account_info(),
            cbs_lpsol: ctx.accounts.cbs_lpsol.to_account_info(),
            cbs_lpbtc: ctx.accounts.cbs_lpbtc.to_account_info(),
            cbs_lpeth: ctx.accounts.cbs_lpeth.to_account_info(),

            system_program: ctx.accounts.system_program.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info()
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        cbs_protocol::cpi::liquidate_lptoken_collateral(cpi_ctx)?;

        Ok(())
    }

    pub fn liquidate (
        ctx: Context<Liquidate>
    ) -> Result<()> {
        msg!("Started liquidate");

        let liquidator = &mut ctx.accounts.liquidator;

        let borrowed_lpusd = liquidator.borrowed_lpusd;       
        let borrowed_lpsol = liquidator.borrowed_lpsol;
        let borrowed_lpbtc = liquidator.borrowed_lpbtc;       
        let borrowed_lpeth = liquidator.borrowed_lpeth;

        let lpsol_amount = liquidator.lpsol_amount;
        let lpusd_amount = liquidator.lpusd_amount;
        let lpbtc_amount = liquidator.lpbtc_amount;
        let lpeth_amount = liquidator.lpeth_amount;

        if borrowed_lpusd == 0 && borrowed_lpsol == 0 && borrowed_lpbtc == 0 && borrowed_lpeth == 0{
            return Err(ErrorCode::NotBorrowedLpToken.into());
        }

        // Fetch the price
        let sol_price: u128 = get_price(ctx.accounts.pyth_sol_account.to_account_info())? as u128;
        let usdc_price: u128 = get_price(ctx.accounts.pyth_usdc_account.to_account_info())? as u128;
        let btc_price: u128 = get_price(ctx.accounts.pyth_btc_account.to_account_info())? as u128;
        let msol_price: u128 = get_price(ctx.accounts.pyth_msol_account.to_account_info())? as u128;
        let eth_price: u128 = get_price(ctx.accounts.pyth_eth_account.to_account_info())? as u128;
        let ust_price: u128 = get_price(ctx.accounts.pyth_ust_account.to_account_info())? as u128;
        let srm_price: u128 = get_price(ctx.accounts.pyth_srm_account.to_account_info())? as u128;
        let scnsol_price: u128 = get_price(ctx.accounts.pyth_scnsol_account.to_account_info())? as u128;
        let stsol_price: u128 = get_price(ctx.accounts.pyth_stsol_account.to_account_info())? as u128;
        let usdt_price: u128 = get_price(ctx.accounts.pyth_usdt_account.to_account_info())? as u128;

        // Total Deposited Price
        let mut total_price: u128 = 0;
        total_price += sol_price * (liquidator.sol_amount + liquidator.lending_sol_amount) as u128;
        total_price += btc_price * (liquidator.btc_amount + liquidator.lending_btc_amount) as u128;
        total_price += usdc_price * (liquidator.usdc_amount + liquidator.lending_usdc_amount) as u128;
        total_price += eth_price * (liquidator.eth_amount + liquidator.lending_eth_amount) as u128;
        total_price += msol_price * (liquidator.msol_amount + liquidator.lending_msol_amount) as u128;
        total_price += ust_price * (liquidator.ust_amount + liquidator.lending_ust_amount) as u128;
        total_price += srm_price * (liquidator.srm_amount + liquidator.lending_srm_amount) as u128;
        total_price += scnsol_price * (liquidator.scnsol_amount + liquidator.lending_scnsol_amount) as u128;
        total_price += stsol_price * (liquidator.stsol_amount + liquidator.lending_stsol_amount) as u128;
        total_price += usdt_price * (liquidator.usdt_amount + liquidator.lending_usdt_amount) as u128;

        total_price += sol_price * lpsol_amount as u128;
        total_price += btc_price * lpbtc_amount as u128;
        total_price += eth_price * lpeth_amount as u128;
        total_price += usdc_price * lpusd_amount as u128;

        // Total Borrowed Price 
        let total_borrowed_price:u128 = borrowed_lpusd as u128 * usdc_price + 
            borrowed_lpsol as u128 * sol_price + 
            borrowed_lpbtc as u128 * btc_price + 
            borrowed_lpeth as u128 * eth_price;

        // LTV should be > 94
        // Formula: LTV = (total_borrowed_price / total_price) * 100 > 94
        // if total_price * LTV_PERMISSION as u128 >= total_borrowed_price * 100{
        //     return Err(ErrorCode::NotEnoughLTV.into());
        // }

        let (program_authority, program_authority_bump) = 
        Pubkey::find_program_address(&[PREFIX.as_bytes()], ctx.program_id);
    
        if program_authority != ctx.accounts.state_account.to_account_info().key() {
            return Err(ErrorCode::InvalidOwner.into());
        }

        let seeds = &[
            PREFIX.as_bytes(),
            &[program_authority_bump]
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
                authority: ctx.accounts.state_account.to_account_info()
            };
    
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, borrowed_lpusd)?;
        }

 
        let mut lpusd_for_swap = lpusd_amount;
        // Liquidate LpSOL (Swap LpUSD to LpSOL and transfer LpSOL to CBS)
        if borrowed_lpsol > 0 {            
            let transfer_amount = (sol_price * borrowed_lpsol as u128 / usdc_price) as u64;
            lpusd_for_swap += transfer_amount;
            msg!("Started request LpSOL from swap");
            // Request LpSOL from SWAP for sending LpSOL back to CBS
            let cpi_program = ctx.accounts.swap_program.to_account_info();
            let cpi_accounts = LiquidateToken {
                state_account: ctx.accounts.state_account.to_account_info(),
                auction_pool: ctx.accounts.cbs_lpsol.to_account_info(),
                swap_pool: ctx.accounts.swap_lpsol.to_account_info(),
                dest_mint: ctx.accounts.lpsol_mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info()
            };
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
            lpfinance_swap::cpi::liquidate_token(cpi_ctx, borrowed_lpsol)?;
            
        }
  
        // Liquidate LpBTC (Swap LpUSD to LpBTC and transfer LpBTC to CBS)
        if borrowed_lpbtc > 0 {            
            let transfer_amount = (sol_price * borrowed_lpbtc as u128 / usdc_price) as u64;
            lpusd_for_swap += transfer_amount;

            msg!("Started request LpBTC from swap");
            // Request LpBTC from SWAP for sending LpBTC back to CBS
            let cpi_program = ctx.accounts.swap_program.to_account_info();
            let cpi_accounts = LiquidateToken {
                state_account: ctx.accounts.state_account.to_account_info(),
                auction_pool: ctx.accounts.cbs_lpbtc.to_account_info(),
                swap_pool: ctx.accounts.swap_lpbtc.to_account_info(),
                dest_mint: ctx.accounts.lpbtc_mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info()
            };
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
            lpfinance_swap::cpi::liquidate_token(cpi_ctx, borrowed_lpbtc)?;
        }
        
        // Liquidate LpETH (Swap LpUSD to LpETH and transfer LpETH to CBS)
        if borrowed_lpeth > 0 {            
            let transfer_amount = (sol_price * borrowed_lpeth as u128 / usdc_price) as u64;
            lpusd_for_swap += transfer_amount;

            msg!("Started request LpETH from swap");
            // Request LpETH from SWAP for sending LpETH back to CBS
            let cpi_program = ctx.accounts.swap_program.to_account_info();
            let cpi_accounts = LiquidateToken {
                state_account: ctx.accounts.state_account.to_account_info(),
                auction_pool: ctx.accounts.cbs_lpeth.to_account_info(),
                swap_pool: ctx.accounts.swap_lpeth.to_account_info(),
                dest_mint: ctx.accounts.lpeth_mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info()
            };
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
            lpfinance_swap::cpi::liquidate_token(cpi_ctx, borrowed_lpeth)?;
        }

        // Request LpUSD amount from SWAP to AUCTION
        let total_request_lpusd = ((total_price - usdc_price * lpusd_for_swap as u128) / usdc_price) as u64;
        
        if total_request_lpusd == 0 {
            return Err(ErrorCode::InvalidAmount.into());
        } else {
            ctx.accounts.user_account.total_request_lpusd = total_request_lpusd;
        }

        let reward = (total_price - total_borrowed_price) / usdc_price;
        
        let config = &mut ctx.accounts.config;
        let total_amount = config.total_lpusd + reward as u64;
        let auction_percent = config.total_percent as u128 * total_amount as u128 / config.total_lpusd as u128;

        config.last_epoch_percent = total_amount * 100 / config.total_lpusd;
        config.last_epoch_profit = reward as u64;
        config.total_lpusd = total_amount;
        config.total_percent = auction_percent as u64;        

        Ok(())
    }

    pub fn liquidate_swap(
        ctx: Context<LiquidateSwap>
    ) -> Result<()> {
        
        msg!("Started liquidate");

        let liquidator = &mut ctx.accounts.liquidator;
        // Request LpUSD amount from SWAP to AUCTION
        let user_account = &mut ctx.accounts.user_account;

        let total_request_lpusd = user_account.total_request_lpusd;
                
        if total_request_lpusd == 0 {
            return Err(ErrorCode::InvalidAmount.into());
        }

        let btc_amount = liquidator.btc_amount + liquidator.lending_btc_amount;
        let sol_amount = liquidator.sol_amount + liquidator.lending_sol_amount;
        let usdc_amount = liquidator.usdc_amount + liquidator.lending_usdc_amount;
        let msol_amount = liquidator.msol_amount + liquidator.lending_msol_amount;
        let eth_amount = liquidator.eth_amount + liquidator.lending_eth_amount;
        let ust_amount = liquidator.ust_amount + liquidator.lending_ust_amount;
        let srm_amount = liquidator.srm_amount + liquidator.lending_srm_amount;
        let scnsol_amount = liquidator.scnsol_amount + liquidator.lending_scnsol_amount;
        let stsol_amount = liquidator.stsol_amount + liquidator.lending_stsol_amount;
        let usdt_amount = liquidator.usdt_amount + liquidator.lending_usdt_amount;

        let lpsol_amount = liquidator.lpsol_amount;
        // let lpusd_amount = liquidator.lpusd_amount;
        let lpbtc_amount = liquidator.lpbtc_amount;
        let lpeth_amount = liquidator.lpeth_amount;

        let (program_authority, program_authority_bump) = 
        Pubkey::find_program_address(&[PREFIX.as_bytes()], ctx.program_id);
    
        if program_authority != ctx.accounts.state_account.to_account_info().key() {
            return Err(ErrorCode::InvalidOwner.into());
        }

        let seeds = &[
            PREFIX.as_bytes(),
            &[program_authority_bump]
        ];
        let signer = &[&seeds[..]];

        // BTC
        if btc_amount > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.auction_btc.to_account_info(),
                to: ctx.accounts.swap_btc.to_account_info(),
                authority: ctx.accounts.state_account.to_account_info()
            };
    
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, btc_amount)?;
        }
        
        // mSOL
        if msol_amount > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.auction_msol.to_account_info(),
                to: ctx.accounts.swap_msol.to_account_info(),
                authority: ctx.accounts.state_account.to_account_info()
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
                authority: ctx.accounts.state_account.to_account_info()
            };
    
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, usdc_amount)?;
        }

        // eth
        if eth_amount > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.auction_eth.to_account_info(),
                to: ctx.accounts.swap_eth.to_account_info(),
                authority: ctx.accounts.state_account.to_account_info()
            };
    
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, eth_amount)?;
        }

        
        // ust
        if ust_amount > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.auction_ust.to_account_info(),
                to: ctx.accounts.swap_ust.to_account_info(),
                authority: ctx.accounts.state_account.to_account_info()
            };
    
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, ust_amount)?;
        }
        
        // srm 
        if srm_amount > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.auction_srm.to_account_info(),
                to: ctx.accounts.swap_srm.to_account_info(),
                authority: ctx.accounts.state_account.to_account_info()
            };
    
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, srm_amount)?;
        }

        // scnsol
        if scnsol_amount > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.auction_scnsol.to_account_info(),
                to: ctx.accounts.swap_scnsol.to_account_info(),
                authority: ctx.accounts.state_account.to_account_info()
            };
    
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, scnsol_amount)?;
        }

        
        // stsol
        if stsol_amount > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.auction_stsol.to_account_info(),
                to: ctx.accounts.swap_stsol.to_account_info(),
                authority: ctx.accounts.state_account.to_account_info()
            };
    
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, stsol_amount)?;
        }
        
        // usdt 
        if usdt_amount > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.auction_usdt.to_account_info(),
                to: ctx.accounts.swap_usdt.to_account_info(),
                authority: ctx.accounts.state_account.to_account_info()
            };
    
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, usdt_amount)?;
        }

        // LpSOL
        if lpsol_amount > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.auction_lpsol.to_account_info(),
                to: ctx.accounts.swap_lpsol.to_account_info(),
                authority: ctx.accounts.state_account.to_account_info()
            };
    
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, lpsol_amount)?;
        }
        // LpBTC
        if lpbtc_amount > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.auction_lpbtc.to_account_info(),
                to: ctx.accounts.swap_lpbtc.to_account_info(),
                authority: ctx.accounts.state_account.to_account_info()
            };
    
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, lpbtc_amount)?;
        }
        // LpETH
        if lpeth_amount > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.auction_lpeth.to_account_info(),
                to: ctx.accounts.swap_lpeth.to_account_info(),
                authority: ctx.accounts.state_account.to_account_info()
            };
    
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, lpeth_amount)?;
        }


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

        // SOL transfer
        if sol_amount > 0 {
            **ctx.accounts.state_account.to_account_info().try_borrow_mut_lamports()? -= sol_amount;
            **ctx.accounts.swap_account.to_account_info().try_borrow_mut_lamports()? += sol_amount;
        }

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
        let config = &mut ctx.accounts.config;

        let total_withdrawable_amount = user_account.lpusd_amount * config.total_percent / DENOMINATOR;
        msg!("Total withdraw amount: !!{:?}!!", total_withdrawable_amount.to_string());
        msg!("pool_lpusd amount: !!{:?}!!", ctx.accounts.pool_lpusd.amount.to_string());

        if ctx.accounts.pool_lpusd.amount < amount {
            return Err(ErrorCode::InsufficientPoolAmount.into());
        }

        if amount > total_withdrawable_amount {
            return Err(ErrorCode::ExceedAmount.into());
        }

        let (program_authority, program_authority_bump) = 
        Pubkey::find_program_address(&[PREFIX.as_bytes()], ctx.program_id);
    
        if program_authority != ctx.accounts.state_account.to_account_info().key() {
            return Err(ErrorCode::InvalidOwner.into());
        }

        let seeds = &[
            PREFIX.as_bytes(),
            &[program_authority_bump]
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
        
        let config = &mut ctx.accounts.config;

        config.total_lpusd = config.total_lpusd - user_account.lpusd_amount;

        // Init user account
        user_account.lpusd_amount = (user_account.lpusd_amount * ctx.accounts.config.total_percent - amount * DENOMINATOR) / ctx.accounts.config.total_percent;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize <'info>{
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(init,
        seeds = [PREFIX.as_bytes()],
        bump,
        payer = authority
    )]
    pub state_account: Box<Account<'info, AuctionStateAccount>>,

    // Config Accounts
    #[account(init,
        payer = authority,
        space = 32 * 27 + 24 * 20 + 8
    )]
    pub config: Box<Account<'info, Config>>,

    pub lpusd_mint: Box<Account<'info,Mint>>,
    pub lpsol_mint: Box<Account<'info,Mint>>,
    pub lpbtc_mint: Box<Account<'info,Mint>>,
    pub lpeth_mint: Box<Account<'info,Mint>>,

    // LpSOL POOL
    #[account(
        init,
        token::mint = lpsol_mint,
        token::authority = state_account,
        seeds = [PREFIX.as_bytes(), b"pool_lpsol".as_ref()],
        bump,
        payer = authority
    )]
    pub pool_lpsol: Box<Account<'info, TokenAccount>>,
    // Lpbtc POOL
    #[account(
        init,
        token::mint = lpbtc_mint,
        token::authority = state_account,
        seeds = [PREFIX.as_bytes(), b"pool_lpbtc".as_ref()],
        bump,
        payer = authority
    )]
    pub pool_lpbtc: Box<Account<'info, TokenAccount>>,
    // Lpeth POOL
    #[account(
        init,
        token::mint = lpeth_mint,
        token::authority = state_account,
        seeds = [PREFIX.as_bytes(), b"pool_lpeth".as_ref()],
        bump,
        payer = authority
    )]
    pub pool_lpeth: Box<Account<'info, TokenAccount>>,
    // Lpusd POOL
    #[account(
        init,
        token::mint = lpusd_mint,
        token::authority = state_account,
        seeds = [PREFIX.as_bytes(), b"pool_lpusd".as_ref()],
        bump,
        payer = authority
    )]
    pub pool_lpusd: Box<Account<'info, TokenAccount>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct InitializePool <'info>{
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub state_account: Box<Account<'info, AuctionStateAccount>>,
    #[account(mut)]
    pub config: Box<Account<'info, Config>>,

    pub usdc_mint: Box<Account<'info, Mint>>,
    pub btc_mint: Box<Account<'info, Mint>>,
    pub msol_mint: Box<Account<'info, Mint>>,
    pub eth_mint: Box<Account<'info, Mint>>,
    pub ust_mint: Box<Account<'info, Mint>>,
    pub srm_mint: Box<Account<'info, Mint>>,
    pub scnsol_mint: Box<Account<'info, Mint>>,
    pub stsol_mint: Box<Account<'info, Mint>>,
    pub usdt_mint: Box<Account<'info, Mint>>,

    // USDC POOL
    #[account(
        init,
        token::mint = usdc_mint,
        token::authority = state_account,
        seeds = [PREFIX.as_bytes(), b"pool_usdc".as_ref()],
        bump,
        payer = authority
    )]
    pub pool_usdc: Box<Account<'info, TokenAccount>>,
    // BTC POOL
    #[account(
        init,
        token::mint = btc_mint,
        token::authority = state_account,
        seeds = [PREFIX.as_bytes(), b"pool_btc".as_ref()],
        bump,
        payer = authority
    )]
    pub pool_btc: Box<Account<'info, TokenAccount>>,
    // mSOL POOL
    #[account(
        init,
        token::mint = msol_mint,
        token::authority = state_account,
        seeds = [PREFIX.as_bytes(), b"pool_msol".as_ref()],
        bump,
        payer = authority
    )]
    pub pool_msol: Box<Account<'info, TokenAccount>>,
    // eth POOL
    #[account(
        init,
        token::mint = eth_mint,
        token::authority = state_account,
        seeds = [PREFIX.as_bytes(), b"pool_eth".as_ref()],
        bump,
        payer = authority
    )]
    pub pool_eth: Box<Account<'info, TokenAccount>>,
    // ust POOL
    #[account(
        init,
        token::mint = ust_mint,
        token::authority = state_account,
        seeds = [PREFIX.as_bytes(), b"pool_ust".as_ref()],
        bump,
        payer = authority
    )]
    pub pool_ust: Box<Account<'info, TokenAccount>>,
    // srm POOL
    #[account(
        init,
        token::mint = srm_mint,
        token::authority = state_account,
        seeds = [PREFIX.as_bytes(), b"pool_srm".as_ref()],
        bump,
        payer = authority
    )]
    pub pool_srm: Box<Account<'info, TokenAccount>>,
    // scnsol POOL
    #[account(
        init,
        token::mint = scnsol_mint,
        token::authority = state_account,
        seeds = [PREFIX.as_bytes(), b"pool_scnsol".as_ref()],
        bump,
        payer = authority
    )]
    pub pool_scnsol: Box<Account<'info, TokenAccount>>,
    // stsol POOL
    #[account(
        init,
        token::mint = stsol_mint,
        token::authority = state_account,
        seeds = [PREFIX.as_bytes(), b"pool_stsol".as_ref()],
        bump,
        payer = authority
    )]
    pub pool_stsol: Box<Account<'info, TokenAccount>>,
    // usdt POOL
    #[account(
        init,
        token::mint = usdt_mint,
        token::authority = state_account,
        seeds = [PREFIX.as_bytes(), b"pool_usdt".as_ref()],
        bump,
        payer = authority
    )]
    pub pool_usdt: Box<Account<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct InitUserAccount<'info> {
    // State account for each user/wallet
    #[account(
        init,
        seeds = [PREFIX.as_ref(), user_authority.key().as_ref()],
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
        seeds = [PREFIX.as_ref()],
        bump
    )]
    pub state_account: Box<Account<'info, AuctionStateAccount>>,
    #[account(mut, has_one = state_account)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut,
        seeds = [PREFIX.as_ref(), b"pool_lpusd".as_ref()],
        bump
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
        seeds = [PREFIX.as_ref()],
        bump
    )]
    pub state_account: Box<Account<'info, AuctionStateAccount>>,
    #[account(mut, has_one = state_account)]
    pub config: Box<Account<'info, Config>>,
    // UserAccount from CBS protocol
    #[account(mut)]
    pub liquidator: Box<Account<'info, UserAccount>>,
    pub swap_program: Program<'info, LpfinanceSwap>,

    #[account(mut)]
    pub auction_lpusd: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub swap_lpsol: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub swap_lpbtc: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub swap_lpeth: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub lpbtc_mint: Box<Account<'info,Mint>>,
    #[account(mut)]
    pub lpeth_mint: Box<Account<'info,Mint>>,
    #[account(mut)]
    pub lpsol_mint: Box<Account<'info,Mint>>,
    #[account(mut)]
    pub lpusd_mint: Box<Account<'info,Mint>>,

    #[account(mut)]
    pub cbs_lpusd: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub cbs_lpsol: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub cbs_lpbtc: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub cbs_lpeth: Box<Account<'info, TokenAccount>>,
    
    #[account(mut, constraint = user_account.owner == liquidator.owner)]
    pub user_account: Box<Account<'info, UserStateAccount>>,
    // pyth
    pub pyth_btc_account: AccountInfo<'info>,
    pub pyth_usdc_account: AccountInfo<'info>,
    pub pyth_sol_account: AccountInfo<'info>,
    pub pyth_msol_account: AccountInfo<'info>,
    pub pyth_ust_account: AccountInfo<'info>,
    pub pyth_srm_account: AccountInfo<'info>,
    pub pyth_scnsol_account: AccountInfo<'info>,
    pub pyth_stsol_account: AccountInfo<'info>,
    pub pyth_usdt_account: AccountInfo<'info>,
    pub pyth_eth_account: AccountInfo<'info>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}


#[derive(Accounts)]
pub struct LiquidateFromCBS<'info> {
    #[account(mut)]
    pub user_authority: Signer<'info>,
    #[account(mut,
        seeds = [PREFIX.as_ref()],
        bump
    )]
    pub state_account: Box<Account<'info, AuctionStateAccount>>,
    #[account(mut, has_one = state_account)]
    pub config: Box<Account<'info, Config>>,
    // UserAccount from CBS protocol
    #[account(mut)]
    pub liquidator: Box<Account<'info, UserAccount>>,
    #[account(mut)]
    pub cbs_account: Box<Account<'info, StateAccount>>,
    pub cbs_program: Program<'info, CbsProtocol>,

    #[account(mut)]
    pub auction_btc: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub auction_msol: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub auction_usdc: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub auction_eth: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub auction_ust: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub auction_srm: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub auction_scnsol: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub auction_stsol: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub auction_usdt: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub cbs_btc: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub cbs_msol: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub cbs_usdc: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub cbs_eth: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub cbs_ust: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub cbs_srm: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub cbs_scnsol: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub cbs_stsol: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub cbs_usdt: Box<Account<'info, TokenAccount>>,
    
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}


#[derive(Accounts)]
pub struct LiquidateLpTokenFromCBS<'info> {
    #[account(mut)]
    pub user_authority: Signer<'info>,
    #[account(mut,
        seeds = [PREFIX.as_ref()],
        bump
    )]
    pub state_account: Box<Account<'info, AuctionStateAccount>>,
    #[account(mut, has_one = state_account)]
    pub config: Box<Account<'info, Config>>,
    // UserAccount from CBS protocol
    #[account(mut)]
    pub liquidator: Box<Account<'info, UserAccount>>,
    #[account(mut)]
    pub cbs_account: Box<Account<'info, StateAccount>>,
    pub cbs_program: Program<'info, CbsProtocol>,

    #[account(mut)]
    pub auction_btc: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub auction_msol: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub auction_lpusd: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub auction_lpsol: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub auction_lpbtc: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub auction_lpeth: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub cbs_btc: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub cbs_msol: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub cbs_lpusd: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub cbs_lpsol: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub cbs_lpbtc: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub cbs_lpeth: Box<Account<'info, TokenAccount>>,
    
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct LiquidateSwap<'info> {
    #[account(mut)]
    pub user_authority: Signer<'info>,
    #[account(mut,
        seeds = [PREFIX.as_ref()],
        bump
    )]
    pub state_account: Box<Account<'info, AuctionStateAccount>>,
    #[account(mut, has_one = state_account)]
    pub config: Box<Account<'info, Config>>,
    // UserAccount from CBS protocol
    #[account(mut)]
    pub liquidator: Box<Account<'info, UserAccount>>,
    #[account(mut)]
    pub swap_account: Box<Account<'info, lpfinance_swap::StateAccount>>,
    pub swap_program: Program<'info, LpfinanceSwap>,

    #[account(mut)]
    pub lpusd_mint: Box<Account<'info,Mint>>,

    #[account(mut)]
    pub swap_lpusd: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub swap_lpsol: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub swap_lpbtc: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub swap_lpeth: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub swap_btc: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub swap_usdc: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub swap_msol: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub swap_eth: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub swap_ust: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub swap_srm: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub swap_scnsol: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub swap_stsol: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub swap_usdt: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub auction_lpusd: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub auction_lpsol: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub auction_lpbtc: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub auction_lpeth: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub auction_btc: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub auction_usdc: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub auction_msol: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub auction_eth: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub auction_ust: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub auction_srm: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub auction_scnsol: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub auction_stsol: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub auction_usdt: Box<Account<'info, TokenAccount>>,

    #[account(mut, constraint = user_account.owner == liquidator.owner)]
    pub user_account: Box<Account<'info, UserStateAccount>>,
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
        seeds = [PREFIX.as_ref()],
        bump
    )]
    pub state_account: Box<Account<'info, AuctionStateAccount>>,
    #[account(mut, has_one = state_account)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut,
        seeds = [PREFIX.as_ref(), b"pool_lpusd".as_ref()],
        bump
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
    pub temp_amount: u64,
    pub total_request_lpusd: u64
}

#[account]
#[derive(Default)]
pub struct AuctionStateAccount {
    pub owner: Pubkey
}

#[account]
#[derive(Default)]
pub struct Config {
    pub state_account: Pubkey,

    pub msol_mint: Pubkey,
    pub btc_mint: Pubkey,
    pub usdc_mint: Pubkey,
    pub eth_mint: Pubkey,
    pub ust_mint: Pubkey,
    pub srm_mint: Pubkey,
    pub scnsol_mint: Pubkey,
    pub stsol_mint: Pubkey,
    pub usdt_mint: Pubkey,

    pub lpsol_mint: Pubkey,
    pub lpusd_mint: Pubkey,
    pub lpbtc_mint: Pubkey,
    pub lpeth_mint: Pubkey,

    pub pool_btc: Pubkey,
    pub pool_usdc: Pubkey,
    pub pool_msol: Pubkey,
    pub pool_eth: Pubkey,
    pub pool_ust: Pubkey,
    pub pool_srm: Pubkey,
    pub pool_scnsol: Pubkey,
    pub pool_stsol: Pubkey,
    pub pool_usdt: Pubkey,

    pub pool_lpsol: Pubkey,
    pub pool_lpusd: Pubkey,
    pub pool_lpbtc: Pubkey,
    pub pool_lpeth: Pubkey,

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
    #[msg("Invalid Owner")]
    InvalidOwner,
    #[msg("Invalid Amount")]
    InvalidAmount,
    #[msg("Exceed Amount")]
    ExceedAmount,
    #[msg("Not Enough For LTV")]
    NotEnoughLTV,
    #[msg("Not Borrowed LpToken")]
    NotBorrowedLpToken
}