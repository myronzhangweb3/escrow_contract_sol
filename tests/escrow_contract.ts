import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { BridgeContract } from "../target/types/escrow_contract";
import { PublicKey, SystemProgram } from '@solana/web3.js';
import * as splToken from '@solana/spl-token';
import { assert, expect } from "chai";
import { getAccount } from "@solana/spl-token";

describe("escrow_contract", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.BridgeContract as Program<BridgeContract>;

  let mint: PublicKey;
  const payer = (provider.wallet as anchor.Wallet).payer;
  const operator = anchor.web3.Keypair.generate();
  const escrowAccount = anchor.web3.Keypair.generate();
  let escrowTokenAccount : PublicKey;
  const amountToDistribute = new anchor.BN(1);
  console.log(`payer: ${payer.publicKey.toBase58()}`)
  console.log(`operator: ${operator.publicKey.toBase58()}`)
  console.log(`escrowAccount: ${escrowAccount.publicKey.toBase58()}`)
  
  before(async () => {
    mint = await splToken.createMint(
      provider.connection,
      payer,
      provider.wallet.publicKey,
      null,
      6
    );

    escrowTokenAccount = await splToken.createAssociatedTokenAccount(
      provider.connection,
      payer,
      mint,
      escrowAccount.publicKey
    );

    await splToken.mintTo(
      provider.connection,
      payer,
      mint,
      escrowTokenAccount,
      payer,
      1000
    );
    const recipientAmount = await splToken.getAccount(provider.connection, escrowTokenAccount);
    console.log(`recipient address token balance: ${recipientAmount.amount}`)

    // Transfer SOL with rent exemption
    const rentExemptAmount = await provider.connection.getMinimumBalanceForRentExemption(0);
    const solTransferTxToOperator = new anchor.web3.Transaction().add(
      anchor.web3.SystemProgram.transfer({
        fromPubkey: payer.publicKey,
        toPubkey: operator.publicKey,
        lamports: rentExemptAmount,
      })
    );
    await provider.sendAndConfirm(solTransferTxToOperator, [payer]);

    const solTransferTxToOperator2 = new anchor.web3.Transaction().add(
      anchor.web3.SystemProgram.transfer({
        fromPubkey: payer.publicKey,
        toPubkey: operator.publicKey,
        lamports: amountToDistribute.mul(new anchor.BN(100000000000)).toNumber(),
      })
    );
    await provider.sendAndConfirm(solTransferTxToOperator2, [payer]);
    console.log(`operator sol balance: ${await program.provider.connection.getBalance(operator.publicKey)}`)
    const solTransferTxToEscrowAccount = new anchor.web3.Transaction().add(
      anchor.web3.SystemProgram.transfer({
        fromPubkey: payer.publicKey,
        toPubkey: escrowAccount.publicKey,
        lamports: rentExemptAmount,
      })
    );
    await provider.sendAndConfirm(solTransferTxToEscrowAccount, [payer]);
    const solTransferTxToEscrowAccount2 = new anchor.web3.Transaction().add(
      anchor.web3.SystemProgram.transfer({
        fromPubkey: payer.publicKey,
        toPubkey: escrowAccount.publicKey,
        lamports: amountToDistribute.mul(new anchor.BN(100000000000)).toNumber(),
      })
    );
    await provider.sendAndConfirm(solTransferTxToEscrowAccount2, [payer]);
    console.log(`escrowAccount sol balance: ${await program.provider.connection.getBalance(escrowAccount.publicKey)}`)

    await program.methods.initialize(operator.publicKey).accounts({
      escrowAccount: escrowAccount.publicKey,
      authority: payer.publicKey,
      systemProgram: SystemProgram.programId,
    }).signers([escrowAccount, payer]).rpc();

  });

  it("Distributes SOL!", async () => {
    const recipient = anchor.web3.Keypair.generate();
    
    const initialBalance = await program.provider.connection.getBalance(recipient.publicKey);
    const initialEscrowBalance = await program.provider.connection.getBalance(escrowAccount.publicKey);
    
    const rentExemptAmount = await program.provider.connection.getMinimumBalanceForRentExemption(0);
    
    const tx = await program.methods.distributeSol(amountToDistribute).accounts({
      recipient: recipient.publicKey,
      escrowAccount: escrowAccount.publicKey,
      operator: operator.publicKey,
      systemProgram: SystemProgram.programId,
    }).signers([operator]).rpc();

    const finalBalance = await program.provider.connection.getBalance(recipient.publicKey);
    const escrowBalance = await program.provider.connection.getBalance(escrowAccount.publicKey);
    expect(finalBalance).to.equal(initialBalance + amountToDistribute.toNumber() + rentExemptAmount);
    expect(escrowBalance).to.equal(initialEscrowBalance - amountToDistribute.toNumber() - rentExemptAmount);
  });

  it("Distributes Tokens!", async () => {
    const recipientAccount = anchor.web3.Keypair.generate();
    const recipientTokenAccount = await splToken.createAssociatedTokenAccount(
      provider.connection,
      payer,
      mint,
      recipientAccount.publicKey
    );
    const initialEscrowBalance = await splToken.getAccount(provider.connection, escrowTokenAccount);

    const approveTx = await program.methods.authorizeOperatorOnce().accounts({
      escrowAccount: escrowAccount.publicKey,
      senderTokenAccount: escrowTokenAccount,
      senderTokenAccountAuthority: escrowAccount.publicKey,
      operator: operator.publicKey,
      tokenProgram: splToken.TOKEN_PROGRAM_ID, 
    }).signers([escrowAccount]).rpc();

    // send token to recipient account
    const distributeTokenTx = await program.methods.distributeToken(amountToDistribute).accounts({
      escrowAccount: escrowAccount.publicKey,
      senderTokenAccount: escrowTokenAccount,
      recipient: recipientTokenAccount,
      operator: operator.publicKey,
      tokenProgram: splToken.TOKEN_PROGRAM_ID,
    }).signers([operator]).rpc();

    // check recipient account balance
    const recipientAmount = await splToken.getAccount(provider.connection, recipientTokenAccount);
    assert.strictEqual(recipientAmount.amount, BigInt(amountToDistribute.toNumber())); // The balance of the recipient's account should equal the distributed amount

    // Verify that the escrow account balance decreased by amountToDistribute
    const escrowAccountBalance = await splToken.getAccount(provider.connection, escrowTokenAccount);
    assert.strictEqual(escrowAccountBalance.amount, initialEscrowBalance.amount - BigInt(amountToDistribute.toNumber())); // The balance of the escrow account decreased by amountToDistribute
  });

  it("Fails to Distribute SOL with Unauthorized Operator", async () => {
    const recipient = anchor.web3.Keypair.generate();
    const unauthorizedOperator = anchor.web3.Keypair.generate();

    try {
      await program.methods.distributeSol(amountToDistribute).accounts({
        sender: escrowAccount.publicKey,
        recipient: recipient.publicKey,
        escrowAccount: escrowAccount.publicKey,
        operator: unauthorizedOperator.publicKey,
        systemProgram: SystemProgram.programId,
      }).signers([unauthorizedOperator]).rpc();
      assert.fail("Should have thrown an error");
    } catch (err) {
      assert.include(err.toString(), "ConstraintHasOne");
    }
  });

  it("Fails to Distribute Tokens with Unauthorized Operator", async () => {
    const recipientAccount = anchor.web3.Keypair.generate();
    const recipientTokenAccount = await splToken.createAssociatedTokenAccount(
      provider.connection,
      payer,
      mint,
      recipientAccount.publicKey
    );
    const unauthorizedOperator = anchor.web3.Keypair.generate();

    try {
      await program.methods.distributeToken(amountToDistribute).accounts({
        senderTokenAccount: escrowTokenAccount,
        recipient: recipientTokenAccount,
        escrowAccount: escrowAccount.publicKey,
        operator: unauthorizedOperator.publicKey,
        tokenProgram: splToken.TOKEN_PROGRAM_ID,
      }).signers([unauthorizedOperator]).rpc();
      assert.fail("Should have thrown an error");
    } catch (err) {
      assert.include(err.toString(), "ConstraintHasOne");
    }
  });

  it("Fails to Distribute with Zero Amount", async () => {
    const recipient = anchor.web3.Keypair.generate();
    
    try {
      await program.methods.distributeSol(new anchor.BN(0)).accounts({
        sender: escrowAccount.publicKey,
        recipient: recipient.publicKey,
        escrowAccount: escrowAccount.publicKey,
        operator: operator.publicKey,
        systemProgram: SystemProgram.programId,
      }).signers([operator]).rpc();
      assert.fail("Should have thrown an error");
    } catch (err) {
      assert.include(err.toString(), "InvalidAmount");
    }
  });

  it("Fails to Distribute SOL to Invalid Recipient Address", async () => {
    const invalidRecipientAddress = "invalidAddress"; // Invalid address format

    try {
      await program.methods.distributeSol(new anchor.BN(1000)).accounts({
        sender: escrowAccount.publicKey,
        recipient: invalidRecipientAddress,
        escrowAccount: escrowAccount.publicKey,
        operator: operator.publicKey,
        systemProgram: SystemProgram.programId,
      }).signers([operator]).rpc();
      assert.fail("Should have thrown an error");
    } catch (err) {
      assert.include(err.toString(), "Non-base58 character"); // Adjust the error message as needed
    }
  });

  it("Fails to Distribute Tokens to Invalid Recipient Address", async () => {
    const invalidRecipientAddress = "invalidAddress"; // Invalid address format

    try {
      await program.methods.distributeToken(amountToDistribute).accounts({
        senderTokenAccount: escrowTokenAccount,
        recipient: invalidRecipientAddress,
        escrowAccount: escrowAccount.publicKey,
        operator: operator.publicKey,
        tokenProgram: splToken.TOKEN_PROGRAM_ID,
      }).signers([operator]).rpc();
      assert.fail("Should have thrown an error");
    } catch (err) {
      assert.include(err.toString(), "Non-base58 character"); // Adjust the error message as needed
    }
  });
});
