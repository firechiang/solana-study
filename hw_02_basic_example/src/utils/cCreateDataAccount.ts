import {
    Connection,
    Keypair,
    PublicKey,
    sendAndConfirmTransaction,
    SystemProgram,
    Transaction,
    TransactionInstruction
} from "@solana/web3.js";

/**
 * 创建数据账户
 * @param connection
 * @param keypair
 * @param programId
 */
const CCreateDataAccount = async (connection: Connection, keypair: Keypair, programId: PublicKey): Promise<string> => {
    try {
        /**
         * 创建数据账户地址（注意：这个就是创建PDA地址，但是数据账户是我们在合约里面创建的而不是直接在前端创建的，因为PDA地址不能直接在前端创建数据账户，原因是它没有私钥不能签名）
         * 说明：因为我们的程序只需要用户创建一个数据账户，所以直接使用用户地址作为种子创建PDA地址。
         * 如果程序是需要用户创建多个数据账户，那么每个PDA地址的种子是需要不一样的，可根据每个数据账户的作用来定。
         * 比如第一个数据账户的种子：[Buffer.from('1')]
         */
        const [pdaAccountKey, _bump] = PublicKey.findProgramAddressSync([keypair.publicKey.toBuffer()],programId);
        console.info(`Create DataAccountKey: ${pdaAccountKey}`);
        // 构建调用智能合约Instruction操作
        const instruction = new TransactionInstruction({
            // 调用只能合约所需要的账户（注意：账户的顺序要和合约里面的一致）
            keys: [{
                pubkey: keypair.publicKey,
                isSigner: true,
                isWritable: true
            },{
                pubkey: pdaAccountKey,
                isSigner: false,
                isWritable: true
            },{
                pubkey: SystemProgram.programId,
                isSigner: false,
                isWritable: false
            }],
            // 调用智能合约的参数（就是input）
            data: Buffer.from(new Uint8Array([0])),
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

export default CCreateDataAccount;