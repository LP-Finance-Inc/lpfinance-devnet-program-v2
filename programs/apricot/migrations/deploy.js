// Migrations are an early feature. Currently, they're nothing more than this
// single deploy script that's invoked from the CLI, injecting a provider
// configured from the workspace's Anchor.toml.

const anchor = require("@project-serum/anchor");
const { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID, Token } = require('@solana/spl-token')
const { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Keypair } = anchor.web3;

const idl = require("../target/idl/apricot.json");
const programID = idl.metadata.address;

console.log("ProgramID", programID);
const PREFIX = "apricot0";

const pool_usdc = "pool_usdc";
const pool_btc = "pool_btc";
const pool_eth = "pool_eth";
const pool_msol = "pool_msol";
const pool_ust = "pool_ust";
const pool_srm = "pool_srm";
const pool_scnsol = "pool_scnsol";
const pool_stsol = "pool_stsol";
const pool_usdt = "pool_usdt";

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


module.exports = async function (provider) {
  // Configure client to use the provider.
  anchor.setProvider(provider);

  // Add your deploy script here
  const program = new anchor.Program(idl, programID);

  try {

    // Signer
    const authority = provider.wallet.publicKey;       
    const config = new PublicKey('7q2j82XkdWDpQVj7cogxX64fGvtpUqDFKQUUVdgiKvdP');
    const [stateAccount, stateAccountBump] = await PublicKey.findProgramAddress(
      [Buffer.from(PREFIX)],
      program.programId
    );

    // updateRate
    await program.rpc.updateRate({
      accounts: {
        owner: authority,
        stateAccount,
        config
      }
    });

  } catch (err) {
    console.log("Transaction error: ", err);
  }
}

// module.exports = async function (provider) {
//   // Configure client to use the provider.
//   anchor.setProvider(provider);

//   // Add your deploy script here
//   const program = new anchor.Program(idl, programID);

//   try {

//     // Signer
//     const authority = provider.wallet.publicKey;       
//     const user = new PublicKey('HKakh92meu61n3kchSPpNDveCwHno9ymeamN9yZbXt1z');
//     const [userAccount, userAccountBump] = await PublicKey.findProgramAddress(
//       [Buffer.from(PREFIX), Buffer.from(user.toBuffer())],
//       program.programId
//     );
//     console.log("CBS apricot account", userAccount.toBase58());

//     // initUserAccount
//     await program.rpc.initUserAccount({
//       accounts: {
//         userAccount,
//         user,
//         userAuthority: authority,
//         systemProgram: SystemProgram.programId,
//         rent: SYSVAR_RENT_PUBKEY,
//       }
//     });

//   } catch (err) {
//     console.log("Transaction error: ", err);
//   }
// }

// module.exports = async function (provider) {
//   // Configure client to use the provider.
//   anchor.setProvider(provider);

//   // Add your deploy script here
//   const program = new anchor.Program(idl, programID);

//   try {
//     // const config  = new PublicKey("6bUzHQxih8vuMtZL7fm2xsfSt55zDuL4m9RwrqXk9YDp");
//     const configAccount = anchor.web3.Keypair.generate();
//     console.log("Config: ", configAccount.publicKey.toBase58());

//     // Find PDA from `cbs protocol` for state account
//     const [stateAccount, stateAccountBump] = await PublicKey.findProgramAddress(
//       [Buffer.from(PREFIX)],
//       program.programId
//     );
//     console.log("State-Account:", stateAccount.toBase58());

//     // Find PDA for `eth pool`
//     const [poolEth, poolEthBump] = await PublicKey.findProgramAddress(
//       [Buffer.from(PREFIX), Buffer.from(pool_eth)],
//       program.programId
//     );
//     console.log("Pool-ETH:", poolEth.toBase58());

//     // Find PDA for `usdc pool`
//     const [poolUsdc, poolUsdcBump] = await PublicKey.findProgramAddress(
//       [Buffer.from(PREFIX), Buffer.from(pool_usdc)],
//       program.programId
//     );
//     console.log("Pool-USDC:", poolUsdc.toBase58());

//     // Find PDA for `btc pool`
//     const [poolBtc, poolBtcBump] = await PublicKey.findProgramAddress(
//       [Buffer.from(PREFIX), Buffer.from(pool_btc)],
//       program.programId
//     );
//     console.log("Pool-BTC:", poolBtc.toBase58());

//     // Find PDA for `msol pool`
//     const [poolMsol, poolMsolBump] = await PublicKey.findProgramAddress(
//       [Buffer.from(PREFIX), Buffer.from(pool_msol)],
//       program.programId
//     );
//     console.log("Pool-MSOL:", poolMsol.toBase58());

//     // Find PDA for `srm pool`
//     const [poolSrm, poolSrmBump] = await PublicKey.findProgramAddress(
//       [Buffer.from(PREFIX), Buffer.from(pool_srm)],
//       program.programId
//     );
//     console.log("Pool-SRM:", poolSrm.toBase58());

//     // Find PDA for `scnsol pool`
//     const [poolScnsol, poolScnsolBump] = await PublicKey.findProgramAddress(
//       [Buffer.from(PREFIX), Buffer.from(pool_scnsol)],
//       program.programId
//     );
//     console.log("Pool-SCNSOL:", poolScnsol.toBase58());

//     // Find PDA for `stsol pool`
//     const [poolStsol, poolStsolBump] = await PublicKey.findProgramAddress(
//       [Buffer.from(PREFIX), Buffer.from(pool_stsol)],
//       program.programId
//     );
//     console.log("Pool-STSOL:", poolStsol.toBase58());
    
//     // Find PDA for `usdt pool`
//     const [poolUsdt, poolUsdtBump] = await PublicKey.findProgramAddress(
//       [Buffer.from(PREFIX), Buffer.from(pool_usdt)],
//       program.programId
//     );
//     console.log("Pool-USDT:", poolUsdt.toBase58());

//     // Signer
//     const authority = provider.wallet.publicKey;       

//     // initialize
//     await program.rpc.initialize({
//       accounts: {
//         authority,
//         stateAccount,
//         config: configAccount.publicKey,
//         usdcMint,
//         btcMint,
//         msolMint,
//         ethMint,
//         ustMint,
//         srmMint,
//         scnsolMint,
//         stsolMint,
//         usdtMint,
//         poolUsdc,
//         poolEth,
//         poolBtc,
//         poolMsol,
//         poolSrm,
//         poolScnsol,
//         poolStsol,
//         poolUsdt,
//         systemProgram: SystemProgram.programId,
//         tokenProgram: TOKEN_PROGRAM_ID,
//         rent: SYSVAR_RENT_PUBKEY,
//       },
//       signers: [configAccount]
//     });

//   } catch (err) {
//     console.log("Transaction error: ", err);
//   }
// }

// 2022-04-22
// CBS apricot account : A7Y1R2jPsS3rGHZiXEcCXRByLBZtvv1b46BZEjeYYhiu

// ProgramID DDr65T1xJYmBi8M8sqEcitEbLSEboXMG1t3JNvYZk8Nx
// Config:  7q2j82XkdWDpQVj7cogxX64fGvtpUqDFKQUUVdgiKvdP
// State-Account: DaYiw3X7Vt3upbKXWciETvbo72TZF9y9sVan6TySnzbn
// Pool-ETH: DNAUfAwU9u4knVgSkrQiedDWCG5cycMNfDzEyTQVkXXG
// Pool-USDC: HWK6BKu1G7pUDJSwXXb5DfLtGdc3JTmzfKXN91dW7jiM
// Pool-BTC: DJ78qgpXMADf56pYgxqPEYV82QMRgah5ma1MEEQ1eYCe
// Pool-MSOL: AzkMZoUv3Ni8jtNSkbdkEXZh6D5TdKfMoQbqEBbqfGtZ
// Pool-SRM: Cp51w8YVX8nP8d4zBf7dSEtd4pWVVUgiw7c7abubyoux
// Pool-SCNSOL: DH6uU98QuuA56NJutgtJxApCc9ffsbGWuV9AJAg9X8vS
// Pool-STSOL: 9BRojuywXGhms3QGGNrCiWZynuSKGGzhwv9abdwfpEck
// Pool-USDT: 9LMiXJAKBTtxQWCARbRdNSE28iAGkMqgq8sxWHqVP8Rn