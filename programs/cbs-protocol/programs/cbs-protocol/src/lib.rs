use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, system_instruction };
use pyth_client;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, MintTo, Transfer, Token, TokenAccount }
};

use lpfinance_accounts::cpi::accounts::AddFromCbsProgram;
use lpfinance_accounts::program::LpfinanceAccounts;
use lpfinance_accounts::{self, WhiteList, Config};

declare_id!("3YhaNLN3oYUaAXjK9yRqVVNUYhqPsVqB5q9GEJ1vWcTM");

const LP_TOKEN_DECIMALS: u8 = 9;

const LTV:u128 = 85;
const DOMINATOR:u128 = 100;

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
        ctx: Context<Initialize>,
        protocol_name: String,
        bumps: ProtocolBumps,
    ) -> Result<()> {
        msg!("INITIALIZE PROTOCAL");

        let state_account = &mut ctx.accounts.state_account;

        let name_bytes = protocol_name.as_bytes();
        let mut name_data = [b' '; 10];
        name_data[..name_bytes.len()].copy_from_slice(name_bytes);

        state_account.protocol_name = name_data;
        state_account.bumps = bumps;
        state_account.btc_mint = ctx.accounts.btc_mint.key();
        state_account.usdc_mint = ctx.accounts.usdc_mint.key();
        state_account.lpusd_mint = ctx.accounts.lpusd_mint.key();
        state_account.lpsol_mint = ctx.accounts.lpsol_mint.key();
        state_account.msol_mint = ctx.accounts.msol_mint.key();
        state_account.pool_btc = ctx.accounts.pool_btc.key();
        state_account.pool_usdc = ctx.accounts.pool_usdc.key();
        state_account.pool_msol = ctx.accounts.pool_msol.key();
        state_account.pool_lpsol = ctx.accounts.pool_lpsol.key();
        state_account.pool_lpusd = ctx.accounts.pool_lpusd.key();
        state_account.owner = ctx.accounts.authority.key();

        state_account.total_borrowed_lpsol = 0;
        state_account.total_borrowed_lpusd = 0;

        state_account.total_deposited_sol = 0;
        state_account.total_deposited_usdc = 0;
        state_account.total_deposited_btc = 0;
        state_account.total_deposited_lpsol = 0;
        state_account.total_deposited_lpusd = 0;
        state_account.total_deposited_msol = 0;

        state_account.liquidation_run = false;

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
        Ok(())
    }

    pub fn deposit_collateral(
        ctx: Context<DepositCollateral>,
        amount: u64,
        _pool_bump: u8,
        _pool_seed: String
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
        let state_account = &mut ctx.accounts.state_account;

        if ctx.accounts.user_collateral.mint == state_account.btc_mint {
            user_account.btc_amount = user_account.btc_amount + amount;
            state_account.total_deposited_btc = state_account.total_deposited_btc + amount;
        }

        if ctx.accounts.user_collateral.mint == state_account.usdc_mint {
            user_account.usdc_amount = user_account.usdc_amount + amount;
            state_account.total_deposited_usdc = state_account.total_deposited_usdc + amount;
        }

        if ctx.accounts.user_collateral.mint == state_account.lpusd_mint {
            user_account.lpusd_amount = user_account.lpusd_amount + amount;
            state_account.total_deposited_lpusd = state_account.total_deposited_lpusd + amount;
        }

        if ctx.accounts.user_collateral.mint == state_account.lpsol_mint {
            user_account.lpsol_amount = user_account.lpsol_amount + amount;
            state_account.total_deposited_lpsol = state_account.total_deposited_lpsol + amount;
        }

        if ctx.accounts.user_collateral.mint == state_account.msol_mint {
            user_account.msol_amount = user_account.msol_amount + amount;
            state_account.total_deposited_msol = state_account.total_deposited_msol + amount;
        }

        // let whitelist = ctx.accounts.whitelist.load_mut()?;
        if ctx.accounts.whitelist.load_mut()?.addresses.contains(&ctx.accounts.user_authority.key()) {
            msg!("Already Exist");
        } else {

            let cpi_program = ctx.accounts.accounts_program.to_account_info();
            let cpi_accounts = AddFromCbsProgram {
                config: ctx.accounts.config.to_account_info(),
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
        let state_account = &mut ctx.accounts.state_account;

        user_account.sol_amount = user_account.sol_amount + amount;
        state_account.total_deposited_sol = state_account.total_deposited_sol + amount;

        // let whitelist = ctx.accounts.whitelist.load_mut()?;
        if ctx.accounts.whitelist.load_mut()?.addresses.contains(&ctx.accounts.user_authority.key()) {
            msg!("Already Exist");
        } else {

            let cpi_program = ctx.accounts.accounts_program.to_account_info();
            let cpi_accounts = AddFromCbsProgram {
                config: ctx.accounts.config.to_account_info(),
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
        islpusd: bool,
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
        let state_account = &mut ctx.accounts.state_account;

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

        // LpUSD price
        let lpusd_price = usdc_price;        
        total_price += lpusd_price * user_account.lpusd_amount as u128;

        // LpSOL price
        let lpsol_price = sol_price;
        total_price += lpsol_price * user_account.lpsol_amount as u128;
        // Total Borrowed AMount
        total_borrowed_price += lpusd_price * user_account.borrowed_lpusd as u128;
        total_borrowed_price += lpsol_price * user_account.borrowed_lpsol as u128;

        let mut borrow_value: u128 = amount as u128;
        
        if islpusd {
            borrow_value = borrow_value * lpusd_price;
            state_account.total_borrowed_lpusd = state_account.total_borrowed_lpusd + amount;
        } else {
            borrow_value = borrow_value * lpsol_price;
            state_account.total_borrowed_lpsol = state_account.total_borrowed_lpsol + amount;
        }

        let borrable_total = total_price * LTV / DOMINATOR - total_borrowed_price;

        if borrable_total > borrow_value {
            // Mint
            let seeds = &[
                ctx.accounts.state_account.protocol_name.as_ref(),
                &[ctx.accounts.state_account.bumps.state_account]
            ];
            let signer = &[&seeds[..]];
            let cpi_accounts = MintTo {
                mint: ctx.accounts.collateral_mint.to_account_info(),
                to: ctx.accounts.user_collateral.to_account_info(),
                authority: ctx.accounts.state_account.to_account_info(),
            };

            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

            token::mint_to(cpi_ctx, amount)?;

            if islpusd {
                user_account.borrowed_lpusd = user_account.borrowed_lpusd + amount;
            } else {
                user_account.borrowed_lpsol = user_account.borrowed_lpsol + amount;
            }
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
        let usdc_amount = user_account.usdc_amount;
        let btc_amount = user_account.btc_amount;
        let sol_amount = user_account.sol_amount;
        let msol_amount = user_account.msol_amount;

        let seeds = &[
            ctx.accounts.state_account.protocol_name.as_ref(),
            &[ctx.accounts.state_account.bumps.state_account],
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
        msg!("sol_amount started");

        if sol_amount > 0 {
            **ctx.accounts.state_account.to_account_info().try_borrow_mut_lamports()? -= sol_amount;
            **ctx.accounts.auction_account.try_borrow_mut_lamports()? += sol_amount;
        }
        msg!("sol_amount ended");

        user_account.lpusd_amount = 0;
        user_account.lpsol_amount = 0;
        user_account.usdc_amount = 0;
        user_account.btc_amount = 0;
        user_account.sol_amount = 0;
        user_account.msol_amount = 0;

        user_account.borrowed_lpusd = 0;
        user_account.borrowed_lpsol = 0;
        
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
        let lpsol_amount = user_account.lpsol_amount as u128;
        let msol_amount = user_account.msol_amount as u128;
        let lpusd_amount = user_account.lpusd_amount as u128;
        let borrowed_lpusd = user_account.borrowed_lpusd as u128;
        let borrowed_lpsol = user_account.borrowed_lpsol as u128;

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

        // LpUSD price
        let lpusd_price = usdc_price;        
        total_price += lpusd_price * lpusd_amount;

        // LpSOL price
        let lpsol_price = sol_price;
        total_price += lpsol_price * lpsol_amount;

        let mut borrowed_total: u128 = 0;
        borrowed_total += borrowed_lpsol * lpsol_price;
        borrowed_total += borrowed_lpusd * lpusd_price;

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
        ctx.accounts.state_account.total_deposited_sol -= amount;

        Ok(())
    }

    pub fn withdraw_token(
        ctx: Context<WithdrawToken>,
        amount: u64
    ) -> Result<()> {
        let user_account = &mut ctx.accounts.user_account;
        let sol_amount = user_account.sol_amount as u128;
        let btc_amount = user_account.btc_amount as u128;
        let usdc_amount = user_account.usdc_amount as u128;
        let lpsol_amount = user_account.lpsol_amount as u128;
        let msol_amount = user_account.msol_amount as u128;
        let lpusd_amount = user_account.lpusd_amount as u128;
        let borrowed_lpusd = user_account.borrowed_lpusd as u128;
        let borrowed_lpsol = user_account.borrowed_lpsol as u128;

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

        // LpUSD price
        let lpusd_price = usdc_price;        
        total_price += lpusd_price * lpusd_amount;

        // LpSOL price
        let lpsol_price = sol_price;
        total_price += lpsol_price * lpsol_amount;

        let mut borrowed_total: u128 = 0;
        borrowed_total += borrowed_lpsol * lpsol_price;
        borrowed_total += borrowed_lpusd * lpusd_price;

        if total_price * LTV < borrowed_total * DOMINATOR {
            return Err(ErrorCode::InvalidAmount.into());
        }        
        
        let mut dest_price:u128;
        let mut owned_amount:u128;
        if ctx.accounts.dest_mint.key() == ctx.accounts.state_account.usdc_mint {
            dest_price = usdc_price;
            owned_amount = usdc_amount;
        } else if ctx.accounts.dest_mint.key() == ctx.accounts.state_account.lpusd_mint {
            dest_price = lpusd_price;
            owned_amount = lpusd_amount;
        } else if ctx.accounts.dest_mint.key() == ctx.accounts.state_account.lpsol_mint {
            dest_price = lpsol_price;
            owned_amount = lpsol_amount;
        } else if ctx.accounts.dest_mint.key() == ctx.accounts.state_account.msol_mint {
            dest_price = msol_price;
            owned_amount = msol_amount;
        } else if ctx.accounts.dest_mint.key() == ctx.accounts.state_account.btc_mint {
            dest_price = btc_price;
            owned_amount = btc_amount;
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
        
        let seeds = &[
            ctx.accounts.state_account.protocol_name.as_ref(),
            &[ctx.accounts.state_account.bumps.state_account],
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

        

        if ctx.accounts.dest_mint.key() == ctx.accounts.state_account.usdc_mint {
            user_account.usdc_amount -= amount;
            ctx.accounts.state_account.total_deposited_usdc -= amount;
        } else if ctx.accounts.dest_mint.key() == ctx.accounts.state_account.lpusd_mint {
            user_account.lpusd_amount -= amount;
            ctx.accounts.state_account.total_deposited_lpusd -= amount;
        } else if ctx.accounts.dest_mint.key() == ctx.accounts.state_account.lpsol_mint {
            user_account.lpsol_amount -= amount;
            ctx.accounts.state_account.total_deposited_lpsol -= amount;
        } else if ctx.accounts.dest_mint.key() == ctx.accounts.state_account.msol_mint {
            user_account.msol_amount -= amount;
            ctx.accounts.state_account.total_deposited_msol -= amount;
        } else if ctx.accounts.dest_mint.key() == ctx.accounts.state_account.btc_mint {
            user_account.btc_amount -= amount;
            ctx.accounts.state_account.total_deposited_btc -= amount;
        }

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
        let state_account = &mut ctx.accounts.state_account;

        if ctx.accounts.user_dest.mint == state_account.usdc_mint {

            let cpi_accounts = Transfer {
                from: ctx.accounts.user_dest.to_account_info(),
                to: ctx.accounts.dest_pool.to_account_info(),
                authority: ctx.accounts.user_authority.to_account_info()
            };
    
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
            token::transfer(cpi_ctx, amount)?;

            user_account.borrowed_lpusd = user_account.borrowed_lpusd - amount;
            state_account.total_borrowed_lpusd = state_account.total_borrowed_lpusd - amount;  
        } else if ctx.accounts.user_dest.mint == state_account.lpusd_mint {

            let cpi_ctx = CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Burn {
                    mint: ctx.accounts.dest_mint.to_account_info(),
                    to: ctx.accounts.user_dest.to_account_info(),
                    authority: ctx.accounts.user_authority.to_account_info()
                }
            );

            token::burn(cpi_ctx, amount)?;

            user_account.borrowed_lpusd = user_account.borrowed_lpusd - amount;
            state_account.total_borrowed_lpusd = state_account.total_borrowed_lpusd - amount;            
        } else if ctx.accounts.user_dest.mint == state_account.lpsol_mint {

            let cpi_ctx = CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Burn {
                    mint: ctx.accounts.dest_mint.to_account_info(),
                    to: ctx.accounts.user_dest.to_account_info(),
                    authority: ctx.accounts.user_authority.to_account_info()
                }
            );

            token::burn(cpi_ctx, amount)?;

            user_account.borrowed_lpsol = user_account.borrowed_lpsol - amount;
            state_account.total_borrowed_lpsol = state_account.total_borrowed_lpsol - amount;            
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
        let state_account = &mut ctx.accounts.state_account;

        user_account.borrowed_lpsol = user_account.borrowed_lpsol - amount;
        state_account.total_borrowed_lpsol = state_account.total_borrowed_lpsol - amount;

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
        seeds = [state_account.protocol_name.as_ref()],
        bump= state_account.bumps.state_account
    )]
    pub state_account: Box<Account<'info, StateAccount>>,
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
        seeds = [state_account.protocol_name.as_ref()],
        bump= state_account.bumps.state_account
    )]
    pub state_account: Box<Account<'info, StateAccount>>,
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
        seeds = [state_account.protocol_name.as_ref()],
        bump= state_account.bumps.state_account
    )]
    pub state_account: Box<Account<'info, StateAccount>>,
    // state account for user's wallet
    #[account(
        mut,
        constraint = user_account.owner == user_authority.key()
    )]
    pub user_account: Box<Account<'info, UserAccount>>,
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
        seeds = [state_account.protocol_name.as_ref()],
        bump= state_account.bumps.state_account
    )]
    pub state_account: Box<Account<'info, StateAccount>>,
    #[account(mut)]
    pub user_dest : Box<Account<'info,TokenAccount>>,
    #[account(mut)]
    pub dest_pool: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub dest_mint: Account<'info,Mint>,
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
#[instruction(protocol_name: String, bumps: ProtocolBumps)]
pub struct Initialize<'info> {
    // Token program authority
    #[account(mut)]
    pub authority: Signer<'info>,
    // State Accounts
    #[account(init,
        seeds = [protocol_name.as_bytes()],
        bump,
        payer = authority
    )]
    pub state_account: Box<Account<'info, StateAccount>>,
    
    pub usdc_mint: Box<Account<'info, Mint>>,
    pub btc_mint: Box<Account<'info, Mint>>,
    pub msol_mint: Box<Account<'info, Mint>>,
    // USDC POOL
    #[account(
        init,
        token::mint = usdc_mint,
        token::authority = state_account,
        seeds = [protocol_name.as_bytes(), b"pool_usdc".as_ref()],
        bump,
        payer = authority
    )]
    pub pool_usdc: Box<Account<'info, TokenAccount>>,
    // BTC POOL
    #[account(
        init,
        token::mint = btc_mint,
        token::authority = state_account,
        seeds = [protocol_name.as_bytes(), b"pool_btc".as_ref()],
        bump,
        payer = authority
    )]
    pub pool_btc: Box<Account<'info, TokenAccount>>,
    // mSOL POOL
    #[account(
        init,
        token::mint = msol_mint,
        token::authority = state_account,
        seeds = [protocol_name.as_bytes(), b"pool_msol".as_ref()],
        bump,
        payer = authority
    )]
    pub pool_msol: Box<Account<'info, TokenAccount>>,

    #[account(init,
        mint::decimals = LP_TOKEN_DECIMALS,
        mint::authority = state_account,
        seeds = [protocol_name.as_bytes(), b"lpsol_mint".as_ref()],
        bump,
        payer = authority
    )]
    pub lpsol_mint: Box<Account<'info, Mint>>,
    

    #[account(init,
        mint::decimals = LP_TOKEN_DECIMALS,
        mint::authority = state_account,
        seeds = [protocol_name.as_bytes(), b"lpusd_mint".as_ref()],
        bump,
        payer = authority
    )]
    pub lpusd_mint: Box<Account<'info, Mint>>,
    // LpSOL POOL
    #[account(
        init,
        token::mint = lpsol_mint,
        token::authority = state_account,
        seeds = [protocol_name.as_bytes(), b"pool_lpsol".as_ref()],
        bump,
        payer = authority
    )]
    pub pool_lpsol: Box<Account<'info, TokenAccount>>,
    // LpUSDC POOL
    #[account(
        init,
        token::mint = lpusd_mint,
        token::authority = state_account,
        seeds = [protocol_name.as_bytes(), b"pool_lpusd".as_ref()],
        bump,
        payer = authority
    )]
    pub pool_lpusd: Box<Account<'info, TokenAccount>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}


#[derive(Accounts)]
#[instruction(amount: u64, pool_bump: u8, pool_seed: String)]
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
    #[account(mut,
        seeds = [state_account.protocol_name.as_ref()],
        bump= state_account.bumps.state_account
    )]
    pub state_account: Box<Account<'info, StateAccount>>,
    #[account(
        mut,
        seeds = [state_account.protocol_name.as_ref(), pool_seed.as_ref()],
        bump = pool_bump)]
    pub collateral_pool: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = user_account.owner == user_authority.key()
    )]
    pub user_account: Box<Account<'info, UserAccount>>,
    #[account(mut)]
    pub whitelist: AccountLoader<'info, WhiteList>,
    #[account(mut)]
    pub config: Box<Account<'info, Config>>,
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
        seeds = [state_account.protocol_name.as_ref()],
        bump= state_account.bumps.state_account
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
        seeds = [state_account.protocol_name.as_ref()],
        bump= state_account.bumps.state_account
    )]
    pub state_account: Box<Account<'info, StateAccount>>,
    #[account(
        mut,
        constraint = user_collateral.owner == user_authority.key(),
        constraint = user_collateral.mint == collateral_mint.key()
    )]
    pub user_collateral : Box<Account<'info,TokenAccount>>,
    #[account(mut)]
    pub collateral_mint: Account<'info,Mint>,
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
pub struct InitUserAccount<'info> {
    // State account for each user/wallet
    #[account(
        init,
        seeds = [state_account.protocol_name.as_ref(), user_authority.key().as_ref()],
        bump,
        payer = user_authority
    )]
    pub user_account: Account<'info, UserAccount>,
    #[account(init,
        payer = user_authority,
        associated_token::mint = lpusd_mint,
        associated_token::authority = user_authority,
        space = 8 + 42
    )]
    pub user_lpusd: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub lpusd_mint: Account<'info,Mint>,
    #[account(init,
        payer = user_authority,
        associated_token::mint = lpsol_mint,
        associated_token::authority = user_authority,
        space = 8 + 42
    )]
    pub user_lpsol: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub lpsol_mint: Account<'info,Mint>,
    #[account(mut)]
    pub state_account: Box<Account<'info, StateAccount>>,
    // Contract Authority accounts
    #[account(mut)]
    pub user_authority: Signer<'info>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
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
    pub auction_msol: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub auction_btc: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub auction_usdc: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub cbs_lpusd: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub cbs_lpsol: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub cbs_msol: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub cbs_btc: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub cbs_usdc: Box<Account<'info, TokenAccount>>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}

#[account]
#[derive(Default)]
pub struct StateAccount {
    pub protocol_name: [u8; 10],
    pub bumps: ProtocolBumps,
    pub owner: Pubkey,
    pub total_borrowed_lpusd: u64,
    pub total_borrowed_lpsol: u64,
    pub total_deposited_sol: u64,
    pub total_deposited_usdc: u64,
    pub total_deposited_btc: u64,
    pub total_deposited_lpsol: u64,
    pub total_deposited_lpusd: u64,
    pub total_deposited_msol: u64,
    pub lpsol_mint: Pubkey,
    pub lpusd_mint: Pubkey,
    pub btc_mint: Pubkey,
    pub usdc_mint: Pubkey,
    pub msol_mint: Pubkey,
    pub pool_btc: Pubkey,
    pub pool_usdc: Pubkey,
    pub pool_lpsol: Pubkey,
    pub pool_lpusd: Pubkey,
    pub pool_msol: Pubkey,
    pub liquidation_run: bool
}

#[account]
#[derive(Default)]
pub struct UserAccount {
    pub borrowed_lpusd: u64,
    pub borrowed_lpsol: u64,
    pub btc_amount: u64,
    pub sol_amount: u64,
    pub usdc_amount: u64,
    pub lpsol_amount: u64,
    pub lpusd_amount: u64,
    pub msol_amount: u64,
    pub owner: Pubkey,
    pub bump: u8
}

#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone)]
pub struct ProtocolBumps{
    pub state_account: u8,
    pub lpusd_mint: u8,
    pub lpsol_mint: u8,
    pub pool_usdc: u8,
    pub pool_btc: u8,
    pub pool_lpusd: u8,
    pub pool_lpsol: u8,
    pub pool_msol: u8,
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
    InvalidToken
}