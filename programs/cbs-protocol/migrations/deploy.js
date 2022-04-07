// Migrations are an early feature. Currently, they're nothing more than this
// single deploy script that's invoked from the CLI, injecting a provider
// configured from the workspace's Anchor.toml.

const anchor = require("@project-serum/anchor");
const { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID, Token } = require('@solana/spl-token')
const { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Keypair } = anchor.web3;

const idl = require("../target/idl/cbs_protocol.json");
const programID = idl.metadata.address;

console.log("ProgramID", programID);
const PREFIX = "cbsprotocol0";
const pool_usdc = "pool_usdc";
const pool_btc = "pool_btc";
const pool_eth = "pool_eth";
const pool_msol = "pool_msol";
const pool_lpsol = "pool_lpsol";
const pool_lpusd = "pool_lpusd";
const pool_lpbtc = "pool_lpbtc";
const pool_lpeth = "pool_lpeth";

// Test Token's MINT
const usdcMint = new PublicKey("8cCs2Th4ivThrJPrkgAWNTegQgMcuBmY7TASv7FPhitj"); 
const btcMint = new PublicKey("25ggxgxMqejf5v9WSQWboqxpsrik1u94PCP5EwPBYeEJ");
const msolMint = new PublicKey("3dDwpZWQqCc5SttGJ2yNnYUnLSBnh9cjWJQPeKNDmDTz");
const ethMint = new PublicKey("6Y9PaAZjDs2n4ZJonCu2uCjRp8tuqe6KJEDs1k6iLkbD");

const lpsolMint = new PublicKey("8cCs2Th4ivThrJPrkgAWNTegQgMcuBmY7TASv7FPhitj"); 
const lpusdMint = new PublicKey("25ggxgxMqejf5v9WSQWboqxpsrik1u94PCP5EwPBYeEJ");
const lpbtcMint = new PublicKey("3dDwpZWQqCc5SttGJ2yNnYUnLSBnh9cjWJQPeKNDmDTz");
const lpethMint = new PublicKey("6Y9PaAZjDs2n4ZJonCu2uCjRp8tuqe6KJEDs1k6iLkbD");

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

    // Find PDA for `btc pool`
    const [poolMsol, poolMsolBump] = await PublicKey.findProgramAddress(
      [Buffer.from(PREFIX), Buffer.from(pool_msol)],
      program.programId
    );
    console.log("Pool-MSOL:", poolMsol.toBase58());

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
        poolUsdc,
        poolEth,
        poolBtc,
        poolMsol,
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

  } catch (err) {
    console.log("Transaction error: ", err);
  }
}

// 2022-04-07 devnet
// ProgramID 6fp8G5rybFqcEJDpXUQhc4ump7aHz59dLbYMpxKsdYVp
// Config:  6bUzHQxih8vuMtZL7fm2xsfSt55zDuL4m9RwrqXk9YDp
// State-Account: Fe3ssjryG7wW7aC7PnTrA8w6TJSKs8CbV3DjoCjJJsdw
// Pool-ETH: 5iPpUr2wtoZ7KLDEvtiYJ3EFKzbgbGPep73mTbgeyp8q
// Pool-USDC: Cdf1MY5c4aR9J8eWChHDvjivdWCsmm6QfdTBWyYBtT7S
// Pool-BTC: 49it3gP5BSpGFLuHRQXdn5vMGyPtgiaVNfRQ6N8NNet4
// Pool-MSOL: 5wJmThdpat6aie8ZxAXPhnphuqjrGpXEai2bNE2TPS6D
// Pool-LpSOL: MKi8ukqLJsfxXb4fCVf1WSWn2UyvP2Tib6qgcfkb25b
// Pool-LpUSD: FiDzogxrkJzZMpWsJRF1ZqftH4G7AmzisEZ1ZBciq6s2
// Pool-LpBTC: HWHn4EjmaMwwpsAkGRFnetfffhRi7BDBnDXqktqDvuEJ
// Pool-LpETH: 3mJM6UXAHifoRvhpo9QPfsBY3guhEMiP4D7T4S72HEHm

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
// const PREFIX = "cbs_pool02";
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