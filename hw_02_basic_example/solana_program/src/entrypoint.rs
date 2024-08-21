use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult,
    program_error::PrintProgramError, pubkey::Pubkey,
};
use crate::processor::BasicExampleProcessor;
use crate::error::BasicExampleError;


// 定义合约入口
entrypoint!(processor_instruction);

fn processor_instruction(program_id: &Pubkey,accounts: &[AccountInfo],input: &[u8]) -> ProgramResult {
    if let Err(error) = BasicExampleProcessor::processor(program_id,accounts,input) {
        error.print::<BasicExampleError>();
        return Err(error);
    }
    Ok(())
}