use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, system_instruction };
use pyth_client;
use anchor_spl::token::{self, Mint, Transfer, Token, TokenAccount };
use anchor_spl::associated_token::AssociatedToken;

declare_id!("9jBjsXqKo6W54Hf65wrgR9k9AVYuCfDQQNUfygFtjWPJ");

#[program]
pub mod lpfinance_swap {
    use super::*;
    pub fn initialize(
        ctx: Context<Initialize>,
        swap_name: String,
        bumps: SwapBumps,
    ) -> Result<()> {
        msg!("INITIALIZE SWAP");

        let state_account = &mut ctx.accounts.state_account;

        let name_bytes = swap_name.as_bytes();
        let mut name_data = [b' '; 10];
        name_data[..name_bytes.len()].copy_from_slice(name_bytes);

        state_account.swap_name = name_data;
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

        state_account.owner = ctx.accounts.authority.key();
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
        let seeds = &[
            ctx.accounts.state_account.swap_name.as_ref(),
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

        let seeds = &[
            ctx.accounts.state_account.swap_name.as_ref(),
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
        token::transfer(cpi_ctx, transfer_amount)?;

        Ok(())
    }

    pub fn liquidate_token(
        ctx: Context<LiquidateToken>,
        amount: u64
    ) -> Result<()> {
        let seeds = &[
            ctx.accounts.state_account.swap_name.as_ref(),
            &[ctx.accounts.state_account.bumps.state_account],
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
        seeds = [state_account.swap_name.as_ref()],
        bump= state_account.bumps.state_account
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
#[instruction(swap_name: String, bumps: SwapBumps)]
pub struct Initialize<'info> {
    // Token program authority
    #[account(mut)]
    pub authority: Signer<'info>,
    // State Accounts
    #[account(init,
        seeds = [swap_name.as_bytes()],
        bump,
        payer = authority
    )]
    pub state_account: Box<Account<'info, StateAccount>>,
    
    pub usdc_mint: Box<Account<'info, Mint>>,
    pub btc_mint: Box<Account<'info, Mint>>,
    pub lpsol_mint: Box<Account<'info, Mint>>,
    pub msol_mint: Box<Account<'info, Mint>>,
    pub lpusd_mint: Box<Account<'info, Mint>>,

    // USDC POOL
    #[account(
        init,
        token::mint = usdc_mint,
        token::authority = state_account,
        seeds = [swap_name.as_bytes(), b"pool_usdc".as_ref()],
        bump,
        payer = authority
    )]
    pub pool_usdc: Box<Account<'info, TokenAccount>>,
    // BTC POOL
    #[account(
        init,
        token::mint = btc_mint,
        token::authority = state_account,
        seeds = [swap_name.as_bytes(), b"pool_btc".as_ref()],
        bump,
        payer = authority
    )]
    pub pool_btc: Box<Account<'info, TokenAccount>>,
    // LpSOL POOL
    #[account(
        init,
        token::mint = lpsol_mint,
        token::authority = state_account,
        seeds = [swap_name.as_bytes(), b"pool_lpsol".as_ref()],
        bump,
        payer = authority
    )]
    pub pool_lpsol: Box<Account<'info, TokenAccount>>,
    // mSOL POOL
    #[account(
        init,
        token::mint = msol_mint,
        token::authority = state_account,
        seeds = [swap_name.as_bytes(), b"pool_msol".as_ref()],
        bump,
        payer = authority
    )]
    pub pool_msol: Box<Account<'info, TokenAccount>>,
    // LpUSDC POOL
    #[account(
        init,
        token::mint = lpusd_mint,
        token::authority = state_account,
        seeds = [swap_name.as_bytes(), b"pool_lpusd".as_ref()],
        bump,
        payer = authority
    )]
    pub pool_lpusd: Box<Account<'info, TokenAccount>>,
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
        seeds = [state_account.swap_name.as_ref()],
        bump= state_account.bumps.state_account
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
        seeds = [state_account.swap_name.as_ref()],
        bump= state_account.bumps.state_account
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
        seeds = [state_account.swap_name.as_ref()],
        bump= state_account.bumps.state_account
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
    pub swap_name: [u8; 10],
    pub bumps: SwapBumps,
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
