import {
    Connection, Keypair,
    PublicKey, sendAndConfirmTransaction,
    Transaction,
    TransactionInstruction
} from "@solana/web3.js";

/**
 * 修改数据账户的数据（注意：构建调用合约的参数不要使用下面这种方式，太基础了，建议使用buffer-layout，示例代码可查看 cMultiParamTest.ts 文件）
 * @param connection
 * @param keypair
 * @param programId
 */
const CUpdateDataAccount = async (connection: Connection, keypair: Keypair, programId: PublicKey): Promise<string> => {
    try {
        /**
         * 我们生成PDA地址的时候就是用用户地址作为种子生成的，所以这里我们直接用这种方式拿到就可以了（注意：种子不变所生成的地址就不会变）
         */
        const [dataAccountKey, _bump] = PublicKey.findProgramAddressSync([keypair.publicKey.toBuffer()],programId);
        console.info(`Update DataAccountKey: ${dataAccountKey}`);
        let dataStr = new Date().getTime().toString();
        console.info(`Update dataStr = ${dataStr}`);
        // 第一个字节是1表示是修改数据账户的数据（合约里面定义的规则）
        let firstBitArray = new Uint8Array([1]);
        // 数据账户的数据
        let dataArray = new TextEncoder().encode(dataStr);
        // 合并两个字节数组（注意：构建调用合约的参数不要使用下面这种方式，太基础了，建议使用buffer-layout，示例代码可查看 cMultiParamTest.ts 文件）
        let inputArray = new Uint8Array(firstBitArray.length + dataArray.length);
        // 从开始位置往数组里面填充 firstBitArray 数据（注意：set函数如果不传数据填充的开始位置，默认从第0个位置开始填充）
        inputArray.set(firstBitArray);
        // 从 firstBitArray.length 位置开始往数组里面填充 dataArray 数据
        inputArray.set(dataArray,firstBitArray.length);
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
            data: Buffer.from(inputArray),
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

export default CUpdateDataAccount;