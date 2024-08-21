
use thiserror::Error;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use solana_program::decode_error::DecodeError;
use solana_program::msg;
use solana_program::program_error::{PrintProgramError, ProgramError};

/**
 * 定义合约错误类型
 */

#[derive(Clone,Debug,Eq,Error,FromPrimitive,PartialEq)]
pub enum BasicExampleError {
    // 抛出这个错误类型返回error包裹的message消息
    #[error("Invalid instruction")]
    InvalidInstruction,

    #[error("Data account initialized")]
    DataAccountInitialized,
}

impl From<BasicExampleError> for ProgramError {
    fn from(e: BasicExampleError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for BasicExampleError {
    fn type_of() -> &'static str {
        "BasicExampleError"
    }
}

impl PrintProgramError for BasicExampleError {
    fn print<E>(&self) where E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive, {
        match self {
            BasicExampleError::InvalidInstruction => msg!("Invalid instruction!"),
            BasicExampleError::DataAccountInitialized => msg!("Data account initialized!"),
        }
    }
}