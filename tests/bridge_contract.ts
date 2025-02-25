import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { BridgeContract } from "../target/types/bridge_contract";
import { Keypair, PublicKey, SystemProgram } from '@solana/web3.js';
import * as splToken from '@solana/spl-token';
import { assert, expect } from "chai";

describe("bridge_contract", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.BridgeContract as Program<BridgeContract>;

  let mint: PublicKey;
  let payerTokenAccount: PublicKey;
  const payer = (provider.wallet as anchor.Wallet).payer;
  const operator = anchor.web3.Keypair.generate();
  const escrowAccount = Keypair.generate();
  const amountToDistribute = new anchor.BN(1);

  it("Is initialized!", async () => {
    mint = await splToken.createMint(
      provider.connection,
      payer,
      provider.wallet.publicKey,
      null,
      6
    );

    payerTokenAccount = await splToken.createAssociatedTokenAccount(
      provider.connection,
      payer,
      mint,
      provider.wallet.publicKey
    );

    await splToken.mintTo(
      provider.connection,
      payer,
      mint,
      payerTokenAccount,
      payer,
      1000
    );

    // Call the initialize method
    const tx = await program.methods.initialize(operator.publicKey).accounts({
      escrowAccount: escrowAccount.publicKey,
      authority: payer.publicKey,
      systemProgram: SystemProgram.programId,
    }).signers([escrowAccount, payer]).rpc();
    console.log("initialize transaction signature", tx);

    // Add further checks to verify initialization (e.g., check the state)
  });

  it("Distributes SOL!", async () => {
    const recipient = anchor.web3.Keypair.generate();
    
    const initialBalance = await program.provider.connection.getBalance(recipient.publicKey);

    const tx = await program.methods.distributeSol(amountToDistribute).accounts({
      sender: payer.publicKey,
      recipient: recipient.publicKey,
      escrowAccount: escrowAccount.publicKey,
      operator: operator.publicKey,
      systemProgram: SystemProgram.programId,
    }).signers([operator]).rpc();
    console.log("Distribute SOL transaction signature", tx);

    const finalBalance = await program.provider.connection.getBalance(recipient.publicKey);
    console.log("Recipient final balance", finalBalance);

    expect(finalBalance).to.be.greaterThan(initialBalance);
  });

  it("Distributes Tokens!", async () => {
    const recipientAccount = anchor.web3.Keypair.generate();
    const recipientTokenAccount = await splToken.createAssociatedTokenAccount(
      provider.connection,
      payer,
      mint,
      recipientAccount.publicKey
    );

    const approveTx = await program.methods.authorizeOperatorOnce().accounts({
      sender: payerTokenAccount,
      senderAuthority: payer.publicKey,
      escrowAccount: escrowAccount.publicKey,
      operator: operator.publicKey,
      tokenProgram: splToken.TOKEN_PROGRAM_ID, 
    }).signers([payer, operator]).rpc();
    console.log("authorizeOperatorOnce transaction signature", approveTx);

    const distributeTokenTx = await program.methods.distributeToken(amountToDistribute).accounts({
      sender: payerTokenAccount,
      recipient: recipientTokenAccount,
      escrowAccount: escrowAccount.publicKey,
      operator: operator.publicKey,
      tokenProgram: splToken.TOKEN_PROGRAM_ID,
    }).signers([operator]).rpc();
    console.log("Distribute Token transaction signature", distributeTokenTx);

    const recipientAmount = await splToken.getAccount(provider.connection, recipientTokenAccount);
    assert.strictEqual(recipientAmount.amount, BigInt(amountToDistribute.toNumber()));

    // Add checks here to verify token transfer
  });
});
