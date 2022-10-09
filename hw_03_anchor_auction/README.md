#### 简单的拍卖程序，使用 Anchor 框架开发
#### Anchor 框架开发搭建
```bash
# 当前命令行窗口使用代理
$ export http_proxy=http://127.0.0.1:58591/
$ export https_proxy=http://127.0.0.1:58591/

# 安装 AVM
$ cargo install --git https://github.com/coral-xyz/anchor avm --locked --force

# 安装最新Anchor套件
$ avm install latest
# 使用最新Anchor套件
$ avm use latest

# 验证Anchor套件是否安装成功
$ anchor --version
```