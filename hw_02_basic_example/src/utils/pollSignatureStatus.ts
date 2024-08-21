import {Connection} from '@solana/web3.js';
import {TLog} from '../types';

// 交易查询时间间隔（单位毫秒）
const POLLING_INTERVAL = 1000;
// 交易确认最大时间（单位秒，交易在规定时间内未确认前端直接显示失败）
const MAX_POLLS = 30;

/**
 * 交易状态查询
 * @param signature
 * @param connection
 * @param createLog
 */
const pollSignatureStatus = async(signature: string,connection: Connection,createLog: (log: TLog) => void,method: any): Promise<void> => {
    let count = 0;
    const interval = setInterval(async () => {
        if(count === MAX_POLLS) {
            clearInterval(interval);
            createLog({
                status: 'error',
                method: method,
                message: `Transaction: ${signature}`,
                messageTwo: `交易未能在 ${MAX_POLLS} 秒内确认交易. 交易可能成功，也可能失败.`,
            });
            return;
        }
        // 查询交易状态
        const {value} = await connection.getSignatureStatus(signature);
        // 交易状态
        const confirmationStatus = value?.confirmationStatus;
        if(confirmationStatus) {
            // 交易状态为已提交或已完成，该值则为true
            const hasReachedSufficientCommitment = confirmationStatus === 'confirmed' || confirmationStatus === 'finalized';
            createLog({
                status: hasReachedSufficientCommitment ? 'success' : 'info',
                method: method,
                message: `Transaction ${signature}`,
                messageTwo: `交易状态: ${confirmationStatus.charAt(0).toUpperCase() + confirmationStatus.slice(1)}`,
            });
            // 交易状态为已提交或已完成
            if(hasReachedSufficientCommitment) {
                clearInterval(interval);
                return;
            }
        } else {
            createLog({
                status: 'info',
                method: method,
                message: `Transaction ${signature}`,
                messageTwo: '交易状态：等待确认...',
            });
        }
        count++;
    },POLLING_INTERVAL);
}

export default pollSignatureStatus;