import {
    Connection, Keypair,
    PublicKey, sendAndConfirmTransaction,
    Transaction,
    TransactionInstruction
} from "@solana/web3.js";

/**
 * 删除数据账户
 * @param connection
 * @param keypair
 * @param programId
 */
const CDeleteDataAccount = async (connection: Connection, keypair: Keypair, programId: PublicKey): Promise<string> => {
    try {
        /**
         * 我们生成PDA地址的时候就是用用户地址作为种子生成的，所以这里我们直接用这种方式拿到就可以了（注意：种子不变所生成的地址就不会变）
         */
        const [dataAccountKey, _bump] = PublicKey.findProgramAddressSync([keypair.publicKey.toBuffer()],programId);
        console.info(`Remove DataAccountKey: ${dataAccountKey}`);
        // 构建调用智能合约Instruction操作
        const instruction = new TransactionInstruction({
            // 调用只能合约所需要的账户（注意：账户的顺序要和合约里面的一致）
            keys: [{
                pubkey: keypair.publicKey,
                isSigner: true,
                isWritable: true
            },{
                pubkey: dataAccountKey,
                isSigner: false,
                isWritable: true
            }],
            // 调用智能合约的参数（就是input）
            data: Buffer.from(new Uint8Array([2])),
            // 合约地址
            programId: programId
        });
        // 构建交易
        let transaction = new Transaction().add(instruction);
        const signature = await sendAndConfirmTransaction(connection, transaction, [keypair]);
        return signature;
    } catch(error) {
        console.error(error);
        // @ts-ignore
        throw new Error(error.message);
    }
}

export default CDeleteDataAccount;