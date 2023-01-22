---
title: Flash Loan Mastery CLI
date: '2023-01-22'
tags: [code, flash-loan-mastery, cli, usage]
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
4. Run `yarn start help` to see all the commands.

```sh
yarn start help
```

You can also run yarn start help command to see what each command is for. e.g.

```sh
yarn start help deposit
```