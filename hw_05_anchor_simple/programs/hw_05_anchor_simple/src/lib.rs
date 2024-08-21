mod state;
mod error;

use state::*;
// 注意：所有和Anchor框架有关系的都需要添加这个依赖（否则编译无法通过）
use anchor_lang::prelude::*;
use anchor_spl::token::{MintTo};

// 指定合约部署地址（就是如果合约部署就使用该地址）
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

// 指定该模块是一个 Solana Program（注意：该模块下的每一个pub函数都是Program函数，在前端可以调用）
#[program]
pub mod hw_06_anchor_simple {
    use super::*;

    /**
     * 初始化一个水龙头（就是创建一个水龙头）
     * @ctx 上下文（里面包含调用该函数所需要的所有AccountInfo账户）
     * @none
     * @drip_volume
     */
    pub fn initialize(ctx: Context<InitializeFaucet>,nonce: u8,drip_volume: u64) -> Result<()> {
        // 获取faucet_config账户里面的数据并解码成 FaucetConfig 结构体
        let faucet_config = &mut ctx.accounts.faucet_config;
        faucet_config.token_program = *ctx.accounts.token_program.key;
        faucet_config.token_mint = *ctx.accounts.token_mint.key;
        faucet_config.token_authority = *ctx.accounts.token_authority.key;
        faucet_config.authority = *ctx.accounts.user.key;
        faucet_config.nonce = nonce;
        faucet_config.drip_volume = drip_volume;
        // 注意：这个到最后数据会自动存储
        Ok(())
    }

    /**
     * 空投
     * @ctx 上下文（里面包含调用该函数所需要的所有AccountInfo账户）
     */
    pub fn drip(ctx: Context<Drip>) -> Result<()> {
        let faucet_config = ctx.accounts.faucet_config.clone();
        // 获取种子（可参考token-swap程序）
        let seeds = &[
            faucet_config.to_account_info().key.as_ref(),
            &[faucet_config.nonce],
        ];
        // 签名种子
        let signer_seeds = &[&seeds[..]];
        // 调用目标合约mint_to函数所需要的账户（这个目标合约是Token合约）
        let cpi_accounts = MintTo {
            // 代币信息账户
            mint: ctx.accounts.token_mint.to_account_info(),
            // 代币接收账户
            to: ctx.accounts.receiver.to_account_info(),
            // 代币信息账户所有者
            authority: ctx.accounts.token_authority.to_account_info(),
        };
        // 目标合约地址
        let cpi_program = ctx.accounts.token_program.clone();
        // 签名
        let cpi_ctx = CpiContext::new_with_signer(cpi_program,cpi_accounts,signer_seeds);
        // 调用目标合约，也就是Token合约的mint_to函数
        anchor_spl::token::mint_to(cpi_ctx, faucet_config.drip_volume)?;
        Ok(())
    }

    /**
     * 修改水龙头一次给多少币
     */
    pub fn set_drip_volume(ctx: Context<DripVolume>, drip_volume: u64) -> Result<()> {
        // 获取faucet_config账户里面的数据并解码成 FaucetConfig 结构体
        let faucet_config = &mut ctx.accounts.faucet_config;
        faucet_config.drip_volume = drip_volume;
        // 注意：这个到最后会数据自动修改并存储
        Ok(())
    }
}