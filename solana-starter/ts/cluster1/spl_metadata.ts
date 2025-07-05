import wallet from "../Turbin3-wallet.json"
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults"
import {
    createMetadataAccountV3,
    CreateMetadataAccountV3InstructionAccounts,
    CreateMetadataAccountV3InstructionArgs,
    DataV2Args
} from "@metaplex-foundation/mpl-token-metadata";
import { createSignerFromKeypair, signerIdentity, publicKey } from "@metaplex-foundation/umi";
import { bs58 } from "@coral-xyz/anchor/dist/cjs/utils/bytes";

// Define our Mint address
const mint = publicKey("4YkAEwtuKLEsDzhT8pn6uhvZxiZcy63VSdsw46t9vY6d")

// Create a UMI connection
const umi = createUmi('https://api.devnet.solana.com');
const keypair = umi.eddsa.createKeypairFromSecretKey(new Uint8Array(wallet));
const signer = createSignerFromKeypair(umi, keypair);
umi.use(signerIdentity(createSignerFromKeypair(umi, keypair)));

(async () => {
    try {
        // Start here
        let accounts: CreateMetadataAccountV3InstructionAccounts = {
            mint,
            mintAuthority: signer,
            updateAuthority: signer,
        }

        let data: DataV2Args = {
            name: "Q3_25_Turbin3_UA",
            symbol: "Q325UA",
            uri: "https://pbs.twimg.com/profile_images/1767222792628383744/pTv9nIHC_400x400.jpg",
            sellerFeeBasisPoints: 10,
            creators: null,
            collection: null,
            uses: null
        }

        let args: CreateMetadataAccountV3InstructionArgs = {
            data,
            isMutable: true,
            collectionDetails: null
        }

        let tx = createMetadataAccountV3(
            umi,
            {
                ...accounts,
                ...args
            }
        )

        let result = await tx.sendAndConfirm(umi);
        console.log(bs58.encode(result.signature));
        console.log(`Succesfully created metadata account! Check out your TX here:\nhttps://explorer.solana.com/tx/${bs58.encode(result.signature)}?cluster=devnet`);
    } catch (e) {
        console.error(`Oops, something went wrong: ${e}`)
    }
})();


// https://explorer.solana.com/tx/3bMNEuSUTQsAsKidn5PNatdua6rHCEK8dbe4R2Cww5GjAkEfjjtc4MBRjfDETBR4bnmLan4aGe2Qgb6ZKkQ9SDX8?cluster=devnet