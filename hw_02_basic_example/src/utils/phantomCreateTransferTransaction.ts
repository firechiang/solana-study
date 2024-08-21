import {Transaction,SystemProgram,Connection,PublicKey} from '@solana/web3.js';

/**
 * 创建Phantom使用的转帐交易
 * @param publicKey
 * @param connection
 */
const phantomCreateTransferTransaction = async (publicKey: PublicKey, connection: Connection):Promise<Transaction> => {
    const transaction = new Transaction().add(
        SystemProgram.transfer({
            fromPubkey: publicKey,
            toPubkey: SystemProgram.programId,
            lamports: 100
        })
    );
    // gas费交易地址
    transaction.feePayer = publicKey;
    const anyTransaction: any = transaction;
    // 获取交易Hash
    anyTransaction.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;

    return transaction;
}

export default phantomCreateTransferTransaction;