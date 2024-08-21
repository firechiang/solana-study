use crate::Pubkey;
// 注意：所有和Anchor框架有关系的都需要添加这个依赖（否则编译无法通过）
use anchor_lang::prelude::*;
// self 表示当前模块可使用 token::xxx 来调用函数
use anchor_spl::{token::{self, Token}};
use crate::error::FaucetError;


// 配置调用initialize函数所需要的AccountInfo账户
#[derive(Accounts)]
pub struct InitializeFaucet<'info> {

    // 要存储水龙头配置信息的账户，配置信息的结构体是 FaucetConfig
    // init 表示初始数据，就是数据没有就创建，但是不能修改数据（注意：这里指的数据是 FaucetConfig 结构体数据）
    // payer 指定存储该数据由谁付钱（这里指定的是user账户，那么创建水龙头就是右user账户签名并付款）
    // space 指定数据空间（就是该数据最大存储空间）
    // 具体各个account的属性说明请参考：https://docs.rs/anchor-lang/0.25.0/anchor_lang/derive.Accounts.html
    #[account(init, payer = user, space = 8 + 8 + 8*1024)]
    pub faucet_config: Account<'info,FaucetConfig>,

    // Token程序地址
    // 注意：双引号引起来的是判断配置，如果 token_program.key ！= token::ID 会抛出异常
    // 还有在当前属性上判断并获取当前属性不需要加 & 符号取引用，但是获取其它属性需要加 & 符号，取引用
    // 说明：token::ID表示调用token的ID函数，这里的token表示公共的Token程序（就是官方的Token程序），也就是如果token_program的地址不等于官方的Token程序地址就抛出异常
    //#[account("token_program.key == &token::ID")]
    //pub token_program: AccountInfo<'info>, 下面这个是新的写法
    // （注意：这个/// CHECK: 表示该字段不需要验证，如果不加这个Anchor框架编译会报错，它会提示你说这个字段没有做权限验证）
    /// CHECK:
    pub token_program: Program<'info, Token>,

    // 代币信息账户（注意：mut声明，表示程序要可以修改代币信息账户里面的数据）
    // （注意：这个/// CHECK: 表示该字段不需要验证，如果不加这个Anchor框架编译会报错，它会提示你说这个字段没有做权限验证）
    /// CHECK:
    #[account(mut)]
    pub token_mint: AccountInfo<'info>,

    // 代币信息账户所有者
    // （注意：这个/// CHECK: 表示该字段不需要验证，如果不加这个Anchor框架编译会报错，它会提示你说这个字段没有做权限验证）
    /// CHECK:
    #[account()]
    pub token_authority: AccountInfo<'info>,

    // 数据存储费用（因为该函数是要初始化存储数据的，所以需要这个，如果是修改数据就不需要传这个了）
    pub rent: Sysvar<'info,Rent>,

    // 签名以及付款用户
    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

// 配置调用drip函数所需要的AccountInfo账户
#[derive(Accounts)]
pub struct Drip<'info> {
    // 水龙头配置信息账户，并自动将数据解码成FaucetConfig结构体
    #[account()]
    pub faucet_config: Account<'info,FaucetConfig>,

    // Token程序地址
    // 注意：双引号引起来的是判断配置，如果 token_program.key ！= token::ID 会抛出异常
    // 还有在当前属性上判断并获取当前属性不需要加 & 符号取引用，但是获取其它属性需要加 & 符号，取引用
    // 说明：token::ID表示调用token的ID函数，这里的token表示公共的Token程序（就是官方的Token程序），也就是如果token_program的地址不等于官方的Token程序地址就抛出异常
    //#[account("token_program.key == &token::ID")] 下面这个是新的写法
    // 注意：使用 &token::ID 当前文件顶部必须声明 使用 token 模块
    // （注意：这个/// CHECK: 表示该字段不需要验证，如果不加这个Anchor框架编译会报错，它会提示你说这个字段没有做权限验证）
    /// CHECK:
    #[account(constraint = token_program.key == &token::ID)]
    pub token_program: AccountInfo<'info>,

    // 代币信息账户（注意：mut声明，表示程序要可以修改代币信息账户里面的数据）
    //#[account(mut,"&faucet_config.token_mint == token_mint.key")] 下面这个是新的写法
    // （注意：这个/// CHECK: 表示该字段不需要验证，如果不加这个Anchor框架编译会报错，它会提示你说这个字段没有做权限验证）
    /// CHECK:
    #[account(mut, constraint = &faucet_config.token_mint == token_mint.key)]
    pub token_mint: AccountInfo<'info>,

    // 代币信息账户所有者
    //#[account("&faucet_config.token_authority == token_authority.key")] 下面这个是新的写法
    // （注意：这个/// CHECK: 表示该字段不需要验证，如果不加这个Anchor框架编译会报错，它会提示你说这个字段没有做权限验证）
    /// CHECK:
    #[account(constraint = &faucet_config.token_authority == token_authority.key)]
    pub token_authority: AccountInfo<'info>,

    // 代币接收账户
    // （注意：这个/// CHECK: 表示该字段不需要验证，如果不加这个Anchor框架编译会报错，它会提示你说这个字段没有做权限验证）
    /// CHECK:
    #[account(mut)]
    pub receiver: AccountInfo<'info>,
}


// 配置调用set_drip_volume函数所需要的AccountInfo账户
#[derive(Accounts)]
pub struct DripVolume<'info> {

    // has_one=authority 表示检查 FaucetConfig 结构体里面的authority属性是否与当前结构里面的属性authority是同一个地址
    // @ FaucetError::Forbidden 表示如果 has_one=authority 没有验证通过就抛 FaucetError::Forbidden 异常
    // 具体各个account的属性说明请参考：https://docs.rs/anchor-lang/0.25.0/anchor_lang/derive.Accounts.html
    #[account(mut, has_one = authority @ FaucetError::Forbidden)]
    pub faucet_config: Account<'info, FaucetConfig>,

    pub authority: Signer<'info>,
}

// 水龙头配置信息（用户账户会存储该信息）
#[account]
pub struct FaucetConfig {
    // Token程序地址
    pub token_program: Pubkey,
    // 代币信息账户
    pub token_mint: Pubkey,
    // 代币信息账户所有者
    pub token_authority: Pubkey,
    // 种子（注意：这个值是在前端使用 findProgramAddress([swap_info.publicKey],program_id) 所得到的）
    // 注意：该值也可以在Solana程序里面使用Pubkey::find_program_address函数获取，具体可参考token-swap程序
    pub nonce: u8,
    // 水龙头一次给多少币
    pub drip_volume: u64,
    // 水龙头的所有者（就是创建该水龙头时的签名付款账户）
    pub authority: Pubkey,
}