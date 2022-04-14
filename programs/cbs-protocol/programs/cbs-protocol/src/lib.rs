use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, system_instruction };
use pyth_client;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Transfer, Token, TokenAccount }
};

use lpfinance_accounts::cpi::accounts::AddFromCbsProgram;
use lpfinance_accounts::program::LpfinanceAccounts;
use lpfinance_accounts::{self, WhiteList};

use lpfinance_tokens::cpi::accounts::MintLpToken;
use lpfinance_tokens::program::LpfinanceTokens;
use lpfinance_tokens::{self, TokenStateAccount};

use solend::program::Solend;
use solend::{self};

use apricot::program::Apricot;
use apricot::{self};

declare_id!("3f39cgs9wPLVv4vGySNecjKtefe5MJYkFEEj3v6bPequ");

const LTV:u128 = 85;
const DOMINATOR:u128 = 100;
const PREFIX: &str = "cbsprotocol2";

const LENDING_PERCENT: u64 = 10;
const W_THRESHHOLD: u64 = 90;
const S_THRESHHOLD: u64 = 75;

pub fn get_price(pyth_account: AccountInfo) -> Result<u128> {
    let pyth_price_info = &pyth_account;
    let pyth_price_data = &pyth_price_info.try_borrow_data()?;
    let pyth_price = pyth_client::cast::<pyth_client::Price>(pyth_price_data);
    let price = pyth_price.agg.price as u128;
    Ok(price)
}

#[program]
pub mod cbs_protocol {
    use super::*;
    pub fn initialize(
        ctx: Context<Initialize>
    ) -> Result<()> {
        msg!("INITIALIZE PROTOCAL");

        let state_account = &mut ctx.accounts.state_account;
        let config = &mut ctx.accounts.config;

        state_account.owner = ctx.accounts.authority.key();
        state_account.liquidation_run = false;

        config.state_account = ctx.accounts.state_account.key();

        config.btc_mint = ctx.accounts.btc_mint.key();
        config.usdc_mint = ctx.accounts.usdc_mint.key();
        config.eth_mint = ctx.accounts.eth_mint.key();
        config.msol_mint = ctx.accounts.msol_mint.key();
        config.ust_mint = ctx.accounts.ust_mint.key();
        config.srm_mint = ctx.accounts.srm_mint.key();
        config.scnsol_mint = ctx.accounts.scnsol_mint.key();
        config.stsol_mint = ctx.accounts.stsol_mint.key();
        config.usdt_mint = ctx.accounts.usdt_mint.key();
        
        config.lpusd_mint = ctx.accounts.lpusd_mint.key();
        config.lpsol_mint = ctx.accounts.lpsol_mint.key();
        config.lpbtc_mint = ctx.accounts.lpbtc_mint.key();
        config.lpeth_mint = ctx.accounts.lpeth_mint.key();

        config.pool_lpsol = ctx.accounts.pool_lpsol.key();
        config.pool_lpusd = ctx.accounts.pool_lpusd.key();
        config.pool_lpbtc = ctx.accounts.pool_lpbtc.key();
        config.pool_lpeth = ctx.accounts.pool_lpeth.key();  

        config.total_borrowed_lpsol = 0;
        config.total_borrowed_lpusd = 0;
        config.total_borrowed_lpeth = 0;
        config.total_borrowed_lpbtc = 0;

        config.total_deposited_sol = 0;
        config.total_deposited_usdc = 0;
        config.total_deposited_btc = 0;
        config.total_deposited_eth = 0;
        config.total_deposited_msol = 0;
        config.total_deposited_ust = 0;
        config.total_deposited_srm = 0;
        config.total_deposited_scnsol = 0;
        config.total_deposited_stsol = 0;
        config.total_deposited_usdt = 0;

        config.total_deposited_lpsol = 0;
        config.total_deposited_lpusd = 0;
        config.total_deposited_lpeth = 0;
        config.total_deposited_lpbtc = 0;        

        Ok(())
    }

    pub fn initialize_pool(
        ctx: Context<InitializePool>
    ) -> Result<()> {
        msg!("INITIALIZE POOL");

        let state_account = &mut ctx.accounts.state_account;
        let config = &mut ctx.accounts.config;

        if state_account.owner != ctx.accounts.authority.key() {
            return Err(ErrorCode::InvalidOwner.into());
        }

        config.state_account = ctx.accounts.state_account.key();

        config.pool_btc = ctx.accounts.pool_btc.key();
        config.pool_usdc = ctx.accounts.pool_usdc.key();
        config.pool_msol = ctx.accounts.pool_msol.key();
        config.pool_eth = ctx.accounts.pool_eth.key();
        config.pool_ust = ctx.accounts.pool_ust.key();
        config.pool_srm = ctx.accounts.pool_srm.key();
        config.pool_scnsol = ctx.accounts.pool_scnsol.key();
        config.pool_stsol = ctx.accounts.pool_stsol.key();
        config.pool_usdt = ctx.accounts.pool_usdt.key();    

        Ok(())
    }

    // Init user account
    pub fn init_user_account(
        ctx: Context<InitUserAccount>, 
        bump: u8
    ) -> Result<()> {
        let user_account = &mut ctx.accounts.user_account;
        user_account.owner = ctx.accounts.user_authority.key();
        user_account.bump = bump;
        Ok(())
    }

    pub fn deposit_collateral(
        ctx: Context<DepositCollateral>,
        amount: u64
    )-> Result<()> {        
        if ctx.accounts.user_collateral.amount < amount {
            return Err(ErrorCode::InsufficientAmount.into());
        }

        let cpi_accounts = Transfer {
            from: ctx.accounts.user_collateral.to_account_info(),
            to: ctx.accounts.collateral_pool.to_account_info(),
            authority: ctx.accounts.user_authority.to_account_info()
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        let user_account =&mut ctx.accounts.user_account;
        let config = &mut ctx.accounts.config;

        if ctx.accounts.user_collateral.mint == config.btc_mint {
            user_account.btc_amount = user_account.btc_amount + amount;
            config.total_deposited_btc = config.total_deposited_btc + amount;
        }

        if ctx.accounts.user_collateral.mint == config.usdc_mint {
            user_account.usdc_amount = user_account.usdc_amount + amount;
            config.total_deposited_usdc = config.total_deposited_usdc + amount;
        }

        if ctx.accounts.user_collateral.mint == config.msol_mint {
            user_account.msol_amount = user_account.msol_amount + amount;
            config.total_deposited_msol = config.total_deposited_msol + amount;
        }
        
        if ctx.accounts.user_collateral.mint == config.eth_mint {
            user_account.eth_amount = user_account.eth_amount + amount;
            config.total_deposited_eth = config.total_deposited_eth + amount;
        }

        if ctx.accounts.user_collateral.mint == config.ust_mint {
            user_account.ust_amount = user_account.ust_amount + amount;
            config.total_deposited_ust = config.total_deposited_ust + amount;
        }

        if ctx.accounts.user_collateral.mint == config.srm_mint {
            user_account.srm_amount = user_account.srm_amount + amount;
            config.total_deposited_srm = config.total_deposited_srm + amount;
        }

        if ctx.accounts.user_collateral.mint == config.scnsol_mint {
            user_account.scnsol_amount = user_account.scnsol_amount + amount;
            config.total_deposited_scnsol = config.total_deposited_scnsol + amount;
        }
        
        if ctx.accounts.user_collateral.mint == config.stsol_mint {
            user_account.stsol_amount = user_account.stsol_amount + amount;
            config.total_deposited_stsol = config.total_deposited_stsol + amount;
        }

        if ctx.accounts.user_collateral.mint == config.usdt_mint {
            user_account.usdt_amount = user_account.usdt_amount + amount;
            config.total_deposited_usdt = config.total_deposited_usdt + amount;
        }

        if ctx.accounts.user_collateral.mint == config.lpusd_mint {
            user_account.lpusd_amount = user_account.lpusd_amount + amount;
            config.total_deposited_lpusd = config.total_deposited_lpusd + amount;
        }

        if ctx.accounts.user_collateral.mint == config.lpsol_mint {
            user_account.lpsol_amount = user_account.lpsol_amount + amount;
            config.total_deposited_lpsol = config.total_deposited_lpsol + amount;
        }

        if ctx.accounts.user_collateral.mint == config.lpeth_mint {
            user_account.lpeth_amount = user_account.lpeth_amount + amount;
            config.total_deposited_lpeth = config.total_deposited_lpeth + amount;
        }

        if ctx.accounts.user_collateral.mint == config.lpbtc_mint {
            user_account.lpbtc_amount = user_account.lpbtc_amount + amount;
            config.total_deposited_lpbtc = config.total_deposited_lpbtc + amount;
        }

        // let whitelist = ctx.accounts.whitelist.load_mut()?;
        if ctx.accounts.whitelist.load_mut()?.addresses.contains(&ctx.accounts.user_authority.key()) {
            msg!("Already Exist");
        } else {

            let cpi_program = ctx.accounts.accounts_program.to_account_info();
            let cpi_accounts = AddFromCbsProgram {
                config: ctx.accounts.accounts_config.to_account_info(),
                whitelist: ctx.accounts.whitelist.to_account_info(),
                cbsprogram: ctx.accounts.state_account.to_account_info()
            };
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

            let new_wallet = ctx.accounts.user_authority.key();
            lpfinance_accounts::cpi::add_from_cbs_program(cpi_ctx, new_wallet)?;
        }
        
        Ok(())
    }

    pub fn deposit_sol(
        ctx: Context<DepositSOL>,
        amount: u64
    ) -> Result<()> {
        msg!("Deposit SOL");

        if **ctx.accounts.user_authority.lamports.borrow() < amount {
            return Err(ErrorCode::InsufficientAmount.into());
        }

        invoke(
            &system_instruction::transfer(
                ctx.accounts.user_authority.key,
                ctx.accounts.state_account.to_account_info().key,
                amount
            ),
            &[
                ctx.accounts.user_authority.to_account_info().clone(),
                ctx.accounts.state_account.to_account_info().clone(),
                ctx.accounts.system_program.to_account_info().clone()
            ]
        )?;

        let user_account = &mut ctx.accounts.user_account;
        let config = &mut ctx.accounts.config;

        user_account.sol_amount = user_account.sol_amount + amount;
        config.total_deposited_sol = config.total_deposited_sol + amount;

        // let whitelist = ctx.accounts.whitelist.load_mut()?;
        if ctx.accounts.whitelist.load_mut()?.addresses.contains(&ctx.accounts.user_authority.key()) {
            msg!("Already Exist");
        } else {

            let cpi_program = ctx.accounts.accounts_program.to_account_info();
            let cpi_accounts = AddFromCbsProgram {
                config: ctx.accounts.whitelist_config.to_account_info(),
                whitelist: ctx.accounts.whitelist.to_account_info(),
                cbsprogram: ctx.accounts.state_account.to_account_info()
            };
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

            let new_wallet = ctx.accounts.user_authority.key();
            lpfinance_accounts::cpi::add_from_cbs_program(cpi_ctx, new_wallet)?;
        }

        Ok(())
    }

    pub fn borrow_lptoken(
        ctx: Context<BorrowLpToken>,
        amount: u64
    ) -> Result<()> {
        msg!("Borrow LpToken");

        if amount < 1 {
            return Err(ErrorCode::InvalidAmount.into());
        }
        // Borrowable TotalPrice. Need to be calculated with LTV
        let mut total_price: u128 = 0;
        let mut total_borrowed_price: u128 = 0;
        let user_account = &mut ctx.accounts.user_account;
        let config = &mut ctx.accounts.config;

        // BTC price        
        let btc_price: u128 = get_price(ctx.accounts.pyth_btc_account.to_account_info())?;    
        total_price += btc_price * user_account.btc_amount as u128;

        // SOL price
        let sol_price: u128 = get_price(ctx.accounts.pyth_sol_account.to_account_info())?;    
        total_price += sol_price * user_account.sol_amount as u128;

        // USDC price
        let usdc_price: u128 = get_price(ctx.accounts.pyth_usdc_account.to_account_info())?;
        total_price += usdc_price * user_account.usdc_amount as u128;

        // mSOL price
        let msol_price: u128 = get_price(ctx.accounts.pyth_msol_account.to_account_info())?;
        total_price += msol_price * user_account.msol_amount as u128;

        // ETH price
        let eth_price: u128 = get_price(ctx.accounts.pyth_eth_account.to_account_info())?;
        total_price += eth_price * user_account.eth_amount as u128;


        // ust price        
        let ust_price: u128 = get_price(ctx.accounts.pyth_ust_account.to_account_info())?;    
        total_price += ust_price * user_account.ust_amount as u128;

        // srm price
        let srm_price: u128 = get_price(ctx.accounts.pyth_srm_account.to_account_info())?;    
        total_price += srm_price * user_account.srm_amount as u128;

        // scnsol price
        let scnsol_price: u128 = get_price(ctx.accounts.pyth_scnsol_account.to_account_info())?;
        total_price += scnsol_price * user_account.scnsol_amount as u128;

        // stsol price
        let stsol_price: u128 = get_price(ctx.accounts.pyth_stsol_account.to_account_info())?;
        total_price += stsol_price * user_account.stsol_amount as u128;

        // usdt price
        let usdt_price: u128 = get_price(ctx.accounts.pyth_usdt_account.to_account_info())?;
        total_price += usdt_price * user_account.usdt_amount as u128;

        // LpUSD price
        let lpusd_price = usdc_price;        
        total_price += lpusd_price * user_account.lpusd_amount as u128;

        // LpSOL price
        let lpsol_price = sol_price;
        total_price += lpsol_price * user_account.lpsol_amount as u128;

        // LpBTC price
        let lpbtc_price = btc_price;        
        total_price += lpbtc_price * user_account.lpbtc_amount as u128;

        // LpETH price
        let lpeth_price: u128 = eth_price;
        total_price += lpeth_price * user_account.lpeth_amount as u128;

        // Total Borrowed AMount
        total_borrowed_price += lpusd_price * user_account.borrowed_lpusd as u128;
        total_borrowed_price += lpsol_price * user_account.borrowed_lpsol as u128;
        total_borrowed_price += lpbtc_price * user_account.borrowed_lpbtc as u128;
        total_borrowed_price += lpeth_price * user_account.borrowed_lpeth as u128;

        let mut borrow_value: u128 = amount as u128;
        
        if ctx.accounts.collateral_mint.key() == config.lpusd_mint {
            borrow_value = borrow_value * lpusd_price;

            config.total_borrowed_lpusd = config.total_borrowed_lpusd + amount;
            user_account.borrowed_lpusd = user_account.borrowed_lpusd + amount;
        } else if ctx.accounts.collateral_mint.key() == config.lpsol_mint {
            borrow_value = borrow_value * lpsol_price;

            config.total_borrowed_lpsol = config.total_borrowed_lpsol + amount;
            user_account.borrowed_lpsol = user_account.borrowed_lpsol + amount;
        } else if ctx.accounts.collateral_mint.key() == config.lpbtc_mint {
            borrow_value = borrow_value * lpbtc_price;

            config.total_borrowed_lpbtc = config.total_borrowed_lpbtc + amount;
            user_account.borrowed_lpbtc = user_account.borrowed_lpbtc + amount;
        } else if ctx.accounts.collateral_mint.key() == config.lpeth_mint {
            borrow_value = borrow_value * lpeth_price;

            config.total_borrowed_lpeth = config.total_borrowed_lpeth + amount;
            user_account.borrowed_lpeth = user_account.borrowed_lpeth + amount;
        } else {
            return Err(ErrorCode::InvalidToken.into());
        }

        let borrable_total = total_price * LTV / DOMINATOR - total_borrowed_price;

        if borrable_total > borrow_value {
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

            // Mint
            let cpi_program = ctx.accounts.lptokens_program.to_account_info();
            let cpi_accounts = MintLpToken {
                signer: ctx.accounts.state_account.to_account_info(),
                state_account: ctx.accounts.tokens_state.to_account_info(),
                config: ctx.accounts.lptoken_config.to_account_info(),
                lptoken_mint: ctx.accounts.collateral_mint.to_account_info(),
                user_lptoken: ctx.accounts.user_collateral.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info()
            };

            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            lpfinance_tokens::cpi::mint_lptoken(cpi_ctx, amount)?;
        } else {
            return Err(ErrorCode::BorrowExceed.into());
        }

        Ok(())
    }

    pub fn liquidate_collateral(
        ctx: Context<LiquidateCollateral>
    ) -> Result<()> {
        msg!("liquidate_collateral started");

        let user_account = &mut ctx.accounts.user_account;

        let lpusd_amount = user_account.lpusd_amount;
        let lpsol_amount = user_account.lpsol_amount;
        let lpbtc_amount = user_account.lpbtc_amount;
        let lpeth_amount = user_account.lpeth_amount;
        let usdc_amount = user_account.usdc_amount;
        let btc_amount = user_account.btc_amount;
        let sol_amount = user_account.sol_amount;
        let msol_amount = user_account.msol_amount;
        let eth_amount = user_account.eth_amount;
        let ust_amount = user_account.ust_amount;
        let srm_amount = user_account.srm_amount;
        let scnsol_amount = user_account.scnsol_amount;
        let stsol_amount = user_account.stsol_amount;
        let usdt_amount = user_account.usdt_amount;

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

        msg!("Lpusd amount: !!{:?}!!", lpusd_amount.to_string());

        if lpusd_amount > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.cbs_lpusd.to_account_info(),
                to: ctx.accounts.auction_lpusd.to_account_info(),
                authority: ctx.accounts.state_account.to_account_info()
            };
    
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, lpusd_amount)?;
        }

        if lpsol_amount > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.cbs_lpsol.to_account_info(),
                to: ctx.accounts.auction_lpsol.to_account_info(),
                authority: ctx.accounts.state_account.to_account_info()
            };
    
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, lpsol_amount)?;
        }

        if lpbtc_amount > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.cbs_lpbtc.to_account_info(),
                to: ctx.accounts.auction_lpbtc.to_account_info(),
                authority: ctx.accounts.state_account.to_account_info()
            };
    
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, lpbtc_amount)?;
        }

        if lpeth_amount > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.cbs_lpeth.to_account_info(),
                to: ctx.accounts.auction_lpeth.to_account_info(),
                authority: ctx.accounts.state_account.to_account_info()
            };
    
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, lpeth_amount)?;
        }

        if msol_amount > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.cbs_msol.to_account_info(),
                to: ctx.accounts.auction_msol.to_account_info(),
                authority: ctx.accounts.state_account.to_account_info()
            };
    
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, msol_amount)?;
        }

        if btc_amount > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.cbs_btc.to_account_info(),
                to: ctx.accounts.auction_btc.to_account_info(),
                authority: ctx.accounts.state_account.to_account_info()
            };
    
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, btc_amount)?;
        }

        if usdc_amount > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.cbs_usdc.to_account_info(),
                to: ctx.accounts.auction_usdc.to_account_info(),
                authority: ctx.accounts.state_account.to_account_info()
            };
    
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, usdc_amount)?;
        }

        if eth_amount > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.cbs_eth.to_account_info(),
                to: ctx.accounts.auction_eth.to_account_info(),
                authority: ctx.accounts.state_account.to_account_info()
            };
    
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, eth_amount)?;
        }

        if ust_amount > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.cbs_ust.to_account_info(),
                to: ctx.accounts.auction_ust.to_account_info(),
                authority: ctx.accounts.state_account.to_account_info()
            };
    
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, ust_amount)?;
        }

        if srm_amount > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.cbs_srm.to_account_info(),
                to: ctx.accounts.auction_srm.to_account_info(),
                authority: ctx.accounts.state_account.to_account_info()
            };
    
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, srm_amount)?;
        }

        if scnsol_amount > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.cbs_scnsol.to_account_info(),
                to: ctx.accounts.auction_scnsol.to_account_info(),
                authority: ctx.accounts.state_account.to_account_info()
            };
    
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, scnsol_amount)?;
        }

        if stsol_amount > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.cbs_stsol.to_account_info(),
                to: ctx.accounts.auction_stsol.to_account_info(),
                authority: ctx.accounts.state_account.to_account_info()
            };
    
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, stsol_amount)?;
        }

        if usdt_amount > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.cbs_usdt.to_account_info(),
                to: ctx.accounts.auction_usdt.to_account_info(),
                authority: ctx.accounts.state_account.to_account_info()
            };
    
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, usdt_amount)?;
        }

        msg!("sol_amount started");

        if sol_amount > 0 {
            **ctx.accounts.state_account.to_account_info().try_borrow_mut_lamports()? -= sol_amount;
            **ctx.accounts.auction_account.try_borrow_mut_lamports()? += sol_amount;
        }
        msg!("sol_amount ended");

        user_account.lpusd_amount = 0;
        user_account.lpsol_amount = 0;
        user_account.lpbtc_amount = 0;
        user_account.lpeth_amount = 0;

        user_account.sol_amount = 0;
        user_account.usdc_amount = 0;
        user_account.btc_amount = 0;
        user_account.msol_amount = 0;
        user_account.eth_amount = 0;
        user_account.ust_amount = 0;
        user_account.srm_amount = 0;
        user_account.scnsol_amount = 0;
        user_account.stsol_amount = 0;
        user_account.usdt_amount = 0;

        user_account.borrowed_lpusd = 0;
        user_account.borrowed_lpsol = 0;
        user_account.borrowed_lpbtc = 0;
        user_account.borrowed_lpeth = 0;
        
        Ok(())
    }

    pub fn withdraw_sol(
        ctx: Context<WithdrawSOL>,
        amount: u64
    ) -> Result<()> {
        let user_account = &mut ctx.accounts.user_account;
        let sol_amount = user_account.sol_amount as u128;
        let btc_amount = user_account.btc_amount as u128;
        let usdc_amount = user_account.usdc_amount as u128;
        let msol_amount = user_account.msol_amount as u128;
        let eth_amount = user_account.eth_amount as u128;
        let ust_amount = user_account.ust_amount as u128;
        let srm_amount = user_account.srm_amount as u128;
        let scnsol_amount = user_account.scnsol_amount as u128;
        let stsol_amount = user_account.stsol_amount as u128;
        let usdt_amount = user_account.usdt_amount as u128;

        let lpsol_amount = user_account.lpsol_amount as u128;
        let lpusd_amount = user_account.lpusd_amount as u128;
        let lpbtc_amount = user_account.lpbtc_amount as u128;
        let lpeth_amount = user_account.lpeth_amount as u128;

        let borrowed_lpusd = user_account.borrowed_lpusd as u128;
        let borrowed_lpsol = user_account.borrowed_lpsol as u128;
        let borrowed_lpbtc = user_account.borrowed_lpbtc as u128;
        let borrowed_lpeth = user_account.borrowed_lpeth as u128;

        let mut total_price: u128 = 0;

        // BTC price
        let btc_price: u128 = get_price(ctx.accounts.pyth_btc_account.to_account_info())?;
        total_price += btc_price * btc_amount;

        // SOL price
        let sol_price: u128 = get_price(ctx.accounts.pyth_sol_account.to_account_info())?; 
        total_price += sol_price * sol_amount;

        // USDC price
        let usdc_price: u128 = get_price(ctx.accounts.pyth_usdc_account.to_account_info())?;      
        total_price += usdc_price * usdc_amount;

        // mSOL price
        let msol_price: u128 = get_price(ctx.accounts.pyth_msol_account.to_account_info())?;
        total_price += msol_price * msol_amount;

        // ETH price
        let eth_price: u128 = get_price(ctx.accounts.pyth_eth_account.to_account_info())?;
        total_price += eth_price * eth_amount;

        // ust price
        let ust_price: u128 = get_price(ctx.accounts.pyth_ust_account.to_account_info())?;
        total_price += ust_price * ust_amount;

        // srm price
        let srm_price: u128 = get_price(ctx.accounts.pyth_srm_account.to_account_info())?; 
        total_price += srm_price * srm_amount;

        // scnsol price
        let scnsol_price: u128 = get_price(ctx.accounts.pyth_scnsol_account.to_account_info())?;      
        total_price += scnsol_price * scnsol_amount;

        // stsol price
        let stsol_price: u128 = get_price(ctx.accounts.pyth_stsol_account.to_account_info())?;
        total_price += stsol_price * stsol_amount;

        // usdt price
        let usdt_price: u128 = get_price(ctx.accounts.pyth_usdt_account.to_account_info())?;
        total_price += usdt_price * usdt_amount;

        // lpETH price
        let lpeth_price: u128 = eth_price;
        total_price += lpeth_price * lpeth_amount;

        // LpUSD price
        let lpusd_price = usdc_price;        
        total_price += lpusd_price * lpusd_amount;

        // LpUSD price
        let lpbtc_price = btc_price;        
        total_price += lpbtc_price * lpbtc_amount;

        // LpSOL price
        let lpsol_price = sol_price;
        total_price += lpsol_price * lpsol_amount;

        let mut borrowed_total: u128 = 0;
        borrowed_total += borrowed_lpsol * lpsol_price;
        borrowed_total += borrowed_lpusd * lpusd_price;
        borrowed_total += borrowed_lpbtc * lpbtc_price;
        borrowed_total += borrowed_lpeth * lpeth_price;

        if total_price * LTV < borrowed_total * DOMINATOR {
            return Err(ErrorCode::InvalidAmount.into());
        }
        
        if amount > sol_amount as u64 {
            return Err(ErrorCode::InvalidAmount.into());
        }
        let borrowable_amount = (total_price - borrowed_total * DOMINATOR / LTV) / sol_price;
        if amount > borrowable_amount as u64{
            return Err(ErrorCode::InvalidAmount.into());
        }
        
        **ctx.accounts.state_account.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.user_authority.try_borrow_mut_lamports()? += amount;

        user_account.sol_amount -= amount;
        ctx.accounts.config.total_deposited_sol -= amount;

        Ok(())
    }

    pub fn withdraw_token(
        ctx: Context<WithdrawToken>,
        amount: u64
    ) -> Result<()> {
        msg!("Withdraw Token");

        let user_account = &mut ctx.accounts.user_account;
        let sol_amount = user_account.sol_amount as u128;
        let btc_amount = user_account.btc_amount as u128;
        let usdc_amount = user_account.usdc_amount as u128;
        let msol_amount = user_account.msol_amount as u128;
        let eth_amount = user_account.eth_amount as u128;
        let ust_amount = user_account.ust_amount as u128;
        let srm_amount = user_account.srm_amount as u128;
        let scnsol_amount = user_account.scnsol_amount as u128;
        let stsol_amount = user_account.stsol_amount as u128;
        let usdt_amount = user_account.usdt_amount as u128;

        let lpsol_amount = user_account.lpsol_amount as u128;
        let lpusd_amount = user_account.lpusd_amount as u128;
        let lpbtc_amount = user_account.lpbtc_amount as u128;
        let lpeth_amount = user_account.lpeth_amount as u128;

        let borrowed_lpusd = user_account.borrowed_lpusd as u128;
        let borrowed_lpsol = user_account.borrowed_lpsol as u128;
        let borrowed_lpbtc = user_account.borrowed_lpbtc as u128;
        let borrowed_lpeth = user_account.borrowed_lpeth as u128;

        let mut total_price: u128 = 0;

        // BTC price
        let btc_price: u128 = get_price(ctx.accounts.pyth_btc_account.to_account_info())?;     
        total_price += btc_price * btc_amount;

        // SOL price
        let sol_price: u128 = get_price(ctx.accounts.pyth_sol_account.to_account_info())?;     
        total_price += sol_price * sol_amount;

        // USDC price
        let usdc_price: u128 = get_price(ctx.accounts.pyth_usdc_account.to_account_info())?;     
        total_price += usdc_price * usdc_amount;

        // mSOL price
        let msol_price: u128 = get_price(ctx.accounts.pyth_msol_account.to_account_info())?;
        total_price += msol_price * msol_amount;

        // ETH price
        let eth_price: u128 = get_price(ctx.accounts.pyth_eth_account.to_account_info())?;   
        total_price += eth_price * eth_amount;

        // ust price
        let ust_price: u128 = get_price(ctx.accounts.pyth_ust_account.to_account_info())?;     
        total_price += ust_price * ust_amount;

        // srm price
        let srm_price: u128 = get_price(ctx.accounts.pyth_srm_account.to_account_info())?;     
        total_price += srm_price * srm_amount;

        // scnsol price
        let scnsol_price: u128 = get_price(ctx.accounts.pyth_scnsol_account.to_account_info())?;     
        total_price += scnsol_price * scnsol_amount;

        // stsol price
        let stsol_price: u128 = get_price(ctx.accounts.pyth_stsol_account.to_account_info())?;
        total_price += stsol_price * stsol_amount;

        // usdt price
        let usdt_price: u128 = get_price(ctx.accounts.pyth_usdt_account.to_account_info())?;   
        total_price += usdt_price * usdt_amount;

        // LpUSD price
        let lpusd_price = usdc_price;        
        total_price += lpusd_price * lpusd_amount;

        // LpSOL price
        let lpsol_price = sol_price;
        total_price += lpsol_price * lpsol_amount;

        // LpETH price
        let lpeth_price: u128 = eth_price;   
        total_price += lpeth_price * lpeth_amount;

        // LpBTC price
        let lpbtc_price = btc_price;
        total_price += lpbtc_price * lpbtc_amount;

        let mut borrowed_total: u128 = 0;
        borrowed_total += borrowed_lpsol * lpsol_price;
        borrowed_total += borrowed_lpusd * lpusd_price;
        borrowed_total += borrowed_lpbtc * lpbtc_price;
        borrowed_total += borrowed_lpeth * lpeth_price;

        if total_price * LTV < borrowed_total * DOMINATOR {
            return Err(ErrorCode::InvalidAmount.into());
        }        
        
        let mut dest_price:u128;
        let mut owned_amount:u128;
        if ctx.accounts.dest_mint.key() == ctx.accounts.config.usdc_mint {
            dest_price = usdc_price;
            owned_amount = usdc_amount;
            user_account.usdc_amount -= amount;
            ctx.accounts.config.total_deposited_usdc -= amount;
        } else if ctx.accounts.dest_mint.key() == ctx.accounts.config.msol_mint {
            dest_price = msol_price;
            owned_amount = msol_amount;
            user_account.msol_amount -= amount;
            ctx.accounts.config.total_deposited_msol -= amount;
        } else if ctx.accounts.dest_mint.key() == ctx.accounts.config.eth_mint {
            dest_price = eth_price;
            owned_amount = eth_amount;
            user_account.eth_amount -= amount;
            ctx.accounts.config.total_deposited_eth -= amount;
        } else if ctx.accounts.dest_mint.key() == ctx.accounts.config.btc_mint {
            dest_price = btc_price;
            owned_amount = btc_amount;
            user_account.btc_amount -= amount;
            ctx.accounts.config.total_deposited_btc -= amount;
        } else if ctx.accounts.dest_mint.key() == ctx.accounts.config.ust_mint {
            dest_price = ust_price;
            owned_amount = ust_amount;
            user_account.ust_amount -= amount;
            ctx.accounts.config.total_deposited_ust -= amount;
        } else if ctx.accounts.dest_mint.key() == ctx.accounts.config.srm_mint {
            dest_price = srm_price;
            owned_amount = srm_amount;
            user_account.srm_amount -= amount;
            ctx.accounts.config.total_deposited_srm -= amount;
        } else if ctx.accounts.dest_mint.key() == ctx.accounts.config.scnsol_mint {
            dest_price = scnsol_price;
            owned_amount = scnsol_amount;
            user_account.scnsol_amount -= amount;
            ctx.accounts.config.total_deposited_scnsol -= amount;
        } else if ctx.accounts.dest_mint.key() == ctx.accounts.config.stsol_mint {
            dest_price = stsol_price;
            owned_amount = stsol_amount;
            user_account.stsol_amount -= amount;
            ctx.accounts.config.total_deposited_stsol -= amount;
        } else if ctx.accounts.dest_mint.key() == ctx.accounts.config.usdt_mint {
            dest_price = usdt_price;
            owned_amount = usdt_amount;
            user_account.usdt_amount -= amount;
            ctx.accounts.config.total_deposited_usdt -= amount;
        } else if ctx.accounts.dest_mint.key() == ctx.accounts.config.lpbtc_mint {
            dest_price = lpbtc_price;
            owned_amount = lpbtc_amount;
            user_account.lpbtc_amount -= amount;
            ctx.accounts.config.total_deposited_lpbtc -= amount;
        } else if ctx.accounts.dest_mint.key() == ctx.accounts.config.lpeth_mint {
            dest_price = lpeth_price;
            owned_amount = lpeth_amount;
            user_account.lpeth_amount -= amount;
            ctx.accounts.config.total_deposited_lpeth -= amount;
        } else if ctx.accounts.dest_mint.key() == ctx.accounts.config.lpusd_mint {
            dest_price = lpusd_price;
            owned_amount = lpusd_amount;
            user_account.lpusd_amount -= amount;
            ctx.accounts.config.total_deposited_lpusd -= amount;
        } else if ctx.accounts.dest_mint.key() == ctx.accounts.config.lpsol_mint {
            dest_price = lpsol_price;
            owned_amount = lpsol_amount;
            user_account.lpsol_amount -= amount;
            ctx.accounts.config.total_deposited_lpsol -= amount;
        } else {
            return Err(ErrorCode::InvalidToken.into());
        }        

        if amount > owned_amount as u64 {
            return Err(ErrorCode::InvalidAmount.into());
        }

        let borrowable_amount = (total_price - borrowed_total * DOMINATOR / LTV) / dest_price;
        if amount > borrowable_amount as u64{
            return Err(ErrorCode::InvalidAmount.into());
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
            from: ctx.accounts.dest_pool.to_account_info(),
            to: ctx.accounts.user_dest.to_account_info(),
            authority: ctx.accounts.state_account.to_account_info()
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, amount)?;

        Ok(())
    }

    pub fn repay_token(
        ctx: Context<RepayToken>,
        amount: u64
    ) -> Result<()> {
        if ctx.accounts.user_dest.amount < amount {
            return Err(ErrorCode::InsufficientAmount.into());
        }

        let user_account =&mut ctx.accounts.user_account;
        let config = &mut ctx.accounts.config;


        if ctx.accounts.user_dest.mint != config.usdc_mint && 
            ctx.accounts.user_dest.mint != config.eth_mint &&
            ctx.accounts.user_dest.mint != config.btc_mint &&
            ctx.accounts.user_dest.mint != config.lpusd_mint &&
            ctx.accounts.user_dest.mint != config.lpeth_mint &&
            ctx.accounts.user_dest.mint != config.lpbtc_mint &&
            ctx.accounts.user_dest.mint != config.lpsol_mint
        {
            return Err(ErrorCode::InvalidToken.into());
        }

        if ctx.accounts.user_dest.mint == config.usdc_mint ||
            ctx.accounts.user_dest.mint == config.eth_mint ||
            ctx.accounts.user_dest.mint == config.btc_mint       
        {
            let cpi_accounts = Transfer {
                from: ctx.accounts.user_dest.to_account_info(),
                to: ctx.accounts.dest_pool.to_account_info(),
                authority: ctx.accounts.user_authority.to_account_info()
            };
    
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
            token::transfer(cpi_ctx, amount)?;
        }

        if ctx.accounts.user_dest.mint == config.lpusd_mint ||
            ctx.accounts.user_dest.mint == config.lpeth_mint ||
            ctx.accounts.user_dest.mint == config.lpbtc_mint ||
            ctx.accounts.user_dest.mint == config.lpsol_mint 
        {
            let cpi_ctx = CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Burn {
                    mint: ctx.accounts.dest_mint.to_account_info(),
                    to: ctx.accounts.user_dest.to_account_info(),
                    authority: ctx.accounts.user_authority.to_account_info()
                }
            );

            token::burn(cpi_ctx, amount)?;
        }

        if ctx.accounts.user_dest.mint == config.usdc_mint || ctx.accounts.user_dest.mint == config.lpusd_mint{
            
            user_account.borrowed_lpusd = user_account.borrowed_lpusd - amount;
            config.total_borrowed_lpusd = config.total_borrowed_lpusd - amount;  
        } else if ctx.accounts.user_dest.mint == config.lpsol_mint {

            user_account.borrowed_lpsol = user_account.borrowed_lpsol - amount;
            config.total_borrowed_lpsol = config.total_borrowed_lpsol - amount;            
        } else if ctx.accounts.user_dest.mint == config.lpbtc_mint || ctx.accounts.user_dest.mint == config.btc_mint{

            user_account.borrowed_lpbtc = user_account.borrowed_lpbtc - amount;
            config.total_borrowed_lpbtc = config.total_borrowed_lpbtc - amount;            
        } else if ctx.accounts.user_dest.mint == config.lpeth_mint || ctx.accounts.user_dest.mint == config.eth_mint{

            user_account.borrowed_lpeth = user_account.borrowed_lpeth - amount;
            config.total_borrowed_lpeth = config.total_borrowed_lpeth - amount;            
        }

        Ok(())
    }

    pub fn repay_sol(
        ctx: Context<RepaySOL>,
        amount: u64
    ) -> Result<()> {
        if **ctx.accounts.user_authority.lamports.borrow() < amount {
            return Err(ErrorCode::InsufficientAmount.into());
        }

        if amount > ctx.accounts.user_account.borrowed_lpsol || amount == 0 {
            return Err(ErrorCode::InvalidAmount.into());
        }

        invoke(
            &system_instruction::transfer(
                ctx.accounts.user_authority.key,
                ctx.accounts.state_account.to_account_info().key,
                amount
            ),
            &[
                ctx.accounts.user_authority.to_account_info().clone(),
                ctx.accounts.state_account.to_account_info().clone(),
                ctx.accounts.system_program.to_account_info().clone()
            ]
        )?;

        let user_account = &mut ctx.accounts.user_account;
        let config = &mut ctx.accounts.config;

        user_account.borrowed_lpsol = user_account.borrowed_lpsol - amount;
        config.total_borrowed_lpsol = config.total_borrowed_lpsol - amount;

        Ok(())
    }

    pub fn update_config(
        ctx: Context<UpdateConfig>
    ) -> Result<()> {
        msg!("Update Config");

        let config = &mut ctx.accounts.config;

        
        config.btc_mint = ctx.accounts.btc_mint.key();
        config.usdc_mint = ctx.accounts.usdc_mint.key();
        config.eth_mint = ctx.accounts.eth_mint.key();
        config.msol_mint = ctx.accounts.msol_mint.key();     
        config.ust_mint = ctx.accounts.ust_mint.key();
        config.srm_mint = ctx.accounts.srm_mint.key();
        config.scnsol_mint = ctx.accounts.scnsol_mint.key();
        config.stsol_mint = ctx.accounts.stsol_mint.key();
        config.usdt_mint = ctx.accounts.usdt_mint.key();      
        config.lpusd_mint = ctx.accounts.lpusd_mint.key();
        config.lpsol_mint = ctx.accounts.lpsol_mint.key();
        config.lpbtc_mint = ctx.accounts.lpbtc_mint.key();
        config.lpeth_mint = ctx.accounts.lpeth_mint.key();
        config.pool_btc = ctx.accounts.pool_btc.key();
        config.pool_usdc = ctx.accounts.pool_usdc.key();
        config.pool_msol = ctx.accounts.pool_msol.key();
        config.pool_eth = ctx.accounts.pool_eth.key();
        config.pool_ust = ctx.accounts.pool_ust.key();
        config.pool_scnsol = ctx.accounts.pool_scnsol.key();
        config.pool_stsol = ctx.accounts.pool_stsol.key();
        config.pool_srm = ctx.accounts.pool_srm.key();
        config.pool_usdt = ctx.accounts.pool_usdt.key();
        config.pool_lpsol = ctx.accounts.pool_lpsol.key();
        config.pool_lpusd = ctx.accounts.pool_lpusd.key();
        config.pool_lpbtc = ctx.accounts.pool_lpbtc.key();
        config.pool_lpeth = ctx.accounts.pool_lpeth.key();


        Ok(())
    }

    pub fn create_user_account_lending(
        ctx: Context<CreateUserAccountLending>
    ) -> Result<()> {

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

        let cpi_program = ctx.accounts.solend_program.to_account_info();
        let cpi_accounts = solend::cpi::accounts::InitUserAccount {
            user_account: ctx.accounts.solend_account.to_account_info(),
            user_authority: ctx.accounts.state_account.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info()
        };
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

        solend::cpi::init_user_account(cpi_ctx)?;
        
        let cpi_program = ctx.accounts.apricot_program.to_account_info();
        let cpi_accounts = apricot::cpi::accounts::InitUserAccount {
            user_account: ctx.accounts.apricot_account.to_account_info(),
            user_authority: ctx.accounts.state_account.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info()
        };
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

        apricot::cpi::init_user_account(cpi_ctx)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct RepayToken<'info> {
    #[account(mut)]
    pub user_authority: Signer<'info>,
    #[account(mut)]
    pub user_dest : Box<Account<'info,TokenAccount>>,
    #[account(mut)]
    pub dest_mint: Account<'info,Mint>,
    // state account for user's wallet
    #[account(mut,
        seeds = [PREFIX.as_bytes()],
        bump
    )]
    pub state_account: Box<Account<'info, StateAccount>>,
    #[account(mut,has_one = state_account)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub dest_pool: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = user_account.owner == user_authority.key()
    )]
    pub user_account: Box<Account<'info, UserAccount>>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct RepaySOL<'info> {
    #[account(mut)]
    pub user_authority: Signer<'info>,
    #[account(mut,
        seeds = [PREFIX.as_bytes()],
        bump
    )]
    pub state_account: Box<Account<'info, StateAccount>>,
    #[account(mut)]
    pub config: Box<Account<'info, Config>>,

    // state account for user's wallet
    #[account(
        mut,
        constraint = user_account.owner == user_authority.key()
    )]
    pub user_account: Box<Account<'info, UserAccount>>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct WithdrawSOL<'info> {
    #[account(mut)]
    pub user_authority: Signer<'info>,
    #[account(mut,
        seeds = [PREFIX.as_bytes()],
        bump
    )]
    pub state_account: Box<Account<'info, StateAccount>>,
    #[account(mut, has_one= state_account)]
    pub config: Box<Account<'info, Config>>,
    // state account for user's wallet
    #[account(
        mut,
        constraint = user_account.owner == user_authority.key()
    )]
    pub user_account: Box<Account<'info, UserAccount>>,
    pub pyth_btc_account: AccountInfo<'info>,
    pub pyth_usdc_account: AccountInfo<'info>,
    pub pyth_sol_account: AccountInfo<'info>,
    pub pyth_eth_account: AccountInfo<'info>,
    pub pyth_msol_account: AccountInfo<'info>,
    pub pyth_ust_account: AccountInfo<'info>,
    pub pyth_srm_account: AccountInfo<'info>,
    pub pyth_scnsol_account: AccountInfo<'info>,
    pub pyth_stsol_account: AccountInfo<'info>,
    pub pyth_usdt_account: AccountInfo<'info>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct WithdrawToken<'info> {
    #[account(mut)]
    pub user_authority: Signer<'info>,
    // state account for user's wallet
    #[account(
        mut,
        constraint = user_account.owner == user_authority.key()
    )]
    pub user_account: Box<Account<'info, UserAccount>>,
    #[account(mut,
        seeds = [PREFIX.as_bytes()],
        bump
    )]
    pub state_account: Box<Account<'info, StateAccount>>,
    #[account(mut, has_one = state_account)]
    pub config: Box<Account<'info, Config>>,

    #[account(mut)]
    pub user_dest : Box<Account<'info,TokenAccount>>,
    #[account(mut)]
    pub dest_pool: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub dest_mint: Account<'info,Mint>,
    pub pyth_btc_account: AccountInfo<'info>,
    pub pyth_usdc_account: AccountInfo<'info>,
    pub pyth_sol_account: AccountInfo<'info>,
    pub pyth_eth_account: AccountInfo<'info>,
    pub pyth_msol_account: AccountInfo<'info>,
    pub pyth_ust_account: AccountInfo<'info>,
    pub pyth_srm_account: AccountInfo<'info>,
    pub pyth_scnsol_account: AccountInfo<'info>,
    pub pyth_stsol_account: AccountInfo<'info>,
    pub pyth_usdt_account: AccountInfo<'info>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    // Token program authority
    #[account(mut)]
    pub authority: Signer<'info>,
    // State Accounts
    #[account(init,
        seeds = [PREFIX.as_bytes()],
        bump,
        payer = authority
    )]
    pub state_account: Box<Account<'info, StateAccount>>,

    // Config Accounts
    #[account(init,
        payer = authority,
        space = 32 * 27 + 24 * 20 + 8
    )]
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

    pub lpsol_mint: Box<Account<'info, Mint>>,   
    pub lpusd_mint: Box<Account<'info, Mint>>,
    pub lpbtc_mint: Box<Account<'info, Mint>>,
    pub lpeth_mint: Box<Account<'info, Mint>>,

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
    // LpUSD POOL
    #[account(
        init,
        token::mint = lpusd_mint,
        token::authority = state_account,
        seeds = [PREFIX.as_bytes(), b"pool_lpusd".as_ref()],
        bump,
        payer = authority
    )]
    pub pool_lpusd: Box<Account<'info, TokenAccount>>,
    // LpBTC POOL
    #[account(
        init,
        token::mint = lpbtc_mint,
        token::authority = state_account,
        seeds = [PREFIX.as_bytes(), b"pool_lpbtc".as_ref()],
        bump,
        payer = authority
    )]
    pub pool_lpbtc: Box<Account<'info, TokenAccount>>,
    // LpETH POOL
    #[account(
        init,
        token::mint = lpeth_mint,
        token::authority = state_account,
        seeds = [PREFIX.as_bytes(), b"pool_lpeth".as_ref()],
        bump,
        payer = authority
    )]
    pub pool_lpeth: Box<Account<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    // Token program authority
    #[account(mut)]
    pub authority: Signer<'info>,
    // State Accounts
    #[account(mut)]
    pub state_account: Box<Account<'info, StateAccount>>,

    // Config Accounts
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
    // USDC POOL
    #[account(
        init,
        token::mint = eth_mint,
        token::authority = state_account,
        seeds = [PREFIX.as_bytes(), b"pool_eth".as_ref()],
        bump,
        payer = authority
    )]
    pub pool_eth: Box<Account<'info, TokenAccount>>,
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
    // UST POOL
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

    #[account(mut)]
    pub lpsol_mint: Box<Account<'info, Mint>>,
    
    #[account(mut)]
    pub lpusd_mint: Box<Account<'info, Mint>>,

    #[account(mut)]
    pub lpbtc_mint: Box<Account<'info, Mint>>,

    #[account(mut)]
    pub lpeth_mint: Box<Account<'info, Mint>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct CreateUserAccountLending<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut, has_one = owner)]
    pub state_account: Box<Account<'info, StateAccount>>,
    #[account(mut)]
    pub solend_account: Box<Account<'info, solend::UserAccount>>,
    #[account(mut)]
    pub apricot_account: Box<Account<'info, apricot::UserAccount>>,
    
    pub apricot_program: Program<'info, Apricot>,
    
    pub solend_program: Program<'info, Solend>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct DepositCollateral<'info> {
    #[account(mut)]
    pub user_authority: Signer<'info>,
    #[account(
        mut,
        constraint = user_collateral.owner == user_authority.key(),
        constraint = user_collateral.mint == collateral_mint.key()
    )]
    pub user_collateral : Box<Account<'info,TokenAccount>>,
    #[account(mut)]
    pub collateral_mint: Account<'info,Mint>,
    // state account for user's wallet
    #[account(mut)]
    pub state_account: Box<Account<'info, StateAccount>>,
    #[account(mut, has_one = state_account)]
    pub config: Box<Account<'info, Config>>,    
    #[account(mut)]
    pub collateral_pool: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = user_account.owner == user_authority.key()
    )]
    pub user_account: Box<Account<'info, UserAccount>>,
    #[account(mut)]
    pub whitelist: AccountLoader<'info, WhiteList>,
    #[account(mut)]
    pub solend_config: Box<Account<'info, solend::Config>>,
    #[account(mut)]
    pub solend_pool: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub solend_account: Box<Account<'info, solend::UserAccount>>,
    #[account(mut)]
    pub apricot_config: Box<Account<'info, apricot::Config>>,
    #[account(mut)]
    pub apricot_pool: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub apricot_account: Box<Account<'info, apricot::UserAccount>>,
    #[account(mut)]
    pub accounts_config: Box<Account<'info, lpfinance_accounts::Config>>,
    pub accounts_program: Program<'info, LpfinanceAccounts>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct DepositSOL<'info> {
    #[account(mut)]
    pub user_authority: Signer<'info>,
    #[account(mut,
        seeds = [PREFIX.as_bytes()],
        bump
    )]
    pub state_account: Box<Account<'info, StateAccount>>,
    // state account for user's wallet
    #[account(
        mut,
        constraint = user_account.owner == user_authority.key()
    )]
    pub user_account: Box<Account<'info, UserAccount>>,
    #[account(mut)]
    pub whitelist: AccountLoader<'info, WhiteList>,

    #[account(mut)]
    pub whitelist_config: Box<Account<'info, lpfinance_accounts::Config>>,
    #[account(mut)]
    pub config: Box<Account<'info, Config>>,
    pub accounts_program: Program<'info, LpfinanceAccounts>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct BorrowLpToken<'info> {
    #[account(mut)]
    pub user_authority: Signer<'info>,
    // state account for user's wallet
    #[account(
        mut,
        constraint = user_account.owner == user_authority.key()
    )]
    pub user_account: Box<Account<'info, UserAccount>>,
    #[account(mut,
        seeds = [PREFIX.as_bytes()],
        bump
    )]
    pub state_account: Box<Account<'info, StateAccount>>,
    #[account(mut)]
    pub tokens_state: Box<Account<'info, TokenStateAccount>>,
    #[account(mut)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub lptoken_config: Box<Account<'info, lpfinance_tokens::Config>>,
    #[account(
        init_if_needed,
        payer = user_authority,
        associated_token::mint = collateral_mint,
        associated_token::authority = user_authority
    )]
    pub user_collateral : Box<Account<'info,TokenAccount>>,
    #[account(mut)]
    pub collateral_mint: Account<'info,Mint>,
    pub pyth_btc_account: AccountInfo<'info>,
    pub pyth_eth_account: AccountInfo<'info>,
    pub pyth_usdc_account: AccountInfo<'info>,
    pub pyth_sol_account: AccountInfo<'info>,
    pub pyth_msol_account: AccountInfo<'info>,
    pub pyth_ust_account: AccountInfo<'info>,
    pub pyth_srm_account: AccountInfo<'info>,
    pub pyth_scnsol_account: AccountInfo<'info>,
    pub pyth_stsol_account: AccountInfo<'info>,
    pub pyth_usdt_account: AccountInfo<'info>,
    // Programs and Sysvars
    pub lptokens_program: Program<'info, LpfinanceTokens>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
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
    pub user_account: Box<Account<'info, UserAccount>>,
    #[account(mut)]
    pub state_account: Box<Account<'info, StateAccount>>,
    // Contract Authority accounts
    #[account(mut)]
    pub user_authority: Signer<'info>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct LiquidateCollateral<'info> {
    #[account(mut)]
    pub user_account: Box<Account<'info, UserAccount>>,
    #[account(mut)]
    pub state_account: Box<Account<'info, StateAccount>>,
    #[account(mut)]
    pub auction_account: AccountInfo<'info>,

    #[account(mut)]
    pub auction_lpusd: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub auction_lpsol: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub auction_lpbtc: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub auction_lpeth: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub auction_msol: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub auction_btc: Box<Account<'info, TokenAccount>>,
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
    pub cbs_lpusd: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub cbs_lpsol: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub cbs_lpbtc: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub cbs_lpeth: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub cbs_msol: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub cbs_btc: Box<Account<'info, TokenAccount>>,
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

#[account]
#[derive(Default)]
pub struct StateAccount {
    pub owner: Pubkey,
    pub liquidation_run: bool
}

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut, has_one = state_account)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut, has_one = owner)]
    pub state_account: Box<Account<'info, StateAccount>>,

    #[account(mut)]
    pub btc_mint: Box<Account<'info,Mint>>,
    #[account(mut)]
    pub usdc_mint: Box<Account<'info,Mint>>,
    #[account(mut)]
    pub eth_mint: Box<Account<'info,Mint>>,
    #[account(mut)]
    pub msol_mint: Box<Account<'info,Mint>>,
    #[account(mut)]
    pub ust_mint: Box<Account<'info,Mint>>,
    #[account(mut)]
    pub srm_mint: Box<Account<'info,Mint>>,
    #[account(mut)]
    pub scnsol_mint: Box<Account<'info,Mint>>,
    #[account(mut)]
    pub stsol_mint: Box<Account<'info,Mint>>,
    #[account(mut)]
    pub usdt_mint: Box<Account<'info,Mint>>,

    #[account(mut)]
    pub lpsol_mint: Box<Account<'info,Mint>>,
    #[account(mut)]
    pub lpusd_mint: Box<Account<'info,Mint>>,
    #[account(mut)]
    pub lpbtc_mint: Box<Account<'info,Mint>>,
    #[account(mut)]
    pub lpeth_mint: Box<Account<'info,Mint>>,

    #[account(mut)]
    pub pool_btc: Box<Account<'info,TokenAccount>>,
    #[account(mut)]
    pub pool_usdc: Box<Account<'info,TokenAccount>>,
    #[account(mut)]
    pub pool_msol: Box<Account<'info,TokenAccount>>,
    #[account(mut)]
    pub pool_eth: Box<Account<'info,TokenAccount>>,
    #[account(mut)]
    pub pool_ust: Box<Account<'info,TokenAccount>>,
    #[account(mut)]
    pub pool_srm: Box<Account<'info,TokenAccount>>,
    #[account(mut)]
    pub pool_scnsol: Box<Account<'info,TokenAccount>>,
    #[account(mut)]
    pub pool_stsol: Box<Account<'info,TokenAccount>>,
    #[account(mut)]
    pub pool_usdt: Box<Account<'info,TokenAccount>>,

    #[account(mut)]
    pub pool_lpsol: Box<Account<'info,TokenAccount>>,
    #[account(mut)]
    pub pool_lpusd: Box<Account<'info,TokenAccount>>,
    #[account(mut)]
    pub pool_lpbtc: Box<Account<'info,TokenAccount>>,
    #[account(mut)]
    pub pool_lpeth: Box<Account<'info,TokenAccount>>,

    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>
}

#[account]
#[derive(Default)]
pub struct Config {
    pub state_account: Pubkey,

    pub total_borrowed_lpusd: u64,
    pub total_borrowed_lpsol: u64,
    pub total_borrowed_lpbtc: u64,
    pub total_borrowed_lpeth: u64,

    pub total_deposited_sol: u64,
    pub total_deposited_usdc: u64,
    pub total_deposited_btc: u64,
    pub total_deposited_eth: u64,
    pub total_deposited_msol: u64,
    pub total_deposited_ust: u64,
    pub total_deposited_srm: u64,
    pub total_deposited_scnsol: u64,
    pub total_deposited_stsol: u64,
    pub total_deposited_usdt: u64,
    pub total_deposited_lpsol: u64,
    pub total_deposited_lpusd: u64,
    pub total_deposited_lpbtc: u64,
    pub total_deposited_lpeth: u64,

    pub lpsol_mint: Pubkey,
    pub lpusd_mint: Pubkey,
    pub lpbtc_mint: Pubkey,
    pub lpeth_mint: Pubkey,
    pub btc_mint: Pubkey,
    pub eth_mint: Pubkey,
    pub usdc_mint: Pubkey,
    pub msol_mint: Pubkey,
    pub ust_mint: Pubkey,
    pub srm_mint: Pubkey,
    pub scnsol_mint: Pubkey,
    pub stsol_mint: Pubkey,
    pub usdt_mint: Pubkey,

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
}

#[account]
#[derive(Default)]
pub struct UserAccount {
    pub borrowed_lpusd: u64,
    pub borrowed_lpsol: u64,
    pub borrowed_lpbtc: u64,
    pub borrowed_lpeth: u64,

    pub btc_amount: u64,
    pub sol_amount: u64,
    pub usdc_amount: u64,
    pub eth_amount: u64,
    pub msol_amount: u64,
    pub ust_amount: u64,
    pub srm_amount: u64,
    pub scnsol_amount: u64,
    pub stsol_amount: u64,
    pub usdt_amount: u64,

    pub lpsol_amount: u64,
    pub lpusd_amount: u64,
    pub lpeth_amount: u64,
    pub lpbtc_amount: u64,
    pub owner: Pubkey,
    pub bump: u8
}

#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient Amount")]
    InsufficientAmount,
    #[msg("Borrow Failed")]
    BorrowFailed,
    #[msg("Borrow Exceed")]
    BorrowExceed,
    #[msg("Invalid Amount")]
    InvalidAmount,
    #[msg("Invalid Token")]
    InvalidToken,
    #[msg("Invalid Owner")]
    InvalidOwner
}
