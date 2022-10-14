//! Program state processor

use crate::constraints::{SwapConstraints, SWAP_CONSTRAINTS};
use crate::{
    curve::{
        base::SwapCurve,
        calculator::{RoundDirection, TradeDirection},
        fees::Fees,
    },
    error::SwapError,
    instruction::{
        DepositAllTokenTypes, DepositSingleTokenTypeExactAmountIn, Initialize, Swap,
        SwapInstruction, WithdrawAllTokenTypes, WithdrawSingleTokenTypeExactAmountOut,
    },
    state::{SwapState, SwapV1, SwapVersion},
};
use num_traits::FromPrimitive;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    decode_error::DecodeError,
    entrypoint::ProgramResult,
    instruction::Instruction,
    msg,
    program::invoke_signed,
    program_error::{PrintProgramError, ProgramError},
    program_option::COption,
    pubkey::Pubkey,
    sysvar::Sysvar,
};
use spl_token_2022::{
    check_spl_token_program_account,
    error::TokenError,
    extension::{
        mint_close_authority::MintCloseAuthority, transfer_fee::TransferFeeConfig,
        StateWithExtensions,
    },
    state::{Account, Mint},
};
use std::{convert::TryInto, error::Error};

/// Program state handler.
pub struct Processor {}

impl Processor {
    /// Unpacks a spl_token `Account`.
    /**
     * 解码账户对应在合约上存储的实际数据（注意：这里解码的是代币合约里面某个账户持有代币的数据所以直接返回 Account 对象）
     * @account_info 账户信息
     * @token_program_id 合约地址
     */
    pub fn unpack_token_account(account_info: &AccountInfo,token_program_id: &Pubkey,) -> Result<Account, SwapError> {
        // 如果账户的持有者不是该合约地址 并且 账户地址不是被标记的 直接抛出异常
        if account_info.owner != token_program_id && check_spl_token_program_account(account_info.owner).is_err() {
            Err(SwapError::IncorrectTokenProgramId)
        } else {
            // 解码持有代币信息
            StateWithExtensions::<Account>::unpack(&account_info.data.borrow()).map(|a| a.base).map_err(|_| SwapError::ExpectedAccount)
        }
    }

    /// Unpacks a spl_token `Mint`.
    pub fn unpack_mint(
        account_info: &AccountInfo,
        token_program_id: &Pubkey,
    ) -> Result<Mint, SwapError> {
        if account_info.owner != token_program_id
            && check_spl_token_program_account(account_info.owner).is_err()
        {
            Err(SwapError::IncorrectTokenProgramId)
        } else {
            StateWithExtensions::<Mint>::unpack(&account_info.data.borrow())
                .map(|m| m.base)
                .map_err(|_| SwapError::ExpectedMint)
        }
    }

    /// Unpacks a spl_token `Mint` with extension data
    /**
     * 解码币种信息 Mint 对象（就是将账户里面实际存储的数据解码出来）
     */
    pub fn unpack_mint_with_extensions<'a>(
        account_data: &'a [u8],
        owner: &Pubkey,
        token_program_id: &Pubkey,
    ) -> Result<StateWithExtensions<'a, Mint>, SwapError> {
        if owner != token_program_id && check_spl_token_program_account(owner).is_err() {
            Err(SwapError::IncorrectTokenProgramId)
        } else {
            StateWithExtensions::<Mint>::unpack(account_data).map_err(|_| SwapError::ExpectedMint)
        }
    }

    /// Calculates the authority id by generating a program address.
    /// 通过生成程序地址计算授权id
    pub fn authority_id(
        program_id: &Pubkey,
        my_info: &Pubkey,
        bump_seed: u8,
    ) -> Result<Pubkey, SwapError> {
        Pubkey::create_program_address(&[&my_info.to_bytes()[..32], &[bump_seed]], program_id).or(Err(SwapError::InvalidProgramAddress))
    }

    /// Issue a spl_token `Burn` instruction.
    pub fn token_burn<'a>(
        swap: &Pubkey,
        token_program: AccountInfo<'a>,
        burn_account: AccountInfo<'a>,
        mint: AccountInfo<'a>,
        authority: AccountInfo<'a>,
        bump_seed: u8,
        amount: u64,
    ) -> Result<(), ProgramError> {
        let swap_bytes = swap.to_bytes();
        let authority_signature_seeds = [&swap_bytes[..32], &[bump_seed]];
        let signers = &[&authority_signature_seeds[..]];

        let ix = spl_token_2022::instruction::burn(
            token_program.key,
            burn_account.key,
            mint.key,
            authority.key,
            &[],
            amount,
        )?;

        invoke_signed_wrapper::<TokenError>(
            &ix,
            &[burn_account, mint, authority, token_program],
            signers,
        )
    }

    /// Issue a spl_token `MintTo` instruction.
    /**
     * 为某个地址Mint代币（注意：这个函数里面有调用其他合约的函数）
     * @swap          交易对信息地址
     * @token_program 发币程序账户信息（就是发币合约地址账户）
     * @mint          币种信息
     * @destination   目的地持币账户
     * @authority     所有者账户信息
     * @bump_seed     交易对信息的种子
     * @amount        铸币数量（交易对池的总量）
     */
    pub fn token_mint_to<'a>(
        swap: &Pubkey,
        token_program: AccountInfo<'a>,
        mint: AccountInfo<'a>,
        destination: AccountInfo<'a>,
        authority: AccountInfo<'a>,
        bump_seed: u8,
        amount: u64,
    ) -> Result<(), ProgramError> {
        // 交易对信息地址转byte数组
        let swap_bytes = swap.to_bytes();
        // 交易对信息地址的数组 和 交易对信息地址种子的数组 再组成数组
        let authority_signature_seeds = [&swap_bytes[..32], &[bump_seed]];
        // 在生成签名种子
        let signers = &[&authority_signature_seeds[..]];
        // 生成可以调用Token合约里面mint_to函数的Instruction
        let ix = spl_token_2022::instruction::mint_to(
            token_program.key,
            mint.key,
            destination.key,
            authority.key,
            &[],
            amount,
        )?;
        // 调用Token合约里面的mint_to函数为目的地持币账户铸造代币
        invoke_signed_wrapper::<TokenError>(
            &ix,
            &[mint, destination, authority, token_program],
            signers,
        )
    }

    /// Issue a spl_token `Transfer` instruction.
    #[allow(clippy::too_many_arguments)]
    pub fn token_transfer<'a>(
        swap: &Pubkey,
        token_program: AccountInfo<'a>,
        source: AccountInfo<'a>,
        mint: AccountInfo<'a>,
        destination: AccountInfo<'a>,
        authority: AccountInfo<'a>,
        bump_seed: u8,
        amount: u64,
        decimals: u8,
    ) -> Result<(), ProgramError> {
        let swap_bytes = swap.to_bytes();
        let authority_signature_seeds = [&swap_bytes[..32], &[bump_seed]];
        let signers = &[&authority_signature_seeds[..]];
        let ix = spl_token_2022::instruction::transfer_checked(
            token_program.key,
            source.key,
            mint.key,
            destination.key,
            authority.key,
            &[],
            amount,
            decimals,
        )?;
        invoke_signed_wrapper::<TokenError>(
            &ix,
            &[source, mint, destination, authority, token_program],
            signers,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn check_accounts(
        token_swap: &dyn SwapState,
        program_id: &Pubkey,
        swap_account_info: &AccountInfo,
        authority_info: &AccountInfo,
        token_a_info: &AccountInfo,
        token_b_info: &AccountInfo,
        pool_mint_info: &AccountInfo,
        pool_token_program_info: &AccountInfo,
        user_token_a_info: Option<&AccountInfo>,
        user_token_b_info: Option<&AccountInfo>,
        pool_fee_account_info: Option<&AccountInfo>,
    ) -> ProgramResult {
        if swap_account_info.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }
        if *authority_info.key
            != Self::authority_id(program_id, swap_account_info.key, token_swap.bump_seed())?
        {
            return Err(SwapError::InvalidProgramAddress.into());
        }
        if *token_a_info.key != *token_swap.token_a_account() {
            return Err(SwapError::IncorrectSwapAccount.into());
        }
        if *token_b_info.key != *token_swap.token_b_account() {
            return Err(SwapError::IncorrectSwapAccount.into());
        }
        if *pool_mint_info.key != *token_swap.pool_mint() {
            return Err(SwapError::IncorrectPoolMint.into());
        }
        if *pool_token_program_info.key != *token_swap.token_program_id() {
            return Err(SwapError::IncorrectTokenProgramId.into());
        }
        if let Some(user_token_a_info) = user_token_a_info {
            if token_a_info.key == user_token_a_info.key {
                return Err(SwapError::InvalidInput.into());
            }
        }
        if let Some(user_token_b_info) = user_token_b_info {
            if token_b_info.key == user_token_b_info.key {
                return Err(SwapError::InvalidInput.into());
            }
        }
        if let Some(pool_fee_account_info) = pool_fee_account_info {
            if *pool_fee_account_info.key != *token_swap.pool_fee_account() {
                return Err(SwapError::IncorrectFeeAccount.into());
            }
        }
        Ok(())
    }

    /// Processes an [Initialize](enum.Instruction.html).
    /**
     * 创建交易对
     * @fees 手续费相关配置
     * @swap_curve 交易对价格计算实现
     * @swap_constraints 交易对默认配置信息
     * 注意：
     * 1,手续账户的所有者要等于Swap常量配置中的所有者否则会抛出异常
     * 2,交易对持有账户必须是两个币以及手续费和目的地账户的所有者
     * 注意：交易对所有者应该是在前端用 findProgramAddress([swap_info.publicKey],program_id)函数生成的，而这个地址是没有私钥的，它只能在对应的program_id程序里面进行签名，再调用其它合约
     * 注意：这个地址的生成账户（也就是swap_info）在调用对应的program_id程序时，程序应该验证其网络中的签名，否则会存在漏洞
     * 3,有续费币种必须是池代币币种
     * 4,交易对持有账户必须可以铸造池代币
     * 5,池代币币种必须是一个新的且还没有生成过代币的币种（就是总量还是0的币种）
     */
    pub fn process_initialize(
        program_id: &Pubkey,
        fees: Fees,
        swap_curve: SwapCurve,
        accounts: &[AccountInfo],
        swap_constraints: &Option<SwapConstraints>,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        // 取迭代器里面的第1个账户为交易对信息
        let swap_info = next_account_info(account_info_iter)?;
        // 取迭代器里面的第2个账户为交易对所有者（注意：该账户要是两个币以及手续费以及目的地账户的所有者）
        // 注意：这个账户应该是在前端用 findProgramAddress([swap_info.publicKey],program_id)函数生成的，而这个地址是没有私钥的，它只能在对应的program_id程序里面进行签名，再调用其它合约
        // 注意：这个地址的生成账户（也就是swap_info）在调用对应的program_id程序时，程序应该验证其网络中的签名，否则会存在漏洞
        let authority_info = next_account_info(account_info_iter)?;
        // 取迭代器里面的第3个账户为交易对代币A流动性账户（后面会解码出来Account对象数据，就是持有来多少个代币A的数据）
        let token_a_info = next_account_info(account_info_iter)?;
        // 取迭代器里面的第4个账户为交易对代币B流动性账户（后面会解码出来Account对象数据，就是持有来多少个代币B的数据）
        let token_b_info = next_account_info(account_info_iter)?;
        // 取迭代器里面的第5个账户为池代币币种信息（注意：手续费账户的币种必须是这个）
        let pool_mint_info = next_account_info(account_info_iter)?;
        // 取迭代器里面的低6个账户为收取手续费账户（注意：这个账户要是某个代币的持有账户，就是该账户持有了多少个某个代币）
        let fee_account_info = next_account_info(account_info_iter)?;
        // 取迭代器里面的低7个账户为目的地账户（注意：这个账户用来装池代币，交易对池初始化时会为这个账户mint一些池代币）
        let destination_info = next_account_info(account_info_iter)?;
        // 取迭代器里面的第8个为发币程序账户信息（就是发币合约地址账户）
        let pool_token_program_info = next_account_info(account_info_iter)?;

        // Token合约地址
        let token_program_id = *pool_token_program_info.key;
        // 如果交易对已经初始化直接抛出异常
        if SwapVersion::is_initialized(&swap_info.data.borrow()) {
            return Err(SwapError::AlreadyInUse.into());
        }
        // 使用类似于前端的findProgramAddress([swap_info.publicKey],program_id)函数生成地址和种子
        let (swap_authority, bump_seed) = Pubkey::find_program_address(&[&swap_info.key.to_bytes()], program_id);
        // 验证生成的地址和传过来的所有者地址是不是一致，不一致的话抛出异常
        if *authority_info.key != swap_authority {
            return Err(SwapError::InvalidProgramAddress.into());
        }
        // 解码交易对代币A流动性账户信息（注意：这个函数里面实际是解码账户对应在合约上存储的实际数据）
        let token_a = Self::unpack_token_account(token_a_info, &token_program_id)?;
        // 解码交易对代币B流动性账户信息（注意：这个函数里面实际是解码账户对应在合约上存储的实际数据）
        let token_b = Self::unpack_token_account(token_b_info, &token_program_id)?;
        // 解码手续费币种的持有信息（注意：这个函数里面实际是解码账户对应在合约上存储的实际数据）
        let fee_account = Self::unpack_token_account(fee_account_info, &token_program_id)?;
        // 解码目的地币种的持有信息（注意：这个函数里面实际是解码账户对应在合约上存储的实际数据）
        let destination = Self::unpack_token_account(destination_info, &token_program_id)?;
        // 池代币币种信息对象（注意：只要可关闭铸造池代币权限地址不存在就正常返回池代币信息对象）
        let pool_mint = {
            let pool_mint_data = pool_mint_info.data.borrow();
            // 解码池代币对象
            let pool_mint = Self::unpack_mint_with_extensions(&pool_mint_data, pool_mint_info.owner, &token_program_id, )?;
            // 验证可关闭铸币权限地址是否不存在
            if let Ok(extension) = pool_mint.get_extension::<MintCloseAuthority>() {
                let close_authority: Option<Pubkey> = extension.close_authority.into();
                if close_authority.is_some() {
                    return Err(SwapError::InvalidCloseAuthority.into());
                }
            }
            pool_mint.base
        };
        // 判断交易对的所有者是不是交易对代币A流动性账户的所有者，如果不是抛出异常
        if *authority_info.key != token_a.owner {
            return Err(SwapError::InvalidOwner.into());
        }
        // 判断交易对的所有者是不是交易对代币B流动性账户的所有者，如果不是抛出异常
        if *authority_info.key != token_b.owner {
            return Err(SwapError::InvalidOwner.into());
        }
        // 判断交易对的所有者是不是目的地账户的所有者，如果不是抛出异常
        if *authority_info.key == destination.owner {
            return Err(SwapError::InvalidOutputOwner.into());
        }
        // 判断交易对的所有者是不是手续费账户的所有者，如果不是抛出异常
        if *authority_info.key == fee_account.owner {
            return Err(SwapError::InvalidOutputOwner.into());
        }
        // 判断交易对的所有者是不是可以铸造池代币，如果不可以抛出异常
        if COption::Some(*authority_info.key) != pool_mint.mint_authority {
            return Err(SwapError::InvalidOwner.into());
        }
        // 如果Token A 和 Token B是同一种代币抛出异常
        if token_a.mint == token_b.mint {
            return Err(SwapError::RepeatedMint.into());
        }
        // 交易对初始化时验证给定的总量
        swap_curve.calculator.validate_supply(token_a.amount, token_b.amount)?;
        // 如果Token A账户有授权地址抛出异常
        if token_a.delegate.is_some() {
            return Err(SwapError::InvalidDelegate.into());
        }
        // 如果交易对代币B流动性账户有授权地址抛出异常
        if token_b.delegate.is_some() {
            return Err(SwapError::InvalidDelegate.into());
        }
        // 如果交易对代币A流动性账户有可以关闭该账户的地址则抛出异常
        if token_a.close_authority.is_some() {
            return Err(SwapError::InvalidCloseAuthority.into());
        }
        // 如果交易对代币B流动性账户有可以关闭该账户的地址则抛出异常
        if token_b.close_authority.is_some() {
            return Err(SwapError::InvalidCloseAuthority.into());
        }
        // 池代币币种的总量必须为0否则抛出异常
        if pool_mint.supply != 0 {
            return Err(SwapError::InvalidSupply.into());
        }
        // 如果池代币的可回收地址存在则抛出异常
        if pool_mint.freeze_authority.is_some() {
            return Err(SwapError::InvalidFreezeAuthority.into());
        }
        // 如果手续费币种不等于池代币币种则抛出异常
        if *pool_mint_info.key != fee_account.mint {
            return Err(SwapError::IncorrectPoolMint.into());
        }
        // 如果交易对常量配置信息不为空
        if let Some(swap_constraints) = swap_constraints {
            // 获取Swap常量配置中的 所有者地址（注意：这个所有者地址是一个系统环境变量）
            let owner_key = swap_constraints.owner_key.parse::<Pubkey>().map_err(|_| SwapError::InvalidOwner)?;
            // 如果手续账户的所有者不等于Swap常量配置中的所有者 则抛出异常
            if fee_account.owner != owner_key {
                return Err(SwapError::InvalidOwner.into());
            }
            // 验证交易对价格计算实现
            swap_constraints.validate_curve(&swap_curve)?;
            // 验证费用配置
            swap_constraints.validate_fees(&fees)?;
        }
        // 验证费用信息
        fees.validate()?;
        // 验证交易对价格计算实现
        swap_curve.calculator.validate()?;
        // 交易对池的总量
        let initial_amount = swap_curve.calculator.new_pool_supply();

        // 调用Token合约里面的mint_to函数为目的地持币账户铸造池代币（铸造数量就是交易对池的总量）
        Self::token_mint_to(
            swap_info.key,
            pool_token_program_info.clone(),
            pool_mint_info.clone(),
            destination_info.clone(),
            authority_info.clone(),
            bump_seed,
            to_u64(initial_amount)?,
        )?;

        let obj = SwapVersion::SwapV1(SwapV1 {
            // 交易对是否已经初始化
            is_initialized: true,
            // 交易对信息账户的种子（如果在合约里面要调用另一个合约可以使用该种子签名）
            bump_seed,
            // Token合约地址
            token_program_id,
            // 交易对代币A流动性账户
            token_a: *token_a_info.key,
            // 交易对代币B流动性账户
            token_b: *token_b_info.key,
            // 池代币币种信息地址
            pool_mint: *pool_mint_info.key,
            // Token A币种信息地址
            token_a_mint: token_a.mint,
            // Token B币种信息地址
            token_b_mint: token_b.mint,
            // 交易和取款手续费存放账户地址（注意：该币种需和池代币币种相同）
            pool_fee_account: *fee_account_info.key,
            // 费用相关配置
            fees,
            // 代币兑换价格计算实现
            swap_curve,
        });
        // 保存数据
        SwapVersion::pack(obj, &mut swap_info.data.borrow_mut())?;
        Ok(())
    }

    /// Processes an [Swap](enum.Instruction.html).
    /**
     * 兑换
     * @amount_in          转入金额
     * @minimum_amount_out 最少兑换到数量（如果实际兑换数量小于该数量直接抛出异常）
     */
    pub fn process_swap(
        program_id: &Pubkey,
        amount_in: u64,
        minimum_amount_out: u64,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        // 取迭代器里面的第1个为交易对信息账户
        let swap_info = next_account_info(account_info_iter)?;
        // 取迭代器里面的第2个为交易对所有者账户
        let authority_info = next_account_info(account_info_iter)?;
        // 取迭代器里面的第3个为转出地址所有者账户
        let user_transfer_authority_info = next_account_info(account_info_iter)?;
        // 取迭代器里面的第4个为个人转出账户
        let source_info = next_account_info(account_info_iter)?;
        // 取迭代器里面的第5个为交易对代币A流动性账户
        let swap_source_info = next_account_info(account_info_iter)?;
        // 取迭代器里面的第6个为交易对代币B流动性账户
        let swap_destination_info = next_account_info(account_info_iter)?;
        // 取迭代器里面的第7个为个人转入账户
        let destination_info = next_account_info(account_info_iter)?;
        // 取迭代器里面的第8个为交易对池代币信息账户
        let pool_mint_info = next_account_info(account_info_iter)?;
        // 取迭代器里面的第9个为交易手续费收取账户
        let pool_fee_account_info = next_account_info(account_info_iter)?;
        // 取迭代器里面的第10个为源币种信息账户
        let source_token_mint_info = next_account_info(account_info_iter)?;
        // 取迭代器里面的第11个为目标币种信息账户
        let destination_token_mint_info = next_account_info(account_info_iter)?;
        // 取迭代器里面的第12个为源币种程序信息
        let source_token_program_info = next_account_info(account_info_iter)?;
        // 取迭代器里面的第13个为目标币种程序信息
        let destination_token_program_info = next_account_info(account_info_iter)?;
        // 取迭代器里面的第14个为代币程序地址
        let pool_token_program_info = next_account_info(account_info_iter)?;

        // 交易对信息账户如果不属于当前程序直接抛出异常
        if swap_info.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }
        // 解码交易对信息账户
        let token_swap = SwapVersion::unpack(&swap_info.data.borrow())?;
        // 通过交易对信息账户的key和已经存储的交易对信息账户的种子来反向计算交易对信息账户的所有者；如果和传过的所有者不一致抛出异常
        if *authority_info.key != Self::authority_id(program_id, swap_info.key, token_swap.bump_seed())? {
            return Err(SwapError::InvalidProgramAddress.into());
        }
        // 如果传过来的交易对流动性账户A 不属于 交易对信息账户里面的流动性账户A或B 抛出异常
        if !(*swap_source_info.key == *token_swap.token_a_account() || *swap_source_info.key == *token_swap.token_b_account()) {
            return Err(SwapError::IncorrectSwapAccount.into());
        }
        // 如果传过来的交易对流动性账户B 不属于 交易对信息账户里面的流动性账户A或B 抛出异常
        if !(*swap_destination_info.key == *token_swap.token_a_account() || *swap_destination_info.key == *token_swap.token_b_account()) {
            return Err(SwapError::IncorrectSwapAccount.into());
        }
        // 如果传过来的交易对流动性账户A和B相等，说明是同币种兑换，直接抛出异常
        if *swap_source_info.key == *swap_destination_info.key {
            return Err(SwapError::InvalidInput.into());
        }
        // 如果传过来的交易对流动性账户A 等于 个人转出账户 抛出异常
        if swap_source_info.key == source_info.key {
            return Err(SwapError::InvalidInput.into());
        }
        // 如果传过来的交易对流动性账户B 等于 转入账户 抛出异常
        if swap_destination_info.key == destination_info.key {
            return Err(SwapError::InvalidInput.into());
        }
        // 如果传过来的池代币账户 不等于 交易对信息里面的池代币账户 抛出异常
        if *pool_mint_info.key != *token_swap.pool_mint() {
            return Err(SwapError::IncorrectPoolMint.into());
        }
        // 如果传过来的手续费收取账户 不等于 交易对信息里面手续费收取账户 抛出异常
        if *pool_fee_account_info.key != *token_swap.pool_fee_account() {
            return Err(SwapError::IncorrectFeeAccount.into());
        }
        // 如果传过来的代币程序地址 不等于 交易对信息里面的程序地址 抛出异常
        if *pool_token_program_info.key != *token_swap.token_program_id() {
            return Err(SwapError::IncorrectTokenProgramId.into());
        }

        // 解码交易对代币A流动性账户
        let source_account = Self::unpack_token_account(swap_source_info, token_swap.token_program_id())?;
        // 解码交易对代币B流动性账户
        let dest_account = Self::unpack_token_account(swap_destination_info, token_swap.token_program_id())?;
        // 解码交易对池代币信息账户
        let pool_mint = Self::unpack_mint(pool_mint_info, token_swap.token_program_id())?;

        // Take transfer fees into account for actual amount transferred in
        // 计算实际源币种转出数量考虑转入费
        let actual_amount_in = {
            // 源币种信息账户数据
            let source_mint_data = source_token_mint_info.data.borrow();
            // 解码源币种信息
            let source_mint = Self::unpack_mint_with_extensions(
                &source_mint_data,
                source_token_mint_info.owner,
                token_swap.token_program_id(),
            )?;
            // 如果源币种有转账配置 则 计算实际源币种可用数量
            if let Ok(transfer_fee_config) = source_mint.get_extension::<TransferFeeConfig>() {
                amount_in.saturating_sub(
                    transfer_fee_config
                        .calculate_epoch_fee(Clock::get()?.epoch, amount_in)
                        .ok_or(SwapError::FeeCalculationFailure)?,
                )
            } else {
                amount_in
            }
        };

        // Calculate the trade amounts
        // 计算交易方向
        let trade_direction = if *swap_source_info.key == *token_swap.token_a_account() {
            TradeDirection::AtoB
        } else {
            TradeDirection::BtoA
        };
        // 兑换并得到兑换结果
        let result = token_swap
            .swap_curve()
            .swap(
                to_u128(actual_amount_in)?,
                to_u128(source_account.amount)?,
                to_u128(dest_account.amount)?,
                trade_direction,
                token_swap.fees(),
            )
            .ok_or(SwapError::ZeroTradingTokens)?;

        // Re-calculate the source amount swapped based on what the curve says
        // 计算实际兑换源币种转出数量（可能池中没有钱实际可兑换的转出数量小于传入的转出数量）
        let (source_transfer_amount, source_mint_decimals) = {
            // 实际兑换源币种转出数量（可能池中没有钱实际可兑换的转出数量小于传入的转出数量）
            let source_amount_swapped = to_u64(result.source_amount_swapped)?;
            // 源币种信息账户数据
            let source_mint_data = source_token_mint_info.data.borrow();
            // 解码源币种信息
            let source_mint = Self::unpack_mint_with_extensions(
                &source_mint_data,
                source_token_mint_info.owner,
                token_swap.token_program_id(),
            )?;
            // 如果源币种有转账配置 则 计算实际源币种可用数量
            let amount =
                if let Ok(transfer_fee_config) = source_mint.get_extension::<TransferFeeConfig>() {
                    source_amount_swapped.saturating_add(
                        transfer_fee_config
                            .calculate_epoch_fee(Clock::get()?.epoch, source_amount_swapped)
                            // 由于版本问题下面这个函数没有，用上面函数替代
                            //.calculate_inverse_epoch_fee(Clock::get()?.epoch, source_amount_swapped)
                            .ok_or(SwapError::FeeCalculationFailure)?,
                    )
                } else {
                    source_amount_swapped
                };
            (amount, source_mint.base.decimals)
        };
        // 计算实际兑换目标币种转入数量（就是实际可兑换到的目标数量）
        let (destination_transfer_amount, destination_mint_decimals) = {
            // 目标币种信息账户数据
            let destination_mint_data = destination_token_mint_info.data.borrow();
            // 解码目标币种信息
            let destination_mint = Self::unpack_mint_with_extensions(
                &destination_mint_data,
                source_token_mint_info.owner,
                token_swap.token_program_id(),
            )?;
            // 实际兑换目标币种转入数量（就是可兑换到的目标数量）
            let amount_out = to_u64(result.destination_amount_swapped)?;
            // 如果目标币种有转账配置 则 计算目标币种实际可接收数量
            let amount_received = if let Ok(transfer_fee_config) = destination_mint.get_extension::<TransferFeeConfig>() {
                amount_out.saturating_sub(
                    transfer_fee_config
                        .calculate_epoch_fee(Clock::get()?.epoch, amount_out)
                        .ok_or(SwapError::FeeCalculationFailure)?,
                )
            } else {
                amount_out
            };
            // 如果实际兑换数量 小于 传入的最少兑换到数量 直接抛出异常
            if amount_received < minimum_amount_out {
                return Err(SwapError::ExceededSlippage.into());
            }
            (amount_out, destination_mint.base.decimals)
        };
        // 根据交易方向得到
        let (swap_token_a_amount, swap_token_b_amount) = match trade_direction {
            TradeDirection::AtoB => (
                result.new_swap_source_amount,
                result.new_swap_destination_amount,
            ),
            TradeDirection::BtoA => (
                result.new_swap_destination_amount,
                result.new_swap_source_amount,
            ),
        };

        // 将源币种转出数量 转到 交易对代币A流动性账户上
        Self::token_transfer(
            // 交易对信息账户地址
            swap_info.key,
            // 源币种程序信息
            source_token_program_info.clone(),
            // 个人转出账户
            source_info.clone(),
            // 源币种信息
            source_token_mint_info.clone(),
            // 交易对代币A流动性账户
            swap_source_info.clone(),
            // 个人转出地址所有者账户
            user_transfer_authority_info.clone(),
            // 交易对信息账户的种子
            token_swap.bump_seed(),
            // 实际兑换源币种转出数量
            source_transfer_amount,
            // 源币种精度
            source_mint_decimals,
        )?;
        /*************************** curve 算法逻辑，可不需要解读 Start ****************************************/
        let mut pool_token_amount = token_swap.swap_curve().withdraw_single_token_type_exact_out(
                result.owner_fee,
                swap_token_a_amount,
                swap_token_b_amount,
                to_u128(pool_mint.supply)?,
                trade_direction,
                token_swap.fees(),
            ).ok_or(SwapError::FeeCalculationFailure)?;

        if pool_token_amount > 0 {
            // Allow error to fall through
            if let Ok(host_fee_account_info) = next_account_info(account_info_iter) {

                let host_fee_account = Self::unpack_token_account(host_fee_account_info, token_swap.token_program_id(), )?;

                if *pool_mint_info.key != host_fee_account.mint {
                    return Err(SwapError::IncorrectPoolMint.into());
                }
                let host_fee = token_swap.fees().host_fee(pool_token_amount).ok_or(SwapError::FeeCalculationFailure)?;

                if host_fee > 0 {
                    pool_token_amount = pool_token_amount.checked_sub(host_fee).ok_or(SwapError::FeeCalculationFailure)?;
                    Self::token_mint_to(
                        swap_info.key,
                        pool_token_program_info.clone(),
                        pool_mint_info.clone(),
                        host_fee_account_info.clone(),
                        authority_info.clone(),
                        token_swap.bump_seed(),
                        to_u64(host_fee)?,
                    )?;
                }
            }
            if token_swap.check_pool_fee_info(pool_fee_account_info).is_ok() {
                Self::token_mint_to(
                    swap_info.key,
                    pool_token_program_info.clone(),
                    pool_mint_info.clone(),
                    pool_fee_account_info.clone(),
                    authority_info.clone(),
                    token_swap.bump_seed(),
                    to_u64(pool_token_amount)?,
                )?;
            };
        }
        /*************************** curve 算法逻辑，可不需要解读 End ****************************************/

        // 从交易对代币B流动性账户将钱转到用户账户，至此交易完成
        Self::token_transfer(
            // 交易对信息账户地址
            swap_info.key,
            // 目标币种程序信息
            destination_token_program_info.clone(),
            // 交易对代币B流动性账户
            swap_destination_info.clone(),
            // 目标币种信息账户
            destination_token_mint_info.clone(),
            // 个人转入账户（就是收款账户）
            destination_info.clone(),
            // 交易对所有者账户
            authority_info.clone(),
            // 交易对信息账户的种子
            token_swap.bump_seed(),
            // 实际兑换目标币种转入数量
            destination_transfer_amount,
            // 目标币种精度
            destination_mint_decimals,
        )?;

        Ok(())
    }

    /// Processes an [DepositAllTokenTypes](enum.Instruction.html).
    pub fn process_deposit_all_token_types(
        program_id: &Pubkey,
        pool_token_amount: u64,
        maximum_token_a_amount: u64,
        maximum_token_b_amount: u64,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let swap_info = next_account_info(account_info_iter)?;
        let authority_info = next_account_info(account_info_iter)?;
        let user_transfer_authority_info = next_account_info(account_info_iter)?;
        let source_a_info = next_account_info(account_info_iter)?;
        let source_b_info = next_account_info(account_info_iter)?;
        let token_a_info = next_account_info(account_info_iter)?;
        let token_b_info = next_account_info(account_info_iter)?;
        let pool_mint_info = next_account_info(account_info_iter)?;
        let dest_info = next_account_info(account_info_iter)?;
        let token_a_mint_info = next_account_info(account_info_iter)?;
        let token_b_mint_info = next_account_info(account_info_iter)?;
        let token_a_program_info = next_account_info(account_info_iter)?;
        let token_b_program_info = next_account_info(account_info_iter)?;
        let pool_token_program_info = next_account_info(account_info_iter)?;

        let token_swap = SwapVersion::unpack(&swap_info.data.borrow())?;
        let calculator = &token_swap.swap_curve().calculator;
        if !calculator.allows_deposits() {
            return Err(SwapError::UnsupportedCurveOperation.into());
        }
        Self::check_accounts(
            token_swap.as_ref(),
            program_id,
            swap_info,
            authority_info,
            token_a_info,
            token_b_info,
            pool_mint_info,
            pool_token_program_info,
            Some(source_a_info),
            Some(source_b_info),
            None,
        )?;

        let token_a = Self::unpack_token_account(token_a_info, token_swap.token_program_id())?;
        let token_b = Self::unpack_token_account(token_b_info, token_swap.token_program_id())?;
        let pool_mint = Self::unpack_mint(pool_mint_info, token_swap.token_program_id())?;
        let current_pool_mint_supply = to_u128(pool_mint.supply)?;
        let (pool_token_amount, pool_mint_supply) = if current_pool_mint_supply > 0 {
            (to_u128(pool_token_amount)?, current_pool_mint_supply)
        } else {
            (calculator.new_pool_supply(), calculator.new_pool_supply())
        };

        let results = calculator
            .pool_tokens_to_trading_tokens(
                pool_token_amount,
                pool_mint_supply,
                to_u128(token_a.amount)?,
                to_u128(token_b.amount)?,
                RoundDirection::Ceiling,
            )
            .ok_or(SwapError::ZeroTradingTokens)?;
        let token_a_amount = to_u64(results.token_a_amount)?;
        if token_a_amount > maximum_token_a_amount {
            return Err(SwapError::ExceededSlippage.into());
        }
        if token_a_amount == 0 {
            return Err(SwapError::ZeroTradingTokens.into());
        }
        let token_b_amount = to_u64(results.token_b_amount)?;
        if token_b_amount > maximum_token_b_amount {
            return Err(SwapError::ExceededSlippage.into());
        }
        if token_b_amount == 0 {
            return Err(SwapError::ZeroTradingTokens.into());
        }

        let pool_token_amount = to_u64(pool_token_amount)?;

        Self::token_transfer(
            swap_info.key,
            token_a_program_info.clone(),
            source_a_info.clone(),
            token_a_mint_info.clone(),
            token_a_info.clone(),
            user_transfer_authority_info.clone(),
            token_swap.bump_seed(),
            token_a_amount,
            Self::unpack_mint(token_a_mint_info, token_swap.token_program_id())?.decimals,
        )?;
        Self::token_transfer(
            swap_info.key,
            token_b_program_info.clone(),
            source_b_info.clone(),
            token_b_mint_info.clone(),
            token_b_info.clone(),
            user_transfer_authority_info.clone(),
            token_swap.bump_seed(),
            token_b_amount,
            Self::unpack_mint(token_b_mint_info, token_swap.token_program_id())?.decimals,
        )?;
        Self::token_mint_to(
            swap_info.key,
            pool_token_program_info.clone(),
            pool_mint_info.clone(),
            dest_info.clone(),
            authority_info.clone(),
            token_swap.bump_seed(),
            pool_token_amount,
        )?;

        Ok(())
    }

    /// Processes an [WithdrawAllTokenTypes](enum.Instruction.html).
    pub fn process_withdraw_all_token_types(
        program_id: &Pubkey,
        pool_token_amount: u64,
        minimum_token_a_amount: u64,
        minimum_token_b_amount: u64,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let swap_info = next_account_info(account_info_iter)?;
        let authority_info = next_account_info(account_info_iter)?;
        let user_transfer_authority_info = next_account_info(account_info_iter)?;
        let pool_mint_info = next_account_info(account_info_iter)?;
        let source_info = next_account_info(account_info_iter)?;
        let token_a_info = next_account_info(account_info_iter)?;
        let token_b_info = next_account_info(account_info_iter)?;
        let dest_token_a_info = next_account_info(account_info_iter)?;
        let dest_token_b_info = next_account_info(account_info_iter)?;
        let pool_fee_account_info = next_account_info(account_info_iter)?;
        let token_a_mint_info = next_account_info(account_info_iter)?;
        let token_b_mint_info = next_account_info(account_info_iter)?;
        let pool_token_program_info = next_account_info(account_info_iter)?;
        let token_a_program_info = next_account_info(account_info_iter)?;
        let token_b_program_info = next_account_info(account_info_iter)?;

        let token_swap = SwapVersion::unpack(&swap_info.data.borrow())?;
        Self::check_accounts(
            token_swap.as_ref(),
            program_id,
            swap_info,
            authority_info,
            token_a_info,
            token_b_info,
            pool_mint_info,
            pool_token_program_info,
            Some(dest_token_a_info),
            Some(dest_token_b_info),
            Some(pool_fee_account_info),
        )?;

        let token_a = Self::unpack_token_account(token_a_info, token_swap.token_program_id())?;
        let token_b = Self::unpack_token_account(token_b_info, token_swap.token_program_id())?;
        let pool_mint = Self::unpack_mint(pool_mint_info, token_swap.token_program_id())?;

        let calculator = &token_swap.swap_curve().calculator;

        let withdraw_fee = match token_swap.check_pool_fee_info(pool_fee_account_info) {
            Ok(_) => {
                if *pool_fee_account_info.key == *source_info.key {
                    // withdrawing from the fee account, don't assess withdraw fee
                    0
                } else {
                    token_swap
                        .fees()
                        .owner_withdraw_fee(to_u128(pool_token_amount)?)
                        .ok_or(SwapError::FeeCalculationFailure)?
                }
            }
            Err(_) => 0,
        };
        let pool_token_amount = to_u128(pool_token_amount)?
            .checked_sub(withdraw_fee)
            .ok_or(SwapError::CalculationFailure)?;

        let results = calculator
            .pool_tokens_to_trading_tokens(
                pool_token_amount,
                to_u128(pool_mint.supply)?,
                to_u128(token_a.amount)?,
                to_u128(token_b.amount)?,
                RoundDirection::Floor,
            )
            .ok_or(SwapError::ZeroTradingTokens)?;
        let token_a_amount = to_u64(results.token_a_amount)?;
        let token_a_amount = std::cmp::min(token_a.amount, token_a_amount);
        if token_a_amount < minimum_token_a_amount {
            return Err(SwapError::ExceededSlippage.into());
        }
        if token_a_amount == 0 && token_a.amount != 0 {
            return Err(SwapError::ZeroTradingTokens.into());
        }
        let token_b_amount = to_u64(results.token_b_amount)?;
        let token_b_amount = std::cmp::min(token_b.amount, token_b_amount);
        if token_b_amount < minimum_token_b_amount {
            return Err(SwapError::ExceededSlippage.into());
        }
        if token_b_amount == 0 && token_b.amount != 0 {
            return Err(SwapError::ZeroTradingTokens.into());
        }

        if withdraw_fee > 0 {
            Self::token_transfer(
                swap_info.key,
                pool_token_program_info.clone(),
                source_info.clone(),
                pool_mint_info.clone(),
                pool_fee_account_info.clone(),
                user_transfer_authority_info.clone(),
                token_swap.bump_seed(),
                to_u64(withdraw_fee)?,
                pool_mint.decimals,
            )?;
        }
        Self::token_burn(
            swap_info.key,
            pool_token_program_info.clone(),
            source_info.clone(),
            pool_mint_info.clone(),
            user_transfer_authority_info.clone(),
            token_swap.bump_seed(),
            to_u64(pool_token_amount)?,
        )?;

        if token_a_amount > 0 {
            Self::token_transfer(
                swap_info.key,
                token_a_program_info.clone(),
                token_a_info.clone(),
                token_a_mint_info.clone(),
                dest_token_a_info.clone(),
                authority_info.clone(),
                token_swap.bump_seed(),
                token_a_amount,
                Self::unpack_mint(token_a_mint_info, token_swap.token_program_id())?.decimals,
            )?;
        }
        if token_b_amount > 0 {
            Self::token_transfer(
                swap_info.key,
                token_b_program_info.clone(),
                token_b_info.clone(),
                token_b_mint_info.clone(),
                dest_token_b_info.clone(),
                authority_info.clone(),
                token_swap.bump_seed(),
                token_b_amount,
                Self::unpack_mint(token_b_mint_info, token_swap.token_program_id())?.decimals,
            )?;
        }
        Ok(())
    }

    /// Processes DepositSingleTokenTypeExactAmountIn
    pub fn process_deposit_single_token_type_exact_amount_in(
        program_id: &Pubkey,
        source_token_amount: u64,
        minimum_pool_token_amount: u64,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let swap_info = next_account_info(account_info_iter)?;
        let authority_info = next_account_info(account_info_iter)?;
        let user_transfer_authority_info = next_account_info(account_info_iter)?;
        let source_info = next_account_info(account_info_iter)?;
        let swap_token_a_info = next_account_info(account_info_iter)?;
        let swap_token_b_info = next_account_info(account_info_iter)?;
        let pool_mint_info = next_account_info(account_info_iter)?;
        let destination_info = next_account_info(account_info_iter)?;
        let source_token_mint_info = next_account_info(account_info_iter)?;
        let source_token_program_info = next_account_info(account_info_iter)?;
        let pool_token_program_info = next_account_info(account_info_iter)?;

        let token_swap = SwapVersion::unpack(&swap_info.data.borrow())?;
        let calculator = &token_swap.swap_curve().calculator;
        if !calculator.allows_deposits() {
            return Err(SwapError::UnsupportedCurveOperation.into());
        }
        let source_account =
            Self::unpack_token_account(source_info, token_swap.token_program_id())?;
        let swap_token_a =
            Self::unpack_token_account(swap_token_a_info, token_swap.token_program_id())?;
        let swap_token_b =
            Self::unpack_token_account(swap_token_b_info, token_swap.token_program_id())?;

        let trade_direction = if source_account.mint == swap_token_a.mint {
            TradeDirection::AtoB
        } else if source_account.mint == swap_token_b.mint {
            TradeDirection::BtoA
        } else {
            return Err(SwapError::IncorrectSwapAccount.into());
        };

        let (source_a_info, source_b_info) = match trade_direction {
            TradeDirection::AtoB => (Some(source_info), None),
            TradeDirection::BtoA => (None, Some(source_info)),
        };

        Self::check_accounts(
            token_swap.as_ref(),
            program_id,
            swap_info,
            authority_info,
            swap_token_a_info,
            swap_token_b_info,
            pool_mint_info,
            pool_token_program_info,
            source_a_info,
            source_b_info,
            None,
        )?;

        let pool_mint = Self::unpack_mint(pool_mint_info, token_swap.token_program_id())?;
        let pool_mint_supply = to_u128(pool_mint.supply)?;
        let pool_token_amount = if pool_mint_supply > 0 {
            token_swap
                .swap_curve()
                .deposit_single_token_type(
                    to_u128(source_token_amount)?,
                    to_u128(swap_token_a.amount)?,
                    to_u128(swap_token_b.amount)?,
                    pool_mint_supply,
                    trade_direction,
                    token_swap.fees(),
                )
                .ok_or(SwapError::ZeroTradingTokens)?
        } else {
            calculator.new_pool_supply()
        };

        let pool_token_amount = to_u64(pool_token_amount)?;
        if pool_token_amount < minimum_pool_token_amount {
            return Err(SwapError::ExceededSlippage.into());
        }
        if pool_token_amount == 0 {
            return Err(SwapError::ZeroTradingTokens.into());
        }

        match trade_direction {
            TradeDirection::AtoB => {
                Self::token_transfer(
                    swap_info.key,
                    source_token_program_info.clone(),
                    source_info.clone(),
                    source_token_mint_info.clone(),
                    swap_token_a_info.clone(),
                    user_transfer_authority_info.clone(),
                    token_swap.bump_seed(),
                    source_token_amount,
                    Self::unpack_mint(source_token_mint_info, token_swap.token_program_id())?
                        .decimals,
                )?;
            }
            TradeDirection::BtoA => {
                Self::token_transfer(
                    swap_info.key,
                    source_token_program_info.clone(),
                    source_info.clone(),
                    source_token_mint_info.clone(),
                    swap_token_b_info.clone(),
                    user_transfer_authority_info.clone(),
                    token_swap.bump_seed(),
                    source_token_amount,
                    Self::unpack_mint(source_token_mint_info, token_swap.token_program_id())?
                        .decimals,
                )?;
            }
        }
        Self::token_mint_to(
            swap_info.key,
            pool_token_program_info.clone(),
            pool_mint_info.clone(),
            destination_info.clone(),
            authority_info.clone(),
            token_swap.bump_seed(),
            pool_token_amount,
        )?;

        Ok(())
    }

    /// Processes a [WithdrawSingleTokenTypeExactAmountOut](enum.Instruction.html).
    pub fn process_withdraw_single_token_type_exact_amount_out(
        program_id: &Pubkey,
        destination_token_amount: u64,
        maximum_pool_token_amount: u64,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let swap_info = next_account_info(account_info_iter)?;
        let authority_info = next_account_info(account_info_iter)?;
        let user_transfer_authority_info = next_account_info(account_info_iter)?;
        let pool_mint_info = next_account_info(account_info_iter)?;
        let source_info = next_account_info(account_info_iter)?;
        let swap_token_a_info = next_account_info(account_info_iter)?;
        let swap_token_b_info = next_account_info(account_info_iter)?;
        let destination_info = next_account_info(account_info_iter)?;
        let pool_fee_account_info = next_account_info(account_info_iter)?;
        let destination_token_mint_info = next_account_info(account_info_iter)?;
        let pool_token_program_info = next_account_info(account_info_iter)?;
        let destination_token_program_info = next_account_info(account_info_iter)?;

        let token_swap = SwapVersion::unpack(&swap_info.data.borrow())?;
        let destination_account =
            Self::unpack_token_account(destination_info, token_swap.token_program_id())?;
        let swap_token_a =
            Self::unpack_token_account(swap_token_a_info, token_swap.token_program_id())?;
        let swap_token_b =
            Self::unpack_token_account(swap_token_b_info, token_swap.token_program_id())?;

        let trade_direction = if destination_account.mint == swap_token_a.mint {
            TradeDirection::AtoB
        } else if destination_account.mint == swap_token_b.mint {
            TradeDirection::BtoA
        } else {
            return Err(SwapError::IncorrectSwapAccount.into());
        };

        let (destination_a_info, destination_b_info) = match trade_direction {
            TradeDirection::AtoB => (Some(destination_info), None),
            TradeDirection::BtoA => (None, Some(destination_info)),
        };
        Self::check_accounts(
            token_swap.as_ref(),
            program_id,
            swap_info,
            authority_info,
            swap_token_a_info,
            swap_token_b_info,
            pool_mint_info,
            pool_token_program_info,
            destination_a_info,
            destination_b_info,
            Some(pool_fee_account_info),
        )?;

        let pool_mint = Self::unpack_mint(pool_mint_info, token_swap.token_program_id())?;
        let pool_mint_supply = to_u128(pool_mint.supply)?;
        let swap_token_a_amount = to_u128(swap_token_a.amount)?;
        let swap_token_b_amount = to_u128(swap_token_b.amount)?;

        let burn_pool_token_amount = token_swap
            .swap_curve()
            .withdraw_single_token_type_exact_out(
                to_u128(destination_token_amount)?,
                swap_token_a_amount,
                swap_token_b_amount,
                pool_mint_supply,
                trade_direction,
                token_swap.fees(),
            )
            .ok_or(SwapError::ZeroTradingTokens)?;

        let withdraw_fee = match token_swap.check_pool_fee_info(pool_fee_account_info) {
            Ok(_) => {
                if *pool_fee_account_info.key == *source_info.key {
                    // withdrawing from the fee account, don't assess withdraw fee
                    0
                } else {
                    token_swap
                        .fees()
                        .owner_withdraw_fee(burn_pool_token_amount)
                        .ok_or(SwapError::FeeCalculationFailure)?
                }
            }
            Err(_) => 0,
        };
        let pool_token_amount = burn_pool_token_amount
            .checked_add(withdraw_fee)
            .ok_or(SwapError::CalculationFailure)?;

        if to_u64(pool_token_amount)? > maximum_pool_token_amount {
            return Err(SwapError::ExceededSlippage.into());
        }
        if pool_token_amount == 0 {
            return Err(SwapError::ZeroTradingTokens.into());
        }

        if withdraw_fee > 0 {
            Self::token_transfer(
                swap_info.key,
                pool_token_program_info.clone(),
                source_info.clone(),
                pool_mint_info.clone(),
                pool_fee_account_info.clone(),
                user_transfer_authority_info.clone(),
                token_swap.bump_seed(),
                to_u64(withdraw_fee)?,
                pool_mint.decimals,
            )?;
        }
        Self::token_burn(
            swap_info.key,
            pool_token_program_info.clone(),
            source_info.clone(),
            pool_mint_info.clone(),
            user_transfer_authority_info.clone(),
            token_swap.bump_seed(),
            to_u64(burn_pool_token_amount)?,
        )?;

        match trade_direction {
            TradeDirection::AtoB => {
                Self::token_transfer(
                    swap_info.key,
                    destination_token_program_info.clone(),
                    swap_token_a_info.clone(),
                    destination_token_mint_info.clone(),
                    destination_info.clone(),
                    authority_info.clone(),
                    token_swap.bump_seed(),
                    destination_token_amount,
                    Self::unpack_mint(destination_token_mint_info, token_swap.token_program_id())?
                        .decimals,
                )?;
            }
            TradeDirection::BtoA => {
                Self::token_transfer(
                    swap_info.key,
                    destination_token_program_info.clone(),
                    swap_token_b_info.clone(),
                    destination_token_mint_info.clone(),
                    destination_info.clone(),
                    authority_info.clone(),
                    token_swap.bump_seed(),
                    destination_token_amount,
                    Self::unpack_mint(destination_token_mint_info, token_swap.token_program_id())?
                        .decimals,
                )?;
            }
        }

        Ok(())
    }

    /// Processes an [Instruction](enum.Instruction.html).
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        Self::process_with_constraints(program_id, accounts, input, &SWAP_CONSTRAINTS)
    }

    /// Processes an instruction given extra constraint
    pub fn process_with_constraints(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        input: &[u8],
        swap_constraints: &Option<SwapConstraints>,
    ) -> ProgramResult {
        let instruction = SwapInstruction::unpack(input)?;
        match instruction {
            SwapInstruction::Initialize(Initialize { fees, swap_curve }) => {
                msg!("Instruction: Init");
                Self::process_initialize(program_id, fees, swap_curve, accounts, swap_constraints)
            }
            SwapInstruction::Swap(Swap {
                amount_in,
                minimum_amount_out,
            }) => {
                msg!("Instruction: Swap");
                Self::process_swap(program_id, amount_in, minimum_amount_out, accounts)
            }
            SwapInstruction::DepositAllTokenTypes(DepositAllTokenTypes {
                pool_token_amount,
                maximum_token_a_amount,
                maximum_token_b_amount,
            }) => {
                msg!("Instruction: DepositAllTokenTypes");
                Self::process_deposit_all_token_types(
                    program_id,
                    pool_token_amount,
                    maximum_token_a_amount,
                    maximum_token_b_amount,
                    accounts,
                )
            }
            SwapInstruction::WithdrawAllTokenTypes(WithdrawAllTokenTypes {
                pool_token_amount,
                minimum_token_a_amount,
                minimum_token_b_amount,
            }) => {
                msg!("Instruction: WithdrawAllTokenTypes");
                Self::process_withdraw_all_token_types(
                    program_id,
                    pool_token_amount,
                    minimum_token_a_amount,
                    minimum_token_b_amount,
                    accounts,
                )
            }
            SwapInstruction::DepositSingleTokenTypeExactAmountIn(
                DepositSingleTokenTypeExactAmountIn {
                    source_token_amount,
                    minimum_pool_token_amount,
                },
            ) => {
                msg!("Instruction: DepositSingleTokenTypeExactAmountIn");
                Self::process_deposit_single_token_type_exact_amount_in(
                    program_id,
                    source_token_amount,
                    minimum_pool_token_amount,
                    accounts,
                )
            }
            SwapInstruction::WithdrawSingleTokenTypeExactAmountOut(
                WithdrawSingleTokenTypeExactAmountOut {
                    destination_token_amount,
                    maximum_pool_token_amount,
                },
            ) => {
                msg!("Instruction: WithdrawSingleTokenTypeExactAmountOut");
                Self::process_withdraw_single_token_type_exact_amount_out(
                    program_id,
                    destination_token_amount,
                    maximum_pool_token_amount,
                    accounts,
                )
            }
        }
    }
}

fn to_u128(val: u64) -> Result<u128, SwapError> {
    val.try_into().map_err(|_| SwapError::ConversionFailure)
}

fn to_u64(val: u128) -> Result<u64, SwapError> {
    val.try_into().map_err(|_| SwapError::ConversionFailure)
}

fn invoke_signed_wrapper<T>(
    instruction: &Instruction,
    account_infos: &[AccountInfo],
    signers_seeds: &[&[&[u8]]],
) -> Result<(), ProgramError>
where
    T: 'static + PrintProgramError + DecodeError<T> + FromPrimitive + Error,
{
    invoke_signed(instruction, account_infos, signers_seeds).map_err(|err| {
        err.print::<T>();
        err
    })
}
