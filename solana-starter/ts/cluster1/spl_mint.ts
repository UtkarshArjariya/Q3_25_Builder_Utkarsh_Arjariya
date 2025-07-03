import { Keypair, PublicKey, Connection, Commitment } from "@solana/web3.js";
import { getOrCreateAssociatedTokenAccount, mintTo } from '@solana/spl-token';
import wallet from "../Turbin3-wallet.json"

// Import our keypair from the wallet file
const keypair = Keypair.fromSecretKey(new Uint8Array(wallet));

//Create a Solana devnet connection
const commitment: Commitment = "confirmed";
const connection = new Connection("https://api.devnet.solana.com", commitment);

const token_decimals = 1_000_000n;

// Mint address
const mint = new PublicKey("4YkAEwtuKLEsDzhT8pn6uhvZxiZcy63VSdsw46t9vY6d");

(async () => {
    try {
        // Create an ATA
        const ata = await getOrCreateAssociatedTokenAccount(
            connection,
            keypair,
            mint,
            keypair.publicKey
        );
        console.log(`Your ata is: ${ata.address.toBase58()}`);

        // Mint to ATA
        const mintTx = await mintTo(
            connection,
            keypair,
            mint,
            ata.address,
            keypair.publicKey,
            5n * token_decimals
        );
        console.log(`Your mint txid: ${mintTx}`);
    } catch (error) {
        console.log(`Oops, something went wrong: ${error}`)
    }
})()

//  Your ata is: CYTKByVPRTpwR8RBdoYUX5kYtVHNuMiAUhRcKCRNPtN1
//  Your mint txid: 3TeDA1PRKa7e86BKppZaDNgRarDsXofGER6mKsEy6A54SbJK9MUpZMyqT1sRiMFSP61KgpPLbarKXH2VBKwaieK1