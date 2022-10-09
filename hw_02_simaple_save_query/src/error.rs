use solana_program::{decode_error::DecodeError,
                     program_error::ProgramError,
                     msg,
                     program_error::PrintProgramError};
use thiserror::Error;
use num_traits::FromPrimitive;
use num_derive::FromPrimitive;

/// Errors that may be returned by the hello-world program.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum HelloWorldError {
    /// Invalid instruction
    #[error("Invalid instruction")]
    InvalidInstruction,
}

impl From<HelloWorldError> for ProgramError {
    fn from(e: HelloWorldError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
impl<T> DecodeError<T> for HelloWorldError {
    fn type_of() -> &'static str {
        "HelloWorldError"
    }
}


impl PrintProgramError for HelloWorldError {
    fn print<E>(&self)
        where
            E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            HelloWorldError::InvalidInstruction => msg!("Invalid instruction"),
        }
    }
}