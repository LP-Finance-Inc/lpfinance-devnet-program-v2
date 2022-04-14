use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, system_instruction };
use pyth_client;
use anchor_spl::token::{self, Mint, Transfer, Token, TokenAccount };
use anchor_spl::associated_token::AssociatedToken;

declare_id!("6dMiU9ZmaFTPeLPco5rMjXCbUUyJZyRvHPccXXTefTLu");
const PREFIX: &str = "lpfiswap";

#[program]
pub mod lpfinance_swap {
    use super::*;
    pub fn initialize(
        ctx: Context<Initialize>
    ) -> Result<()> {
        msg!("INITIALIZE SWAP");

        let state_account = &mut ctx.accounts.state_account;
        let config = &mut ctx.accounts.config;

        state_account.owner = ctx.accounts.authority.key();

        config.state_account = ctx.accounts.state_account.key();

        config.lpusd_mint = ctx.accounts.lpusd_mint.key();
        config.lpsol_mint = ctx.accounts.lpsol_mint.key();
        config.lpbtc_mint = ctx.accounts.lpbtc_mint.key();
        config.lpeth_mint = ctx.accounts.lpeth_mint.key();

        config.pool_lpsol = ctx.accounts.pool_lpsol.key();
        config.pool_lpusd = ctx.accounts.pool_lpusd.key();
        config.pool_lpbtc = ctx.accounts.pool_lpbtc.key();
        config.pool_lpeth = ctx.accounts.pool_lpeth.key();

        Ok(())
    }

    pub fn initialize_pool(
        ctx: Context<InitializePool>
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

        Ok(())
    }

    pub fn swap_sol_to_token(
        ctx: Context<SwapSOLToToken>,
        quote_amount: u64
    ) -> Result<()> {
        if quote_amount == 0 {
            return Err(ErrorCode::InvalidQuoteAmount.into());
        }

        if **ctx.accounts.user_authority.lamports.borrow()  < quote_amount {
            return Err(ErrorCode::InsufficientAmount.into());
        }

        let pyth_price_info = &ctx.accounts.pyth_quote_account;
        let pyth_price_data = &pyth_price_info.try_borrow_data()?;
        let pyth_price = pyth_client::cast::<pyth_client::Price>(pyth_price_data);

        let quote_price = pyth_price.agg.price as u64;
        let quote_total: u128 = quote_price as u128 * quote_amount as u128;

        // destination token
        let pyth_price_info = &ctx.accounts.pyth_dest_account;
        let pyth_price_data = &pyth_price_info.try_borrow_data()?;
        let pyth_price = pyth_client::cast::<pyth_client::Price>(pyth_price_data);

        let dest_price = pyth_price.agg.price as u64;

        let transfer_amount = (quote_total/dest_price as u128) as u64;

        if ctx.accounts.dest_pool.amount < transfer_amount {
            return Err(ErrorCode::InsufficientAmount.into());
        }

        msg!("Sending SOL");

        invoke(
            &system_instruction::transfer(
                ctx.accounts.user_authority.key,
                ctx.accounts.state_account.to_account_info().key,
                quote_amount
            ),
            &[
                ctx.accounts.user_authority.to_account_info().clone(),
                ctx.accounts.state_account.to_account_info().clone(),
                ctx.accounts.system_program.to_account_info().clone()
            ]
        )?;

        msg!("Sending Token: !!{:?}!!", transfer_amount.to_string());
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
        token::transfer(cpi_ctx, transfer_amount)?;

        Ok(())
    }

    pub fn swap_token_to_sol(
        ctx: Context<SwapTokenToSOL>,
        quote_amount: u64
    ) -> Result<()> {
        if quote_amount == 0 {
            return Err(ErrorCode::InvalidQuoteAmount.into());
        }
        if ctx.accounts.user_quote.amount < quote_amount {
            return Err(ErrorCode::InsufficientAmount.into());
        }

        let cpi_accounts = Transfer {
            from: ctx.accounts.user_quote.to_account_info(),
            to: ctx.accounts.quote_pool.to_account_info(),
            authority: ctx.accounts.user_authority.to_account_info()
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, quote_amount)?;

        let pyth_price_info = &ctx.accounts.pyth_quote_account;
        let pyth_price_data = &pyth_price_info.try_borrow_data()?;
        let pyth_price = pyth_client::cast::<pyth_client::Price>(pyth_price_data);

        let quote_price = pyth_price.agg.price as u128;
        let quote_total = quote_price * quote_amount as u128;

        // destination token
        let pyth_price_info = &ctx.accounts.pyth_dest_account;
        let pyth_price_data = &pyth_price_info.try_borrow_data()?;
        let pyth_price = pyth_client::cast::<pyth_client::Price>(pyth_price_data);

        let dest_price = pyth_price.agg.price as u128;


        let transfer_amount = (quote_total/dest_price) as u64;
        let pool_balance = **ctx.accounts.state_account.to_account_info().lamports.borrow();
        msg!("Pool Balance: !!{:?}!!", pool_balance.to_string());

        if pool_balance < transfer_amount {
            return Err(ErrorCode::InsufficientAmount.into());
        }

        **ctx.accounts.state_account.to_account_info().try_borrow_mut_lamports()? -= transfer_amount;
        **ctx.accounts.user_authority.try_borrow_mut_lamports()? += transfer_amount;
        
        Ok(())
    }

    pub fn swap_token_to_token(
        ctx: Context<SwapTokenToToken>,
        quote_amount: u64
    ) -> Result<()> {

        if quote_amount == 0 {
            return Err(ErrorCode::InvalidQuoteAmount.into());
        }

        if ctx.accounts.user_quote.amount < quote_amount {
            return Err(ErrorCode::InsufficientAmount.into());
        }

        let pyth_price_info = &ctx.accounts.pyth_quote_account;
        let pyth_price_data = &pyth_price_info.try_borrow_data()?;
        let pyth_price = pyth_client::cast::<pyth_client::Price>(pyth_price_data);

        let quote_price = pyth_price.agg.price as u128;
        let quote_total: u128 = quote_price * quote_amount as u128;

        // destination token
        let pyth_price_info = &ctx.accounts.pyth_dest_account;
        let pyth_price_data = &pyth_price_info.try_borrow_data()?;
        let pyth_price = pyth_client::cast::<pyth_client::Price>(pyth_price_data);

        let dest_price = pyth_price.agg.price as u128;

        msg!("Quote Price: !!{:?}!!", quote_price.to_string());
        msg!("Dest Price: !!{:?}!!", dest_price.to_string());
        msg!("Quote Amount: !!{:?}!!", quote_amount.to_string());

        let transfer_amount = (quote_total/dest_price) as u64;
        msg!("Transfer Amount: !!{:?}!!", transfer_amount.to_string());
        msg!("Quote Total: !!{:?}!!", quote_total.to_string());

        let cpi_accounts = Transfer {
            from: ctx.accounts.user_quote.to_account_info(),
            to: ctx.accounts.quote_pool.to_account_info(),
            authority: ctx.accounts.user_authority.to_account_info()
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, quote_amount)?;

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
        token::transfer(cpi_ctx, transfer_amount)?;

        Ok(())
    }

    pub fn liquidate_token(
        ctx: Context<LiquidateToken>,
        amount: u64
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

        
        let cpi_accounts = Transfer {
            from: ctx.accounts.swap_pool.to_account_info(),
            to: ctx.accounts.auction_pool.to_account_info(),
            authority: ctx.accounts.state_account.to_account_info()
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, amount)?;

        Ok(())
    }
}


#[derive(Accounts)]
pub struct LiquidateToken<'info>{
    #[account(mut,
        seeds = [PREFIX.as_bytes()],
        bump
    )]
    pub state_account: Box<Account<'info, StateAccount>>,

    #[account(
        mut,
        constraint = auction_pool.mint == dest_mint.key()
    )]
    pub auction_pool : Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = swap_pool.owner == state_account.key(),
        constraint = swap_pool.mint == dest_mint.key()
    )]
    pub swap_pool : Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub dest_mint: Account<'info,Mint>,
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
        space = 32 * 27 + 8
    )]
    pub config: Box<Account<'info, Config>>,

    pub lpusd_mint: Box<Account<'info, Mint>>,
    pub lpsol_mint: Box<Account<'info, Mint>>,
    pub lpbtc_mint: Box<Account<'info, Mint>>,
    pub lpeth_mint: Box<Account<'info, Mint>>,

    // LpUSDC POOL
    #[account(
        init,
        token::mint = lpusd_mint,
        token::authority = state_account,
        seeds = [PREFIX.as_bytes(), b"pool_lpusd".as_ref()],
        bump,
        payer = authority
    )]
    pub pool_lpusd: Box<Account<'info, TokenAccount>>,
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
    #[account(mut, has_one = state_account)]
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
pub struct SwapSOLToToken<'info> {
    #[account(mut)]
    pub user_authority: Signer<'info>,
    // NOTE: this will also be SOL account.
    #[account(mut,
        seeds = [PREFIX.as_bytes()],
        bump
    )]
    pub state_account: Box<Account<'info, StateAccount>>,
    #[account(mut)]
    pub dest_mint: Account<'info,Mint>,
    // User's Token account For dest_mint
    #[account(
        init_if_needed,
        payer = user_authority,
        associated_token::mint = dest_mint,
        associated_token::authority = user_authority
    )]
    pub user_dest : Box<Account<'info, TokenAccount>>,
    // Swap Pool for dest_mint token
    #[account(
        mut,
        constraint = dest_pool.owner == state_account.key(),
        constraint = dest_pool.mint == dest_mint.key()
    )]
    pub dest_pool : Box<Account<'info, TokenAccount>>,
    // SOL pyth account
    pub pyth_quote_account: AccountInfo<'info>,
    // Token pyth account
    pub pyth_dest_account: AccountInfo<'info>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct SwapTokenToSOL<'info> {
    #[account(mut)]
    pub user_authority: Signer<'info>,
    // NOTE: this will also be SOL account.
    #[account(mut,
        seeds = [PREFIX.as_bytes()],
        bump
    )]
    pub state_account: Box<Account<'info, StateAccount>>,
    #[account(mut)]
    pub quote_mint: Account<'info,Mint>,
    #[account(
        mut,
        constraint = user_quote.owner == user_authority.key(),
        constraint = user_quote.mint == quote_mint.key()
    )]
    pub user_quote : Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = quote_pool.owner == state_account.key(),
        constraint = quote_pool.mint == quote_mint.key()
    )]
    pub quote_pool : Box<Account<'info, TokenAccount>>,
    // Token pyth
    pub pyth_quote_account: AccountInfo<'info>,
    // SOL pyth
    pub pyth_dest_account: AccountInfo<'info>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct SwapTokenToToken<'info> {
    #[account(mut)]
    pub user_authority: Signer<'info>,
    #[account(mut,
        seeds = [PREFIX.as_bytes()],
        bump
    )]
    pub state_account: Box<Account<'info, StateAccount>>,

    #[account(
        mut,
        constraint = user_quote.owner == user_authority.key(),
        constraint = user_quote.mint == quote_mint.key()
    )]
    pub user_quote : Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = quote_pool.owner == state_account.key(),
        constraint = quote_pool.mint == quote_mint.key()
    )]
    pub quote_pool : Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub quote_mint: Account<'info,Mint>,
    #[account(mut)]
    pub dest_mint: Account<'info,Mint>,
    #[account(
        init_if_needed,
        payer = user_authority,
        associated_token::mint = dest_mint,
        associated_token::authority = user_authority
    )]
    pub user_dest : Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub dest_pool : Box<Account<'info, TokenAccount>>,
    pub pyth_quote_account: AccountInfo<'info>,
    pub pyth_dest_account: AccountInfo<'info>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
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

    pub lpsol_mint: Pubkey,
    pub lpusd_mint: Pubkey,
    pub lpbtc_mint: Pubkey,
    pub lpeth_mint: Pubkey,

    pub msol_mint: Pubkey,
    pub btc_mint: Pubkey,
    pub usdc_mint: Pubkey,
    pub eth_mint: Pubkey,
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

#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone)]
pub struct SwapBumps{
    pub state_account: u8,
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

    #[msg("Invalid Quote Amount")]
    InvalidQuoteAmount,

    #[msg("Invalid Owner")]
    InvalidOwner
}
