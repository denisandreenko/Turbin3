import bs58 from "bs58";

function base58ToWallet(base58String: string): string {
    const bytes = bs58.decode(base58String);
    console.log(bytes);
    return Buffer.from(bytes).toString('hex');
}
