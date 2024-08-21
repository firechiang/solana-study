import {
    Connection,
    Keypair,
    PublicKey,
} from "@solana/web3.js";

import { cstr, u8, struct } from "@solana/buffer-layout";
import { publicKey } from "@solana/buffer-layout-utils";

// 定义账户数据结构体
interface DataStruct {
    accountKey: PublicKey,
    state: number,
    dataLen: number,
    data: String
}
// 构建账户数据结构体解码器（注意：顺序要和智能合约里面编码结构体的顺序一致）
const DataStructDecode = struct<DataStruct>([
    publicKey('accountKey'),
    u8('state'),
    u8('dataLen'),
    cstr('data')
]);

/**
 * 查询账户数据
 * @param connection
 * @param keypair
 * @param programId
 */
const CQueryDataAccount = async (connection: Connection, keypair: Keypair, programId: PublicKey): Promise<string> => {
    try {
        // 数据账户地址（我们创建数据账户的时候就是用这种方式生成的地址）
        const [pdaAccountKey, _bump] = PublicKey.findProgramAddressSync([keypair.publicKey.toBuffer()],programId);
        console.info(`Query DataAccountKey: ${pdaAccountKey}`);
        // 查询数据对象
        let accountInfo = await connection.getAccountInfo(pdaAccountKey);
        // 解码数据
        let dataInfo = DataStructDecode.decode(accountInfo.data);
        // @ts-ignore
        return `account_key: ${dataInfo.accountKey.toBase58()},state: ${dataInfo.state},data_len; ${dataInfo.dataLen},data: ${dataInfo.data}`;
    } catch(error) {
        console.error(error);
        // @ts-ignore
        throw new Error(error.message);
    }
}

export default CQueryDataAccount;