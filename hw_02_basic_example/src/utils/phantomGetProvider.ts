import {PhantomProvider} from '../types'

/**
 * 获取Phantom插件实例
 */
const phantomGetProvider = () : PhantomProvider | undefined => {
    if('phantom' in window) {
        const anyWindow: any = window;
        const provider = anyWindow.phantom?.solana;
        if(provider?.isPhantom) {
            return provider;
        }
    }
    window.open('https://phantom.app/','_blank')
}

export default phantomGetProvider;