import {PhantomProvider} from '../types'

/**
 * Phantom消息签名
 * @param provider
 * @param message
 */
const phantomSignMessage = async (provider: PhantomProvider, message: string): Promise<string> => {
    try {
        // 编码消息
        const encodedMessage = new TextEncoder().encode(message);
        // 消息签名
        const signMessage = await provider.signMessage(encodedMessage);
        return signMessage;
    } catch(error) {
        console.warn(error);
        // @ts-ignore
        throw new Error(error.message);
    }
}

export default phantomSignMessage;