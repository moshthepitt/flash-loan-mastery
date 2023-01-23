# Flash Loan Mastery

**Flash Loan Mastery** is a smart contract that enables flash loans on Solana.  Flash loans are uncollateralized loans without borrowing limits in which a user borrows funds and returns them in the same transaction. If the user can't repay the loan before the transaction is completed, the transaction will fail and the money will be returned.

## Documentation

Visit our [website](https://flashloanmastery.com/) for more documentation.  Here is a [mirror](https://github.com/moshthepitt/flash-loan-mastery/blob/master/app/site/src/about.md) in case there is an issue with the website.

## Testing

1. Clone the repo, and [install Solana & Anchor](https://www.anchor-lang.com/docs/installation)
2. Run `yarn` to install the packages
3. Run `anchor test`

## Related

1. [Smart contract](https://github.com/moshthepitt/flash-loan-mastery)
2. [Javascript SDK](https://github.com/moshthepitt/flash-loan-mastery-js)
3. [CLI tool](https://github.com/moshthepitt/flash-loan-mastery-cli)
3. [Jupiter Arbitrage Trading](https://github.com/moshthepitt/flm-jupiter-arb)
