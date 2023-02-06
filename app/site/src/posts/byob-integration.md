---
title: BYOB Integration is now live!
date: '2023-02-06'
tags: [flash-loan-mastery, byob, integrations]
description: Flash Loan Mastery has been integrated with BYOB.
permalink: posts/{{ title | slug }}/index.html
---

 [BYOB](https://www.byob.so/) (or Build Your Own Bot) is basically a Solana transaction builder; a user interface where you can create actions (called "instructions" in the Solana ecosystem) and chain them together into one big atomic transaction, quickly and easily.

 We first heard of BYOB in [this tweet](https://twitter.com/Rockooor/status/1621767659745312768) because it had won two awards in the recently concluded Solana Sandstorm hackathon, and were struck with how simple and refreshing it was to use it to compose complex transactions using simple individual actions.  The one thing that was missing was that it did not have a Flash Loan Mastery integration.

 Well, that problem was solved after a couple of beers and some coding which resulted in [this pull request (PR)](https://github.com/rockooor/byob/pull/1) that added Flash Loan Mastery to BYOB.  To BYOB's credit doing this was quite [straight-forward](https://twitter.com/moshthepitt/status/1622532536554426371).

 What does this mean for you and I?  A couple of things:

1. BYOB is the first ever graphical user interface where you can use Flash Loan Mastery without needing to know how to operate a command line.  You can do these actions:

- Deposit
- Withdraw
- Initiate a flash loan

2. Flash Loan Mastery becomes more powerful if you can use it as a tool to achieve your objectives - whatever those might be.  The BYOB integration allows you to really tap into [Solana's famed composability](https://twitter.com/syndica_io/status/1499822888777654273).  For example this [hilarious transaction](https://twitter.com/Rockooor/status/1622537684957773826) has flash loans from both Flash Loan Mastery and Solend and is trivial to do using BYOB.


 Check out [BYOB](https://www.byob.so/) and give Flash Loan Mastery a try there, and let us know what you think. 🍺
