import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { LpusdAuction } from '../target/types/lpusd_auction';

describe('lpusd-auction', () => {

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.LpusdAuction;

  it('Is initialized!', async () => {
    // Add your test here.
    const tx = await program.rpc.initialize({});
    console.log("Your transaction signature", tx);
  });
});
