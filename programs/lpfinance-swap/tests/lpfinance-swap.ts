import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { LpfinanceSwap } from '../target/types/lpfinance_swap';

describe('lpfinance-swap', () => {

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.LpfinanceSwap as Program<LpfinanceSwap>;

  it('Is initialized!', async () => {
    // Add your test here.
    const tx = await program.rpc.initialize({});
    console.log("Your transaction signature", tx);
  });
});
