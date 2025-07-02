import { Keypair } from "@solana/web3.js";

let kp = Keypair.generate()
console.log(`You've generated a new Solana wallet: ${kp.publicKey.toBase58()}`)

console.log(`[${kp.secretKey}]`)

// You've generated a new Solana wallet: 5Nm99W6tj2ad9h8BvskDxt8yZqG7iiyexRwhheE5XFUi
