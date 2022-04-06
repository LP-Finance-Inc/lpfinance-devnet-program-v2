import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { LendingTokens } from '../target/types/lending_tokens';

describe('lending-tokens', () => {

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.LendingTokens as Program<LendingTokens>;

  it('Is initialized!', async () => {
    // Add your test here.
    const tx = await program.rpc.initialize({});
    console.log("Your transaction signature", tx);
  });
});
