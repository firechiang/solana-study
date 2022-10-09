

// 该文件定义实际要存储数据的结构体

use std::str::from_utf8;
use arrayref::{array_mut_ref,mut_array_refs, array_ref, array_refs};
use solana_program::program_error::ProgramError;
use solana_program::program_pack::{IsInitialized, Pack, Sealed};
use solana_program::pubkey::Pubkey;

#[repr(C)]
#[derive(Clone,Debug,Default,PartialEq)]
pub struct HelloWorldState {
    pub account_key: Pubkey,
    pub message: String,
}


impl Sealed for HelloWorldState {}

impl IsInitialized for HelloWorldState {
    fn is_initialized(&self) -> bool {
        return true;
    }
}

// 实现数据打包解包（注意：这个实现我们其实可以不用写直接在结构体上面添加 BorshSerialize和BorshDeserialize 注解即可）
impl Pack for HelloWorldState {
    // 数据总长度
    const LEN: usize = 289;

    // 数据解包
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        // 复制数组0-289个字节
        let src = array_ref![src,0,289];
        // 拆分字节数组0-32个字节是公钥，第33个字节是message的长度，后面的256个字节用来存储message的数据
        let (account_key_buf, message_len_buf, message_buf) = array_refs![src, 32, 1, 256];
        // 转换公钥
        let account_key = Pubkey::new_from_array(*account_key_buf);
        // 转换message数据长度
        let message_len = message_len_buf[0] as u8;
        // 截取字节数组得到实际message数据
        let (msg_buf,_) = message_buf.split_at(message_len.into());
        // 转换message数据
        let message = String::from(from_utf8(msg_buf).unwrap());
        Ok(HelloWorldState {
            account_key,
            message
        })
    }

    // 数据打包存储
    fn pack_into_slice(&self, dst: &mut [u8]) {
        // 修改字节数组0-289位置的数据
        let dst = array_mut_ref![dst,0,289];
        // 修改字节数组0-32个字节是公钥，第33个字节是message的长度，后面的256个字节用来存储message的数据
        let (account_key_buf,message_len_buf,message_buf) = mut_array_refs![dst,32,1,256];
        // 将公钥转换成字节数组再填充上去
        account_key_buf.copy_from_slice(self.account_key.as_ref());
        // 填充message数据的长度
        message_len_buf[0] = self.message.len() as u8;
        // 将message转换成字节数组再填充上去
        //message_buf.copy_from_slice(&self.message.as_bytes());
        message_buf[..self.message.len()].copy_from_slice(&self.message.as_bytes());
    }
}