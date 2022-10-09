use std::mem::size_of;
use std::str::from_utf8;
use solana_program::msg;
use solana_program::program_error::ProgramError;
use crate::error::HelloWorldError;

// 该文件是解析调用合约入口参数

// 该枚举定义合约有那些函数以及函数的参数
#[repr(C)]
#[derive(Debug)]
pub enum HelloWorldInstruction {
    // 函数Hello参数是message
    Hello {
        message: String,
    },
    // 函数Erase没有参数
    Erase,
    // 函数Query
    Query,
}

impl HelloWorldInstruction {
    // 解析 input 参数
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        use HelloWorldError::InvalidInstruction;
        msg!("正在解析 Input！");
        let (&tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            // 调用合约 Hello函数
            0 => {
                let message= String::from(from_utf8(rest).unwrap());
                Self::Hello{
                    message,
                }
            },
            // 调用合约 Erase函数
            1 => Self::Erase,
            // 调用Query函数
            2 => Self::Query,
            _ => return Err(HelloWorldError::InvalidInstruction.into()),
        })
    }

    // 打包参数成 input
    pub fn pack(&self) -> Vec<u8> {
        let mut buf : Vec<u8>;
        let self_len= size_of::<Self>();
        match self {
            &Self::Hello {
                ref message,
            } => {
                // 初始化缓冲区大小
                buf = Vec::with_capacity(self_len+1);
                // 前面第一个字节是函数标识
                buf.push(0); // tag
                buf.extend_from_slice(message.as_bytes());
            }
            Self::Erase => {
                // 初始化缓冲区大小
                buf = Vec::with_capacity(self_len);
                // 前面第一个字节是函数标识
                buf.push(1); //tag
            },
            Self::Query => {
                // 初始化缓冲区大小
                buf = Vec::with_capacity(self_len);
                // 前面第一个字节是函数标识
                buf.push(2);
            }
        };
        buf
    }
}