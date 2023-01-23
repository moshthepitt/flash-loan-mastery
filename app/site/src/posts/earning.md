---
title: Earning With Flash Loan Mastery
date: '2023-01-23'
tags: [depositor, referrer, profit, earn, code, developer]
description: How to earn with Flash Loan Mastery.
permalink: posts/{{ title | slug }}/index.html
---

Apart from using Flash Loan Mastery profitably (e.g. [arbitrage trading](/posts/jupiter-arbitrage-trading/)), you can earn a return from Flash Loan Mastery in the following ways:

## 1. Earn as a Depositor

When you deposit your funds into Flash Loan Mastery mastery, you are entitled to a share of the flash loan fees that accrue to that flash loan pool.  For instance, if you supply USDC then any time that a user uses USDC flash loans, the [loan fee](/posts/charges-and-fees/) is deposited into the USDC flash loan pool.  When it comes for you to withdraw your USDC, you get the amount that you originally deposited + your share of the loan fees.

## 2. Earn as a Referrer

Flash Loan Mastery allows you to specify a referrer whenever requesting a flash loan.  **0.05% of the loan amount** is paid out to this referrer.

This is meant to incentivize developers to integrate Flash Loan Mastery into their apps, websites and tools.  [Here is a good example of how to do this in code](https://github.com/moshthepitt/flm-jupiter-arb/blob/master/src/jup.ts#L149).

```ts
   // ...lots of other code

  const flashLoan = await getFlashLoanInstructions(
    connection,
    wallet,
    mint1,
    amount,
    DEFAULT_REFERRER // this could be YOUR wallet address
  );
```
