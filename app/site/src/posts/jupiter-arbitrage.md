---
title: Jupiter Arbitrage Trading
date: '2023-01-23'
tags: [arbitrage, code, flash-loan-mastery, cli, usage, bonk, versioned-transactions]
description: Jupiter arbitrage trading using Flash Loan Mastery.
permalink: posts/{{ title | slug }}/index.html
---

## Installation

1. [Git Clone](https://git-scm.com/docs/git-clone) the [repository](https://github.com/moshthepitt/flm-jupiter-arb).
2. Install all the dependencies with `yarn`.

```sh
yarn
```
3. Locate or [setup your file system wallet](https://docs.solana.com/wallet-guide/file-system-wallet#:~:text=A%20file%20system%20wallet%20exists,system%20wallet%20is%20not%20recommended.) file.
4. Set up a file named `.env` (right next to where the file named .env.sample is located) and put in at least the Solana RPC URL you want to use.  The file contents would be something like this:

```yaml
NODE_ENV=production
RPC_URI=https://api.mainnet-beta.solana.com
FLM_PROGRAM_ID=1oanfPPN8r1i4UbugXHDxWMbWVJ5qLSN5qzNFZkz6Fg
```
5. Run `yarn start help` to see all the commands.

```sh
yarn start help
```

## Examples

```sh
yarn start help simple-jupiter-arb
```

For example, here's how you would run a trade between USDC and BONK:

First, create common token accounts to avoid setup during trades:

```sh
yarn start create-token-accounts -k /path/to/solana-wallet.json
```

Then the actual arb command:

```sh
yarn start simple-jupiter-arb -k /path/to/solana-wallet.json -m1 DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263 -m2 EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v -a 1000000
```

[Here is what a profitable transaction would look like](https://solscan.io/tx/2wbbaNidGLfefRGkfJQwVCVgCkV8X8yaJLZ3gGqwu5Bhzmnh6dLF2dfZmAYTPDoCrzGpryCuZP75eNGWT4NgJLLJ).
