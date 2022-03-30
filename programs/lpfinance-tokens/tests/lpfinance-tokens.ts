import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { LpfinanceTokens } from '../target/types/lpfinance_tokens';

describe('lpfinance-tokens', () => {

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.LpfinanceTokens as Program<LpfinanceTokens>;

  it('Is initialized!', async () => {
    // Add your test here.
    const tx = await program.rpc.initialize({});
    console.log("Your transaction signature", tx);
  });
});
