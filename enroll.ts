import {
    address,
    appendTransactionMessageInstructions,
    assertIsTransactionWithinSizeLimit,
    createKeyPairSignerFromBytes,
    createSolanaRpc,
    createSolanaRpcSubscriptions,
    createTransactionMessage,
    devnet,
    getSignatureFromTransaction,
    pipe,
    sendAndConfirmTransactionFactory,
    setTransactionMessageFeePayerSigner,
    setTransactionMessageLifetimeUsingBlockhash,
    signTransactionMessageWithSigners,
    addSignersToTransactionMessage,
    getProgramDerivedAddress,
    generateKeyPairSigner,
    getAddressEncoder
  } from "@solana/kit";
  
import { getInitializeInstruction, getSubmitTsInstruction } from "./clients/js/src/generated/index.js";
import wallet from "./Turbin3-wallet.json";
  
  const MPL_CORE_PROGRAM = address("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d");
  const PROGRAM_ADDRESS = address("TRBZyQHB3m68FGeVsqTK39Wm4xejadjVhP5MAZaKWDM");
  const SYSTEM_PROGRAM = address("11111111111111111111111111111111");
  const COLLECTION = address("5ebsp5RChCGK7ssRZMVMufgVZhd2kFbNaotcZ5UvytN2");
  
  const keypair = await createKeyPairSignerFromBytes(new Uint8Array(wallet));
  console.log(`Your Turbin3 wallet address: ${keypair.address}`);
  
  // Create devnet connection
  const rpc = createSolanaRpc(devnet("https://api.devnet.solana.com"));
  const rpcSubscriptions = createSolanaRpcSubscriptions(devnet('ws://api.devnet.solana.com'));
  
  const addressEncoder = getAddressEncoder();
  
  const accountSeeds = [Buffer.from("prereqs"), addressEncoder.encode(keypair.address)];
  const [account, _bump] = await getProgramDerivedAddress({
    programAddress: PROGRAM_ADDRESS,
    seeds: accountSeeds
  });
  
  console.log(`PDA account address: ${account}`);
  
  const authoritySeeds = [Buffer.from("collection"), addressEncoder.encode(COLLECTION)];
  const [authority, _authorityBump] = await getProgramDerivedAddress({
    programAddress: PROGRAM_ADDRESS,
    seeds: authoritySeeds
  });
  
  console.log(`Authority PDA: ${authority}`);
  
  // Generate mint keypair for the NFT
  const mintKeyPair = await generateKeyPairSigner();
  console.log(`Mint address: ${mintKeyPair.address}`);
  
  async function initialize() {
    const initializeIx = getInitializeInstruction({
      github: "ayyush1738",    
      user: keypair,
      account,
      systemProgram: SYSTEM_PROGRAM
    });
  
    const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
  
    const transactionMessageInit = pipe(
      createTransactionMessage({ version: 0 }),
      tx => setTransactionMessageFeePayerSigner(keypair, tx),
      tx => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
      tx => appendTransactionMessageInstructions([initializeIx], tx)
    );
  
    const signedTxInit = await signTransactionMessageWithSigners(transactionMessageInit);
    assertIsTransactionWithinSizeLimit(signedTxInit);
  
    const sendAndConfirmTransaction = sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions });
  
    try {
      await sendAndConfirmTransaction(
        signedTxInit,
        { commitment: 'confirmed', skipPreflight: false }
      );
      const signatureInit = getSignatureFromTransaction(signedTxInit);
      console.log(`Initialize Success! Check out your TX here:
  https://explorer.solana.com/tx/${signatureInit}?cluster=devnet`);
    } catch (e) {
      console.error(`Initialize failed: ${e}`);
    }
  }
  
  async function submitTs() {
    console.log('\n=== Account Addresses ===');
    console.log('User:', keypair.address);
    console.log('Account (PDA):', account);
    console.log('Mint:', mintKeyPair.address);
    console.log('Collection:', COLLECTION);
    console.log('Authority:', authority);
    console.log('MPL Core Program:', MPL_CORE_PROGRAM);
    console.log('System Program:', SYSTEM_PROGRAM);
    
    const submitIx = getSubmitTsInstruction({
      user: keypair,
      account,
      mint: mintKeyPair,
      collection: COLLECTION,
      authority,
      mplCoreProgram: MPL_CORE_PROGRAM,
      systemProgram: SYSTEM_PROGRAM
    });
  
    const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
  
    const transactionMessageSubmit = pipe(
      createTransactionMessage({ version: 0 }),
      tx => setTransactionMessageFeePayerSigner(keypair, tx),
      tx => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
      tx => appendTransactionMessageInstructions([submitIx], tx),
      tx => addSignersToTransactionMessage([mintKeyPair], tx)
    );
  
    const signedTxSubmit = await signTransactionMessageWithSigners(transactionMessageSubmit);
    assertIsTransactionWithinSizeLimit(signedTxSubmit);
  
    const sendAndConfirmTransaction = sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions });
  
    try {
      await sendAndConfirmTransaction(
        signedTxSubmit,
        { commitment: 'confirmed', skipPreflight: true }
      );
      const signatureSubmit = getSignatureFromTransaction(signedTxSubmit);
      console.log(`Submit TS Success! Check out your TX here:
  https://explorer.solana.com/tx/${signatureSubmit}?cluster=devnet`);
    } catch (e: any) {
      console.error(`Submit TS failed:`, e);
      if (e.cause?.logs) {
        console.error('Transaction logs:', e.cause.logs);
      }
    }
  }
  
  await initialize(); 
  await submitTs();