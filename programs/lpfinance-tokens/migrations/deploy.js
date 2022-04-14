// Migrations are an early feature. Currently, they're nothing more than this
// single deploy script that's invoked from the CLI, injecting a provider
// configured from the workspace's Anchor.toml.

const anchor = require("@project-serum/anchor");
const { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID, Token } = require('@solana/spl-token')
const { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Keypair } = anchor.web3;

const idl = require("../target/idl/lpfinance_tokens.json");
const programID = idl.metadata.address;

console.log("ProgramID", programID);

const PREFIX = "lptokens";

const lpsol_mint = "lpsol_mint";
const lpusd_mint = "lpusd_mint";
const lpbtc_mint = "lpbtc_mint";
const lpeth_mint = "lpeth_mint";
const lpdao_mint = "lpdao_mint";

module.exports = async function (provider) {
  // Configure client to use the provider.
  anchor.setProvider(provider);

  // Add your deploy script here
  const program = new anchor.Program(idl, programID);

  try {
    // Find PDA from `cbs protocol` for state account
    const [stateAccount, stateAccountBump] = await PublicKey.findProgramAddress(
      [Buffer.from(PREFIX)],
      program.programId
    );
    // bumps.stateAccount = stateAccountBump;
    console.log("State-Account:", stateAccount.toBase58());

    const [lpsolMint, lpsolMintBump] = await PublicKey.findProgramAddress(
      [Buffer.from(PREFIX), Buffer.from(lpsol_mint)],
      program.programId
    );
    // bumps.poolUsdc = poolUsdcBump;
    console.log("LpSOL mint:", lpsolMint.toBase58());

    const [lpusdMint, lpusdMintBump] = await PublicKey.findProgramAddress(
      [Buffer.from(PREFIX), Buffer.from(lpusd_mint)],
      program.programId
    );
    // bumps.poolBtc = poolBtcBump;
    console.log("LpUSD Mint:", lpusdMint.toBase58());

    const [lpbtcMint, lpbtcMintBump] = await PublicKey.findProgramAddress(
      [Buffer.from(PREFIX), Buffer.from(lpbtc_mint)],
      program.programId
    );
    // bumps.poolMsol = poolMsolBump;
    console.log("LpBTC Mint:", lpbtcMint.toBase58());

    const [lpethMint, lpethMintBump] = await PublicKey.findProgramAddress(
      [Buffer.from(PREFIX), Buffer.from(lpeth_mint)],
      program.programId
    );
    // bumps.poolLpsol = poolLpsolBump;
    console.log("LpETH Mint:", lpethMint.toBase58());

    const [lpdaoMint, lpdaoMintBump] = await PublicKey.findProgramAddress(
      [Buffer.from(PREFIX), Buffer.from(lpdao_mint)],
      program.programId
    );
    // bumps.poolLpusd = poolLpusdBump;
    console.log("LpFI Mint:", lpdaoMint.toBase58());
    
    // Signer
    const authority = provider.wallet.publicKey;

    // const configAccount = anchor.web3.Keypair.generate();
    // console.log("Config: ", configAccount.publicKey.toBase58());
    const config = new PublicKey("2KoT2ifTjzWd773nUa9aZD6fTVzD9kJgzddLbCFbVU71");
    const cbs_account = new PublicKey("HeuPo1nG7uVQhNmBUxmJAyTsCPN9F99uyuC1UdeGMhZe");
    await program.rpc.updateCbsAccount(cbs_account, {
      accounts: {
        owner: authority,
        stateAccount,
        config
      }
    });
    // const userDaotoken = await Token.getAssociatedTokenAddress(
    //   ASSOCIATED_TOKEN_PROGRAM_ID,
    //   TOKEN_PROGRAM_ID,
    //   lpdaoMint,
    //   authority, true
    // )

    // initialize
    // await program.rpc.initialize({
    //   accounts: {
    //     authority,
    //     stateAccount,
    //     config: configAccount.publicKey,
    //     lpsolMint,
    //     lpusdMint,
    //     lpbtcMint,
    //     lpethMint,
    //     lpdaoMint,
    //     userDaotoken,
    //     systemProgram: SystemProgram.programId,
    //     associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    //     tokenProgram: TOKEN_PROGRAM_ID,
    //     rent: SYSVAR_RENT_PUBKEY,
    //   },
    //   signers: [configAccount]
    // });

  } catch (err) {
    console.log("Transaction error: ", err);
  }
}

// 2022-04-06
// ProgramID 3QDe67WmbnSubjdrzsrdYs7ywVYVvjyuoTRECNVgojRr
// State-Account: 9nzCGf6BhFgQogav4Kar1DwFVBZdb3FCRuyJVJK3ZKwL
// LpSOL mint: 9Mcq5PQsEXuSY19ei8CqzRawPdPSAH1VM63GqtZU3x18
// LpUSD Mint: 8YawjpcTDs3SsR7bsCHDb4b1Yv3PAKULB5xZ5VNunroJ
// LpBTC Mint: B8w6e1gSCHE4xNhPhaK5y3cYYBwKMmfJqfe3C9692mGW
// LpETH Mint: 8ZwwTyZ3PSyAzpqPeTXnvxdTF88CxzDQ57hF48WQvK7c
// LpFI Mint: ApThTspa1JouZqmGoY5qbgdkMeo9eqEbs1dziaVA9kKH
// Config:  2KoT2ifTjzWd773nUa9aZD6fTVzD9kJgzddLbCFbVU71