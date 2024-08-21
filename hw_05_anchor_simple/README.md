##### 使用说明（注意：如果要测试部署请先修改程序地址；程序地址在 /programs/hw_05_anchor_simple/src/lib.rs 文件 和 /Anchor.toml 文件 以及 /client/client.ts文件里面）
```bash
# 编译源码（注意：该命令实际执行的是 anchor build 就是利用Anchor框架来编译Rust源码）
$ npm run build

# 将程序部署到开发网络
$ npm run deploy:devnet
```