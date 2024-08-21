use solana_program::account_info::{AccountInfo, next_account_info};
use solana_program::entrypoint::ProgramResult;
use solana_program::{msg, system_program};
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use crate::state::{BasicExampleState, BasicExampleStruct};
use crate::instruction::BasicExampleInstruction;
use crate::tools::account::create_pda_account;

/**
 * 定义合约函数具体实现
 */
pub struct BasicExampleProcessor {}

impl BasicExampleProcessor {

    /**
     * 初始化数据存储账户
     * @param program_id 当前程序ID
     * @param accounts 账户列表
     */
    pub fn initialize(_program_id: &Pubkey,accounts: &[AccountInfo]) -> ProgramResult {
        // 获取账户列表可变迭代器
        let accounts_iter = &mut accounts.iter();
        // 交易发起者账户
        let client_account_info = next_account_info(accounts_iter)?;
        // 要创建数据账户的PDA地址
        let pda_account_info = next_account_info(accounts_iter)?;
        // 系统程序账户
        let sys_program_info = next_account_info(accounts_iter)?;
        // 验证交易发起者签名
        if !client_account_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        // 如果PDA地址已有余额就判断其数据账户是否已经创建
        if pda_account_info.lamports() > 0 {
            // 解码数据存储对象
            let basic_example_struct = BasicExampleStruct::unpack_unchecked(&pda_account_info.data.borrow())?;
            // 判断数据账户是否已经初始化
            if basic_example_struct.state == BasicExampleState::Initialize {
                msg!("数据账户: {:#?} 已初始化。",pda_account_info.key);
                return Err(ProgramError::AccountAlreadyInitialized);
            }
        }
        // 数据租金信息
        let rent = Rent::get()?;
        // 判断余额是否可以抵扣存储数据的租金（注意：lamports 表示账户余额，就是有多少个SOL代币）
        if rent.minimum_balance(BasicExampleStruct::LEN) > client_account_info.lamports() {
            return Err(ProgramError::AccountNotRentExempt);
        };
        // 获取PDA账户地址（注意：这个种子要和前端获取PDA地址的种子一致否则生成的地址会不一样）
        let (pda_account_key,bump_seeds) = Pubkey::find_program_address(&[&client_account_info.key.to_bytes()],_program_id);
        // 如果前端生成的PDA地址和我们生成的PDA地址不一致，说明所使用的种子不一样，直接报错
        if *pda_account_info.key != pda_account_key {
            return Err(ProgramError::InvalidSeeds);
        }
        // PDA地址的所有者如果不是系统程序抛出异常
        if *pda_account_info.owner != system_program::id() {
            return Err(ProgramError::IllegalOwner);
        }
        // PDA账户种子签名
        let pda_signer_seeds:&[&[_]] = &[&client_account_info.key.to_bytes(),&[bump_seeds]];
        // 创建PDA地址数据账户
        create_pda_account(
            client_account_info,
            &rent,
            BasicExampleStruct::LEN,
            _program_id,
            sys_program_info,
            pda_account_info,
            pda_signer_seeds)?;
        // 解码数据存储对象
        let mut basic_example_struct = BasicExampleStruct::unpack_unchecked(&pda_account_info.data.borrow())?;
        // 初始数据
        basic_example_struct.account_key = *client_account_info.key;
        basic_example_struct.state = BasicExampleState::Initialize;
        // 编码并存储存储数据
        BasicExampleStruct::pack(basic_example_struct,&mut pda_account_info.data.borrow_mut())?;
        Ok(())
    }

    /**
     * 修改数据
     * @param program_id 当前程序ID
     * @param accounts 账户列表
     * @param data     数据
     */
    pub fn update(_program_id: &Pubkey,accounts: &[AccountInfo],data: String) -> ProgramResult {
        // 获取账户列表可变迭代器
        let accounts_iter = &mut accounts.iter();
        // 交易发起者账户
        let client_account = next_account_info(accounts_iter)?;
        // 数据账户
        let data_account = next_account_info(accounts_iter)?;
        // 验证交易发起者签名
        if !client_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        // 将旧数据解包生成结构体对象
        let mut basic_example_struct = BasicExampleStruct::unpack_unchecked(&data_account.data.borrow())?;

        if basic_example_struct.state != BasicExampleState::Initialize {
            return Err(ProgramError::UninitializedAccount);
        }
        // 修改结构体对象
        basic_example_struct.account_key = *client_account.key;
        basic_example_struct.data = data;
        // 打包并存储数据
        BasicExampleStruct::pack(basic_example_struct,&mut data_account.data.borrow_mut())?;
        Ok(())
    }

    /**
     * 删除数据
     * 说明：因为Solana上存储数据是要付费的，删除数据，我们只需要将存储数据账户上的余额全部转走，就等于删除了数据
     * @param program_id 当前程序ID
     * @param accounts 账户列表
     */
    pub fn delete(_program_id: &Pubkey,accounts: &[AccountInfo]) -> ProgramResult {
        // 获取账户列表可变迭代器
        let accounts_iter = &mut accounts.iter();
        // 交易发起者账户
        let client_account = next_account_info(accounts_iter)?;
        // 数据账户
        let data_account = next_account_info(accounts_iter)?;
        // 验证交易发起者签名
        if !client_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        // 获取交易发起者账户余额
        let client_account_lamports = client_account.lamports();
        // 将数据账户余额全部转到交易发起者账户（逻辑就是：交易发起者账户余额 = 交易发起者账户余额 + 数据账户余额）
        **client_account.lamports.borrow_mut() = client_account_lamports + data_account.lamports();
        // 将数据账户余额置为0（这样数据账户上的数据就会自动被删除）
        **data_account.lamports.borrow_mut() = 0;
        msg!("数据删除完成！");
        Ok(())
    }

    /**
     * 查询数据
     * @param program_id 当前程序ID
     * @param accounts 账户列表
     */
    pub fn query(_program_id: &Pubkey,accounts: &[AccountInfo]) -> ProgramResult {
        // 获取账户列表可变迭代器
        let accounts_iter = &mut accounts.iter();
        // 交易发起者账户
        let client_account = next_account_info(accounts_iter)?;
        // 数据账户
        let data_account = next_account_info(accounts_iter)?;
        // 验证交易发起者签名
        if !client_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let basic_example_struct = BasicExampleStruct::unpack_unchecked(&data_account.data.borrow())?;
        msg!("查询到链上数据：{:#?}",&basic_example_struct);
        Ok(())
    }

    /**
     * 合约函数调用分发
     * @param program_id 合约程序地址
     * @param accounts   账户列表
     * @param input      调用合约参数
     */
    pub fn processor(_program_id: &Pubkey,accounts: &[AccountInfo],input: &[u8]) -> ProgramResult {
        // 解析合约调用参数将其转换为函数定义枚举
        let instruction = BasicExampleInstruction::unpack(input)?;
        match instruction {
            BasicExampleInstruction::Initialize => {
                Self::initialize(_program_id,accounts)
            },
            BasicExampleInstruction::Update {data} => {
                Self::update(_program_id, accounts,data)
            },
            BasicExampleInstruction::Delete => {
                Self::delete(_program_id ,accounts)
            },
            BasicExampleInstruction::MultiParamTest {aa,bb,cc,dd} => {
                msg!("调用函数 MultiParamTest 参数 aa={:#?},bb={:#?},cc={:#?},dd={:#?}",aa,bb,cc,dd);
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::mem;
    use arrayref::array_mut_ref;
    use solana_program::account_info::AccountInfo;
    use solana_program::clock::Epoch;
    use solana_program::program_pack::Pack;
    use solana_program::pubkey::Pubkey;
    use crate::instruction::BasicExampleInstruction;
    use crate::processor::BasicExampleProcessor;
    use crate::state::{BasicExampleState, BasicExampleStruct};

    #[test]
    fn test_processor() {
        let program_id = Pubkey::default();

        let client_key = Pubkey::default();
        let mut client_key_lamports: u64 = 100000000;
        let mut client_key_data = vec![0;mem::size_of::<u32>()];

        let data_key = Pubkey::default();
        let mut data_key_lamports: u64 = 0;
        let mut data_key_data = vec![0;BasicExampleStruct::LEN];
        let data_key_data = array_mut_ref![data_key_data,0,BasicExampleStruct::LEN];
        BasicExampleStruct {
            account_key: data_key,
            state: BasicExampleState::Initialize,
            data: String::from("datadatadata")
        }.pack_into_slice(data_key_data);

        let client_account = AccountInfo::new(
            &client_key,
            true,
            true,
            &mut client_key_lamports,
            &mut client_key_data,
            &client_key,
            false,
            Epoch::default()
        );

        let data_account = AccountInfo::new(
            &data_key,
            false,
            true,
            &mut data_key_lamports,
            data_key_data,
            &data_key,
            false,
            Epoch::default()
        );

        let basic_example_instruction = BasicExampleInstruction::Delete {

        };
        let input = basic_example_instruction.pack();
        let accounts = vec![client_account,data_account];
        let _=BasicExampleProcessor::processor(&program_id,&accounts,&input);
    }
}