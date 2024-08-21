import {Transaction} from '@solana/web3.js';
import {PhantomProvider} from '../types';

/**
 * Phantom签名交易
 * @param provider
 * @param transaction
 */
const phantomSignTransaction = async (provider: PhantomProvider,transaction: Transaction):Promise<Transaction> => {
    try{
        // 签名交易
        const signTransaction = await provider.signTransaction(transaction);
        // 签名多个交易
        //const transactions = await provider.signAllTransactions([transaction1,transaction2]);
        // 使用密钥继续签名
        //signTransaction.partialSign(keypair);
        return signTransaction;
    }catch(error) {
        console.warn(error);
        // @ts-ignore
        throw new Error(error.message);
    }
}

export default phantomSignTransaction;