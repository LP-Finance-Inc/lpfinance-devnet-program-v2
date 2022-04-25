// Migrations are an early feature. Currently, they're nothing more than this
// single deploy script that's invoked from the CLI, injecting a provider
// configured from the workspace's Anchor.toml.

const anchor = require("@project-serum/anchor");
const whiteListJson = require("./whitelist.json");

const { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY } = anchor.web3;

const idl = require("../target/idl/lpfinance_accounts.json");
const programID = idl.metadata.address;

console.log("ProgramID", programID);

const cbsprogram = new PublicKey("HKakh92meu61n3kchSPpNDveCwHno9ymeamN9yZbXt1z"); 
const BIG_WHITELIST_LEN = 10000;

module.exports = async function (provider) {
  // Configure client to use the provider.
  anchor.setProvider(provider);

  // Add your deploy script here
  const program = new anchor.Program(idl, programID);

  try {
    const whitelistAccountSize = 8 + (32 * BIG_WHITELIST_LEN);

    // const configAccount = anchor.web3.Keypair.generate();
    // const whiteListData = anchor.web3.Keypair.generate();
    // console.log("ConfigData: ", configAccount.secretKey);
    // console.log("WhiteListData: ", whiteListData.secretKey);
    // console.log("ConfigAccount:", configAccount.publicKey.toBase58());
    // console.log("WhiteListAccount:", whiteListData.publicKey.toBase58());

    // Signer
    const authority = provider.wallet.publicKey;
       
    // initialize
    // const init_tx = await program.rpc.initialize(cbsprogram, {
    //   accounts: {
    //     authority,
    //     whitelist: whiteListData.publicKey,
    //     config: configAccount.publicKey,
    //     systemProgram: SystemProgram.programId,
    //     rent: SYSVAR_RENT_PUBKEY,
    //   },
    //   signers: [configAccount, whiteListData],
    //   instructions: [
    //     SystemProgram.createAccount({
    //       fromPubkey: program.provider.wallet.publicKey,
    //       lamports:
    //          await program.provider.connection.getMinimumBalanceForRentExemption(
    //             whitelistAccountSize
    //          ),
    //       newAccountPubkey: whiteListData.publicKey,
    //       programId: program.programId,
    //       space: whitelistAccountSize,
    //     }),
    //   ],
    // });

    // console.log("Initialize tx", init_tx);

    // const addys = [];
    // addys.push(new PublicKey("FuRNteV4mDLdvBG1dwPZXKdY5MopQz8pCAx5BJ1XUojw"));
    // addys.push(new PublicKey("YwwpaoBBeNT6zHNT3n1EqhWdCeHjQsCC7Y8ZFdTy6RL"));
    // // addys.push(new PublicKey("YwwpaoBBeNT6zHNT3n1EqhWdCeHjQsCC7Y8ZFdTy6RL"));

    // const tx = await program.rpc.addWhitelistAddresses(addys, {
    //   accounts: {
    //     config: configAccount.publicKey,
    //     whitelist: whiteListData.publicKey,
    //     authority
    //   }
    // });

    // console.log("Tx: ", tx);

    // let accountData = await program.account.whiteList.fetch(whiteListData.publicKey);
    // console.log("Account List1: ", accountData.addresses[0].toBase58());
    // console.log("Account List2: ", accountData.addresses[1].toBase58());

    const addresses = whiteListJson.addresses;
    const config = new PublicKey("2N9QkRVTD7nxsPADPYjscZ2LEFyidT4ke3gX2K9xiQV2");
    const whitelist = new PublicKey("C5LzyP3dxUsLhExJwsrYeN2qgTiPZiw69RAwZmoCJ9uZ");
    const configData = await program.account.config.fetch(config);
    // console.log("Counter:", configData);
    const counter = configData.counter;
    // return;
    for (let i = counter + 1; i < addresses.length; i++) { // 
      const addys = [];
      addys.push(new PublicKey(addresses[i]));

      const tx = await program.rpc.addWhitelistAddresses(addys, {
        accounts: {
          whitelist,
          config,
          authority
        }
      });
      console.log(i);
    }


  } catch (err) {
    console.log("Transaction error: ", err);
  }
}

// 2022-04-22 devnet (2)  10000
// ConfigAccount: 2N9QkRVTD7nxsPADPYjscZ2LEFyidT4ke3gX2K9xiQV2
// WhiteListAccount: C5LzyP3dxUsLhExJwsrYeN2qgTiPZiw69RAwZmoCJ9uZ

// 2022-04-22 devnet (1)
// ProgramID 6gUZGnEHSjVmZ3dq99wd7Ut8ER7PSoEAqPjFfjPKkuMv
// ConfigAccount: 9EzbCTagJ1Xxfb5T5HLu1voN5HcMN6SfeCyod2KhAvgG
// WhiteListAccount: 6QDWT8jYDCTx5zrpqNmtm3SpHUZuEn8wVXW7WESqmpd9

// 2022-04-07 devnet
// ProgramID 6gUZGnEHSjVmZ3dq99wd7Ut8ER7PSoEAqPjFfjPKkuMv
// ConfigAccount: CZmYgJ7wHVQm8ww66JyitYM67qDLpE6gGP7vRkrEPGXq
// WhiteListAccount: J3eH9PAzTh6xuaguT2J3Sti6iiFqVR1yxiJjM7ssYaKt

// 2022-03-22 devnet
// ProgramID CaBy6Mh16bVQpnqY7Crt13hU4Zyv8QbW55GfTvVFwxYh
// ConfigAccount: E9vtd7bYeCK5w4RYau7EGttmReHJ24hf3NUL7thzsd2n
// WhiteListAccount: 9Ln3atZznayRjWM73THuCGpzcPgGH9MvjAjHWJEFKQwW
// Initialize tx 33hAoZmxNa4E4iLTY6MpbCHQ5FCHem9FHJ6Yq7CUvkKPjSWCQVGeTp2bo5wUBfhNy5J5ovbAEwsPbXCCkem93Drm