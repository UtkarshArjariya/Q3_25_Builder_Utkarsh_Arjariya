import wallet from "../Turbin3-wallet.json"
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults"
import { createGenericFile, createSignerFromKeypair, signerIdentity } from "@metaplex-foundation/umi"
import { irysUploader } from "@metaplex-foundation/umi-uploader-irys"

// Create a devnet connection
const umi = createUmi('https://api.devnet.solana.com');

let keypair = umi.eddsa.createKeypairFromSecretKey(new Uint8Array(wallet));
const signer = createSignerFromKeypair(umi, keypair);

umi.use(irysUploader());
umi.use(signerIdentity(signer));

(async () => {
    try {
        // Follow this JSON structure
        // https://docs.metaplex.com/programs/token-metadata/changelog/v1.0#json-structure

        const image = "https://gateway.irys.xyz/56jcuk6nA4Hhq96ztWaqV7ShEXwyd6aeizots1MHo3if";
        const metadata = {
            name: "Multi Color Generative Rug #1",
            symbol: "MCGNRUG1",
            description: "A multi color generative rug NFT",
            image,
            attributes: [
                { trait_type: '?', value: '?' }
            ],
            properties: {
                files: [
                    {
                        type: "image/png",
                        uri: "image"
                    },
                ]
            },
            creators: []
        };
        const myUri = await umi.uploader.uploadJson(metadata);
        console.log("Your metadata URI: ", myUri);
    }
    catch (error) {
        console.log("Oops.. Something went wrong", error);
    }
})();

//  Your metadata URI:  https://gateway.irys.xyz/3gJsftQwbmBn2271GLwJiYdAjMf5Mhw7rEz46ZcqxEwt