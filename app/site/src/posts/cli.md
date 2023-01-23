---
title: Flash Loan Mastery CLI
date: '2023-01-22'
tags: [code, flash-loan-mastery, cli, usage, bonk, versioned-transactions]
description: All about the Flash Loan Mastery CLI tool.
permalink: posts/{{ title | slug }}/index.html
---

Our CLI tool allows you to use Flash Loan Mastery from the command line.

What the CLI can do:

- Initialize flash loan pools
- Deposit tokens into flash loan pools
- Withdraw tokens from flash loan pools
- Run example flash loan instructions
- Arbitrage using our flash loans and [Jupiter](https://jup.ag/)


## Installation

1. [Git Clone](https://git-scm.com/docs/git-clone) the [CLI repository](https://github.com/moshthepitt/flash-loan-mastery-cli).
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

You can also run yarn start help command to see what each command is for. e.g.

```sh
yarn start help deposit
```

## Examples

### Init Pool

To get help on this command:

```sh
yarn start help init-pool
```

To set up a new pool:

```sh
yarn start init-pool -k /path/to/solana-wallet.json -tm DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263 -pm EEgkPj5Z4J9KMCFSchiMz9wGusJgw6wqGPMyMJT9hoEZ
```

Note that the `pm` (Pool Mint) option:

- should be a new mint that is owned by the user
- should not have any tokens issued
- should have the same number of decimals as the `tm` (Token Mint) option

Note that a pool for any mint can only bet up once, and this can be done by anyone.

### Deposit

To get help on this command:

```sh
yarn start help deposit
```

For example, to deposit some [BONK](https://twitter.com/bonk_inu):

```sh
yarn start deposit -k /path/to/solana-wallet.json -tm DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263 -tf Hm4ebPskgjJVesKLowyEhpLW6axodBbH82k5CHz3ynSa -a 10310517.49915
```

### Withdraw

To get help on this command:

```sh
yarn start help withdraw
```

For example, to withdraw some USDC:

```sh
yarn start withdraw -k /path/to/solana-wallet.json -tm EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v -ptf 6H7ahCDN8hny2mWKoQBycUqLXoq4aaZ6V32rpmh99eGr -a 4.249834
```

### Example Flash Loan

To get help on this command:

```sh
yarn start help example-flash-loan
```

For example, an example of a flash loan of 10 million BONK:

```sh
yarn start example-flash-loan -k /path/to/solana-wallet.json -tm DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263 -a 10000000
```

[Here is what the transaction itself](https://solana.fm/tx/5wbvKGpQcojidPoAr5Kc8xeJijy2UyYgu3ZrFCDkJL7DHbx4tmPd7wgafL54NMv6oxYDEf6JKw1yAaXYN5hx9f3J?cluster=mainnet-qn1) looks like.  We are fairly certain that this might be the very first BONK flash loan ever! :)

### Example Flash Loan Using Versioned Transactions

We support flash loans using Solana's [versioned transactions](https://docs.solana.com/developing/versioned-transactions).

For example, for a versioned flash loan of 1,000 USDC:

#### 1. Create keys/addresses that should be saved in the address lookup table

```sh
yarn start seed-example-flash-loan-lookup-table -k /path/to/solana-wallet.json -tm EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v -a 1000
```
#### 2. Create the address lookup table

```sh
yarn start create-versioned-flash-loan-lookup-table-from-cache -k /path/to/solana-wallet.json -tm EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
```

#### 3. Execute the flash loan using versioned transactions

```sh
yarn start example-versioned-flash-loan -k /path/to/solana-wallet.json -tm EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v -a 1000
```

[Here is what this versioned flash loan transaction](https://solana.fm/tx/2utP7HZNCbx3BtZSNuZbpmhDz1Gxbw7h1MygPjvVEo1MQry7VHPqHxevXy5AEYiFkJFsdBMg7vaWXW6iKWz8jmkv?cluster=mainnet-qn1) ends up looking like.