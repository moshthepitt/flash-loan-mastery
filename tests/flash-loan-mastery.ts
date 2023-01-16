import {
  PublicKey,
  Keypair,
  SystemProgram,
  SYSVAR_INSTRUCTIONS_PUBKEY,
} from "@solana/web3.js";
import * as anchor from "@project-serum/anchor";
import { BN, Program } from "@project-serum/anchor";
import {
  getAccount,
  getMint,
  getAssociatedTokenAddress,
  createInitializeMintInstruction,
  createTransferInstruction,
  createAssociatedTokenAccountInstruction,
  createMintToInstruction,
  MINT_SIZE,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { FlashLoanMastery } from "../target/types/flash_loan_mastery";
import { expect } from "chai";

export const LOAN_FEE = 900;
export const REFERRAL_FEE = 50;
export const LOAN_FEE_DENOMINATOR = 10000;
export const ONE_HUNDRED = 100;

describe("flash-loan-mastery", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace
    .FlashLoanMastery as Program<FlashLoanMastery>;
  const wallet = program.provider.publicKey;
  const tokenMint = new Keypair();
  const poolMint = new Keypair();
  const depositor2 = new Keypair();
  const depositor3 = new Keypair();
  let poolAuthorityKey: PublicKey;

  it("init pool", async () => {
    // set up the mint and token accounts
    const mintCost =
      await program.provider.connection.getMinimumBalanceForRentExemption(
        MINT_SIZE,
        "confirmed"
      );
    // create mints
    const instructions = [tokenMint, poolMint].map((it) => [
      SystemProgram.createAccount({
        fromPubkey: wallet,
        lamports: mintCost,
        newAccountPubkey: it.publicKey,
        programId: TOKEN_PROGRAM_ID,
        space: MINT_SIZE,
      }),
      createInitializeMintInstruction(it.publicKey, 9, wallet, wallet),
    ]);
    const tx = new anchor.web3.Transaction().add(...instructions.flat());
    await program.provider.sendAndConfirm(tx, [tokenMint, poolMint]);

    // create pool
    const poolAuthority = PublicKey.findProgramAddressSync(
      [Buffer.from("flash_loan"), tokenMint.publicKey.toBuffer()],
      program.programId
    );
    poolAuthorityKey = poolAuthority[0];

    const initPoolIx = await program.methods
      .initPool()
      .accountsStrict({
        funder: wallet,
        mint: tokenMint.publicKey,
        poolShareMint: poolMint.publicKey,
        poolShareMintAuthority: wallet,
        poolAuthority: poolAuthority[0],
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .instruction();

    const tx2 = new anchor.web3.Transaction().add(initPoolIx);
    await program.provider.sendAndConfirm(tx2);

    const poolAuthorityAccount = await program.account.poolAuthority.fetch(
      poolAuthority[0]
    );
    expect(poolAuthorityAccount.bump).eq(poolAuthority[1]);
    expect(poolAuthorityAccount.poolShareMint.equals(poolMint.publicKey)).to.be
      .true;
    expect(poolAuthorityAccount.mint.equals(tokenMint.publicKey)).to.be.true;

    const poolShareMintAcc = await getMint(
      program.provider.connection,
      poolMint.publicKey
    );
    expect(poolShareMintAcc.mintAuthority.equals(poolAuthority[0])).to.be.true;
    expect(poolShareMintAcc.freezeAuthority).to.be.null;
  });

  it("deposit into pool", async () => {
    // create token accounts
    const createTokenIxs = [tokenMint, poolMint].map(async (it) => {
      const walletToken = await getAssociatedTokenAddress(it.publicKey, wallet);
      const poolAuthorityToken = await getAssociatedTokenAddress(
        it.publicKey,
        poolAuthorityKey,
        true
      );
      return [
        createAssociatedTokenAccountInstruction(
          wallet,
          walletToken,
          wallet,
          it.publicKey
        ),
        createAssociatedTokenAccountInstruction(
          wallet,
          poolAuthorityToken,
          poolAuthorityKey,
          it.publicKey
        ),
      ];
    });
    // mint token to wallet
    const instructions = (await Promise.all(createTokenIxs)).flat();
    instructions.push(
      createMintToInstruction(
        tokenMint.publicKey,
        await getAssociatedTokenAddress(tokenMint.publicKey, wallet),
        wallet,
        100_000_000
      )
    );
    // create pool share account for depositor2
    instructions.push(
      createAssociatedTokenAccountInstruction(
        wallet,
        await getAssociatedTokenAddress(
          poolMint.publicKey,
          depositor2.publicKey
        ),
        depositor2.publicKey,
        poolMint.publicKey
      )
    );
    // create pool share account for depositor3
    instructions.push(
      createAssociatedTokenAccountInstruction(
        wallet,
        await getAssociatedTokenAddress(
          poolMint.publicKey,
          depositor3.publicKey
        ),
        depositor3.publicKey,
        poolMint.publicKey
      )
    );

    await program.provider.sendAndConfirm(
      new anchor.web3.Transaction().add(...instructions)
    );

    const tokenFrom = await getAssociatedTokenAddress(
      tokenMint.publicKey,
      wallet
    );
    const tokenTo = await getAssociatedTokenAddress(
      tokenMint.publicKey,
      poolAuthorityKey,
      true
    );
    const poolShareTokenTo = await getAssociatedTokenAddress(
      poolMint.publicKey,
      wallet
    );
    const poolShareTokenToDepositor2 = await getAssociatedTokenAddress(
      poolMint.publicKey,
      depositor2.publicKey
    );
    const poolShareTokenToDepositor3 = await getAssociatedTokenAddress(
      poolMint.publicKey,
      depositor3.publicKey
    );

    const amount1 = new BN(100_000);
    const ix = await program.methods
      .deposit(amount1)
      .accountsStrict({
        depositor: wallet,
        tokenFrom,
        tokenTo,
        poolShareTokenTo,
        poolShareMint: poolMint.publicKey,
        poolAuthority: poolAuthorityKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .instruction();

    await program.provider.sendAndConfirm(
      new anchor.web3.Transaction().add(ix)
    );

    let tokenToAccAfter = await getAccount(
      program.provider.connection,
      tokenTo,
      "processed"
    );
    let poolShareTokenToAccAfter = await getAccount(
      program.provider.connection,
      poolShareTokenTo,
      "processed"
    );
    let poolShareMintAccAfter = await getMint(
      program.provider.connection,
      poolMint.publicKey,
      "processed"
    );
    expect(tokenToAccAfter.amount).equals(BigInt(amount1.toString()));
    expect(poolShareTokenToAccAfter.amount).equals(BigInt(amount1.toString()));
    expect(poolShareMintAccAfter.supply).equals(BigInt(amount1.toString()));
    // 100% of pool shares
    expect(
      Number(poolShareTokenToAccAfter.amount) /
        Number(poolShareMintAccAfter.supply)
    ).eq(1);

    // deposit again, different account
    const amount2 = new BN(100_000);
    await program.provider.sendAndConfirm(
      new anchor.web3.Transaction().add(
        await program.methods
          .deposit(amount2)
          .accountsStrict({
            depositor: wallet,
            tokenFrom,
            tokenTo,
            poolShareTokenTo: poolShareTokenToDepositor2,
            poolShareMint: poolMint.publicKey,
            poolAuthority: poolAuthorityKey,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .instruction()
      )
    );
    let tokenToAccAfter2 = await getAccount(
      program.provider.connection,
      tokenTo,
      "processed"
    );
    let poolShareTokenToAccAfter2 = await getAccount(
      program.provider.connection,
      poolShareTokenTo,
      "processed"
    );
    let poolShareMintAccAfter2 = await getMint(
      program.provider.connection,
      poolMint.publicKey,
      "processed"
    );
    expect(tokenToAccAfter2.amount).equals(
      BigInt(amount1.toString()) + BigInt(amount2.toString())
    );
    // 50% of pool shares
    expect(
      Number(poolShareTokenToAccAfter2.amount) /
        Number(poolShareMintAccAfter2.supply)
    ).eq(0.5);

    // simulate pool profits by transferring directly to pool
    const profits = 234_567;
    await program.provider.sendAndConfirm(
      new anchor.web3.Transaction().add(
        createTransferInstruction(tokenFrom, tokenTo, wallet, profits)
      )
    );
    let tokenToAccAfter2b = await getAccount(
      program.provider.connection,
      tokenTo,
      "processed"
    );

    // deposit again, yet another different account
    const amount3 = new BN(33_000);
    await program.provider.sendAndConfirm(
      new anchor.web3.Transaction().add(
        await program.methods
          .deposit(amount3)
          .accountsStrict({
            depositor: wallet,
            tokenFrom,
            tokenTo,
            poolShareTokenTo: poolShareTokenToDepositor3,
            poolShareMint: poolMint.publicKey,
            poolAuthority: poolAuthorityKey,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .instruction()
      )
    );
    let poolShareTokenToAccAfter3 = await getAccount(
      program.provider.connection,
      poolShareTokenTo,
      "processed"
    );
    let poolShareMintAccAfter3 = await getMint(
      program.provider.connection,
      poolMint.publicKey,
      "processed"
    );

    const depositor3Shares = Math.floor(
      (amount3.toNumber() * Number(poolShareMintAccAfter2.supply)) /
        Number(tokenToAccAfter2b.amount)
    );
    expect(Number(poolShareMintAccAfter3.supply)).equals(
      amount1.add(amount2).toNumber() + depositor3Shares
    );
    // ~46% of pool shares
    expect(
      Number(poolShareTokenToAccAfter3.amount) /
        Number(poolShareMintAccAfter3.supply)
    ).eq(100_000 / Number(poolShareMintAccAfter3.supply));
  });

  it("withdraw from pool", async () => {
    const tokenTo = await getAssociatedTokenAddress(
      tokenMint.publicKey,
      wallet
    );
    const tokenFrom = await getAssociatedTokenAddress(
      tokenMint.publicKey,
      poolAuthorityKey,
      true
    );
    const poolShareTokenFrom = await getAssociatedTokenAddress(
      poolMint.publicKey,
      wallet
    );

    let tokenToBefore = await getAccount(
      program.provider.connection,
      tokenTo,
      "processed"
    );
    let tokenFromBefore = await getAccount(
      program.provider.connection,
      tokenFrom,
      "processed"
    );
    let poolShareTokenFromBefore = await getAccount(
      program.provider.connection,
      poolShareTokenFrom,
      "processed"
    );
    let poolShareMintAccBefore = await getMint(
      program.provider.connection,
      poolMint.publicKey,
      "processed"
    );

    const amount1 = new BN(50);
    await program.provider.sendAndConfirm(
      new anchor.web3.Transaction().add(
        await program.methods
          .withdraw(amount1)
          .accountsStrict({
            withdrawer: wallet,
            tokenFrom,
            tokenTo,
            poolShareTokenFrom,
            poolShareMint: poolMint.publicKey,
            poolAuthority: poolAuthorityKey,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .instruction()
      )
    );

    let tokenToAfter = await getAccount(
      program.provider.connection,
      tokenTo,
      "processed"
    );
    let tokenFromAfter = await getAccount(
      program.provider.connection,
      tokenFrom,
      "processed"
    );
    let poolShareTokenFromAfter = await getAccount(
      program.provider.connection,
      poolShareTokenFrom,
      "processed"
    );
    let poolShareMintAccAfter = await getMint(
      program.provider.connection,
      poolMint.publicKey,
      "processed"
    );

    const tokenValue = Math.floor(
      (amount1.toNumber() * Number(tokenFromBefore.amount)) /
        Number(poolShareMintAccBefore.supply)
    );
    expect(poolShareTokenFromAfter.amount).equals(
      poolShareTokenFromBefore.amount - BigInt(amount1.toString())
    );
    expect(poolShareMintAccAfter.supply).equals(
      poolShareMintAccBefore.supply - BigInt(amount1.toString())
    );
    expect(tokenFromAfter.amount).equals(
      tokenFromBefore.amount - BigInt(tokenValue)
    );
    expect(tokenToAfter.amount).equals(
      tokenToBefore.amount + BigInt(tokenValue)
    );
  });

  it("process flash loans!", async () => {
    const lenderFrom = await getAssociatedTokenAddress(
      tokenMint.publicKey,
      poolAuthorityKey,
      true
    );
    const borrowerTo = await getAssociatedTokenAddress(
      tokenMint.publicKey,
      depositor2.publicKey
    );
    const repayerFrom = await getAssociatedTokenAddress(
      tokenMint.publicKey,
      wallet,
      true
    );

    let lenderFromBefore = await getAccount(
      program.provider.connection,
      lenderFrom,
      "processed"
    );
    let repayerFromBefore = await getAccount(
      program.provider.connection,
      repayerFrom,
      "processed"
    );

    const createDepositor2TokenIx = createAssociatedTokenAccountInstruction(
      wallet,
      borrowerTo,
      depositor2.publicKey,
      tokenMint.publicKey
    );
    const amount1 = new BN(Number(400_000));
    const borrowIx = await program.methods
      .borrow(amount1)
      .accountsStrict({
        borrower: wallet,
        tokenFrom: lenderFrom,
        tokenTo: borrowerTo,
        poolAuthority: poolAuthorityKey,
        instructionsSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .instruction();

    const loanFees = amount1
      .mul(new BN(LOAN_FEE))
      .div(new BN(LOAN_FEE_DENOMINATOR))
      .div(new BN(ONE_HUNDRED));
    const referralFee = amount1
      .mul(new BN(REFERRAL_FEE))
      .div(new BN(LOAN_FEE_DENOMINATOR))
      .div(new BN(ONE_HUNDRED));
    const totalFees = amount1
      .mul(new BN(LOAN_FEE + REFERRAL_FEE))
      .div(new BN(LOAN_FEE_DENOMINATOR))
      .div(new BN(ONE_HUNDRED));
    const repaymentAmount = amount1.add(totalFees);
    const repaymentAmountNoReferral = amount1.add(loanFees);
    const repayIx = await program.methods
      .repay(repaymentAmount)
      .accountsStrict({
        repayer: wallet,
        tokenFrom: repayerFrom,
        tokenTo: lenderFrom,
        poolAuthority: poolAuthorityKey,
        instructionsSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .instruction();

    await program.provider.sendAndConfirm(
      new anchor.web3.Transaction().add(
        ...[createDepositor2TokenIx, borrowIx, repayIx]
      )
    );

    let lenderFromAfter = await getAccount(
      program.provider.connection,
      lenderFrom,
      "processed"
    );
    let borrowerToAfter = await getAccount(
      program.provider.connection,
      borrowerTo,
      "processed"
    );
    let repayerFromAfter = await getAccount(
      program.provider.connection,
      repayerFrom,
      "processed"
    );

    expect(borrowerToAfter.amount).equals(BigInt(amount1.toNumber()));
    expect(Number(lenderFromAfter.amount)).gt(Number(lenderFromBefore.amount));
    expect(Number(lenderFromAfter.amount)).equals(
      new BN(lenderFromBefore.amount.toString()).add(loanFees).toNumber()
    );
    expect(repayerFromAfter.amount).equals(
      repayerFromBefore.amount - BigInt(repaymentAmountNoReferral.toNumber())
    ) /** no referral fees charged */;

    // inclusion of referral fee works
    const referralTokenTo = await getAssociatedTokenAddress(
      tokenMint.publicKey,
      depositor3.publicKey,
      true
    );
    const createReferralTokenIx = createAssociatedTokenAccountInstruction(
      wallet,
      referralTokenTo,
      depositor3.publicKey,
      tokenMint.publicKey
    );
    const repayWithReferralIx = await program.methods
      .repay(repaymentAmount)
      .accountsStrict({
        repayer: wallet,
        tokenFrom: repayerFrom,
        tokenTo: lenderFrom,
        poolAuthority: poolAuthorityKey,
        instructionsSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .remainingAccounts([
        { pubkey: referralTokenTo, isSigner: false, isWritable: true },
      ])
      .instruction();
    await program.provider.sendAndConfirm(
      new anchor.web3.Transaction().add(
        ...[createReferralTokenIx, borrowIx, repayWithReferralIx]
      )
    );
    let lenderFromAfter2 = await getAccount(
      program.provider.connection,
      lenderFrom,
      "processed"
    );
    let repayerFromAfter2 = await getAccount(
      program.provider.connection,
      repayerFrom,
      "processed"
    );
    let referralTokenToAfter = await getAccount(
      program.provider.connection,
      referralTokenTo,
      "processed"
    );
    expect(referralTokenToAfter.amount).equals(
      BigInt(referralFee.toNumber())
    ) /** referral fee paid */;
    expect(Number(lenderFromAfter2.amount)).gt(Number(lenderFromAfter.amount));
    expect(Number(lenderFromAfter2.amount)).equals(
      new BN(lenderFromAfter.amount.toString())
        .add(loanFees)
        .toNumber()
    ) /** no change in expected payment amount */;
    expect(repayerFromAfter2.amount).equals(
      repayerFromAfter.amount - BigInt(repaymentAmount.toNumber())
    ) /** referral fees have been charged */;

      // wrong repayment fails
      let success1 = true;
      try {
        await program.provider.sendAndConfirm(
          new anchor.web3.Transaction().add(
            ...[
              await program.methods
                .borrow(new BN(100_000))
                .accountsStrict({
                  borrower: wallet,
                  tokenFrom: lenderFrom,
                  tokenTo: borrowerTo,
                  poolAuthority: poolAuthorityKey,
                  instructionsSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
                  tokenProgram: TOKEN_PROGRAM_ID,
                })
                .instruction(),
              await program.methods
                .repay(new BN(90_000))
                .accountsStrict({
                  repayer: wallet,
                  tokenFrom: repayerFrom,
                  tokenTo: lenderFrom,
                  poolAuthority: poolAuthorityKey,
                  instructionsSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
                  tokenProgram: TOKEN_PROGRAM_ID,
                })
                .instruction(),
            ]
          )
        );
      } catch {
        success1 = false;
      }
      expect(success1).to.be.false;

      // no repayment fails
      let success2 = true;
      try {
        await program.provider.sendAndConfirm(
          new anchor.web3.Transaction().add(
            ...[
              await program.methods
                .borrow(new BN(100_000))
                .accountsStrict({
                  borrower: wallet,
                  tokenFrom: lenderFrom,
                  tokenTo: borrowerTo,
                  poolAuthority: poolAuthorityKey,
                  instructionsSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
                  tokenProgram: TOKEN_PROGRAM_ID,
                })
                .instruction(),
              createTransferInstruction(repayerFrom, lenderFrom, wallet, 1337),
            ]
          )
        );
      } catch {
        success2 = false;
      }
      expect(success2).to.be.false;

      // wrong repayment token account fails
      let success3 = true;
      try {
        await program.provider.sendAndConfirm(
          new anchor.web3.Transaction().add(
            ...[
              await program.methods
                .borrow(new BN(100_000))
                .accountsStrict({
                  borrower: wallet,
                  tokenFrom: lenderFrom,
                  tokenTo: borrowerTo,
                  poolAuthority: poolAuthorityKey,
                  instructionsSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
                  tokenProgram: TOKEN_PROGRAM_ID,
                })
                .instruction(),
              await program.methods
                .repay(new BN(90_000))
                .accountsStrict({
                  repayer: wallet,
                  tokenFrom: repayerFrom,
                  tokenTo: borrowerTo /** this is wrong */,
                  poolAuthority: poolAuthorityKey,
                  instructionsSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
                  tokenProgram: TOKEN_PROGRAM_ID,
                })
                .instruction(),
            ]
          )
        );
      } catch {
        success3 = false;
      }
      expect(success3).to.be.false;

      // repayment with no borrow works
      await program.provider.sendAndConfirm(
        new anchor.web3.Transaction().add(
          ...[
            await program.methods
              .repay(new BN(90_000))
              .accountsStrict({
                repayer: wallet,
                tokenFrom: repayerFrom,
                tokenTo: lenderFrom,
                poolAuthority: poolAuthorityKey,
                instructionsSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
                tokenProgram: TOKEN_PROGRAM_ID,
              })
              .instruction(),
          ]
        )
      );

      // re-borrow before repaying fails
      let success4 = true;
      try {
        await program.provider.sendAndConfirm(
          new anchor.web3.Transaction().add(
            ...[
              await program.methods
                .borrow(amount1)
                .accountsStrict({
                  borrower: wallet,
                  tokenFrom: lenderFrom,
                  tokenTo: borrowerTo,
                  poolAuthority: poolAuthorityKey,
                  instructionsSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
                  tokenProgram: TOKEN_PROGRAM_ID,
                })
                .instruction(),
              await program.methods
                .borrow(new BN(10))
                .accountsStrict({
                  borrower: wallet,
                  tokenFrom: lenderFrom,
                  tokenTo: borrowerTo,
                  poolAuthority: poolAuthorityKey,
                  instructionsSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
                  tokenProgram: TOKEN_PROGRAM_ID,
                })
                .instruction() /** borrow again */,
              await program.methods
                .repay(repaymentAmount)
                .accountsStrict({
                  repayer: wallet,
                  tokenFrom: repayerFrom,
                  tokenTo: lenderFrom,
                  poolAuthority: poolAuthorityKey,
                  instructionsSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
                  tokenProgram: TOKEN_PROGRAM_ID,
                })
                .instruction(),
            ]
          )
        );
      } catch {
        success4 = false;
      }
      expect(success4).to.be.false;
  });
});
