// Migrations are an early feature. Currently, they're nothing more than this
// single deploy script that's invoked from the CLI, injecting a provider
// configured from the workspace's Anchor.toml.

const anchor = require("@project-serum/anchor");
const { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID, Token } = require('@solana/spl-token')
const { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Keypair } = anchor.web3;
const { cbsAddrs } = require('./wallets');

const idl = require("../target/idl/cbs_protocol.json");
const programID = idl.metadata.address;

console.log("ProgramID", programID);
const PREFIX = "cbsprotocol3";

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

    for (const idx in cbsAddrs) {
      try {
        console.log(cbsAddrs[idx])
        const authority = new PublicKey(cbsAddrs[idx]);
        const [userAccount, userAccountBump] = await PublicKey.findProgramAddress(
          [Buffer.from(PREFIX), Buffer.from(authority.toBuffer())],
          program.programId
        );
    
        await program.rpc.fixUserAccount( new anchor.BN("0"), {
          accounts: {
            userAccount
          }
        });
      } catch (err) {
        console.log(err)
      }
    }
}
/*
module.exports = async function (provider) {
  // Configure client to use the provider.
  anchor.setProvider(provider);

  // Add your deploy script here
  const program = new anchor.Program(idl, programID);

  try {
    // const config  = new PublicKey("6bUzHQxih8vuMtZL7fm2xsfSt55zDuL4m9RwrqXk9YDp");
    const configAccount = anchor.web3.Keypair.generate();
    console.log("Config: ", configAccount.publicKey.toBase58());

    // Find PDA from `cbs protocol` for state account
    const [stateAccount, stateAccountBump] = await PublicKey.findProgramAddress(
      [Buffer.from(PREFIX)],
      program.programId
    );
    console.log("State-Account:", stateAccount.toBase58());

    // Find PDA for `eth pool`
    const [poolEth, poolEthBump] = await PublicKey.findProgramAddress(
      [Buffer.from(PREFIX), Buffer.from(pool_eth)],
      program.programId
    );
    console.log("Pool-ETH:", poolEth.toBase58());

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
       
    // UpdateConfig
    // await program.rpc.updateConfig({
    //   accounts: {
    //     owner: authority,
    //     config,
    //     stateAccount,
    //     usdcMint,
    //     btcMint,
    //     msolMint,
    //     ethMint,
    //     poolUsdc,
    //     poolEth,
    //     poolBtc,
    //     poolMsol,
    //     lpsolMint,
    //     lpusdMint,
    //     lpbtcMint,
    //     lpethMint,
    //     poolLpsol,
    //     poolLpusd,
    //     poolLpbtc,
    //     poolLpeth,
    //     systemProgram: SystemProgram.programId,
    //     tokenProgram: TOKEN_PROGRAM_ID,
    //     rent: SYSVAR_RENT_PUBKEY,
    //   }
    // });

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
} */

// 2022-04-22
// ProgramID 3f39cgs9wPLVv4vGySNecjKtefe5MJYkFEEj3v6bPequ
// Config:  4mND9qtpmZN5fEk48TMy2tUEaSh5QFGL64ruFXMwuwRA
// State-Account: HKakh92meu61n3kchSPpNDveCwHno9ymeamN9yZbXt1z
// Pool-ETH: 5N45aAx4aAj5CmD5wiievuG9bnY3wwzLNzXUdgsXhpy
// Pool-USDC: HoKqptd4zJzE5w5RvAXt39hHs4WDFvXnSry8xyedrrYc
// Pool-BTC: J1bgGA5Khj3t9bPepVpS6rPD5o1Pa4JdxypWJQrXt6Zu
// Pool-MSOL: GpCTx7w81RAqdJgC9HmJxwshUmagb1tpqG76beVpPwYA
// Pool-UST: 8jde3rbntGh34KSk2V48q7w6WeFCCjEFLsxtV6AB2pWp
// Pool-SRM: G7W5Dzby8sV8FobviTvogkzuksneFBCncy1kwjuoQea5
// Pool-SCNSOL: 5RVTEUCfePKfwnvQdgW3nhKyndyPXAM2TJfUKXuT1dzP
// Pool-STSOL: GYWxoNG6adwwe6Za6V7QQCxKFsDvJXxDNdhPrLuGu1CC
// Pool-USDT: 51FovGmzpBmxy31yHAnhu4Ftb5VgoYV4bYxNhQdoPVAZ
// Pool-LpSOL: HbYcBPKmKcNVFgvQt8in3KFTG6GQtYHXDM7Dwq2q7JKo
// Pool-LpUSD: Bc7AzMJcUbE1c3CLHpXtyY7g9x23BBryJgqXRPvwHyCB
// Pool-LpBTC: E6Bu3gzRbTuPmWRiCArJqtAvUzYFwJYf9nYZSjSarTJT
// Pool-LpETH: GvbQ59hQsovMGaz2W4rq6xCxtF43UJdugq9mJocYWTpU

// 2022-04-13
// ProgramID 3f39cgs9wPLVv4vGySNecjKtefe5MJYkFEEj3v6bPequ
// Config:  4vtJgTh9V2dMnQXB9Fv95rR9AXx9oxDbQ46rjdh2vrpx
// State-Account: HeuPo1nG7uVQhNmBUxmJAyTsCPN9F99uyuC1UdeGMhZe
// Pool-ETH: Awfy6rkdvjeDiNGmnjigGp6NPKjYS57FCh4cTZ3U1qKZ
// Pool-USDC: 4qNyPmdrdwbgKVWxg7BdDRzECaD14zzy9NPPcwwSkmBp
// Pool-BTC: SAiFRd6vgmjYLuDgAqL4cF6qsqrYQoF6RL4sZVKbX5V
// Pool-MSOL: 7EQp5hkTGX6fhZMc9eo8rMXBUbq1UF6LTy1BhDWGAZpm
// Pool-UST: Htmo9oubJvgiDm4ELBmcd8wq6Y1Rqw6PsK5wdptyUGs9
// Pool-SRM: EQKZFuKL3pNgoqRjJZXNLyJRmVKbprcbDyZ6DfrEyyHv
// Pool-SCNSOL: 4vTCcTA2DhtsQzdswxbYFy1tMFfEDun4cjsd3jXNyx7R
// Pool-STSOL: GBaM5muFk513yrY1ha4seqGuo3Vjy2XTPotoBjTyNWQL
// Pool-USDT: 2feJnASCnKyC7hzRoBehf4RnkZqw4HSokg6yS9FnQ1am
// Pool-LpSOL: FYgRnHZQTVGeNNDwBrGYXZZhs4pQC47EFPgfAaZbe43a
// Pool-LpUSD: GA94vWMGzujMsSqLpKtWfd24F8AstPqcpgWZuETQEwww
// Pool-LpBTC: 5nA17sck1GBy1ztS5UbqEQmPxEwdU2DSfDD9BdYo9A7u
// Pool-LpETH: 2CHQiC3xsjWKy5nSinxdXvY6SuxxSqapfriD1MKj5xj5
