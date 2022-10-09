use solana_program::account_info::{AccountInfo, next_account_info};
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use crate::instruction::HelloWorldInstruction;
use crate::state::HelloWorldState;


// 该文件是合约里面各个函数的逻辑实现
pub struct Processor{}

impl Processor {


    // 存储数据
    pub fn processor_hello(message: String,accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        // 发起者账户信息
        let client_info = next_account_info(account_info_iter)?;
        // 发起者要存储的数据信息
        let message_info = next_account_info(account_info_iter)?;
        //
        if !client_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        // 解包生成 HelloWorldState
        let mut state = HelloWorldState::unpack_unchecked(&message_info.data.borrow())?;
        state.account_key = *client_info.key;
        state.message = message;
        // 打包存储
        HelloWorldState::pack(state, &mut message_info.data.borrow_mut())?;
        Ok(())
    }

    // 删除数据（说明：因为Solana上存储数据是要付费的，删除数据，我们只需要将存储数据账户上的余额全部转走，就等于删除了数据）
    pub fn processor_erase(accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let client_info = next_account_info(account_info_iter)?;
        let message_info = next_account_info(account_info_iter)?;


        if !client_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        // 获取用户账户余额（就是有多少SOL）
        let client_starting_lamports = client_info.lamports();
        // 将消息账户余额全部转到用户账户（逻辑就是：用户账户余额 = 用户账户余额本身余额 + 消息账户余额）
        **client_info.lamports.borrow_mut() = client_starting_lamports + message_info.lamports();
        // 将消息账户余额置为0（这样消息账户上的数据就会自动被删除）
        **message_info.lamports.borrow_mut() = 0;
        Ok(())
    }

    // 查询函数
    pub fn processor_query(accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let client_info = next_account_info(account_info_iter)?;
        let message_info = next_account_info(account_info_iter)?;
        if !client_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let state = HelloWorldState::unpack_unchecked(&message_info.data.borrow())?;
        msg!("查询到链上数据: {:#?}",&state);
        Ok(())
    }



    pub fn processor(program_id: &Pubkey,accounts: &[AccountInfo],input: &[u8]) -> ProgramResult {
        // 解析合约参数转换成实际调用函数和参数
        let instruction = HelloWorldInstruction::unpack(input)?;
        match instruction {
            HelloWorldInstruction::Hello {message} => {
                msg!("调用智能合约Hello函数");
                Self::processor_hello(message,accounts)
            },
            HelloWorldInstruction::Erase => {
                msg!("调用智能合约Erase函数");
                Self::processor_erase(accounts)
            },
            HelloWorldInstruction::Query => {
                msg!("调用智能合约Query函数");
                Self::processor_query(accounts)
            },
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}