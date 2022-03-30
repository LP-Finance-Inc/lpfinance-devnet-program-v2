import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { LpfinanceStaking } from '../target/types/lpfinance_staking';

describe('lpfinance-staking', () => {

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.LpfinanceStaking as Program<LpfinanceStaking>;

  it('Is initialized!', async () => {
    // Add your test here.
    const tx = await program.rpc.initialize({});
    console.log("Your transaction signature", tx);
  });
});
