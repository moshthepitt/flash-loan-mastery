# Flash Loan Mastery

**Flash Loan Mastery** is a smart contract that enables flash loans on Solana.  Flash loans are uncollateralized loans without borrowing limits in which a user borrows funds and returns them in the same transaction. If the user can't repay the loan before the transaction is completed, the transaction will fail and the money will be returned.

## Testing

1. Clone the repo, and [install Solana & Anchor](https://www.anchor-lang.com/docs/installation)
2. Run `yarn` to install the packages
3. Run `anchor test`
