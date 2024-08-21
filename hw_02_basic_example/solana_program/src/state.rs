
use std::str::from_utf8;
use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use num_enum::TryFromPrimitive;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::{IsInitialized, Pack, Sealed};
use solana_program::pubkey::Pubkey;


/**
 * 定义数据存储账户状态
 * 注意：repr(u8) 表示结构数据转用字节数组表示的时候，每个字段用一个u8字节表示，而且是按顺序来的。0表示Uninitialize，1表示Initialize，2表示Delete
 */
#[repr(u8)]
#[derive(Clone,Copy,Debug,PartialEq,TryFromPrimitive)]
pub enum BasicExampleState {
    // 未初始化
    Uninitialize,
    // 已初始化
    Initialize,
    // 已删除
    Delete,
}

impl Default for BasicExampleState {
    fn default() -> Self {
        BasicExampleState::Uninitialize
    }
}

/**
 * 定义数据存储结构
 * 注意：repr(C) 表示结构体数据用字节数组表示的时候，每个字段用其本身数据类型表示
 */
#[repr(C)]
#[derive(Clone,Debug,Default,PartialEq)]
pub struct BasicExampleStruct {
    pub account_key: Pubkey,
    pub state: BasicExampleState,
    pub data: String
}

impl Sealed for BasicExampleStruct {}

impl IsInitialized for BasicExampleStruct {
    fn is_initialized(&self) -> bool {
        self.state != BasicExampleState::Uninitialize
    }
}





/**
 * 实现结构体数据解包打包（注意：这个实现我们其实可以不用写，直接在结构体上面添加注解 BorshSerialize和BorshDeserialize 即可）
 */
impl Pack for BasicExampleStruct {
    // 可存储数据总长度290个字节
    const LEN: usize = 290;

    /**
     * 数据解包（参数是u8数组，返回值是结构体）
     */
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        // 获取参数src的可切片引用（注意：src的总长度是 BasicExampleStruct::LEN 是我们在代码里面早就定好的）
        let src = array_ref![src,0,BasicExampleStruct::LEN];
        // 拆分字节数组，前32个字节是公钥，第33个字节是data的长度，后面256个字节是data数据
        let (account_key_buf,state_buf,data_len_buf,data_buf) = array_refs![src,32,1,1,BasicExampleStruct::LEN-32-1-1];
        // 转换公钥
        let account_key = Pubkey::new_from_array(*account_key_buf);
        // 存储数据账户状态
        let state = BasicExampleState::try_from_primitive(state_buf[0]).or(Err(ProgramError::InvalidAccountData))?;
        // 转换data长度
        let data_len = data_len_buf[0] as u8;
        // 截取字节数组得到实际data数据
        let (data_data,_) = data_buf.split_at(data_len.into());
        // 转化data数据
        let data = String::from(from_utf8(data_data).unwrap());

        Ok(BasicExampleStruct {
            account_key,
            state,
            data
        })
    }

    /**
     * 打包并存储数据（参数是结构体对象和一个可变数组这个可变数组用于存储打包后的结构体数据，然后由Solana会拿着这个数据进行存储）
     */
    fn pack_into_slice(&self, dst: &mut [u8]) {
        // 获取dst数组的可变引用（已方便往里面插入数据）
        let dst = array_mut_ref![dst,0,BasicExampleStruct::LEN];
        // 获取数组分段引用，前32个字节是公钥，第33个字节是data的长度，后面的256个字节是data数据
        let (account_key_buf,state_buf,data_len_buf,data_buf) = mut_array_refs![dst,32,1,1,BasicExampleStruct::LEN-32-1-1];
        // 将公钥转换成字节数组再填充上去（注意：填上去的数据长度要和被填数组的长度一致否者报错）
        account_key_buf.copy_from_slice(self.account_key.as_ref());
        // 数据存储账户状态
        state_buf[0] = self.state as u8;
        // 填充data数据的长度
        data_len_buf[0] = self.data.len() as u8;
        // 包装data数据使其长度为256个字节
        let mut data_bytes = Vec::new();
        // 往数组后面的位置写入data数据（注意：这个填充不是从空位置开始而是从数组的长度位置开始，它会扩张数组的长度）
        data_bytes.extend_from_slice(self.data.as_bytes());
        // 重新指定data_bytes数组长度为256个字节，空位置用0填充
        data_bytes.resize(BasicExampleStruct::LEN-32-1-1, 0);
        // 将新的data数据填充上去（注意：填上去的数据长度要和被填数组的长度一致否者报错）
        data_buf.copy_from_slice(&data_bytes);
    }

}

#[cfg(test)]
mod test {
    use arrayref::array_mut_ref;
    use solana_program::msg;
    use solana_program::program_pack::Pack;
    use solana_program::pubkey::Pubkey;
    use crate::state::BasicExampleStruct;
    use crate::state::BasicExampleState;

    #[test]
    fn test_unpack_from_slice() {
        let account_key = Pubkey::default();
        let data = String::from("data");

        let mut array = Vec::new();
        // 往数组里面填充数据（注意：这个填充不是从空位置开始而是从数组的长度位置开始，它会扩张数组的长度）
        array.extend_from_slice(account_key.as_ref());
        // 往数组里面填充数据存储账户状态
        array.push(2 as u8);
        array.push(data.len() as u8);
        // 往数组里面填充数据（注意：这个填充不是从空位置开始而是从数组的长度位置开始，它会扩张数组的长度）
        array.extend_from_slice(data.as_bytes());
        // 重新指定数组长度为290，空位置用0填充（原因：我们解包时会默认该数组长度为290）
        array.resize(BasicExampleStruct::LEN, 0);
        let state = BasicExampleStruct::unpack_from_slice(&array).unwrap();
        msg!("BasicExampleState={:#?}",state);
    }

    #[test]
    fn test_pack_init_slice() {

        let old_state = BasicExampleStruct {
            account_key: Pubkey::default(),
            state: BasicExampleState::Initialize,
            data: String::from("datadatadata")
        };
        let mut array = vec![0 as u8;BasicExampleStruct::LEN];
        let dst = array_mut_ref![array,0,BasicExampleStruct::LEN];
        old_state.pack_into_slice(dst);
        let new_state = BasicExampleStruct::unpack_from_slice(dst).unwrap();
        msg!("New BasicExampleState={:#?}",new_state);
    }

}

