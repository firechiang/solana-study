use borsh::BorshDeserialize;
use helloworld::{process_instruction, GreetingAccount};
use solana_program_test::*;
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Signer,
    transaction::Transaction,
};
use std::mem;

#[tokio::test]
async fn test_helloworld() {
    let program_id = Pubkey::new_unique();
    let greeted_pubkey = Pubkey::new_unique();
    println!("program_id={}",program_id);
    println!("greeted_pubkey={}",greeted_pubkey);

    /**
     * 模拟部署合约
     * @param program_name        合约名称
     * @param program_id          合约程序ID
     * @param process_instruction 合约入口函数
     */
    let mut program_test = ProgramTest::new("helloworld",program_id,processor!(process_instruction),);

    /**
     * 给用户地址添加合约账户
     * @param address 账户地址
     * @param account 账户信息
     */
    program_test.add_account(greeted_pubkey,Account {
            lamports: 5,
            data: vec![0_u8; mem::size_of::<u32>()],
            owner: program_id,
            ..Account::default()
        },
    );
    // 启动合约
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // 获取用户地址的合约账户
    let greeted_account = banks_client.get_account(greeted_pubkey).await.expect("get_account").expect("greeted_account not found");
    // 验证该账户是否没有访问过计数器合约
    assert_eq!(GreetingAccount::try_from_slice(&greeted_account.data).unwrap().counter,0);

    /*--------------------------------------第一次模拟交易-------------------------------------------------*/
    // 模拟创建交易
    let mut transaction = Transaction::new_with_payer(&[Instruction::new_with_bincode(
            program_id, //要访问的智能合约ID（合约地址）
            &[0], // ignored but makes the instruction unique in the slot
            vec![AccountMeta::new(greeted_pubkey, false)],// 访问者地址（is_signer 表示访问者是否持有私钥，is_writable 表示程序是否可以修改账户信息）
        )],
        Some(&payer.pubkey()),
    );
    // 交易签名
    transaction.sign(&[&payer], recent_blockhash);
    // 使用合约处理交易（就是访问智能合约）
    let res = banks_client.process_transaction(transaction).await.unwrap();
    println!("TransactionRes={:#?}",res);

    // 获取用户地址的合约账户
    let greeted_account = banks_client.get_account(greeted_pubkey).await.expect("get_account").expect("greeted_account not found");
    // 验证该账户是否已经访问过1次计数器合约（因为上面模拟交易访问过一次）
    assert_eq!(GreetingAccount::try_from_slice(&greeted_account.data).unwrap().counter,1);

    /*--------------------------------------第二次模拟交易-------------------------------------------------*/
    // 再次模拟创建交易
    let mut transaction = Transaction::new_with_payer(&[Instruction::new_with_bincode(
            program_id, //要访问的智能合约ID（合约地址）
            &[1], // ignored but makes the instruction unique in the slot
            vec![AccountMeta::new(greeted_pubkey, false)], // 访问者地址（就是先通过这个地址从合约中拿到账户，再用账户去调用合约）。注意：账户信息在上面已经添加到合约当中
        )],
        Some(&payer.pubkey()),
    );
    // 交易签名
    transaction.sign(&[&payer], recent_blockhash);
    // 使用合约处理交易（就是访问智能合约）
    banks_client.process_transaction(transaction).await.unwrap();

    // 获取用户地址的合约账户
    let greeted_account = banks_client.get_account(greeted_pubkey).await.expect("get_account").expect("greeted_account not found");
    // 验证该账户是否已经访问过2次计数器合约（因为上面模拟交易总共访问过2次）
    assert_eq!(GreetingAccount::try_from_slice(&greeted_account.data).unwrap().counter,2);
}