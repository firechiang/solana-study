use crate::{error::HelloWorldError, processor::Processor};
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult,
    program_error::PrintProgramError, pubkey::Pubkey,
};

// 该文件是合约入口
entrypoint!(processor_instruction);

// 合约入口实现
fn processor_instruction(program_id: &Pubkey,accounts: &[AccountInfo],instruction_data: &[u8],) -> ProgramResult {
    if let Err(error) = Processor::processor(program_id, accounts, instruction_data) {
        error.print::<HelloWorldError>();
        return Err(error);
    }
    Ok(())
}