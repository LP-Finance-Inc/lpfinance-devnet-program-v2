# LP Finance Program
LP Finance is a decentralized synthetic asset protocol on Solana. LP Finance Protocol allows users to earn interest on supplied tokens and borrow USD and SOL, BTC pegged stablecoin interest-free.

## Contributing/Building
The LP Finance protocol is open source with a focus on developer friendliness and integrations.

## LP Finance Program Structure

1. [lpfinance token program](https://github.com/LP-Finance-Inc/lpfinance-devnet-program/tree/main/programs/lpfinance-tokens) to mint/burn LPFi(LP Finance DAO) token as well as CBS Tokens (lpBTC, lpSOL, lpUSD).
LP Finance Program has been configured with several programs
2. [cbs program](https://github.com/LP-Finance-Inc/lpfinance-devnet-program/tree/main/programs/cbs-protocol) to deposit collaterals and borrow CBS Tokens (lpBTC, lpSOL, lpUSD).
3. [LpFi staking program](https://github.com/LP-Finance-Inc/lpfinance-devnet-program/tree/main/programs/lpfinace-staking) to stake LPFI(LP Finance DAO) token and get daily reward.
4. [Lpfinance auction program](https://github.com/LP-Finance-Inc/lpfinance-devnet-program/tree/main/programs/lpusd-auction) to liquidate the collaterals and give interest reward.
## Development
Program was developed with [anchor](https://github.com/project-serum/anchor) framework.

### Environment Setup

1. Install the lastest Rust stable from https://rustup.rs/
2. Install Solana v1.6.1 or later from https://docs.solana.com/cli/install-solana-cli-tools
3. Install anchor environment from [here](https://project-serum.github.io/anchor/getting-started/installation.html)
4. Install the `libudev` development package for your distribution (`libudev-dev` on Debian-derived distros, `libudev-devel` on Redhat-derived).

### Build

The anchor build is available for building programs against your host machine:

```
$ anchor build
```

To deploy the program
```
$ anchor deploy
```

To initialize the program with tokens and other configurations
```
$ anchor migrate
```

### Test
To test the program
```
anchor test
```
