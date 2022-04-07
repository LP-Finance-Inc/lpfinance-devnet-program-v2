// Migrations are an early feature. Currently, they're nothing more than this
// single deploy script that's invoked from the CLI, injecting a provider
// configured from the workspace's Anchor.toml.

const anchor = require("@project-serum/anchor");
const { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID, Token } = require('@solana/spl-token')
const { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Keypair } = anchor.web3;

const idl = require("../target/idl/lending_tokens.json");
const programID = idl.metadata.address;

console.log("ProgramID", programID);

const PREFIX = "lendtokens";

const ust_mint = "ust_mint";
const usdc_mint = "usdc_mint";
const msol_mint = "msol_mint";
const srm_mint = "srm_mint";
const scnsol_mint = "scnsol_mint";
const stsol_mint = "stsol_mint";
const btc_mint = "btc_mint";
const usdt_mint = "usdt_mint";
const eth_mint = "weth_mint";

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

    const [ethMint, ethMintBump] = await PublicKey.findProgramAddress(
      [Buffer.from(PREFIX), Buffer.from(eth_mint)],
      program.programId
    );
    // bumps.poolUsdc = poolUsdcBump;
    console.log("ETH mint:", ethMint.toBase58());

    const [ustMint, ustMintBump] = await PublicKey.findProgramAddress(
      [Buffer.from(PREFIX), Buffer.from(ust_mint)],
      program.programId
    );
    // bumps.poolUsdc = poolUsdcBump;
    console.log("UST mint:", ustMint.toBase58());

    const [usdcMint, usdcMintBump] = await PublicKey.findProgramAddress(
      [Buffer.from(PREFIX), Buffer.from(usdc_mint)],
      program.programId
    );
    // bumps.poolBtc = poolBtcBump;
    console.log("USDC Mint:", usdcMint.toBase58());

    const [msolMint, msolMintBump] = await PublicKey.findProgramAddress(
      [Buffer.from(PREFIX), Buffer.from(msol_mint)],
      program.programId
    );
    // bumps.poolMsol = poolMsolBump;
    console.log("MSOL Mint:", msolMint.toBase58());

    const [srmMint, srmMintBump] = await PublicKey.findProgramAddress(
      [Buffer.from(PREFIX), Buffer.from(srm_mint)],
      program.programId
    );
    // bumps.poolLpsol = poolLpsolBump;
    console.log("SRM Mint:", srmMint.toBase58());

    const [scnsolMint, scnsolMintBump] = await PublicKey.findProgramAddress(
      [Buffer.from(PREFIX), Buffer.from(scnsol_mint)],
      program.programId
    );
    // bumps.poolLpusd = poolLpusdBump;
    console.log("scnSOL Mint:", scnsolMint.toBase58());

    const [stsolMint, stsolMintBump] = await PublicKey.findProgramAddress(
      [Buffer.from(PREFIX), Buffer.from(stsol_mint)],
      program.programId
    );
    // bumps.lpsolMint = lpsolMintBump;
    console.log("stsol-Mint:", stsolMint.toBase58());

    const [btcMint, btcMintBump] = await PublicKey.findProgramAddress(
      [Buffer.from(PREFIX), Buffer.from(btc_mint)],
      program.programId
    );
    // bumps.lpusdMint = lpusdMintBump;
    console.log("Btc-Mint:", btcMint.toBase58());

    const [usdtMint, usdtMintBump] = await PublicKey.findProgramAddress(
      [Buffer.from(PREFIX), Buffer.from(usdt_mint)],
      program.programId
    );
    // bumps.lpusdMint = lpusdMintBump;
    console.log("USDT-Mint:", usdtMint.toBase58());
    // console.log("Bumps", bumps);

    // await add_new();
    const configAccount = anchor.web3.Keypair.generate();
    console.log("Config: ", configAccount.publicKey.toBase58());
    const authority = provider.wallet.publicKey;

    // ADD Token
    await program.rpc.addToken({
      accounts: {
        owner: authority,
        stateAccount,
        config: configAccount.publicKey,
        ethMint,
        ustMint,
        usdcMint,
        msolMint,
        srmMint,
        scnsolMint,
        stsolMint,
        btcMint,
        usdtMint,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
      },
      signers: [configAccount]
    });
    // await init_program()
    // await mint()    
    // await burn()
  } catch (err) {
    console.log("Transaction error: ", err);
  }
}
const add_new = async () => {
  const configAccount = anchor.web3.Keypair.generate();
  console.log("Config: ", configAccount.publicKey);
  const authority = provider.wallet.publicKey;

  // initialize
  await program.rpc.addToken({
    accounts: {
      owner: authority,
      stateAccount,
      config: configAccount.publicKey,
      ethMint,
      ustMint,
      usdcMint,
      msolMint,
      srmMint,
      scnsolMint,
      stsolMint,
      btcMint,
      usdtMint,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY,
    },
    signers: [configAccount]
  });
}
const init_program = async () => {
  const configAccount = anchor.web3.Keypair.generate();
  console.log("Config: ", configAccount.publicKey);

  // initialize
  await program.rpc.initialize({
    accounts: {
      authority,
      stateAccount,
      config: configAccount.publicKey,
      ustMint,
      usdcMint,
      msolMint,
      srmMint,
      scnsolMint,
      stsolMint,
      btcMint,
      usdtMint,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY,
    },
    signers: [configAccount]
  });
}

// TEST mint and burn
const mint = async () => {
  const userToken = await Token.getAssociatedTokenAddress(
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    ustMint,
    authority
  )
  const mintAmount = new anchor.BN("10000000000");
  await program.rpc.mintToken(mintAmount, {
    accounts: {
      owner: authority,
      stateAccount,
      userToken,
      tokenMint: ustMint,
      systemProgram: SystemProgram.programId,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      tokenProgram: TOKEN_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY
    }
  });
}

const burn = async () => {

  const userToken = await Token.getAssociatedTokenAddress(
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    ustMint,
    authority
  )
  const mintAmount = new anchor.BN("10000000000");

  await program.rpc.burnToken(mintAmount, {
    accounts: {
      owner: authority,
      stateAccount,
      userToken,
      tokenMint: ustMint,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY
    }
  });
}
// 2022-04-06
// State-Account: DjhuhArv5yhnE6atZDqpQTwsgMYM7xcqKzSUXei24Fks
// Config: GgRw24thLTL4EGdRitKViPQPfBuPUfE3oyiw1Y8umVE3
// UST mint: CZqXAbuUzGngd97oLjR1bcWkkZrz7MsKAbTJX9oT5Epv
// USDC Mint: 8cCs2Th4ivThrJPrkgAWNTegQgMcuBmY7TASv7FPhitj
// MSOL Mint: 3dDwpZWQqCc5SttGJ2yNnYUnLSBnh9cjWJQPeKNDmDTz
// SRM Mint: GB8u3PRkQoi73v5Tctqj5he4M441S2QfqMpcaAsnozE6
// scnSOL Mint: GXFmXhwBMfXq5utccyNcQRrfQuBVjjprHKSqLzi3P7vn
// stsol Mint: CJGeMYvL7s2k8VHooJ1JvgZsCJqrSEExmPkpFBZskAfV
// Btc Mint: 25ggxgxMqejf5v9WSQWboqxpsrik1u94PCP5EwPBYeEJ
// USDT Mint: DpsmMkLP5yAeBSh7yAMHNuBurLnc8LNxvoddAoKo27dk
// ETH Mint: 6Y9PaAZjDs2n4ZJonCu2uCjRp8tuqe6KJEDs1k6iLkbD