import React, { useState, useEffect, useCallback, useMemo } from 'react';
import styled from '@emotion/styled';
import {clusterApiUrl, Connection, Keypair, PublicKey} from '@solana/web3.js';

import { TLog } from './types';
import { Logs, Sidebar, NoProvider } from './components';

import {
    phantomGetProvider,
    phantomSignMessage,
    phantomCreateDataAccount,
    pollSignatureStatus,
    createDataAccount,
    updateDataAccount,
    deleteDataAccount,
    queryDataAccount,
    multiParamTest
} from './utils';


const fromB = Keypair.generate();
const fromS = btoa(String.fromCharCode.apply(null, fromB.secretKey));
console.info(fromS);
// 修改 fromS
const fromA = Uint8Array.from(atob(fromS), (m) => m.codePointAt(0));
const fromK = Keypair.fromSecretKey(fromA);
console.info(`From Key: ${fromK.publicKey}`);


const remoteNetwork = clusterApiUrl('devnet');
const localhostNetwork = 'http://localhost:8899';
const provider = phantomGetProvider();
const connection = new Connection(localhostNetwork);
const programId = new PublicKey('9XSoH2btchBESWhABvjYRRTp7vovAkkiNMQvV9c7BT2C');

// 自定义类型
export type ConnectedMethods = | { name: string; onClick: () => Promise<string>; } | { name: string; onClick: () => Promise<void>; };

interface Props {
    publicKey: PublicKey | null;
    connectedMethods: ConnectedMethods[];
    handleConnect: () => Promise<void>;
    logs: TLog[];
    clearLogs: () => void;
};

/**
 * 自定义Hooks（包含程序所有处理逻辑）
 */
const useProps = (): Props => {
    // 日志数据状态管理
    const [logs, setLogs] = useState<TLog[]>([]);

    // 控制日志数据生成
    const createLog = useCallback((log: TLog) => {
        // 注意：set函数在执行时如果发现传递给它的是函数，它会将现有值传递给该函数并执行该函数获取新值
        setLogs((logs) => [...logs, log]);
    },[setLogs]);

    // 清空日志数据
    const clearLogs = useCallback(() => {
        setLogs([]);
    }, [setLogs]);

    // 绑定相关事件（每次日志发生变化重新绑定）
    useEffect(() => {
        if (!provider) return;

        // 监听Phantom钱包connect连接事件
        provider.on('connect', (publicKey: PublicKey) => {
            createLog({
                status: 'success',
                method: 'connect',
                message: `Connected to account ${publicKey.toBase58()}`,
            });
        });

        // 监听Phantom钱包disconnect断开连接事件
        provider.on('disconnect', () => {
            createLog({
                status: 'warning',
                method: 'disconnect',
                message: '👋',
            });
        });

        // 监听Phantom钱包accountChanged切换账户事件
        provider.on('accountChanged', (publicKey: PublicKey | null) => {
            // 已连接Phantom钱包
            if (publicKey) {
                createLog({
                    status: 'info',
                    method: 'accountChanged',
                    message: `Switched to account ${publicKey.toBase58()}`,
                });

            // 未连接Phantom钱包
            } else {
                // 打印日志
                createLog({
                    status: 'info',
                    method: 'accountChanged',
                    message: 'Attempting to switch accounts.',
                });
                // 重新连接Phantom钱包
                provider.connect().catch((error) => {
                    createLog({
                        status: 'error',
                        method: 'accountChanged',
                        message: `Failed to re-connect: ${error.message}`,
                    });
                });
            }
        });

        return () => {
            provider.disconnect();
        };

    }, [createLog]);

    // 通过Phantom签名提交交易创建数据账户
    const handlePhantomCreateDataAccount = useCallback(async () => {
        // 没有获取到Phantom插件实例
        if (!provider) return;
        try {
            createLog({
                status: 'info',
                method: 'phantomCreateDataAccount',
                message: 'Phantom创建数据账户交易提交中...',
            });
            const signature = await phantomCreateDataAccount(connection,provider,programId);
            createLog({
                status: 'info',
                method: 'phantomCreateDataAccount',
                message: `交易Hash: ${signature}.`,
            });
            // 交易状态查询
            pollSignatureStatus(signature, connection, createLog,'createDataAccountByPhantom');
        } catch(error) {
            createLog({
                status: 'error',
                method: 'phantomCreateDataAccount',
                message: error.message,
            });
        }
    },[createLog]);

    // 创建数据账户
    const handleCreateDataAccount = useCallback(async () => {
        try {
            createLog({
                status: 'info',
                method: 'createDataAccount',
                message: '创建数据账户交易提交中...',
            });
            const signature = await createDataAccount(connection,fromK,programId);
            createLog({
                status: 'info',
                method: 'createDataAccount',
                message: `交易Hash: ${signature}.`,
            });
            // 交易状态查询
            pollSignatureStatus(signature, connection, createLog,'createDataAccount');
        } catch(error) {
            createLog({
                status: 'error',
                method: 'createDataAccount',
                message: error.message,
            });
        }
    },[createLog]);


    // 修改账户数据
    const handleUpdateDataAccount = useCallback(async () => {
        try {
            createLog({
                status: 'info',
                method: 'updateDataAccount',
                message: '修改账户数据交易提交中...',
            });
            const signature = await updateDataAccount(connection,fromK,programId);
            createLog({
                status: 'info',
                method: 'updateDataAccount',
                message: `交易Hash: ${signature}.`,
            });
            // 交易状态查询
            pollSignatureStatus(signature, connection, createLog,'updateDataAccount');
        } catch(error) {
            createLog({
                status: 'error',
                method: 'updateDataAccount',
                message: error.message,
            });
        }
    },[createLog]);

    // 删除数据账户
    const handleDeleteDataAccount = useCallback(async () => {
        try {
            createLog({
                status: 'info',
                method: 'deleteDataAccount',
                message: '删除数据账户交易提交中...',
            });
            const signature = await deleteDataAccount(connection,fromK,programId);
            createLog({
                status: 'info',
                method: 'deleteDataAccount',
                message: `交易Hash: ${signature}.`,
            });
            // 交易状态查询
            pollSignatureStatus(signature, connection, createLog,'deleteDataAccount');
        } catch(error) {
            createLog({
                status: 'error',
                method: 'deleteDataAccount',
                message: error.message,
            });
        }
    },[createLog]);

    // 查询数据账户
    const handleQueryDataAccount = useCallback(async () => {
        try {
            const res = await queryDataAccount(connection,fromK,programId);
            createLog({
                status: 'info',
                method: 'queryDataAccount',
                message: `查询到数据账户: ${res}.`,
            });
        } catch(error) {
            createLog({
                status: 'error',
                method: 'queryDataAccount',
                message: error.message,
            });
        }
    },[createLog]);

    // 多个参数传递测试
    const handleMultiParamTest = useCallback(async () => {
        try {
            createLog({
                status: 'info',
                method: 'multiParamTest',
                message: '多个参数传递交易提交中...',
            });
            const signature = await multiParamTest(connection,fromK,programId);
            createLog({
                status: 'info',
                method: 'multiParamTest',
                message: `交易Hash: ${signature}.`,
            });
            // 交易状态查询
            pollSignatureStatus(signature, connection, createLog,'multiParamTest');
        } catch(error) {
            createLog({
                status: 'error',
                method: 'multiParamTest',
                message: error.message,
            });
        }
    },[createLog]);


    // Phantom处理消息签名
    const handlePhantomSignMessage = useCallback(async () => {
        if (!provider) return;

        try {
            const message = new Date().getTime().toString();
            // 签名消息
            const signedMessage = await phantomSignMessage(provider, message);
            createLog({
                status: 'success',
                method: 'phantomSignMessage',
                message: `Message signed: ${JSON.stringify(signedMessage)}`,
            });
            // 返回签名
            return signedMessage;
        } catch (error) {
            createLog({
                status: 'error',
                method: 'phantomSignMessage',
                message: error.message,
            });
        }
    }, [createLog]);

    // 处理连接Phantom钱包
    const handleConnect = useCallback(async () => {
        if (!provider) return;

        try {
            // 连接Phantom钱包
            await provider.connect();
        } catch (error) {
            createLog({
                status: 'error',
                method: 'connect',
                message: error.message,
            });
        }
    }, [createLog]);

    // 处理断开连接Phantom钱包
    const handlePhantomDisconnect = useCallback(async () => {
        if (!provider) return;
        try {
            await provider.disconnect();
        } catch (error) {
            createLog({
                status: 'error',
                method: 'disconnect',
                message: error.message,
            });
        }
    }, [createLog]);

    // 定义相关操作
    const connectedMethods = useMemo(() => {
        return [{
            name: '创建数据账户',
            onClick: handleCreateDataAccount,
        },{
            name: '修改账户数据',
            onClick: handleUpdateDataAccount,
        },{
            name: '查询账户数据',
            onClick: handleQueryDataAccount,
        },{
            name: '删除数据账户',
            onClick: handleDeleteDataAccount,
        },{
            name: '多个参数传递测试',
            onClick: handleMultiParamTest,
        },{
            name: 'Phantom创建数据账户',
            onClick: handlePhantomCreateDataAccount,
        },{
            name: 'Phantom签名数据',
            onClick: handlePhantomSignMessage,
        },{
            name: 'Phantom断开连接',
            onClick: handlePhantomDisconnect,
        }];
    },[
        handleCreateDataAccount,
        handleUpdateDataAccount,
        handleDeleteDataAccount,
        handlePhantomCreateDataAccount,
        handlePhantomSignMessage,
        handlePhantomDisconnect,
    ]);

    return {
        // @ts-ignore
        publicKey: provider?.publicKey || null,
        connectedMethods,
        handleConnect,
        logs,
        clearLogs,
    };
};


// 主界面组件
const StatelessApp = React.memo((props: Props) => {
    const { publicKey, connectedMethods, handleConnect, logs, clearLogs } = props;
    return (
        <StyledApp>
            <Sidebar publicKey={publicKey} connectedMethods={connectedMethods} connect={handleConnect} />
            <Logs publicKey={publicKey} logs={logs} clearLogs={clearLogs} />
        </StyledApp>
    );
});


// 程序入口组件
const App = () => {
    const props = useProps();

    if (!provider) {
        return <NoProvider />;
    }

    return <StatelessApp {...props} />;
};

export default App;

// 主界面样式
const StyledApp = styled.div`
    display: flex;
    flex-direction: row;
    height: 100vh;
    @media (max-width: 768px) {
        flex-direction: column;
    }
`;