// 注意：所有和Anchor框架有关系的都需要添加这个依赖（否则编译无法通过）
use anchor_lang::prelude::*;

/**
 * 自定义 Anchor 框架异常
 */
#[error_code]
pub enum FaucetError {

    #[msg("Authority error")]
    Forbidden,

}