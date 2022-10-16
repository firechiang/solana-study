##### 简要说明
##### 1、由于纯使用react-scripts插件报@solana/buffer-layout/src/Layout.ts no such file or directory错误，所以使用react-app-rewired插件重写webpack配置来规避前面的错误
##### 2、由于错误说明由于项目使用TS而@solana/web3.js兼容TS类型的文件并没有自动加载，导致开发启动时抱上述错误
