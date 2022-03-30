// Migrations are an early feature. Currently, they're nothing more than this
// single deploy script that's invoked from the CLI, injecting a provider
// configured from the workspace's Anchor.toml.

const anchor = require("@project-serum/anchor");
const { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID, Token } = require('@solana/spl-token')
const { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Keypair } = anchor.web3;

const idl = require("../target/idl/cbs_protocol.json");
const programID = idl.metadata.address;

console.log("ProgramID", programID);
const protocol_name = "cbs_pool05";
const pool_usdc = "pool_usdc";
const pool_btc = "pool_btc";
const pool_lpsol = "pool_lpsol";
const pool_lpusd = "pool_lpusd";
const pool_msol = "pool_msol";
const lpsol_mint = "lpsol_mint";
const lpusd_mint = "lpusd_mint";

// Test Token's MINT
const usdcMint = new PublicKey("2Q1WAAgnpEox5Y4b6Y8YyXVwFNhDdGot467XfvdBJaPf"); 
const btcMint = new PublicKey("Hv96pk4HkhGcbNxkBvb7evTU88KzedvgVy2oddBB1ySB");
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
      lpusdMint: 0,
      lpsolMint: 0,
      poolUsdc: 0,
      poolBtc: 0,
      poolMsol: 0,
      poolLpsol: 0,
      poolLpusd: 0
    };

    // Find PDA from `cbs protocol` for state account
    const [stateAccount, stateAccountBump] = await PublicKey.findProgramAddress(
      [Buffer.from(protocol_name)],
      program.programId
    );
    bumps.stateAccount = stateAccountBump;
    console.log("State-Account:", stateAccount.toBase58());

    // Find PDA for `usdc pool`
    const [poolUsdc, poolUsdcBump] = await PublicKey.findProgramAddress(
      [Buffer.from(protocol_name), Buffer.from(pool_usdc)],
      program.programId
    );
    bumps.poolUsdc = poolUsdcBump;
    console.log("Pool-USDC:", poolUsdc.toBase58());

    // Find PDA for `btc pool`
    const [poolBtc, poolBtcBump] = await PublicKey.findProgramAddress(
      [Buffer.from(protocol_name), Buffer.from(pool_btc)],
      program.programId
    );
    bumps.poolBtc = poolBtcBump;
    console.log("Pool-BTC:", poolBtc.toBase58());

    // Find PDA for `btc pool`
    const [poolMsol, poolMsolBump] = await PublicKey.findProgramAddress(
      [Buffer.from(protocol_name), Buffer.from(pool_msol)],
      program.programId
    );
    bumps.poolMsol = poolMsolBump;
    console.log("Pool-MSOL:", poolMsol.toBase58());

    // Find PDA for `lpsol pool`
    const [poolLpsol, poolLpsolBump] = await PublicKey.findProgramAddress(
      [Buffer.from(protocol_name), Buffer.from(pool_lpsol)],
      program.programId
    );
    bumps.poolLpsol = poolLpsolBump;
    console.log("Pool-LpSOL:", poolLpsol.toBase58());

    // Find PDA for `lpusd pool`
    const [poolLpusd, poolLpusdBump] = await PublicKey.findProgramAddress(
      [Buffer.from(protocol_name), Buffer.from(pool_lpusd)],
      program.programId
    );
    bumps.poolLpusd = poolLpusdBump;
    console.log("Pool-LpUSD:", poolLpusd.toBase58());

    // Find PDA for `lpsol mint`
    const [lpsolMint, lpsolMintBump] = await PublicKey.findProgramAddress(
      [Buffer.from(protocol_name), Buffer.from(lpsol_mint)],
      program.programId
    );
    bumps.lpsolMint = lpsolMintBump;
    console.log("LpSOL-Mint:", lpsolMint.toBase58());

    // Find PDA for `lpsol mint`
    const [lpusdMint, lpusdMintBump] = await PublicKey.findProgramAddress(
      [Buffer.from(protocol_name), Buffer.from(lpusd_mint)],
      program.programId
    );
    bumps.lpusdMint = lpusdMintBump;
    console.log("LpUSD-Mint:", lpusdMint.toBase58());

    console.log("Bumps", bumps);

    // Signer
    const authority = provider.wallet.publicKey;
       
    // initialize
    await program.rpc.initialize(protocol_name, bumps, {
      accounts: {
        authority,
        stateAccount,
        usdcMint,
        btcMint,
        msolMint,
        poolUsdc,
        poolBtc,
        poolMsol,
        lpsolMint,
        lpusdMint,
        poolLpsol,
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
// 2022-03-14 devnet
// ProgramID 3YhaNLN3oYUaAXjK9yRqVVNUYhqPsVqB5q9GEJ1vWcTM
// State-Account: 2bpEcaTSRtenzbtVuQmygXWn69ccj2voJ59PjbPuthtJ
// Pool-USDC: 6KJ8uDFnEjPo3VvLoNHhpNq17E3JB9iVzUPFNwUMRzGq
// Pool-BTC: t8ehVs5jAqYVwfLs2F4goQ7jqprAkcZpJDax8LQAcS6
// Pool-MSOL: 7cgwUfB5cHFGPDH2ojkYWP4eZcoBzsvzG2tmRtXz1dU3
// Pool-LpSOL: GoT7kwnXsmxYCMAz8Cp9zCqx9XkEaYwksxKTDv1WoGHZ
// Pool-LpUSD: DjzPeokasEPme9V19861Y5oNgjaxFHHFDz9k6RjvAHBG
// LpSOL-Mint: BseXpATR4hqy7UHvyNztLK711mYPHNCsS5AcBzWzSq7X
// LpUSD-Mint: GPNCGAjyhA1qcSgSotQvJsM1xcGnDMgtTr9TJ1HVVQgG
// Bumps {
//   stateAccount: 252,
//   lpusdMint: 255,
//   lpsolMint: 252,
//   poolUsdc: 255,
//   poolBtc: 255,
//   poolMsol: 253,
//   poolLpsol: 254,
//   poolLpusd: 254
// }

// 2022-03-09 devnet
// ProgramID 3YhaNLN3oYUaAXjK9yRqVVNUYhqPsVqB5q9GEJ1vWcTM
// State-Account: 6ZitbU1D6EZBYqKHq74zCkzoZBvGDGdgzKH1oUh5RM8j
// Pool-USDC: GpzZjubiozLk9K6obfm1dcnKQSqqZGn69dooX2Q1C3Uk
// Pool-BTC: 2tDLPfwz7KCtzSBvUZcsuwmDSZBvy7nT7JEYfQTWApzn
// Pool-LpSOL: 8sB3gqYZ13J4hTmhQ6P5XRcHvoviwymU8Ru9dE2MUrcB
// Pool-LpUSD: GXHTqCiQuFc6k5UT8Rc4aS1gVTr45sTP1NjhQMMtSK82
// LpSOL-Mint: FCSUDXzfqc393wVcv4tWBU4LgRhJeDi8YA6BGTs3qVPP
// LpUSD-Mint: AL9fyDTSmJavYxjftxBHxkLtwv9FcsUJfVvEheW6vfdq
// Bumps {
//   stateAccount: 254,
//   lpusdMint: 255,
//   lpsolMint: 254,
//   poolUsdc: 252,
//   poolBtc: 254,
//   poolLpsol: 255,
//   poolLpusd: 253
// }

// 2022-03-1 devnet
// const protocol_name = "cbs_pool02";
// ProgramID 3YhaNLN3oYUaAXjK9yRqVVNUYhqPsVqB5q9GEJ1vWcTM
// State-Account: ES4ob9B6ngcM5FXDShXA6SHUvFSv1DQiZY5bG9NakXaR
// Pool-USDC: 3FT6VJn3kPAdaBjnNq9oxk47cJVtfyrt8EFFWwArxtMX
// Pool-BTC: HtrG19tdTKXhJHYFLD6PmjZN22Xi9o9EyVq49P6BBjCj
// Pool-LpSOL: 8fYcxYJxCWrQtoZuCrwnBU6sPbbnTW5eWpM7S1spsvZR
// Pool-LpUSD: 5Pb9Cq3Ho5w4JDLvZFjXQyahcpbSVMEpPHT2bAN4HiPy
// LpSOL-Mint: CCFfxDcVY6iCd4EiocQNymZRhZapuGrxVP4TK1PJrVqh
// LpUSD-Mint: C6DHbFE8eFmiiZPcY1mTPaG928q6cXuE9vD2NHuDL5TH
// Bumps {
//   stateAccount: 254,
//   lpusdMint: 253,
//   lpsolMint: 255,
//   poolUsdc: 255,
//   poolBtc: 255,
//   poolLpsol: 255,
//   poolLpusd: 253
// }

// 2022-02-26 devnet
// ProgramID 3YhaNLN3oYUaAXjK9yRqVVNUYhqPsVqB5q9GEJ1vWcTM
// State-Account: EvFeLhQYAjUgg992feVFdAogHKnb8wdKZBKmYA1XyBY7
// Pool-USDC: AkCsz9jBudmPKN47rFS16RZQo3rJ7xkvVAkYJpDwYM9V
// Pool-BTC: HrkssFAVtEdky7SZtj5U8nbF1dvGagvs5Wwi7aUKgF4K
// LpSOL-Mint: BPxhUPCcuJ51ugnTFtK6H8xcZu5QiGeC7DtCdYiyyfrM
// LpUSD-Mint: 7LrqbpCQhVFDJD3X3k6HzgYAtpZe4be7biTDBWrZi2Qs
// Bumps {
//   stateAccount: 255,
//   lpusdMint: 255,
//   lpsolMint: 255,
//   poolUsdc: 255,
//   poolBtc: 255
// }