import {
    Connection, Keypair,
    PublicKey, sendAndConfirmTransaction,
    Transaction,
    TransactionInstruction
} from "@solana/web3.js";

import {struct,u8,u32,nu64,blob} from '@solana/buffer-layout';

// 定义参数结构体
interface ParamStruct {
    i: number,
    aa: number,
    bb: number,
    cc: number,
    dd: Uint8Array
}

/**
 * 多参数调用合约测试（使用Solana buffer-layout装配合约调用参数）
 * @param connection
 * @param keypair
 * @param programId
 */
const CMultiParamTest = async (connection: Connection, keypair: Keypair, programId: PublicKey): Promise<string> => {
    try {
        // 参数
        let param:ParamStruct = {
            i: 3,
            aa: 10211,
            bb: 233,
            cc: 1456161561546,
            dd: new TextEncoder().encode(new Date().getTime().toString())
        }
        console.info(`aaValue=${param.aa},bbValue=${param.bb},ccValue=${param.cc},ddValue=${new TextDecoder().decode(param.dd)}`);
        // 构建参数编码器（注意：顺序要和智能合约里面解析参数的顺序一致）
        let paramEncode = struct<ParamStruct>([
            u8('i'),
            u32('aa'),
            u8('bb'),
            nu64('cc'),
            blob(param.dd.length,"dd"),
        ]);
        // 初始化参数字节数组缓冲区
        const inputArray = Buffer.alloc(paramEncode.span);
        // 编码参数（将所有参数的值转换成一个字节数组）
        paramEncode.encode(param,inputArray);
        // 构建调用智能合约Instruction操作
        const instruction = new TransactionInstruction({
            // 调用只能合约所需要的账户（注意：账户的顺序要和合约里面的一致）
            keys: [{
                pubkey: keypair.publicKey,
                isSigner: true,
                isWritable: true
            }],
            // 调用智能合约的参数（就是input）
            data: inputArray,
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

export default CMultiParamTest;