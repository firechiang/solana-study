#### 一、Solana智能合约简单说明
##### 1，Solana智能合约不存储任何状态信息（因为每一个智能合约都会在用户地址上产生一个合约账户），所有的数据都是存储在用户地址上的合约账户里面。这是Etherscan智能合约和Solana智能合约的最大区别
##### 2，Solana的智能合约存储是需要付费的（也就是如果你操作了智能合约并存储了数据就需要Gas费）
##### 3，要存储数据的长度可以在数据账户创建时指定
##### 4，合约地址不能存储和与转移代币，但是发起地址和合约地址一起可以生成一个没有私钥的地址，这个地址的签名只在合约里面完成，可以使用这个地址来持有其他所有代币，然后在合约里面用这个地址来转币
##### 5，智能合约可使用的堆栈是可以配置的（就是SDK的BpfComputeBudget对象）
##### 6，Solana上有些Rust SDK是不能用的比如 HashMap
##### 7，合约币转账如果接收方没有合约账户，发送方需要先帮接收方创建好合约账户再进行转账
##### 8，一个地址只能拥有一种代币，因为一个地址只能存储一条数据。但是多条数据的所有者可以指定为同一个地址，这样也就实现了一个地址拥有多个币种
##### 9，CPI和PDA概念说明：一个智能合约调用另一个智能合约这个操作叫CPI，但是调用另一个合约需要签名，所以我们需要用调用当前合约的发起者和当前合约一起可以生成一个没有私钥的地址，而这个地址可以在合约里面签名去调用另一个合约。这一整个操作我们叫PDA

#### 二、安装Solana客户端，[官方文档](https://docs.solana.com/getstarted/local)
```bash
$ export http_proxy=http://127.0.0.1:58591/
$ export https_proxy=http://127.0.0.1:58591/
# 注意：执行这个脚本可能需要代理
$ sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

# 验证Solana客户端是否安装成功
$ solana --version
```

#### 三、使用Solana客户端创建代币
```bash
# 创建代币（注意：spl-token命令是安装好Solana客户端后就会有）
# --decimals 指定精度为12，
# --program-id 指定Solana代币标准程序ID创建代币（注意：这里使用的是Token-2022 Program ID也是当前最新的）
# --enable-metadata 开启元数据（后面可以指定代币的一些信息数据，比如名称等等）
$ spl-token create-token --decimals 12 --program-id TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb --enable-metadata
               # 代币地址                                                  # 代币程序地址（就是代币Program Id）
Creating token 8kDYBqzYrKayd2fQ63BPd8ed1sk8hsUWQ2y6JcBUhHdT under program TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb
# 代币地址（注意：代币地址存储所有代币数据，该数据所有权就是代币创建者）
Address:  8kDYBqzYrKayd2fQ63BPd8ed1sk8hsUWQ2y6JcBUhHdT
Decimals:  12
# 交易签名也是交易Hash
Signature: 3yc6xVVWZBoVZuqucbmSWKj87QwXtk1YhwkKRpqv2xhu53Vyodwy25t7EjFJxZJmcVRbEBHDBH3kejCsSSBryBSm

# 为上面刚刚创建的代币初始化元数据（就是设置代币名称，简称，代币网址URL）
# 注意：8kDYBqzYrKayd2fQ63BPd8ed1sk8hsUWQ2y6JcBUhHdT 是代币地址
$ spl-token initialize-metadata 8kDYBqzYrKayd2fQ63BPd8ed1sk8hsUWQ2y6JcBUhHdT "TokenName" "TokenSymbol" "https://spl.solana.com/token-2022"

# 查询代币分配总量
$ spl-token supply 8kDYBqzYrKayd2fQ63BPd8ed1sk8hsUWQ2y6JcBUhHdT

# 创建代币持有账户（因为Solana地址本身不能持有ERC20代币，只能创建代币持有账户持有代币，然后我拥有代币持有账户的所有权）
# --fee-payer 指定交易签名地址密钥对文件
# --owner     指定代币持有账户的所有者地址
# 最后一个参数是代币地址
$ spl-token create-account --fee-payer ~/.config/solana/id.json --owner CokWw92izG3TrnkZJK3RujGwnUKq1i29pzL4shpUpVaE 8kDYBqzYrKayd2fQ63BPd8ed1sk8hsUWQ2y6JcBUhHdT
# 代币持有账户地址
Creating account 4GLLKhsTCkm7roqFiTEm5fg4LDrjBZmgyT94cJ6kAaTr
# 交易签名也是交易Hash
Signature: SUqvh7h3qgZV3sywXEmYzegiemffr51fQeoAuWbETTzEqfs8vuxr8KAVF6Nx7httt13wripV2mHtMhqhoEFuwHv

# 为代币持有账户Mint（分配）代币。第一个参数是代币地址，第二个参数是分配数量，第三个参数是代币持有账户地址
$ spl-token mint 8kDYBqzYrKayd2fQ63BPd8ed1sk8hsUWQ2y6JcBUhHdT 10000 -- 4GLLKhsTCkm7roqFiTEm5fg4LDrjBZmgyT94cJ6kAaTr

# 查看某个代币余额（参数是代币地址）
$ spl-token balance 8kDYBqzYrKayd2fQ63BPd8ed1sk8hsUWQ2y6JcBUhHdT

# 转帐代币（除前两个选项以外，第一个参数是代币地址，第二个参数是代币数量，第三个参数是接收地址）
# --fund-recipient           表示如果接收地址没有该代币持有账户会帮其创建，如果有则不创建
# --allow-unfunded-recipient 表示如果接收地址没有Solana余额也执行转帐
$ spl-token transfer --fund-recipient --allow-unfunded-recipient 8kDYBqzYrKayd2fQ63BPd8ed1sk8hsUWQ2y6JcBUhHdT 11 3f2PCtqDp1vNm5yJk53t5peTL15ZjXcTm8bsUXpUWE2q
```

#### 四、打包智能合约代码
```bash
# 测试智能合约代码
$ cargo test-bpf --manifest-path=./Cargo.toml

# 打包Solana智能合约程序
# --bpf-out-dir 指定打包后文件输出目录
$ cargo build-bpf --manifest-path=./Cargo.toml --bpf-out-dir=dist/program

# 清空Solana打包程序
$ cargo clean --manifest-path=./Cargo.toml && rm -rf ./dist
```

#### 五、部署智能合约到本地集群
```bash
# 启动本地Solana伪集群
$ solana-test-validator

# 创建密钱包（就是部署智能合约的账户），钱包密钥对默认创建在 ~/.config/solana/id.json
# 指定密钥对存储路径示例: solana-keygen new -o /home/chiangfire/data-data/dev-tools/Solana/test-key/id.json
# 可指定参数 --no-outfile 表示不存储密钥对数据
$ solana-keygen new

# 将Solana服务端设置成本地（或者使用：solana config set --url http://127.0.0.1:8899）
$ solana config set --url localhost
# 将Solana服务端设置成开发网络
$ solana config set --url devnet
# 将Solana服务端设置成测试网络
$ solana config set --url testnet
# 将Solana服务端设置成主网
$ solana config set --url mainnet-beta
# 获取Solana服务端配置
$ solana config get

# 指定Solana部署所使用的钱包账户（如果不指定会默认使用 ~/.config/solana/id.json）
$ solana config set -k ~/.config/solana/id.json

# 领取空投（注意：不领取的话账户里面没有钱）
$ solana airdrop 100

# 查看钱包余额
$ solana balance

# 部署智能合约（注意：/home/helloworld.so 是已经打包好的合约程序）
$ solana program deploy /home/helloworld.so
# 部署成功后的合约地址
Program Id: EjS5rkqgXAUWqhvUip9nWN9mmdRzKLxmsfoXnUggn7pM
```

#### 六、Anchor框架开发搭建（现在开发Solana智能合约建议使用该框架）
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

# 如果前端要使用 Anchor的话需要安装一下插件
$ npm install -g mocha
$ npm install -g @project-serum/anchor
```

#### 七、Anchor框架简单使用
```bash
# 使用Anchor创建项目
$ anchor init "项目名称"

# 使用Anchor编译项目，编译完成以后在target/idl目录下会生成IDL json文件（类似于ABI）（注意：该命令需要在项目目录下执行）
$ anchor build

# 使用客户端JS代码测试链上程序（注意：这个测试代码是写在tests目录下的（具体可参考hw_06_anchor_simple项目））
# --skip-local-validator 表示不自动启动本地Solana伪集群
$ anchor test --skip-local-validator
```





