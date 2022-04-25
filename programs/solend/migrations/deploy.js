// Migrations are an early feature. Currently, they're nothing more than this
// single deploy script that's invoked from the CLI, injecting a provider
// configured from the workspace's Anchor.toml.

const anchor = require("@project-serum/anchor");
const { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID, Token } = require('@solana/spl-token')
const { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Keypair } = anchor.web3;

const idl = require("../target/idl/solend.json");
const programID = idl.metadata.address;

console.log("ProgramID", programID);
const PREFIX = "solend0";

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
    const config = new PublicKey('EG3zgt7HjmyA5y6urhw9AiEWNvEfPHLNZgHG2KCNscon');
    const [stateAccount, stateAccountBump] = await PublicKey.findProgramAddress(
      [Buffer.from(PREFIX)],
      program.programId
    );

    console.log("State Account:", stateAccount.toBase58());

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
//     console.log("CBS solend account", userAccount.toBase58());

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

//     // Find PDA for `ust pool`
//     const [poolUst, poolUstBump] = await PublicKey.findProgramAddress(
//       [Buffer.from(PREFIX), Buffer.from(pool_ust)],
//       program.programId
//     );
//     console.log("Pool-UST:", poolUst.toBase58());

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
//         poolUst,
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

// CBS solend account 37xCKPErUb9q625EaofefgS2pTVQkHxugPjQykPmYmQF

// ProgramID CwB4GeieH48q4p864wammdzFYBMNWopYRQPYkS8zbZwL
// Config:  EG3zgt7HjmyA5y6urhw9AiEWNvEfPHLNZgHG2KCNscon
// State-Account: 9rKSqNo2AfTGj6rbiZ3kWXBGKg6unGnMKLSsCYNpQ8iG
// Pool-ETH: 2KpBSwfKpeipbVMApf5ed4tD943LwoDwtaV541mEM9A2
// Pool-USDC: PN9fZopu5q7XKrK7N3kmxznDF4gw4qgpgDj9YA52uMD
// Pool-BTC: 7pACpuf3PxBurF4BF5JukQqJkZK1CUFf3SmvsMKBYZDY
// Pool-MSOL: HjKVfq9xeqAqv4GTa6wCDZHsNuCenJBaLetLhF4Nk59Q
// Pool-UST: FYccwVZ4kYqePG8VyGFrBYqAsJNdvSDf5Z6FZNUx8DfR
// Pool-SRM: 4LALBawUGE54jgKD42Pf92PLa9nsagRPLXgHwcsjKi4n
// Pool-SCNSOL: 8RRsFhRVZjrCn1xhtHnNPzpGuxtjhw5bZLvkmG7ihYs
// Pool-STSOL: GDdhUP6XWgsVp3FaLhZTVNSGRV72ZbD1k3m4Vq5eCcYG
// Pool-USDT: 8n2uPU7UTQJjGdrtVYkTUCGmXeECY7B5TeDySZLN1xA4