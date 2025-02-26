// import * as anchor from '@coral-xyz/anchor';
// import { PublicKey } from '@solana/web3.js';
// import * as splToken from '@solana/spl-token';
// import { assert } from 'chai';

// describe('usdt-transfer', () => {
//   const provider = anchor.AnchorProvider.env();
//   anchor.setProvider(provider);

//   const recipientAccount = anchor.web3.Keypair.generate();
//   const sender = (provider.wallet as anchor.Wallet).payer;

//   let mint: PublicKey;
//   let senderTokenAccount: anchor.web3.PublicKey;
//   let recipientTokenAccount: anchor.web3.PublicKey;

//   it('Creates USDT mint and token accounts', async () => {
//     // USDT mint to sender
//     mint = await splToken.createMint(
//       provider.connection,
//       sender,
//       provider.wallet.publicKey,
//       null,
//       6,
//     );

//     // Create token accounts for senders and receivers
//     senderTokenAccount = await splToken.createAssociatedTokenAccount(
//       provider.connection,
//       sender,
//       mint,
//       provider.wallet.publicKey
//     );

//     recipientTokenAccount = await splToken.createAssociatedTokenAccount(
//       provider.connection,
//       sender,
//       mint,
//       recipientAccount.publicKey
//     );

//   });

//   it('Transfers USDT successfully', async () => {
//     // Mint some USDT to the sender's account
//     await splToken.mintTo(
//       provider.connection,
//       sender,
//       mint,
//       senderTokenAccount,
//       sender,
//       1000
//     );

//     // Use sdk directly to transfer money
//     await splToken.transfer(
//       provider.connection,
//       sender,
//       senderTokenAccount,
//       recipientTokenAccount,
//       sender,
//       100
//     );

//     // Get account information and verify the transfer
//     const senderAccount = await splToken.getAccount(provider.connection, senderTokenAccount);
//     const recipientAccount = await splToken.getAccount(provider.connection, recipientTokenAccount);

//     assert.strictEqual(senderAccount.amount, BigInt(900)); // The balance of the sender's account decreased by 100
//     assert.strictEqual(recipientAccount.amount, BigInt(100)); // The balance of the recipient's account increased by 100
//   });

// });
