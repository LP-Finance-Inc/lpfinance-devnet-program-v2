import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { Faucet } from '../target/types/faucet';

describe('faucet', () => {

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.Faucet as Program<Faucet>;

  it('Is initialized!', async () => {
    // Add your test here.
    const tx = await program.rpc.initialize({});
    console.log("Your transaction signature", tx);
  });
});
