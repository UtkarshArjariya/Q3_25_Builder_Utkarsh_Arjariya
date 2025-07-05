import { Commitment, Connection, Keypair, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js"
import wallet from "../Turbin3-wallet.json"
import { getOrCreateAssociatedTokenAccount, transfer } from "@solana/spl-token";

// We're going to import our keypair from the wallet file
const keypair = Keypair.fromSecretKey(new Uint8Array(wallet));

//Create a Solana devnet connection
const commitment: Commitment = "confirmed";
const connection = new Connection("https://api.devnet.solana.com", commitment);

// Mint address
const mint = new PublicKey("4YkAEwtuKLEsDzhT8pn6uhvZxiZcy63VSdsw46t9vY6d");

// Recipient address
const to = new PublicKey("A6NoSTXvx2xdHSMtbHbLY55rg7hz4dKZUgKfVtknWAsc");

(async () => {
    try {
        // Get the token account of the fromWallet address, and if it does not exist, create it
        const fromATA = await getOrCreateAssociatedTokenAccount(
            connection,
            keypair,
            mint,
            keypair.publicKey
        );

        // Get the token account of the toWallet address, and if it does not exist, create it
        const toATA = await getOrCreateAssociatedTokenAccount(
            connection,
            keypair,
            mint,
            to
        );

        // Transfer the new token to the "toTokenAccount" we just created
        const txSign = await transfer(
            connection,
            keypair,
            fromATA.address,
            toATA.address,
            keypair,
            1 * LAMPORTS_PER_SOL
        );

        console.log(`Succesfully transferred! Check out your TX here:\nhttps://explorer.solana.com/tx/${txSign}?cluster=devnet`);
        console.log(`From: ${fromATA.address.toBase58()}`);
        console.log(`To: ${toATA.address.toBase58()}`);
    } catch (e) {
        console.error(`Oops, something went wrong: ${e}`)
    }
})();


// Succesfully transferred! Check out your TX here:
// https://explorer.solana.com/tx/76bxeMZ8VPLjsWrQaZQ4x97bq52sG29mpA6dVWh1QGwCta3bpijZWobYez2DFaFCvfbQLrproVUrpTbcAwAsE2X?cluster=devnet
// From: CYTKByVPRTpwR8RBdoYUX5kYtVHNuMiAUhRcKCRNPtN1
// To: CYTKByVPRTpwR8RBdoYUX5kYtVHNuMiAUhRcKCRNPtN1