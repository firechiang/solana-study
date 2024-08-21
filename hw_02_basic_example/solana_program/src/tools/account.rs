use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::program::{invoke, invoke_signed};
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::{msg, system_instruction};

/**
 * 创建数据账户
 * @param payer 数据账户所有者
 * @param rent 数据租金信息
 * @param space 数据空间大小（数据长度）
 * @param program_id 数据账户所属程序ID
 * @param system_program 系统程序ID
 * @param new_pda_account PDA账户地址
 * @param new_pda_signer_seeds PDA地址种子签名
 */
pub fn create_pda_account<'a> (
    payer: &AccountInfo<'a>,
    rent: &Rent,
    space: usize,
    program_id: &Pubkey,
    system_program: &AccountInfo<'a>,
    new_pda_account: &AccountInfo<'a>,
    new_pda_signer_seeds: &[&[u8]]
) -> ProgramResult {
    // 要创建的pda数据账户余额大于0
    if new_pda_account.lamports() > 0 {
        // 计算存储数据还需多少金额（注意：这个计算是建立在数据账户已有余额的前提下）
        let required_lamports = rent.minimum_balance(space).max(1).saturating_sub(new_pda_account.lamports());
        // 存储数据还需金额大于0，直接使用所有者账户向其转帐
        // 注意：这个invoke函数是跨程序调用俗称CPI，第一个参数是构建instruction操作，第二个参数是调用所需账户（如果被调用函数里面要验证pda地址签名，我们就要使用invoke_signed函数跨程序调用）
        if required_lamports > 0 {
            invoke(
                &system_instruction::transfer(payer.key, new_pda_account.key, required_lamports),
                &[payer.clone(),new_pda_account.clone(),system_program.clone()]
            )?;
        }
        // 为pda地址数据账户分配空间（因为pda地址有余额，所以是有空间的，这里调用等于是重新分配空间）
        // 注意：这个invoke_signed函数是跨程序调用俗称CPI，第一个参数是构建instruction操作，第二个参数是调用所需账户，第三个参数是pda账户种子
        invoke_signed(
            &system_instruction::allocate(new_pda_account.key, space as u64),
            &[new_pda_account.clone(), system_program.clone()],
            &[new_pda_signer_seeds],
        )?;
        // 指定pda地址数据账户所属程序（因为pda地址有余额，所以很有可能所属于另外某个程序）
        invoke_signed(
            &system_instruction::assign(new_pda_account.key, program_id),
            &[new_pda_account.clone(), system_program.clone()],
            &[new_pda_signer_seeds],
        )
    } else {
        msg!("创建数据账户前一步 space: {:#?},lamports: {:#?}",space as u64,rent.minimum_balance(space).max(1));
        // 创建pda地址的数据账户
        // 注意：这个invoke_signed函数是跨程序调用俗称CPI，第一个参数是构建instruction操作，第二个参数是调用所需账户，第三个参数是pda账户种子
        invoke_signed(
            &system_instruction::create_account(
                payer.key,
                new_pda_account.key,
                rent.minimum_balance(space).max(1),
                space as u64,
                program_id,
            ),
            &[
                payer.clone(),
                new_pda_account.clone(),
                //system_program.clone(),
            ],
            &[new_pda_signer_seeds],
        )
    }
    //************************************************************
    // CPI 跨程序调用基础示例
    //************************************************************
    // 构建instruction的keys
    //let account_metas = vec![
    //    AccountMeta::new(*from_pubkey, true),
    //    AccountMeta::new(*to_pubkey, true),
    //];


    // 构建instruction
    //let instruction = {
    //    // 被调用程序ID
    //    program_id: Pubkey,
    //    // 被调用程序所要的keys（也就是账户信息）
    //    accounts: Vec<AccountMeta>,
    //    // 被调用程序的input data
    //    data: Vec<u8>,
    //}


    // 注意：这个invoke函数是跨程序调用俗称CPI，第一个参数是构建instruction操作，第二个参数是调用所需账户（如果被调用函数里面要验证pda地址签名，我们就要使用invoke_signed函数跨程序调用）
    //invoke(&instruction,&[from_pubkey.clone(),to_pubkey.clone()]);


    // 注意：这个invoke_signed函数是跨程序调用俗称CPI，第一个参数是构建instruction操作，第二个参数是调用所需账户，第三个参数是pda账户种子
    //invoke_signed(&instruction,&[from_pubkey.clone(),to_pubkey.clone()],&[构建pda账户的种子]);

    //***************************************************************
    // 关于PDA地址的种子说明
    //***************************************************************
    // 前端代码获得pda_address（PDA地址），bump_seed（凹凸种子），在后端是使用 [Buffer.from("test")] + [bump_seed] 作为PDA地址的种子签名，后端的示例代码在最下面
    // let [pda_address, bump_seed] = PublicKey.findProgramAddressSync([Buffer.from("test")],programId,);


    // 在后段端后获得pda_address（PDA地址），bump_seed（凹凸种子）这个和前端的效果一样的，如果参数一致那么所得到的地址也是一样的
    // let (pda_address, bump_seed) = Pubkey::find_program_address(&[&address.key.to_bytes()],program_id)；
    // 获得PDA地址的签名种子，可以拿这个种子去调用其它程序
    // let pda_signer_seeds: &[&[_]] = &[&address.key.to_bytes(),&[bump_seed]];
}