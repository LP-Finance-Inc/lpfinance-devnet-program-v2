import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { Solend } from '../target/types/solend';

describe('solend', () => {

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.Solend as Program<Solend>;

  it('Is initialized!', async () => {
    // Add your test here.
    const tx = await program.rpc.initialize({});
    console.log("Your transaction signature", tx);
  });
});
