// Migrations are an early feature. Currently, they're nothing more than this
// single deploy script that's invoked from the CLI, injecting a provider
// configured from the workspace's Anchor.toml.

const anchor = require("@project-serum/anchor");
const { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID, Token } = require('@solana/spl-token')
const { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Keypair } = anchor.web3;

const idl = require("../target/idl/faucet.json");
const programID = idl.metadata.address;

console.log("ProgramID", programID);
const faucet_name = "faucet_001";
const pool_tusdc = "pool_tusdc";
const pool_tbtc = "pool_tbtc";
const pool_tmsol = "pool_tmsol";

// Test Token's MINT
const tusdcMint = new PublicKey("2Q1WAAgnpEox5Y4b6Y8YyXVwFNhDdGot467XfvdBJaPf"); 
const tbtcMint = new PublicKey("Hv96pk4HkhGcbNxkBvb7evTU88KzedvgVy2oddBB1ySB");
const tmsolMint = new PublicKey("EJ94TwhddyUAra7i3qttQ64Q1wExJYb8GmACbHbAnvKF");

module.exports = async function (provider) {
  // Configure client to use the provider.
  anchor.setProvider(provider);

  // Add your deploy script here
  const program = new anchor.Program(idl, programID);

  try {
    /* interact with the program via rpc */
    let bumps = {
      stateAccount: 0,
      poolTusdc: 0,
      poolTbtc: 0,
      poolTmsol: 0
    };

    // Find PDA from `cbs protocol` for state account
    const [stateAccount, stateAccountBump] = await PublicKey.findProgramAddress(
      [Buffer.from(faucet_name)],
      program.programId
    );
    bumps.stateAccount = stateAccountBump;
    console.log("State-Account:", stateAccount.toBase58());

    // Find PDA for `usdc pool`
    const [poolTusdc, poolTusdcBump] = await PublicKey.findProgramAddress(
      [Buffer.from(faucet_name), Buffer.from(pool_tusdc)],
      program.programId
    );
    bumps.poolTusdc = poolTusdcBump;
    console.log("Pool-USDC:", poolTusdc.toBase58());

    // Find PDA for `btc pool`
    const [poolTbtc, poolTbtcBump] = await PublicKey.findProgramAddress(
      [Buffer.from(faucet_name), Buffer.from(pool_tbtc)],
      program.programId
    );
    bumps.poolTbtc = poolTbtcBump;
    console.log("Pool-BTC:", poolTbtc.toBase58());

    // Find PDA for `tmsol pool`
    const [poolTmsol, poolTmsolBump] = await PublicKey.findProgramAddress(
      [Buffer.from(faucet_name), Buffer.from(pool_tmsol)],
      program.programId
    );
    bumps.poolTmsol = poolTmsolBump;
    console.log("Pool-Tmsol:", poolTmsol.toBase58());

    console.log("Bumps", bumps);

    // Signer
    const authority = provider.wallet.publicKey;
       
    // initialize
    await program.rpc.initialize(faucet_name, bumps, {
      accounts: {
        authority,
        stateAccount,
        tusdcMint,
        tbtcMint,
        tmsolMint,
        poolTusdc,
        poolTbtc,
        poolTmsol,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
      },
    });

  } catch (err) {
    console.log("Transaction error: ", err);
  }
}
// 2022-03-14 2
// ProgramID 7G5gNHT2T2fdb9EHY5K6FEPc6Z1mAF8M5uhURWDbsBjE
// State-Account: G9vKpJfDcfu5SqmLVsqQSTDEsLw5QfyYAAVKkZAYHmyE
// Pool-USDC: 8EDkavvKkrnSH5fsrL69nb9St3dehNaVmwskucFzKWcJ
// Pool-BTC: A8oA9Eh4NzWRPLRULRBPpDdxpV8H8o7Nhi4rQ6FansKq
// Pool-Tmsol: 2cEhXc4ShLZpdo2fhG7Rhk3MeHGd9gYx9vhhV2HC1JU5
// Bumps { stateAccount: 255, poolTusdc: 255, poolTbtc: 255, poolTmsol: 255 }

// 2022-03-14
// ProgramID 7G5gNHT2T2fdb9EHY5K6FEPc6Z1mAF8M5uhURWDbsBjE
// State-Account: ob9WHKk3a3cnrTo2SojP6wH5piBXRDKjEJaDFaRn7Dd
// Pool-tUSDC: 67tFAd5JrHT8vrMZofcrwaReabsNdRW2RyDmGnpXGv2j
// Pool-tBTC: DcGNuUFPEqhGfGFVrttVKr5iMfy9oZpx99fHkTroHR5R
// Bumps { stateAccount: 254, poolTusdc: 254, poolTbtc: 254 }