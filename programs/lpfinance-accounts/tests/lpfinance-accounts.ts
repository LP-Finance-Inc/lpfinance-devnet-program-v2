import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { LpfinanceAccounts } from '../target/types/lpfinance_accounts';

describe('lpfinance-accounts', () => {

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.LpfinanceAccounts as Program<LpfinanceAccounts>;

  it('Is initialized!', async () => {
    // Add your test here.
    const tx = await program.rpc.initialize({});
    console.log("Your transaction signature", tx);
  });
});
