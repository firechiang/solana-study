//! Program state processor

use crate::{
    amount_to_ui_amount_string_trimmed,
    error::TokenError,
    instruction::{is_valid_signer_index, AuthorityType, TokenInstruction, MAX_SIGNERS},
    state::{Account, AccountState, Mint, Multisig},
    try_ui_amount_into_amount,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::set_return_data,
    program_error::ProgramError,
    program_memory::sol_memcmp,
    program_option::COption,
    program_pack::{IsInitialized, Pack},
    pubkey::{Pubkey, PUBKEY_BYTES},
    system_program,
    sysvar::{rent::Rent, Sysvar},
};

/// Program state handler.
pub struct Processor {}

impl Processor {

    /// 初始化一个代币合约合约（就像Solidity上部署一个代币合约一样）
    fn _process_initialize_mint(
        accounts: &[AccountInfo],
        decimals: u8,
        mint_authority: Pubkey,
        freeze_authority: COption<Pubkey>,
        rent_sysvar_account: bool,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let mint_info = next_account_info(account_info_iter)?;
        // 数据长度
        let mint_data_len = mint_info.data_len();
        // 获取手动配置的数据租金信息（Solana数据存储需要收费）
        let rent = if rent_sysvar_account {
            Rent::from_account_info(next_account_info(account_info_iter)?)?
        } else {
            Rent::get()?
        };
        // 解码生成数据对象 Mint
        let mut mint = Mint::unpack_unchecked(&mint_info.data.borrow())?;
        // 如果数据对象已经存在了
        if mint.is_initialized {
            return Err(TokenError::AlreadyInUse.into());
        }
        // 判断余额是否可以抵扣存储数据的租金（注意：lamports 表示账户余额，就是有多少个SOL代币）
        if !rent.is_exempt(mint_info.lamports(), mint_data_len) {
            return Err(TokenError::NotRentExempt.into());
        }

        mint.mint_authority = COption::Some(mint_authority);
        mint.decimals = decimals;
        mint.is_initialized = true;
        mint.freeze_authority = freeze_authority;
        // 存储数据
        Mint::pack(mint, &mut mint_info.data.borrow_mut())?;

        Ok(())
    }

    /// Processes an [InitializeMint](enum.TokenInstruction.html) instruction.
    /// 初始化一个代币合约合约（就像Solidity上部署一个代币合约一样），可手动配置数据租金信息（Solana数据存储需要收费）
    pub fn process_initialize_mint(
        accounts: &[AccountInfo],
        decimals: u8,
        mint_authority: Pubkey,
        freeze_authority: COption<Pubkey>,
    ) -> ProgramResult {
        Self::_process_initialize_mint(accounts, decimals, mint_authority, freeze_authority, true)
    }

    /// Processes an [InitializeMint2](enum.TokenInstruction.html) instruction.
    /// 初始化一个代币合约合约（就像Solidity上部署一个代币合约一样），不可以手动配置数据租金信息（Solana数据存储需要收费）
    pub fn process_initialize_mint2(
        accounts: &[AccountInfo],
        decimals: u8,
        mint_authority: Pubkey,
        freeze_authority: COption<Pubkey>,
    ) -> ProgramResult {
        Self::_process_initialize_mint(accounts, decimals, mint_authority, freeze_authority, false)
    }

    /// 初始化用户地址的代币账户
    fn _process_initialize_account(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        owner: Option<&Pubkey>,
        rent_sysvar_account: bool,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        // 用户地址信息
        let new_account_info = next_account_info(account_info_iter)?;
        // 代币地址信息（类似于Solidity合约信息）
        let mint_info = next_account_info(account_info_iter)?;
        // 如果owner不等于Null就是使用owner（注意：if let 是语法糖，就是match的单个匹配。也就是如果owner等于Some（也就是不等于Null））
        let owner = if let Some(owner) = owner {
            owner
        } else {
            next_account_info(account_info_iter)?.key
        };
        // 用户地址数据长度
        let new_account_info_data_len = new_account_info.data_len();
        // 获取手动配置的数据租金信息（Solana数据存储需要收费）
        let rent = if rent_sysvar_account {
            Rent::from_account_info(next_account_info(account_info_iter)?)?
        } else {
            Rent::get()?
        };
        // 解码生成数据对象 Account
        let mut account = Account::unpack_unchecked(&new_account_info.data.borrow())?;
        // 如果数据对象已存在，抛出异常
        if account.is_initialized() {
            return Err(TokenError::AlreadyInUse.into());
        }
        // 判断余额是否可以抵扣存储数据的租金（注意：lamports 表示账户余额，就是有多少个SOL代币）
        if !rent.is_exempt(new_account_info.lamports(), new_account_info_data_len) {
            return Err(TokenError::NotRentExempt.into());
        }
        // 判断代币地址是不是等于本地配置的一个地址（如果相等表示这个代币是SOL）
        let is_native_mint = Self::cmp_pubkeys(mint_info.key, &crate::native_mint::id());
        // 如果不相等就是用户来创建持有代币账户
        if !is_native_mint {
            // 判断代币地址的账户信息是不是当前智能合约的
            Self::check_account_owner(program_id, mint_info)?;
            // 显示解码代币地址的账户信息（也就是生成 Mint对象），只要没有异常就说明代币地址的账户数据正常
            let _ = Mint::unpack(&mint_info.data.borrow_mut())
                .map_err(|_| Into::<ProgramError>::into(TokenError::InvalidMint))?;
        }
        // 代币地址（简单理解就是合约地址）
        account.mint = *mint_info.key;
        // 账户拥有者
        account.owner = *owner;
        // 可关闭当前账户的地址
        account.close_authority = COption::None;
        // 授权地址（就是授权某个地址可以操控该账户）
        account.delegate = COption::None;
        // 授权金额
        account.delegated_amount = 0;
        // 账户状态信息（AccountState::Initialized表示已创建）
        account.state = AccountState::Initialized;
        // 初始化用户地址的代币账户是SOL
        if is_native_mint {
            // 获取最低的数据存储费用
            let rent_exempt_reserve = rent.minimum_balance(new_account_info_data_len);
            // 是系统代币（类似于Etherscan上的 WETH）
            account.is_native = COption::Some(rent_exempt_reserve);
            // 代币余额 = 用户地址上的SOL - 最低的数据存储费用
            account.amount = new_account_info
                .lamports()
                .checked_sub(rent_exempt_reserve)
                .ok_or(TokenError::Overflow)?;
        } else {
            // 不是系统代币
            account.is_native = COption::None;
            // 代币余额
            account.amount = 0;
        };
        // 存储数据
        Account::pack(account, &mut new_account_info.data.borrow_mut())?;

        Ok(())
    }

    /// Processes an [InitializeAccount](enum.TokenInstruction.html) instruction.
    /// 初始化用户地址的代币账户（可手动配置数据租金信息（Solana数据存储需要收费））
    pub fn process_initialize_account(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        Self::_process_initialize_account(program_id, accounts, None, true)
    }

    /// Processes an [InitializeAccount2](enum.TokenInstruction.html) instruction.
    /// 初始化用户地址的代币账户并指定所有者（可手动配置数据租金信息（Solana数据存储需要收费））
    pub fn process_initialize_account2(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        owner: Pubkey,
    ) -> ProgramResult {
        Self::_process_initialize_account(program_id, accounts, Some(&owner), true)
    }

    /// Processes an [InitializeAccount3](enum.TokenInstruction.html) instruction.
    /// 初始化用户地址的代币账户并指定所有者（不可手动配置数据租金信息（Solana数据存储需要收费））
    pub fn process_initialize_account3(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        owner: Pubkey,
    ) -> ProgramResult {
        Self::_process_initialize_account(program_id, accounts, Some(&owner), false)
    }
    /// 初始化一个多签账户，就是多签钱包
    fn _process_initialize_multisig(
        accounts: &[AccountInfo],
        m: u8,
        rent_sysvar_account: bool,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        // 获取第一个地址账户信息（注意：这个地址就是多签账户地址信息）
        let multisig_info = next_account_info(account_info_iter)?;
        // 多签账户数据长度
        let multisig_info_data_len = multisig_info.data_len();
        // 获取手动配置的数据租金信息（Solana数据存储需要收费）
        let rent = if rent_sysvar_account {
            Rent::from_account_info(next_account_info(account_info_iter)?)?
        } else {
            Rent::get()?
        };
        // 解码生成数据对象 Multisig
        let mut multisig = Multisig::unpack_unchecked(&multisig_info.data.borrow())?;
        // 钱包已经生成过抛出异常
        if multisig.is_initialized {
            return Err(TokenError::AlreadyInUse.into());
        }
        // 判断余额是否可以抵扣存储数据的租金（注意：lamports 表示账户余额，就是有多少个SOL代币）
        if !rent.is_exempt(multisig_info.lamports(), multisig_info_data_len) {
            return Err(TokenError::NotRentExempt.into());
        }
        // 从迭代器里面获取到所有未被迭代的账户地址，示为参与签名的账户（也就是除第一个以外的，因为上面代码只迭代了一个）
        let signer_infos = account_info_iter.as_slice();
        // 总共签名者数量
        multisig.m = m;
        // 有效签名者数量
        multisig.n = signer_infos.len() as u8;
        // 判断有效签名者数量是不是在最小和最大签名数之间
        if !is_valid_signer_index(multisig.n as usize) {
            return Err(TokenError::InvalidNumberOfProvidedSigners.into());
        }
        // 判断总共签名者数量是不是在最小和最大签名数之间
        if !is_valid_signer_index(multisig.m as usize) {
            return Err(TokenError::InvalidNumberOfRequiredSigners.into());
        }
        // 循环配置参与签名者的地址
        for (i, signer_info) in signer_infos.iter().enumerate() {
            multisig.signers[i] = *signer_info.key;
        }
        // 是否已初始化
        multisig.is_initialized = true;
        // 保存数据
        Multisig::pack(multisig, &mut multisig_info.data.borrow_mut())?;

        Ok(())
    }

    /// Processes a [InitializeMultisig](enum.TokenInstruction.html) instruction.
    /// 初始化一个多签账户，就是多签钱包（可手动配置数据租金信息（Solana数据存储需要收费））
    pub fn process_initialize_multisig(accounts: &[AccountInfo], m: u8) -> ProgramResult {
        Self::_process_initialize_multisig(accounts, m, true)
    }

    /// Processes a [InitializeMultisig2](enum.TokenInstruction.html) instruction.
    /// 初始化一个多签账户，就是多签钱包（不可手动配置数据租金信息（Solana数据存储需要收费））
    pub fn process_initialize_multisig2(accounts: &[AccountInfo], m: u8) -> ProgramResult {
        Self::_process_initialize_multisig(accounts, m, false)
    }

    /// Processes a [Transfer](enum.TokenInstruction.html) instruction.
    /**
     * 转账
     * @program_id 合约ID
     * @accounts 账户信息（注意：授权账户是一定要传的，如果是自己转币授权账户就传自己）
     * @amount   转出金额
     * @expected_decimals 代币精度
     */
    pub fn process_transfer(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
        expected_decimals: Option<u8>,
    ) -> ProgramResult {
        // 获取所有账户的迭代器
        let account_info_iter = &mut accounts.iter();
        // 取第一个账户为转出账户
        let source_account_info = next_account_info(account_info_iter)?;
        // 如果expected_decimals不等于Null就是使用迭代器里面的第二个账户为代币信息账户
        // 注意：if let 是语法糖，就是match的单个匹配。也就是如果expected_decimals等于Some（也就是不等于Null）
        let expected_mint_info = if let Some(expected_decimals) = expected_decimals {
            Some((next_account_info(account_info_iter)?, expected_decimals))
        } else {
            None
        };
        // 取下一个账户为接收者账户（应该是迭代器里面第2个或第3个账户）
        let destination_account_info = next_account_info(account_info_iter)?;
        // 取下一个账户为授权账户（应该是迭代器里面第3个或第4个账户）
        let authority_info = next_account_info(account_info_iter)?;
        // 解码转出账户信息
        let mut source_account = Account::unpack(&source_account_info.data.borrow())?;
        // 解码转入账户信息
        let mut destination_account = Account::unpack(&destination_account_info.data.borrow())?;
        // 判断两个账户是否被冻结
        if source_account.is_frozen() || destination_account.is_frozen() {
            return Err(TokenError::AccountFrozen.into());
        }
        // 判断转出账户余额是否大于转出数量
        if source_account.amount < amount {
            return Err(TokenError::InsufficientFunds.into());
        }
        // 判断两个地址是不是同一种代币
        if !Self::cmp_pubkeys(&source_account.mint, &destination_account.mint) {
            return Err(TokenError::MintMismatch.into());
        }
        // 如果代币信息账户不为空，就验证代币信息
        // 注意：if let 是语法糖，就是match的单个匹配。也就是如果expected_mint_info等于Some（也就是不等于Null）
        if let Some((mint_info, expected_decimals)) = expected_mint_info {
            // 判断代币信息是不是转出地址的代币
            if !Self::cmp_pubkeys(mint_info.key, &source_account.mint) {
                return Err(TokenError::MintMismatch.into());
            }
            // 解码出代币信息
            let mint = Mint::unpack(&mint_info.data.borrow_mut())?;
            // 判断代币信息的精度是否和传入的精度相同
            if expected_decimals != mint.decimals {
                return Err(TokenError::MintDecimalsMismatch.into());
            }
        }
        // 是不是相同地址转账
        let self_transfer = Self::cmp_pubkeys(source_account_info.key, destination_account_info.key);
        // 匹配转出账户的授权地址（就是授权某个地址可以操控该账户）
        match source_account.delegate {
            // 如果授权地址不为空并且授权地址等于授权账户
            COption::Some(ref delegate) if Self::cmp_pubkeys(authority_info.key, delegate) => {
                // 验证所有者签名，这里是验证授权地址签名（注意：如果所有者是多签钱包则需要通过多地址签名）
                Self::validate_owner(
                    program_id,
                    delegate,
                    authority_info,
                    account_info_iter.as_slice(),
                )?;
                // 如果授权金额小于转出数量抛出异常
                if source_account.delegated_amount < amount {
                    return Err(TokenError::InsufficientFunds.into());
                }
                // 如果不是同地址转账
                if !self_transfer {
                    // 转出账户授权金额 = 转出账户授权金额 - 转出金额
                    source_account.delegated_amount = source_account.delegated_amount.checked_sub(amount).ok_or(TokenError::Overflow)?;
                    // 减去转出金额之后如果授权金额等于0则置空
                    if source_account.delegated_amount == 0 {
                        source_account.delegate = COption::None;
                    }
                }
            }
            // 验证所有者签名（注意：如果所有者是多签钱包则需要通过多地址签名）
            _ => Self::validate_owner(
                program_id,
                &source_account.owner,
                authority_info,
                account_info_iter.as_slice(),
            )?,
        };
        // 如果是同地址转账或者转出金额等于0
        if self_transfer || amount == 0 {
            // 检查某个账户的所有者是不是合约ID
            Self::check_account_owner(program_id, source_account_info)?;
            Self::check_account_owner(program_id, destination_account_info)?;
        }

        // This check MUST occur just before the amounts are manipulated
        // to ensure self-transfers are fully validated
        // 如果是同地址转账
        if self_transfer {
            return Ok(());
        }

        // 转出账户的余额 = 转出账户余额 - 转出金额
        source_account.amount = source_account.amount.checked_sub(amount).ok_or(TokenError::Overflow)?;

        // 转入账户的余额 = 转入账户余额 + 转入金额
        destination_account.amount = destination_account.amount.checked_add(amount).ok_or(TokenError::Overflow)?;

        // 如果转出账户是系统代币（类似于Etherscan上的 WETH）
        if source_account.is_native() {
            // 转出账户余额
            let source_starting_lamports = source_account_info.lamports();
            // 转出账户的余额 = 转出账户余额 - 转出金额
            **source_account_info.lamports.borrow_mut() = source_starting_lamports.checked_sub(amount).ok_or(TokenError::Overflow)?;

            // 转入账户余额
            let destination_starting_lamports = destination_account_info.lamports();
            // 转入账户的余额 = 转入账户余额 + 转入金额
            **destination_account_info.lamports.borrow_mut() = destination_starting_lamports.checked_add(amount).ok_or(TokenError::Overflow)?;
        }

        // 存储数据
        Account::pack(source_account, &mut source_account_info.data.borrow_mut())?;
        Account::pack(destination_account, &mut destination_account_info.data.borrow_mut(), )?;

        Ok(())
    }

    /// Processes an [Approve](enum.TokenInstruction.html) instruction.
    /**
     * 授权
     * @program_id 合约ID
     * @accounts   账户信息
     * @amount     授权金额
     * @expected_decimals 代币精度
     */
    pub fn process_approve(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
        expected_decimals: Option<u8>,
    ) -> ProgramResult {
        // 获取账户迭代器
        let account_info_iter = &mut accounts.iter();
        // 取迭代器里面的第一个为持有代币的账户（也就是要将这个账户授权出去）
        let source_account_info = next_account_info(account_info_iter)?;
        // 如果expected_decimals不等于Null就是使用迭代器里面的第二个账户为代币信息账户
        // 注意：if let 是语法糖，就是match的单个匹配。也就是如果expected_decimals等于Some（也就是不等于Null）
        let expected_mint_info = if let Some(expected_decimals) = expected_decimals {
            Some((next_account_info(account_info_iter)?, expected_decimals))
        } else {
            None
        };
        // 取下一个账户为被授权账户（就是这个账户可以操控持有代币的账户）（应该是迭代器里面第2个或第3个账户）
        let delegate_info = next_account_info(account_info_iter)?;
        // 取下一个账户为授权发起账户（就是持有代币账户的所有者）（应该是迭代器里面第3个或第4个账户）
        let owner_info = next_account_info(account_info_iter)?;
        // 解码持有代币的账户信息
        let mut source_account = Account::unpack(&source_account_info.data.borrow())?;
        // 持有代币的账户信息是否被冻结
        if source_account.is_frozen() {
            return Err(TokenError::AccountFrozen.into());
        }
        // 如果代币信息账户不为空，就验证代币信息
        // 注意：if let 是语法糖，就是match的单个匹配。也就是如果expected_mint_info等于Some（也就是不等于Null）
        if let Some((mint_info, expected_decimals)) = expected_mint_info {
            // 判断代币信息是不是转出地址的代币
            if !Self::cmp_pubkeys(mint_info.key, &source_account.mint) {
                return Err(TokenError::MintMismatch.into());
            }
            // 解码出代币信息
            let mint = Mint::unpack(&mint_info.data.borrow_mut())?;
            // 判断代币信息的精度是否和传入的精度相同
            if expected_decimals != mint.decimals {
                return Err(TokenError::MintDecimalsMismatch.into());
            }
        }

        // 验证持有代币账户的所有者签名（注意：如果所有者是多签钱包则需要通过多地址签名）
        Self::validate_owner(
            program_id,
            &source_account.owner,
            owner_info,
            account_info_iter.as_slice(),
        )?;
        // 修改持有代币账户的授权地址
        source_account.delegate = COption::Some(*delegate_info.key);
        // 修改授权金额
        source_account.delegated_amount = amount;
        // 存储数据
        Account::pack(source_account, &mut source_account_info.data.borrow_mut())?;

        Ok(())
    }

    /// Processes an [Revoke](enum.TokenInstruction.html) instruction.
    /**
     * 取消授权
     */
    pub fn process_revoke(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        // 取迭代器里面的第一个为持有代币的账户
        let source_account_info = next_account_info(account_info_iter)?;
        // 解码持有代币账户的信息
        let mut source_account = Account::unpack(&source_account_info.data.borrow())?;
        // 取迭代器里面的第二个为持有代币账户的所有者
        let owner_info = next_account_info(account_info_iter)?;
        // 持有代币账户是否被冻结
        if source_account.is_frozen() {
            return Err(TokenError::AccountFrozen.into());
        }
        // 验证持有代币账户所有者的签名
        Self::validate_owner(
            program_id,
            &source_account.owner,
            owner_info,
            account_info_iter.as_slice(),
        )?;
        // 修改授权地址为空
        source_account.delegate = COption::None;
        // 修改授权金额为0
        source_account.delegated_amount = 0;
        // 存储数据
        Account::pack(source_account, &mut source_account_info.data.borrow_mut())?;

        Ok(())
    }

    /// Processes a [SetAuthority](enum.TokenInstruction.html) instruction.
    /**
     * 1，修改持有代币账户的所有者或者是修改可以关闭持有代币账户的地址
     * 2，修改代币信息账户的铸币人（也是所有者）或者是修改可以冻结账户的地址
     * @authority_type 操作类型
     * @new_authority  新的所有者 或者 是新的可以关闭持有代币账户的地址 或者是 新可以冻结账户的地址
     *
     */
    pub fn process_set_authority(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        authority_type: AuthorityType,
        new_authority: COption<Pubkey>,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        // 取迭代器里面的第一个为代币信息账户或是持有代币账户
        let account_info = next_account_info(account_info_iter)?;
        // 取迭代器里面的第二个为所有者账户
        let authority_info = next_account_info(account_info_iter)?;
        // 如果account_info是持有代币账户
        if account_info.data_len() == Account::get_packed_len() {
            // 解码持有代币账户
            let mut account = Account::unpack(&account_info.data.borrow())?;
            // 持有代币账户是否被冻结
            if account.is_frozen() {
                return Err(TokenError::AccountFrozen.into());
            }
            // 操作类型
            match authority_type {
                // 指定新的所有者
                AuthorityType::AccountOwner => {
                    // 验证持有代币账户的所有者签名
                    Self::validate_owner(
                        program_id,
                        &account.owner,
                        authority_info,
                        account_info_iter.as_slice(),
                    )?;
                    // 如果新的所有者不为空则修改否则抛出异常
                    if let COption::Some(authority) = new_authority {
                        account.owner = authority;
                    } else {
                        return Err(TokenError::InvalidInstruction.into());
                    }
                    // 清空授权信息
                    account.delegate = COption::None;
                    account.delegated_amount = 0;
                    // 如果是系统代币（类似于WETH）就清空其持有关闭权限的地址
                    if account.is_native() {
                        account.close_authority = COption::None;
                    }
                }
                // 指定新的可以关闭账户的地址
                AuthorityType::CloseAccount => {
                    // 获取可关闭持有代币账户的地址
                    let authority = account.close_authority.unwrap_or(account.owner);
                    // 验证所有者签名
                    Self::validate_owner(
                        program_id,
                        &authority,
                        authority_info,
                        account_info_iter.as_slice(),
                    )?;
                    // 修改可关闭持有代币账户的地址
                    account.close_authority = new_authority;
                }
                _ => {
                    return Err(TokenError::AuthorityTypeNotSupported.into());
                }
            }
            // 存储信息
            Account::pack(account, &mut account_info.data.borrow_mut())?;

        // 如果account_info是代币信息账户
        } else if account_info.data_len() == Mint::get_packed_len() {
            // 解码代币信息账户
            let mut mint = Mint::unpack(&account_info.data.borrow())?;
            // 操作类型
            match authority_type {
                // 修改铸币人（就是代币信息账户所有者）
                AuthorityType::MintTokens => {
                    // Once a mint's supply is fixed, it cannot be undone by setting a new
                    // mint_authority
                    // 获取旧的铸币人（也是旧的所有者）
                    let mint_authority = mint.mint_authority.ok_or(Into::<ProgramError>::into(TokenError::FixedSupply))?;
                    // 验证旧的所有者签名（只有旧的所有者才能指定新的所有者）
                    Self::validate_owner(
                        program_id,
                        &mint_authority,
                        authority_info,
                        account_info_iter.as_slice(),
                    )?;
                    // 修改成新的所有者
                    mint.mint_authority = new_authority;
                }
                // 指定新的可以冻结账户的地址
                AuthorityType::FreezeAccount => {
                    // Once a mint's freeze authority is disabled, it cannot be re-enabled by
                    // setting a new freeze_authority
                    // 获取到旧的可以冻结账户的地址
                    let freeze_authority = mint.freeze_authority.ok_or(Into::<ProgramError>::into(TokenError::MintCannotFreeze))?;
                    // 验证旧的可以冻结账户的地址签名（只有旧的才能指定新的）
                    Self::validate_owner(
                        program_id,
                        &freeze_authority,
                        authority_info,
                        account_info_iter.as_slice(),
                    )?;
                    // 修改可以冻结账户的地址
                    mint.freeze_authority = new_authority;
                }
                _ => {
                    return Err(TokenError::AuthorityTypeNotSupported.into());
                }
            }
            // 存储数据
            Mint::pack(mint, &mut account_info.data.borrow_mut())?;
        } else {
            return Err(ProgramError::InvalidArgument);
        }

        Ok(())
    }

    /// Processes a [MintTo](enum.TokenInstruction.html) instruction.
    /**
     * 为某个地址生成代币
     * @amount 生成数量
     * @expected_decimals 代币精度
     */
    pub fn process_mint_to(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
        expected_decimals: Option<u8>,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        // 取迭代器里面的第一个为代币信息账户
        let mint_info = next_account_info(account_info_iter)?;
        // 取迭代器里面的第二个为接收代币的账户
        let destination_account_info = next_account_info(account_info_iter)?;
        // 取迭代器里面的第三个为代币信息账户的所有者
        let owner_info = next_account_info(account_info_iter)?;
        // 解码代币接收账户
        let mut destination_account = Account::unpack(&destination_account_info.data.borrow())?;
        // 判断代币接收账户是否被冻结
        if destination_account.is_frozen() {
            return Err(TokenError::AccountFrozen.into());
        }
        // 判断代币接收账户是否为系统代币（类似于WETH）
        if destination_account.is_native() {
            return Err(TokenError::NativeNotSupported.into());
        }
        // 判断代币接收账户是不是用来接收 mint_info 这个代币的
        if !Self::cmp_pubkeys(mint_info.key, &destination_account.mint) {
            return Err(TokenError::MintMismatch.into());
        }
        // 解码代币信息并判断精度是否与传过来的精度相同
        let mut mint = Mint::unpack(&mint_info.data.borrow())?;
        if let Some(expected_decimals) = expected_decimals {
            if expected_decimals != mint.decimals {
                return Err(TokenError::MintDecimalsMismatch.into());
            }
        }

        // 验证代币信息账户的所有者签名
        match mint.mint_authority {
            COption::Some(mint_authority) => Self::validate_owner(
                program_id,
                &mint_authority,
                owner_info,
                account_info_iter.as_slice(),
            )?,
            COption::None => return Err(TokenError::FixedSupply.into()),
        }

        if amount == 0 {
            Self::check_account_owner(program_id, mint_info)?;
            Self::check_account_owner(program_id, destination_account_info)?;
        }
        // 代币接收账户添加余额
        destination_account.amount = destination_account.amount.checked_add(amount).ok_or(TokenError::Overflow)?;
        // 代币总量增加
        mint.supply = mint.supply.checked_add(amount).ok_or(TokenError::Overflow)?;
        // 存储代币接收账户数据
        Account::pack(destination_account, &mut destination_account_info.data.borrow_mut(), )?;
        // 存储代币信息账户数据
        Mint::pack(mint, &mut mint_info.data.borrow_mut())?;

        Ok(())
    }

    /// Processes a [Burn](enum.TokenInstruction.html) instruction.
    /**
     * 燃烧代币（注意：授权账户也可以调用该函数只是把币燃烧掉了并没有把币转走）
     * @amount 燃烧数量
     * @expected_decimals 代币精度
     */
    pub fn process_burn(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
        expected_decimals: Option<u8>,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        // 取迭代器里面的第一个账户为被燃烧账户（就是要将这个账户里面的代币清除一些）
        let source_account_info = next_account_info(account_info_iter)?;
        // 取迭代器里面的第二个账户为代币信息账户
        let mint_info = next_account_info(account_info_iter)?;
        // 取迭代器里面的第三个账户为被燃烧账户的所有者或者是被授权账户
        let authority_info = next_account_info(account_info_iter)?;

        // 解码被燃烧账户
        let mut source_account = Account::unpack(&source_account_info.data.borrow())?;
        // 解码代币信息
        let mut mint = Mint::unpack(&mint_info.data.borrow())?;
        // 判断被燃烧账户是否被冻结
        if source_account.is_frozen() {
            return Err(TokenError::AccountFrozen.into());
        }
        // 判断被燃烧账户是否为系统代币（类似于WETH）
        if source_account.is_native() {
            return Err(TokenError::NativeNotSupported.into());
        }
        // 判断被燃烧账户的余额是否充足
        if source_account.amount < amount {
            return Err(TokenError::InsufficientFunds.into());
        }
        // 判断被燃烧账户的代币是否属于mint_info的代币
        if !Self::cmp_pubkeys(mint_info.key, &source_account.mint) {
            return Err(TokenError::MintMismatch.into());
        }
        // 如果传进来的代币精度不为空则判断这个精度是否与代币精度一致
        if let Some(expected_decimals) = expected_decimals {
            if expected_decimals != mint.decimals {
                return Err(TokenError::MintDecimalsMismatch.into());
            }
        }
        // 如果被燃烧账户的所有者不是 system_program（系统账户） 或者 incinerator（Solana系统燃烧账户）
        if !source_account.is_owned_by_system_program_or_incinerator() {
            match source_account.delegate {
                // 授权地址不为空并且所有者账户就是授权地址
                COption::Some(ref delegate) if Self::cmp_pubkeys(authority_info.key, delegate) => {
                    // 验证授权账户签名
                    Self::validate_owner(
                        program_id,
                        delegate,
                        authority_info,
                        account_info_iter.as_slice(),
                    )?;
                    // 判断授权金额是否大于燃烧金额
                    if source_account.delegated_amount < amount {
                        return Err(TokenError::InsufficientFunds.into());
                    }
                    // 授权金额 = 旧的授权金额 - 燃烧金额
                    source_account.delegated_amount = source_account.delegated_amount.checked_sub(amount).ok_or(TokenError::Overflow)?;
                    // 如果新授权金额等于0就是清空授权地址
                    if source_account.delegated_amount == 0 {
                        source_account.delegate = COption::None;
                    }
                }
                // 如果授权地址为空就验证所有者签名
                _ => Self::validate_owner(
                    program_id,
                    &source_account.owner,
                    authority_info,
                    account_info_iter.as_slice(),
                )?,
            }
        }

        if amount == 0 {
            Self::check_account_owner(program_id, source_account_info)?;
            Self::check_account_owner(program_id, mint_info)?;
        }
        // 燃烧账户的余额等于减去燃烧数量
        source_account.amount = source_account.amount.checked_sub(amount).ok_or(TokenError::Overflow)?;
        // 代币的总量等于减去燃烧数量
        mint.supply = mint.supply.checked_sub(amount).ok_or(TokenError::Overflow)?;
        // 存储被燃烧账户信息
        Account::pack(source_account, &mut source_account_info.data.borrow_mut())?;
        // 存储代币信息
        Mint::pack(mint, &mut mint_info.data.borrow_mut())?;

        Ok(())
    }

    /// Processes a [CloseAccount](enum.TokenInstruction.html) instruction.
    /**
     * 关闭某个代币账户
     * 注意：如果接收余额的账户要用某个系统账户那么就只能是 incinerator（Solana系统燃烧账户）否则直接抛出异常）
     * incinerator账户地址 = 1nc1nerator11111111111111111111111111111111
     */
    pub fn process_close_account(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        // 取迭代器里面的第一个为将要被关闭的账户
        let source_account_info = next_account_info(account_info_iter)?;
        // 取迭代器里面的第二个为接收将要被关闭账户的余额的账户（关闭账户的逻辑就是将其SOL全部转移，那么该账户就不能为存储付费数据自然被自动删除）
        let destination_account_info = next_account_info(account_info_iter)?;
        // 取迭代器里面的第三个为可关闭账户的账户
        let authority_info = next_account_info(account_info_iter)?;
        // 判断将要被关闭的账户 是不是等于 接收余额的账户
        if Self::cmp_pubkeys(source_account_info.key, destination_account_info.key) {
            return Err(ProgramError::InvalidAccountData);
        }
        // 解码将要被关闭的账户
        let source_account = Account::unpack(&source_account_info.data.borrow())?;
        // 判断将要被关闭的账户不是系统代币账户（类似于WETH）并且余额不等于0
        if !source_account.is_native() && source_account.amount != 0 {
            return Err(TokenError::NonNativeHasBalance.into());
        }
        // 获取到可以关闭该账户的地址
        let authority = source_account.close_authority.unwrap_or(source_account.owner);
        // 将要被关闭的账户的所有者不是 system_program（系统账户） 或者不是 incinerator（Solana系统燃烧账户）
        if !source_account.is_owned_by_system_program_or_incinerator() {
            // 验证可关闭账户的账户签名
            Self::validate_owner(
                program_id,
                &authority,
                authority_info,
                account_info_iter.as_slice(),
            )?;
        // 如果接收余额的账户是某个系统账户那么就一定只能是 incinerator（Solana系统燃烧账户）否则直接抛出异常
        } else if !solana_program::incinerator::check_id(destination_account_info.key) {
            return Err(ProgramError::InvalidAccountData);
        }
        // 接收余额账户的余额
        let destination_starting_lamports = destination_account_info.lamports();
        // 接收余额账户的新余额等于加上被关闭账户的余额
        **destination_account_info.lamports.borrow_mut() = destination_starting_lamports.checked_add(source_account_info.lamports()).ok_or(TokenError::Overflow)?;
        // 被关闭账户的余额等于0
        **source_account_info.lamports.borrow_mut() = 0;
        // 清空数据
        delete_account(source_account_info)?;

        Ok(())
    }

    /// Processes a [FreezeAccount](enum.TokenInstruction.html) or a
    /// [ThawAccount](enum.TokenInstruction.html) instruction.
    /**
     * 冻结或解冻账户
     * @freeze true=冻结，false=解冻
     */
    pub fn process_toggle_freeze_account(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        freeze: bool,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        // 取迭代器里面的第一个为将要被冻结的账户
        let source_account_info = next_account_info(account_info_iter)?;
        // 取迭代器里面的低二个为代币信息
        let mint_info = next_account_info(account_info_iter)?;
        // 取迭代器里面的第三个为有权冻结账户的账户
        let authority_info = next_account_info(account_info_iter)?;
        // 解码将要被冻结的账户
        let mut source_account = Account::unpack(&source_account_info.data.borrow())?;
        // 如果将要被冻结的账户已经被冻结了 或者 解冻账户本身没有被冻结 直接抛出异常
        if freeze && source_account.is_frozen() || !freeze && !source_account.is_frozen() {
            return Err(TokenError::InvalidState.into());
        }
        // 将要被冻结账户属于系统代币（类似于WETH）直接抛出异常
        if source_account.is_native() {
            return Err(TokenError::NativeNotSupported.into());
        }
        // 将要被冻结的账户不是属于 mint_info 代币
        if !Self::cmp_pubkeys(mint_info.key, &source_account.mint) {
            return Err(TokenError::MintMismatch.into());
        }
        // 解码代币信息
        let mint = Mint::unpack(&mint_info.data.borrow_mut())?;
        // 验证有权冻结账户的账户签名
        match mint.freeze_authority {
            COption::Some(authority) => Self::validate_owner(
                program_id,
                &authority,
                authority_info,
                account_info_iter.as_slice(),
            ),
            COption::None => Err(TokenError::MintCannotFreeze.into()),
        }?;
        // 如果 freeze=true 账户状态就等于冻结；如果freeze=false 账户状态就等于正常
        source_account.state = if freeze {
            AccountState::Frozen
        } else {
            AccountState::Initialized
        };
        // 保存数据
        Account::pack(source_account, &mut source_account_info.data.borrow_mut())?;

        Ok(())
    }

    /// Processes a [SyncNative](enum.TokenInstruction.html) instruction
    /**
     * 同步系统代币余额（就是某个地址上有SOL，而且这个地址上也有一个系统代币账户（类似于WETH），就将SOL的数量同步给系统代币）
     */
    pub fn process_sync_native(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        // 取迭代器里面的第一个为系统账户（就是用来存系统代币的类似于WETH）
        let native_account_info = next_account_info(account_info_iter)?;
        // 判断系统账户是否属于当前程序
        Self::check_account_owner(program_id, native_account_info)?;
        // 解码系统账户
        let mut native_account = Account::unpack(&native_account_info.data.borrow())?;
        // 如果是系统代币账户
        if let COption::Some(rent_exempt_reserve) = native_account.is_native {
            // 系统代币新余额等于减去存储数据的费用
            let new_amount = native_account_info.lamports().checked_sub(rent_exempt_reserve).ok_or(TokenError::Overflow)?;
            // 如果新余额小于系统代币账户本身的余额就抛出异常（因为不能同步嘛）
            if new_amount < native_account.amount {
                return Err(TokenError::InvalidState.into());
            }
            // 更新余额
            native_account.amount = new_amount;
        } else {
            return Err(TokenError::NonNativeNotSupported.into());
        }
        // 保存数据
        Account::pack(native_account, &mut native_account_info.data.borrow_mut())?;
        Ok(())
    }

    /// Processes a [GetAccountDataSize](enum.TokenInstruction.html) instruction
    /**
     * 获取代币信息账户数据大小
     */
    pub fn process_get_account_data_size(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        // make sure the mint is valid
        // 取迭代器里面的第一个为代币信息账户
        let mint_info = next_account_info(account_info_iter)?;
        // 判断代币信息账户是否属于当前程序
        Self::check_account_owner(program_id, mint_info)?;
        // 显示解码代币信息只要不会抛出异常就说明代币正常
        let _ = Mint::unpack(&mint_info.data.borrow()).map_err(|_| Into::<ProgramError>::into(TokenError::InvalidMint))?;
        // 设置需要返回的数据
        set_return_data(&Account::LEN.to_le_bytes());
        Ok(())
    }

    /// Processes an [InitializeImmutableOwner](enum.TokenInstruction.html) instruction
    /**
     * 判断某个地址账户是否没有被初始化（已经被初始化了则抛出异常）
     */
    pub fn process_initialize_immutable_owner(accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let token_account_info = next_account_info(account_info_iter)?;
        let account = Account::unpack_unchecked(&token_account_info.data.borrow())?;
        if account.is_initialized() {
            return Err(TokenError::AlreadyInUse.into());
        }
        msg!("Please upgrade to SPL Token 2022 for immutable owner support");
        Ok(())
    }

    /// Processes an [AmountToUiAmount](enum.TokenInstruction.html) instruction
    /**
     * 计算某个金额的正真数量也就是除以过精度的
     * @amount 待计算金额
     */
    pub fn process_amount_to_ui_amount(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        // 取迭代器里面的第一个为代币信息账户
        let mint_info = next_account_info(account_info_iter)?;
        // 判断代币信息账户是否属于当前程序
        Self::check_account_owner(program_id, mint_info)?;
        // 解码代币信息数据
        let mint = Mint::unpack(&mint_info.data.borrow_mut()).map_err(|_| Into::<ProgramError>::into(TokenError::InvalidMint))?;
        // 计算正真数量（也就是去除以精度）
        let ui_amount = amount_to_ui_amount_string_trimmed(amount, mint.decimals);
        // 返回正真数量
        set_return_data(&ui_amount.into_bytes());
        Ok(())
    }

    /// Processes an [AmountToUiAmount](enum.TokenInstruction.html) instruction
    /**
     * 还原某个金额的正真数量也就是乘以过精度的
     * @amount 待计算金额
     */
    pub fn process_ui_amount_to_amount(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        ui_amount: &str,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        // 取迭代器里面的第一个为代币信息账户
        let mint_info = next_account_info(account_info_iter)?;
        // 判断代币信息账户是否属于当前程序
        Self::check_account_owner(program_id, mint_info)?;
        // 解码代币信息数据
        let mint = Mint::unpack(&mint_info.data.borrow_mut()).map_err(|_| Into::<ProgramError>::into(TokenError::InvalidMint))?;
        // 还原数量（也就是去乘以精度）
        let amount = try_ui_amount_into_amount(ui_amount.to_string(), mint.decimals)?;
        // 返回正真数量
        set_return_data(&amount.to_le_bytes());
        Ok(())
    }

    /// Processes an [Instruction](enum.Instruction.html).
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = TokenInstruction::unpack(input)?;

        match instruction {
            TokenInstruction::InitializeMint {
                decimals,
                mint_authority,
                freeze_authority,
            } => {
                msg!("Instruction: InitializeMint");
                // 初始化一个代币合约合约（就像Solidity上部署一个代币合约一样），可手动配置数据租金信息（Solana数据存储需要收费）
                Self::process_initialize_mint(accounts, decimals, mint_authority, freeze_authority)
            }
            TokenInstruction::InitializeMint2 {
                decimals,
                mint_authority,
                freeze_authority,
            } => {
                msg!("Instruction: InitializeMint2");
                // 初始化一个代币合约合约（就像Solidity上部署一个代币合约一样），不可以手动配置数据租金信息（Solana数据存储需要收费）
                Self::process_initialize_mint2(accounts, decimals, mint_authority, freeze_authority)
            }
            TokenInstruction::InitializeAccount => {
                msg!("Instruction: InitializeAccount");
                // 初始化用户地址的代币账户（可手动配置数据租金信息（Solana数据存储需要收费））
                Self::process_initialize_account(program_id, accounts)
            }
            TokenInstruction::InitializeAccount2 { owner } => {
                msg!("Instruction: InitializeAccount2");
                // 初始化用户地址的代币账户并指定所有者（可手动配置数据租金信息（Solana数据存储需要收费））
                Self::process_initialize_account2(program_id, accounts, owner)
            }
            TokenInstruction::InitializeAccount3 { owner } => {
                msg!("Instruction: InitializeAccount3");
                // 初始化用户地址的代币账户并指定所有者（不可手动配置数据租金信息（Solana数据存储需要收费））
                Self::process_initialize_account3(program_id, accounts, owner)
            }
            TokenInstruction::InitializeMultisig { m } => {
                msg!("Instruction: InitializeMultisig");
                // 初始化一个多签账户，就是多签钱包（可手动配置数据租金信息（Solana数据存储需要收费））
                Self::process_initialize_multisig(accounts, m)
            }
            TokenInstruction::InitializeMultisig2 { m } => {
                msg!("Instruction: InitializeMultisig2");
                // 初始化一个多签账户，就是多签钱包（不可手动配置数据租金信息（Solana数据存储需要收费））
                Self::process_initialize_multisig2(accounts, m)
            }
            TokenInstruction::Transfer { amount } => {
                msg!("Instruction: Transfer");
                /**
                 * 转账
                 * @program_id 合约ID
                 * @accounts 账户信息（注意：授权账户是一定要传的，如果是自己转币授权账户就传自己）
                 * @amount   转出金额
                 * @expected_decimals 代币精度
                 */
                Self::process_transfer(program_id, accounts, amount, None)
            }
            TokenInstruction::Approve { amount } => {
                msg!("Instruction: Approve");
                /**
                 * 授权
                 * @program_id 合约ID
                 * @accounts   账户信息
                 * @amount     授权金额
                 * @expected_decimals 代币精度
                 */
                Self::process_approve(program_id, accounts, amount, None)
            }
            TokenInstruction::Revoke => {
                msg!("Instruction: Revoke");
                /**
                 * 取消授权
                 */
                Self::process_revoke(program_id, accounts)
            }
            TokenInstruction::SetAuthority {
                authority_type,
                new_authority,
            } => {
                msg!("Instruction: SetAuthority");
                /**
                 * 1，修改持有代币账户的所有者或者是修改可以关闭持有代币账户的地址
                 * 2，修改代币信息账户的铸币人（也是所有者）或者是修改可以冻结账户的地址
                 * @authority_type 操作类型
                 * @new_authority  新的所有者 或者 是新的可以关闭持有代币账户的地址 或者是 新可以冻结账户的地址
                 *
                 */
                Self::process_set_authority(program_id, accounts, authority_type, new_authority)
            }
            TokenInstruction::MintTo { amount } => {
                msg!("Instruction: MintTo");
                /**
                 * 为某个地址生成代币
                 * @amount 生成数量
                 * @expected_decimals 代币精度
                 */
                Self::process_mint_to(program_id, accounts, amount, None)
            }
            TokenInstruction::Burn { amount } => {
                msg!("Instruction: Burn");
                /**
                 * 燃烧代币（注意：授权账户也可以调用该函数只是把币燃烧掉了并没有把币转走）
                 * @amount 燃烧数量
                 * @expected_decimals 代币精度
                 */
                Self::process_burn(program_id, accounts, amount, None)
            }
            TokenInstruction::CloseAccount => {
                msg!("Instruction: CloseAccount");
                /**
                 * 关闭某个代币账户
                 * 注意：如果接收余额的账户要用某个系统账户那么就只能是 incinerator（Solana系统燃烧账户）否则直接抛出异常）
                 * incinerator账户地址 = 1nc1nerator11111111111111111111111111111111
                 */
                Self::process_close_account(program_id, accounts)
            }
            TokenInstruction::FreezeAccount => {
                msg!("Instruction: FreezeAccount");
                /**
                 * 冻结账户
                 * @freeze true=冻结，false=解冻
                 */
                Self::process_toggle_freeze_account(program_id, accounts, true)
            }
            TokenInstruction::ThawAccount => {
                msg!("Instruction: ThawAccount");
                /**
                 * 解冻账户
                 * @freeze true=冻结，false=解冻
                 */
                Self::process_toggle_freeze_account(program_id, accounts, false)
            }
            TokenInstruction::TransferChecked { amount, decimals } => {
                msg!("Instruction: TransferChecked");
                /**
                 * 转账
                 * @program_id 合约ID
                 * @accounts 账户信息（注意：授权账户是一定要传的，如果是自己转币授权账户就传自己）
                 * @amount   转出金额
                 * @expected_decimals 代币精度
                 */
                Self::process_transfer(program_id, accounts, amount, Some(decimals))
            }
            TokenInstruction::ApproveChecked { amount, decimals } => {
                msg!("Instruction: ApproveChecked");
                /**
                 * 授权
                 * @program_id 合约ID
                 * @accounts   账户信息
                 * @amount     授权金额
                 * @expected_decimals 代币精度
                 */
                Self::process_approve(program_id, accounts, amount, Some(decimals))
            }
            TokenInstruction::MintToChecked { amount, decimals } => {
                msg!("Instruction: MintToChecked");
                /**
                 * 为某个地址生成代币
                 * @amount 生成数量
                 * @expected_decimals 代币精度
                 */
                Self::process_mint_to(program_id, accounts, amount, Some(decimals))
            }
            TokenInstruction::BurnChecked { amount, decimals } => {
                msg!("Instruction: BurnChecked");
                /**
                 * 燃烧代币（注意：授权账户也可以调用该函数只是把币燃烧掉了并没有把币转走）
                 * @amount 燃烧数量
                 * @expected_decimals 代币精度
                 */
                Self::process_burn(program_id, accounts, amount, Some(decimals))
            }
            TokenInstruction::SyncNative => {
                msg!("Instruction: SyncNative");
                /**
                 * 同步系统代币余额（就是某个地址上有SOL，而且这个地址上也有一个系统代币账户（类似于WETH），就将SOL的数量同步给系统代币）
                 */
                Self::process_sync_native(program_id, accounts)
            }
            TokenInstruction::GetAccountDataSize => {
                msg!("Instruction: GetAccountDataSize");
                /**
                 * 获取代币信息账户数据大小
                 */
                Self::process_get_account_data_size(program_id, accounts)
            }
            TokenInstruction::InitializeImmutableOwner => {
                msg!("Instruction: InitializeImmutableOwner");
                /**
                 * 判断某个地址账户是否没有被初始化（已经被初始化了则抛出异常）
                 */
                Self::process_initialize_immutable_owner(accounts)
            }
            TokenInstruction::AmountToUiAmount { amount } => {
                msg!("Instruction: AmountToUiAmount");
                /**
                 * 计算某个金额的正真数量也就是除以过精度的
                 * @amount 待计算金额
                 */
                Self::process_amount_to_ui_amount(program_id, accounts, amount)
            }
            TokenInstruction::UiAmountToAmount { ui_amount } => {
                msg!("Instruction: UiAmountToAmount");
                /**
                 * 还原某个金额的正真数量也就是乘以过精度的
                 * @amount 待计算金额
                 */
                Self::process_ui_amount_to_amount(program_id, accounts, ui_amount)
            }
        }
    }

    /// 检查某个账户的所有者是不是合约ID
    pub fn check_account_owner(program_id: &Pubkey, account_info: &AccountInfo) -> ProgramResult {
        if !Self::cmp_pubkeys(program_id, account_info.owner) {
            Err(ProgramError::IncorrectProgramId)
        } else {
            Ok(())
        }
    }

    /// Checks two pubkeys for equality in a computationally cheap way using
    /// `sol_memcmp`
    /// 判断两个地址是否相等
    pub fn cmp_pubkeys(a: &Pubkey, b: &Pubkey) -> bool {
        sol_memcmp(a.as_ref(), b.as_ref(), PUBKEY_BYTES) == 0
    }

    /// Validates owner(s) are present
    /// 验证所有者签名（注意：如果所有者是多签钱包则需要通过多地址签名）
    /**
     * @program_id 合约ID
     * @expected_owner 授权地址
     * @owner_account_info 授权账户
     * @signers 签名账户
     */
    pub fn validate_owner(
        program_id: &Pubkey,
        expected_owner: &Pubkey,
        owner_account_info: &AccountInfo,
        signers: &[AccountInfo],
    ) -> ProgramResult {
        // 判断授权地址和授权账户地址是否相同
        if !Self::cmp_pubkeys(expected_owner, owner_account_info.key) {
            return Err(TokenError::OwnerMismatch.into());
        }
        // 判断授权账户是不是属于合约 并且 签名账户数量等于有效签名数量（注意：这个判断也说明授权账户是一个多签钱包）
        if Self::cmp_pubkeys(program_id, owner_account_info.owner) && owner_account_info.data_len() == Multisig::get_packed_len() {
            // 解码多签钱包信息
            let multisig = Multisig::unpack(&owner_account_info.data.borrow())?;
            // 已同意签名数量
            let mut num_signers = 0;
            let mut matched = [false; MAX_SIGNERS];
            // 循环迭代所有签名者信息
            for signer in signers.iter() {
                // 循环迭代已存储的签名地址并且只迭代有效的数量
                for (position, key) in multisig.signers[0..multisig.n as usize].iter().enumerate() {
                    // 如果已存储的地址和已签名的地址相同，说明有一个人已经同意了
                    if Self::cmp_pubkeys(key, signer.key) && !matched[position] {
                        // 地址没有签名抛出异常
                        if !signer.is_signer {
                            return Err(ProgramError::MissingRequiredSignature);
                        }
                        matched[position] = true;
                        num_signers += 1;
                    }
                }
            }
            // 如果已通过的签名数量 小于 需要通过的签名数量 验证失败
            if num_signers < multisig.m {
                return Err(ProgramError::MissingRequiredSignature);
            }
            return Ok(());
        } else if !owner_account_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        Ok(())
    }
}

/// Helper function to mostly delete an account in a test environment.  We could
/// potentially muck around the bytes assuming that a vec is passed in, but that
/// would be more trouble than it's worth.
/// 删除账户（前提是先将账户的余额置为0）（注意：这个函数是在当前系统环境为非Solana时执行，也就是我们在调试时在本地系统上会执行这个函数）
#[cfg(not(target_os = "solana"))]
fn delete_account(account_info: &AccountInfo) -> Result<(), ProgramError> {
    // 将账户的所有者修改为系统程序地址
    account_info.assign(&system_program::id());
    // 获取账户数据索引
    let mut account_data = account_info.data.borrow_mut();
    // 获取账户数据长度
    let data_len = account_data.len();
    // 将数据长度置为0
    solana_program::program_memory::sol_memset(*account_data, 0, data_len);
    Ok(())
}

/// Helper function to totally delete an account on-chain
/// 删除账户（前提是先将账户的余额置为0）（注意：这个函数是在当前系统环境为Solana时执行，也就是真正的在链上会执行这个函数）
#[cfg(target_os = "solana")]
fn delete_account(account_info: &AccountInfo) -> Result<(), ProgramError> {
    // 将账户的所有者修改为系统程序地址
    account_info.assign(&system_program::id());
    // 将数据清0
    account_info.realloc(0, false)
}