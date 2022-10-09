use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};
/**
这是一个简单的计数器合约（记录账户访问合约的次数）
*/

/// 存储在账户中的计数器结构体
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct GreetingAccount {
    /// 账户访问合约次数
    pub counter: u32,
}

// 配置合约的入口函数（相当于main函数）
entrypoint!(process_instruction);

// 合约入口函数实现
/**
* @param program_id        合约程序ID（合约地址）
* @param accounts          发起者发起交易时所发送的所有账户信息（这个是在前端控制的）
* @param _instruction_data 调用智能合约的参数
*/
pub fn process_instruction(program_id: &Pubkey,accounts: &[AccountInfo],_instruction_data: &[u8],) -> ProgramResult {

    // 获取合约账户的迭代器
    let accounts_iter = &mut accounts.iter();

    // 获取合约账户
    let account = next_account_info(accounts_iter)?;
    msg!("开始进入合约入口函数,program_id={},owner={}",program_id,account.owner);
    // 判断这个合约账户是不是用来访问当前合约的（就是这个账户是不是用来访问这个合约的，因为每一个智能合约都会在用户地址上产生一个合约账户）
    if account.owner != program_id {
        msg!("该账户不是用来访问当前合约的，拒绝访问!");
        return Err(ProgramError::IncorrectProgramId);
    }

    // 合约程序存储在账户中的信息（就是GreetingAccount 存储在账户中的计数器结构体）
    let mut greeting_account = GreetingAccount::try_from_slice(&account.data.borrow())?;
    // 计数器加1
    greeting_account.counter += 1;
    // 再将修改后的数据存储到账户当中
    greeting_account.serialize(&mut &mut account.data.borrow_mut()[..])?;

    msg!("当前账户第 {} 次访问合约!", greeting_account.counter);

    Ok(())
}

// Sanity tests
#[cfg(test)]
mod test {
    use super::*;
    use solana_program::clock::Epoch;
    use std::mem;

    #[test]
    fn test_sanity() {
        // 注意：Pubkey::default()就是等于0
        let program_id = Pubkey::default();
        let key = Pubkey::default();
        let mut lamports = 0;
        let mut data = vec![0; mem::size_of::<u32>()];
        // 合约地址
        let owner = Pubkey::default();
        // 模拟账户
        let account = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            Epoch::default(),
        );
        let instruction_data: Vec<u8> = Vec::new();
        // 将账户信息加入集合
        let accounts = vec![account];
        // 将合约存储在账户中的信息转换成GreetingAccount对象，并拿到计数
        let counter = GreetingAccount::try_from_slice(&accounts[0].data.borrow()).unwrap().counter;
        // 判断计数是不是等于0
        assert_eq!(counter,0);
        // 调用合约入口函数
        process_instruction(&program_id, &accounts, &instruction_data).unwrap();
        // 因为上面调用了一次合约，所以这里计数器变成了 1
        assert_eq!(GreetingAccount::try_from_slice(&accounts[0].data.borrow()).unwrap().counter,1);
        // 再次调用合约入口函数
        process_instruction(&program_id, &accounts, &instruction_data).unwrap();
        // 因为上面调用了两次合约，所以这里计数器变成了 2
        assert_eq!(GreetingAccount::try_from_slice(&accounts[0].data.borrow()).unwrap().counter,2);
    }
}