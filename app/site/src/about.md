---
title: About Flash Loan Mastery
layout: about.njk
name: Flash Loan Mastery
---

**Flash Loan Mastery** is a smart contract that enables flash loans on Solana.  We think that this is the simplest and best Solana flash loan program.

## What are flash loans?

Flash loans are uncollateralized loans without borrowing limits in which a user borrows funds and returns them in the same transaction. If the user can't repay the loan before the transaction is completed, the transaction will fail and the money will be returned.

## How does it work?

Our aim is to make Flash Loan Mastery as simple as possible, with very easy to understand code.  Unlike most Solana smart contracts, Flash Loan Mastery is completely open source and we aim to freeze the smart contract (make it un-upgradeable) within the next 3 months (of January 2023).

[Here is an example of a Flash Loan Mastery flash loan transaction](https://explorer.solana.com/tx/FmeRtC2TZTTBMERiBq71gwTE7MUgHztJBePUzfJY3j3UZrytYD5pWAUmiMDR8MaupgaRsN2atpQjwLHy8M3671C).

There are three steps involved when using Flash Loan Mastery:

1. Borrow funds
2. Interact with smart contracts for other operations
3. Return the funds

Our [javascript SDK](https://github.com/moshthepitt/flash-loan-mastery-js) can be used to interact with Flash Loan Mastery.

## How much does it cost?

We charge a flat fee of **0.095%** for each successful flash loan.  [Read more](/posts/charges-and-fees/).

## Source Code

1. [Smart contract](https://github.com/moshthepitt/flash-loan-mastery)
2. [Javascript SDK](https://github.com/moshthepitt/flash-loan-mastery-js)
3. [CLI tool](https://github.com/moshthepitt/flash-loan-mastery-cli)
3. [Jupiter Arbitrage Trading](https://github.com/moshthepitt/flm-jupiter-arb)

## Deployments

The Flash Loan Mastery contract/program address is **1oanfPPN8r1i4UbugXHDxWMbWVJ5qLSN5qzNFZkz6Fg**

1. [Solana mainnet-beta](https://explorer.solana.com/address/1oanfPPN8r1i4UbugXHDxWMbWVJ5qLSN5qzNFZkz6Fg)
2. [Solana devnet](https://explorer.solana.com/address/1oanfPPN8r1i4UbugXHDxWMbWVJ5qLSN5qzNFZkz6Fg?cluster=devnet)
