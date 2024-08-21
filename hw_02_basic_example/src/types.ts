import {PublicKey,Transaction,SendOptions} from '@solana/web3.js';

// 编码类型
type DisplayEncodeing = 'utf8' | 'hex';

// Phantom相关事件类型
type PhantomEvent = 'connect' | 'disconnect' | 'accountChanged';

// Phantom请求函数类型
type PhantomRequestMethod =
    | 'connect'
    | 'disconnect'
    | 'phantomCreateDataAccount'
    | 'phantomSignMessage'
    | 'createDataAccount'
    | 'updateDataAccount'
    | 'deleteDataAccount'
    | 'queryDataAccount'
    | 'multiParamTest';

interface ConnectOpts {
    onlyIfTrusted: boolean;
}

// 与Phantom相关交互定义
export interface PhantomProvider {
    publicKey: PublicKey | null;
    isConnected: boolean | null;
    signAndSendTransaction: (transaction: Transaction, opts?: SendOptions) => Promise<{ signature: string; publicKey: PublicKey }>;
    signTransaction: (transaction: Transaction) => Promise<Transaction>;
    signAllTransactions: (transactions: Transaction[]) => Promise<Transaction[]>;
    signMessage: (message: Uint8Array | string, display?: DisplayEncodeing) => Promise<any>;
    connect: (opts?: Partial<ConnectOpts>) => Promise<{ publicKey: PublicKey }>;
    disconnect: () => Promise<void>;
    on: (event: PhantomEvent, handler: (args: any) => void) => void;
    request: (method: PhantomRequestMethod, params: any) => Promise<unknown>;
}

// 日志级别类型
export type Status = 'success' | 'warning' | 'error' | 'info';

// 日志相关定义
export interface TLog {
    status: Status;
    method?: PhantomRequestMethod | Extract<PhantomEvent, 'accountChanged'>;
    message: string;
    messageTwo?: string;
}