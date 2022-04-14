// Migrations are an early feature. Currently, they're nothing more than this
// single deploy script that's invoked from the CLI, injecting a provider
// configured from the workspace's Anchor.toml.

const anchor = require("@project-serum/anchor");
const TOKEN = require("@solana/spl-token");
const { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID, Token } = require('@solana/spl-token')
const { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Keypair } = anchor.web3;

const idl = require("../target/idl/lpusd_auction.json");
const programID = idl.metadata.address;

console.log("ProgramID", programID);
const PREFIX = "lpauction0";

const pool_usdc = "pool_usdc";
const pool_btc = "pool_btc";
const pool_eth = "pool_eth";
const pool_msol = "pool_msol";
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
    const configAccount = anchor.web3.Keypair.generate();
    console.log("Config: ", configAccount.publicKey.toBase58());
    
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
    console.log("Pool-mSOL:", poolMsol.toBase58());

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

    // Find PDA for `lpsol pool`
    const [poolLpsol, poolLpsolBump] = await PublicKey.findProgramAddress(
      [Buffer.from(PREFIX), Buffer.from(pool_lpsol)],
      program.programId
    );
    console.log("Pool-LpSOL:", poolLpsol.toBase58());

    // Find PDA for `lpusd pool`
    const [poolLpusd, poolLpusdBump] = await PublicKey.findProgramAddress(
      [Buffer.from(PREFIX), Buffer.from(pool_lpusd)],
      program.programId
    );
    console.log("Pool-LpUSD:", poolLpusd.toBase58());

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
       
    // initialize
    await program.rpc.initialize({
      accounts: {
        authority,
        stateAccount,
        config: configAccount.publicKey,
        usdcMint,
        btcMint,
        msolMint,
        ethMint,
        ustMint,
        srmMint,
        scnsolMint,
        stsolMint,
        usdtMint,
        lpsolMint,
        lpusdMint,
        lpbtcMint,
        lpethMint,
        poolLpsol,
        poolLpusd,
        poolLpbtc,
        poolLpeth,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
      },
      signers: [configAccount]
    });

    await program.rpc.initializePool({
      accounts: {
        authority,
        stateAccount,
        config: configAccount.publicKey,
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
        lpsolMint,
        lpusdMint,
        lpbtcMint,
        lpethMint,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
      }
    });

  } catch (err) {
    console.log("Transaction error: ", err);
  }
}

// 2022-04-13
// ProgramID E3tXtRu4xvVCxUHiM9cEMpjhuSUXkNBd3gxr5RdKzSRw
// Config:  CxdZdbyQUPRdu5rPNvP6nW39JnQMb3Yuo9TswvbNKXBL
// State-Account: HnaQW5GKx9KR5XnFU6oHp89tDoTdnG1aB2RSzTqJ71XR
// Pool-USDC: 2PEkt1Fyji31jdsDk1V53TErXdGUY244aXS8ww5Uyp4r
// Pool-BTC: 4ZYQkhouzSmoXwMkAS44iUuoBdZpL6ZxZatcJVrRBQTv
// Pool-mSOL: J2U9DJy4nSkzJwfdSTfpEqZDjb6ZZn7sF9cjyYHr7eeW
// Pool-ETH: 3uLxAFtSAwgy6MqKVGx1RT9xBAyttaiGUwBJkjF5QgL7
// Pool-UST: DUjLpkCGh6YfwsDmJboLE1QJDHKW2PDM6LGNPDmD3JQP
// Pool-SRM: HLwnuihmcewXypJxZdfxyQxMS3NQnMwkQREqvBrYVJdp
// Pool-SCNSOL: 4N9EhRyvzxwGN8gTSNyBCqZHiP1gAZYYpNcETChptDXf
// Pool-STSOL: 2UHb9PHSXiqGbqUt1hySxeTe6BFSDrjc3Tq9cwTabQqK
// Pool-USDT: 8fKwrDv8fQ1adgLXea6dpqGV1LVtZh1NHh4kHzBggNWU
// Pool-LpSOL: CMjd7T5z3eTuCYcd7aXcJow4bRS29jWhF3DoT57YWYLq
// Pool-LpUSD: 3Y2stxxHkLN7PeutAGmdvSyMHcdr1vyVW1QXw1bSZUBD
// Pool-LpBTC: 4kobd9H4ViQ3eUBUxRXHGiT8zHBb1EWVRXtCy5xftdkw
// Pool-LpETH: 8itfBdbVUWcEB55scjp37ShojbEuNLhgApm1jYwSd85M
