import { createUmi } from "@metaplex-foundation/umi-bundle-defaults"
import { createSignerFromKeypair, signerIdentity, generateSigner, percentAmount } from "@metaplex-foundation/umi"
import { createNft, mplTokenMetadata } from "@metaplex-foundation/mpl-token-metadata";

import wallet from "../Turbin3-wallet.json"
import base58 from "bs58";

const RPC_ENDPOINT = "https://api.devnet.solana.com";
const umi = createUmi(RPC_ENDPOINT);

let keypair = umi.eddsa.createKeypairFromSecretKey(new Uint8Array(wallet));
const myKeypairSigner = createSignerFromKeypair(umi, keypair);
umi.use(signerIdentity(myKeypairSigner));
umi.use(mplTokenMetadata())

const mint = generateSigner(umi);

(async () => {
    let tx = createNft(umi, {
        mint,
        name: "Multi Color Generative Rug #1",
        symbol: "MCGNRUG1",
        uri: "https://gateway.irys.xyz/3gJsftQwbmBn2271GLwJiYdAjMf5Mhw7rEz46ZcqxEwt",
        sellerFeeBasisPoints: percentAmount(5)
    })
    let result = await tx.sendAndConfirm(umi);
    const signature = base58.encode(result.signature);

    console.log(`Succesfully Minted! Check out your TX here:\nhttps://explorer.solana.com/tx/${signature}?cluster=devnet`)

    console.log("Mint Address: ", mint.publicKey);
})();

// Succesfully Minted! Check out your TX here:
// https://explorer.solana.com/tx/4t5pceHmTzVtrqhaFYAair6kP9pZ8gt3qCZ65uf6k1So4oHbAkLHM1iVzjVRjsQ16SK2mYwB3JDzqyhoZ4UztZ78?cluster=devnet
// Mint Address:  CvhPLk7Hr21ERryydWVXABAoR6UhiSMumGDzXAp3p4sE