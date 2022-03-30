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
const auction_name = "auction_03";
const pool_usdc = "pool_usdc";
const pool_btc = "pool_btc";
const pool_lpsol = "pool_lpsol";
const pool_lpusd = "pool_lpusd";
const pool_msol = "pool_msol";

// Test Token's MINT
const usdcMint = new PublicKey("2Q1WAAgnpEox5Y4b6Y8YyXVwFNhDdGot467XfvdBJaPf"); 
const btcMint = new PublicKey("Hv96pk4HkhGcbNxkBvb7evTU88KzedvgVy2oddBB1ySB");
const lpsolMint = new PublicKey("BseXpATR4hqy7UHvyNztLK711mYPHNCsS5AcBzWzSq7X"); 
const lpusdMint = new PublicKey("GPNCGAjyhA1qcSgSotQvJsM1xcGnDMgtTr9TJ1HVVQgG");
const msolMint = new PublicKey("EJ94TwhddyUAra7i3qttQ64Q1wExJYb8GmACbHbAnvKF");

module.exports = async function (provider) {
  // Configure client to use the provider.
  anchor.setProvider(provider);

  // Add your deploy script here
  const program = new anchor.Program(idl, programID);

  try {
    /* interact with the program via rpc */
    let bumps = {
      stateAccount: 0,
      poolUsdc: 0,
      poolBtc: 0,
      poolLpsol: 0,
      poolLpusd: 0,
      poolMsol: 0
    };

    // Find PDA from `cbs protocol` for state account
    const [stateAccount, stateAccountBump] = await PublicKey.findProgramAddress(
      [Buffer.from(auction_name)],
      program.programId
    );
    bumps.stateAccount = stateAccountBump;
    console.log("State-Account:", stateAccount.toBase58());

    // Find PDA for `usdc pool`
    const [poolUsdc, poolUsdcBump] = await PublicKey.findProgramAddress(
      [Buffer.from(auction_name), Buffer.from(pool_usdc)],
      program.programId
    );
    bumps.poolUsdc = poolUsdcBump;
    console.log("Pool-USDC:", poolUsdc.toBase58());

    // Find PDA for `btc pool`
    const [poolBtc, poolBtcBump] = await PublicKey.findProgramAddress(
      [Buffer.from(auction_name), Buffer.from(pool_btc)],
      program.programId
    );
    bumps.poolBtc = poolBtcBump;
    console.log("Pool-BTC:", poolBtc.toBase58());

    // Find PDA for `lpsol pool`
    const [poolMsol, poolMsolBump] = await PublicKey.findProgramAddress(
      [Buffer.from(auction_name), Buffer.from(pool_msol)],
      program.programId
    );
    bumps.poolMsol = poolMsolBump;
    console.log("Pool-mSOL:", poolMsol.toBase58());

    // Find PDA for `lpsol pool`
    const [poolLpsol, poolLpsolBump] = await PublicKey.findProgramAddress(
      [Buffer.from(auction_name), Buffer.from(pool_lpsol)],
      program.programId
    );
    bumps.poolLpsol = poolLpsolBump;
    console.log("Pool-LpSOL:", poolLpsol.toBase58());

    // Find PDA for `lpusd pool`
    const [poolLpusd, poolLpusdBump] = await PublicKey.findProgramAddress(
      [Buffer.from(auction_name), Buffer.from(pool_lpusd)],
      program.programId
    );
    bumps.poolLpusd = poolLpusdBump;
    console.log("Pool-LpUSD:", poolLpusd.toBase58());

    console.log("Bumps", bumps);

    // Signer
    const authority = provider.wallet.publicKey;
       
    // initialize
    await program.rpc.initialize(auction_name, bumps, {
      accounts: {
        authority,
        stateAccount,
        usdcMint,
        btcMint,
        msolMint,
        lpsolMint,
        lpusdMint,
        poolUsdc,
        poolBtc,
        poolLpsol,
        poolMsol,
        poolLpusd,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
      },
    });

  } catch (err) {
    console.log("Transaction error: ", err);
  }
}

// 2022-03-17
// ProgramID 6KS4ho2CDvr7MGofHU6F6WJfQ5j6DL8nhBWJtkhMTzqt
// State-Account: kr6SYFJZvfVmFWg4gJPW7Qm4Q2jBANT5NLTieFq6EHK
// Pool-USDC: 4MroHBr3m4zEsRKNGCCockwPSkqJacKoVLMknELWgSyi
// Pool-BTC: 5eQBRPJtYAc29wQJtpWAhFCUsFwDZs2QdbahdZRC4CJT
// Pool-mSOL: AEQagn3g3xqoatAuSxfMtnDU7qehsv1etyUTo6HFxFkx
// Pool-LpSOL: 8XKjY2QVfnZrZAdBrycVSTgDQP9KujoQDowvdRpKFvzc
// Pool-LpUSD: Ddk2z7KBieLv5vyadEGBwm7M2bqeSGRGpVCuSZmxCWtT
// Bumps {
//   stateAccount: 253,
//   poolUsdc: 255,
//   poolBtc: 255,
//   poolLpsol: 255,
//   poolLpusd: 254,
//   poolMsol: 254
// }

// 2022-03-11 env
// ProgramID 6KS4ho2CDvr7MGofHU6F6WJfQ5j6DL8nhBWJtkhMTzqt
// State-Account: 4rU15gLvdV2SwfooxmTX13iRyE2ruCGyVn5p5QeZd5ks
// Pool-USDC: 6sghnjhcQgw64uehscJtDPhsTdrQm7ty7ed2h6WhhyrC
// Pool-BTC: DHzVDgbogJvs3L8UPFHxcYhL8LdmWBj8PcbUeph7J2CZ
// Pool-mSOL: 9GwzM5FnVa7DHyqFYSeZzM39jN8TA21d5VskarV17cHU
// Pool-LpSOL: 9qhUKBqqdvyeGmj7rPatdgyphgWFM7WY6n3r1R12Eu4b
// Pool-LpUSD: 2XCUWyG6Bw5jMtYScSkpTFbhwRD8Y4MSiDLiekuUrW3o
// Bumps {
//   stateAccount: 255,
//   poolUsdc: 255,
//   poolBtc: 255,
//   poolLpsol: 255,
//   poolLpusd: 255,
//   poolMsol: 255
// }

// 2022-03-11 env
// ProgramID 6KS4ho2CDvr7MGofHU6F6WJfQ5j6DL8nhBWJtkhMTzqt
// State-Account: DXkKu4JNp3QoPPrPWhQk9fXCDVwYjn3bJEfnHRJoJrC7
// Pool-USDC: 73xGUk2R7cooqVqHdNsvEhmEdUxvmRAvtnoVrfV78Cvj
// Pool-BTC: 6UPvPTsnYSNTHp1nuTESQPrQfhnpoe5gFHXxbj9daiMU
// Pool-LpSOL: FpzXqnXjuUrYRhmGB27BMLxpRWHsE6e26J62EaDUvfmr
// Pool-LpUSD: DCaRFdbkfHHCvMK2mv7o3ZFcTH1m9Nb9q89oKacghzqG
// Bumps {
//   stateAccount: 252,
//   poolUsdc: 254,
//   poolBtc: 254,
//   poolLpsol: 250,
//   poolLpusd: 255
// }