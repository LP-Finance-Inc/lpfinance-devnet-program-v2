// Migrations are an early feature. Currently, they're nothing more than this
// single deploy script that's invoked from the CLI, injecting a provider
// configured from the workspace's Anchor.toml.

const anchor = require("@project-serum/anchor");
const TOKEN = require("@solana/spl-token");
const { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID, Token } = require('@solana/spl-token')
const { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Keypair } = anchor.web3;

const idl = require("../target/idl/lpfinance_swap.json");
const programID = idl.metadata.address;

console.log("ProgramID", programID);
const PREFIX = "lpfiswap";

const pool_usdc = "pool_usdc";
const pool_btc = "pool_btc";
const pool_msol = "pool_msol";
const pool_eth = "pool_eth";
const pool_ust = "pool_ust";
const pool_srm = "pool_srm";
const pool_scnsol = "pool_scnsol";
const pool_stsol = "pool_stsol";
const pool_usdt = "pool_usdt";

const pool_lpsol = "pool_lpsol";
const pool_lpusd = "pool_lpusd";
const pool_lpbtc = "pool_lpbtc";
const pool_lpeth = "pool_lpeth";

// Test Token's MINT
const usdcMint = new PublicKey("8cCs2Th4ivThrJPrkgAWNTegQgMcuBmY7TASv7FPhitj"); 
const btcMint = new PublicKey("25ggxgxMqejf5v9WSQWboqxpsrik1u94PCP5EwPBYeEJ");
const msolMint = new PublicKey("3dDwpZWQqCc5SttGJ2yNnYUnLSBnh9cjWJQPeKNDmDTz");
const ethMint = new PublicKey("6Y9PaAZjDs2n4ZJonCu2uCjRp8tuqe6KJEDs1k6iLkbD");
const ustMint = new PublicKey("CZqXAbuUzGngd97oLjR1bcWkkZrz7MsKAbTJX9oT5Epv"); 
const srmMint = new PublicKey("GB8u3PRkQoi73v5Tctqj5he4M441S2QfqMpcaAsnozE6");
const scnsolMint = new PublicKey("GXFmXhwBMfXq5utccyNcQRrfQuBVjjprHKSqLzi3P7vn");
const stsolMint = new PublicKey("CJGeMYvL7s2k8VHooJ1JvgZsCJqrSEExmPkpFBZskAfV");
const usdtMint = new PublicKey("DpsmMkLP5yAeBSh7yAMHNuBurLnc8LNxvoddAoKo27dk");

const lpsolMint = new PublicKey("9Mcq5PQsEXuSY19ei8CqzRawPdPSAH1VM63GqtZU3x18"); 
const lpusdMint = new PublicKey("8YawjpcTDs3SsR7bsCHDb4b1Yv3PAKULB5xZ5VNunroJ");
const lpbtcMint = new PublicKey("B8w6e1gSCHE4xNhPhaK5y3cYYBwKMmfJqfe3C9692mGW");
const lpethMint = new PublicKey("8ZwwTyZ3PSyAzpqPeTXnvxdTF88CxzDQ57hF48WQvK7c");

module.exports = async function (provider) {
  // Configure client to use the provider.
  anchor.setProvider(provider);

  // Add your deploy script here
  const program = new anchor.Program(idl, programID);

  try {

    // const configAccount = anchor.web3.Keypair.generate();
    // console.log("Config: ", configAccount.publicKey.toBase58());

    // Find PDA from `cbs protocol` for state account
    const [stateAccount, stateAccountBump] = await PublicKey.findProgramAddress(
      [Buffer.from(PREFIX)],
      program.programId
    );
    console.log("State-Account:", stateAccount.toBase58());

    // Find PDA for `usdc pool`
    const [poolUsdc, poolUsdcBump] = await PublicKey.findProgramAddress(
      [Buffer.from(PREFIX), Buffer.from(pool_usdc)],
      program.programId
    );
    console.log("Pool-USDC:", poolUsdc.toBase58());

    // Find PDA for `btc pool`
    const [poolBtc, poolBtcBump] = await PublicKey.findProgramAddress(
      [Buffer.from(PREFIX), Buffer.from(pool_btc)],
      program.programId
    );
    console.log("Pool-BTC:", poolBtc.toBase58());


    // Find PDA for `msol pool`
    const [poolMsol, poolMsolBump] = await PublicKey.findProgramAddress(
      [Buffer.from(PREFIX), Buffer.from(pool_msol)],
      program.programId
    );
    console.log("Pool-MSOL:", poolMsol.toBase58());


    // Find PDA for `eth pool`
    const [poolEth, poolEthBump] = await PublicKey.findProgramAddress(
      [Buffer.from(PREFIX), Buffer.from(pool_eth)],
      program.programId
    );
    console.log("Pool-ETH:", poolEth.toBase58());

    // Find PDA for `ust pool`
    const [poolUst, poolUstBump] = await PublicKey.findProgramAddress(
      [Buffer.from(PREFIX), Buffer.from(pool_ust)],
      program.programId
    );
    console.log("Pool-UST:", poolUst.toBase58());

    // Find PDA for `srm pool`
    const [poolSrm, poolSrmBump] = await PublicKey.findProgramAddress(
      [Buffer.from(PREFIX), Buffer.from(pool_srm)],
      program.programId
    );
    console.log("Pool-SRM:", poolSrm.toBase58());

    // Find PDA for `scnsol pool`
    const [poolScnsol, poolScnsolBump] = await PublicKey.findProgramAddress(
      [Buffer.from(PREFIX), Buffer.from(pool_scnsol)],
      program.programId
    );
    console.log("Pool-SCNSOL:", poolScnsol.toBase58());

    // Find PDA for `stsol pool`
    const [poolStsol, poolStsolBump] = await PublicKey.findProgramAddress(
      [Buffer.from(PREFIX), Buffer.from(pool_stsol)],
      program.programId
    );
    console.log("Pool-STSOL:", poolStsol.toBase58());
    
    // Find PDA for `usdt pool`
    const [poolUsdt, poolUsdtBump] = await PublicKey.findProgramAddress(
      [Buffer.from(PREFIX), Buffer.from(pool_usdt)],
      program.programId
    );
    console.log("Pool-USDT:", poolUsdt.toBase58());

    // Find PDA for `lpusd pool`
    const [poolLpusd, poolLpusdBump] = await PublicKey.findProgramAddress(
      [Buffer.from(PREFIX), Buffer.from(pool_lpusd)],
      program.programId
    );
    console.log("Pool-LpUSD:", poolLpusd.toBase58());

    // Find PDA for `lpsol pool`
    const [poolLpsol, poolLpsolBump] = await PublicKey.findProgramAddress(
      [Buffer.from(PREFIX), Buffer.from(pool_lpsol)],
      program.programId
    );
    console.log("Pool-LpSOL:", poolLpsol.toBase58());

    // Find PDA for `lpbtc pool`
    const [poolLpbtc, poolLpbtcBump] = await PublicKey.findProgramAddress(
      [Buffer.from(PREFIX), Buffer.from(pool_lpbtc)],
      program.programId
    );
    console.log("Pool-LpBTC:", poolLpbtc.toBase58());

    // Find PDA for `lpeth pool`
    const [poolLpeth, poolLpethBump] = await PublicKey.findProgramAddress(
      [Buffer.from(PREFIX), Buffer.from(pool_lpeth)],
      program.programId
    );
    console.log("Pool-LpETH:", poolLpeth.toBase58());

    // Signer
    const authority = provider.wallet.publicKey;
    
    const config = new PublicKey("7jgbQyMsLkinSJ6fQHtUHwaUyKdwkGGE7scTFeTV8qzw");

    // initialize
    // await program.rpc.initialize({
    //   accounts: {
    //     authority,
    //     stateAccount,
    //     config,
    //     // config: configAccount.publicKey,
    //     lpusdMint,
    //     lpsolMint,
    //     lpbtcMint,
    //     lpethMint,
    //     poolLpbtc,
    //     poolLpeth,
    //     poolLpsol,
    //     poolLpusd,
    //     systemProgram: SystemProgram.programId,
    //     tokenProgram: TOKEN_PROGRAM_ID,
    //     rent: SYSVAR_RENT_PUBKEY,
    //   },
    //   signers: [configAccount]
    // });

    await program.rpc.initializePool({
      accounts: {
        authority,
        stateAccount,
        config,
        // config: configAccount.publicKey,
        usdcMint,
        btcMint,
        msolMint,
        ethMint,
        ustMint,
        srmMint,
        scnsolMint,
        stsolMint,
        usdtMint,
        poolUsdc,
        poolEth,
        poolBtc,
        poolMsol,
        poolUst,
        poolSrm,
        poolScnsol,
        poolStsol,
        poolUsdt,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
      }
    });

  } catch (err) {
    console.log("Transaction error: ", err);
  }
}

// 2022-04-13 devnet
// ProgramID 6dMiU9ZmaFTPeLPco5rMjXCbUUyJZyRvHPccXXTefTLu
// Config-Account: 7jgbQyMsLkinSJ6fQHtUHwaUyKdwkGGE7scTFeTV8qzw
// State-Account: 35oKiStiHmkrfCaFEyHs5suMiLHsM5VsAFQ3peKknkDV
// Pool-USDC: DPeobw5yJS1dkfZaE2Z67gGXsfdDfq5PdXpkBY5HheLG
// Pool-BTC: 84mg2xQAYsVLLGCesLWh1As8YicVkfcWiGKwDes2xg5R
// Pool-MSOL: Ci8PRat8mtgspMyVxHJLj4G32TXJphBKdex8Ka2Ej2TF
// Pool-ETH: 4KcLB2PVsitzKu2pzHuQF2ACB8EQVnKa463Wjn8EtwkB
// Pool-UST: F9uxM2ijA2wVBbnbX92QAP85PfxwaqMBiYA1QoNZrrst
// Pool-SRM: 9cZi4DnSWELQPFaZdSD5fWD77sPEeGALPQhz5E7FYMFT
// Pool-SCNSOL: F4RFismMeTCaDjVGwkTv6oHaNEipQu43EbpbszfFxvAz
// Pool-STSOL: CSQvYuTZzuFfSx8ZxvDnhFkB7gVJHsQEiw8w36KCwmEN
// Pool-USDT: Gdqm6TiL1rnHCXaurBDr4Tek5t1jFNeiAzxc5KEZYMvD
// Pool-LpUSD: 6PTciQETNSwB3FkiivjnU4KLWTr78KBhuPfBt8TazUWZ
// Pool-LpSOL: BTX4Wauvb4GRkThrwXBJpGfm3AkqGCKAWhnD4z9MSsPs
// Pool-LpBTC: HSWbnqVXb8YHMha3rk2Hc2dedP7iLZtvKETzj33cF6FC
// Pool-LpETH: 8YfoUygmJfPgBmGaNqX7oWC2ZggYexJ6Bs21P7f1ct3C