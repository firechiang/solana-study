use crate::{error::TokenError,processor::Processor};
use solana_program:: {
    account_info::AccountInfo,entrypoint,entrypoint::ProgramResult,
    program_error::PrintProgramError,pubkey::Pubkey
};

entrypoint!(process_instruction);

fn process_instruction(program_id: &Pubkey,accounts: &[AccountInfo],input: &[u8]) -> ProgramResult {
    if let Err(error) = Processor::process(program_id,accounts,input) {
        error.print::<TokenError>();
        return Err(error);
    }
    Ok(())
}

