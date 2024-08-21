use std::mem::size_of;
use std::str::from_utf8;
use arrayref::{array_ref, array_refs};
use num_traits::ToBytes;
use solana_program::program_error::ProgramError;
use crate::error::BasicExampleError;

/**
 * 定义合约所有函数（注意：约定俗成的命名方式是在最后面加上Instruction）
 */
#[repr(C)]
#[derive(Clone,Debug,PartialEq)]
pub enum BasicExampleInstruction {
    // 初始化数据账户
    Initialize,
    // 修改数据
    Update {
        data: String
    },
    // 删除数据
    Delete,
    // 多参数测试
    MultiParamTest {
        // 占4个字节
        aa: u32,
        // 占1个字节
        bb: u8,
        // 占8个字节
        cc: u64,
        // 未知长度
        dd: String
    },
}

impl BasicExampleInstruction {
    /**
     * 解码调用合约的参数数据（参数是u8数组，返回值是我们定义的枚举类型）
     */
    pub fn unpack(input: &[u8]) -> Result<Self,ProgramError> {
        // 将参数的第一个字节和后面的所有字节分开（如果拆分是出现错误则抛出BasicExampleError异常）
        let (&tag,rest) = input.split_first().ok_or(BasicExampleError::InvalidInstruction)?;
        Ok(match tag {
            // 如果参数的第一个字节是0表示调用Initialize函数
            0 => Self::Initialize,
            // 如果参数的第一个字节是1表示调用Update函数
            1 => {
                let data = String::from(from_utf8(rest).unwrap());
                Self::Update {
                    data
                }
            },
            // 如果参数的第一个字节是2表示调用的是Delete函数
            2 => Self::Delete,
            // 如果参数的第一个字节是3表示调用的是MultiParamTest函数
            3 => {
                // 前面3个参数总共占13字节，故截取数组0到13的位置（注意：第13个位置不会取，实际是取0到12个位置共13字节）
                let first_params_array = &rest[..13];
                // 获取前3个参数字节数组的可切片引用
                let first_params_array_ref = array_ref![first_params_array,0,13];
                // 拆分字节数组，前4个字节是参数aa，第5个字节是参数bb，后面8个字节是参数cc
                let (aa_ref,bb_ref,cc_ref) = array_refs![first_params_array_ref,4,1,8];

                let aa = u32::from_le_bytes(*aa_ref);
                let bb = bb_ref[0];

                // 初始化一个长度为8的u8字节数组，每个元素用0填充
                //let mut cc_array = [0u8;8];
                // 将cc_ref字节数组填充到cc_array字节数组上（注意：填上去的数据长度要和被填数组的长度一致否者报错）
                //cc_array.copy_from_slice(cc_ref);
                //let cc = u64::from_le_bytes(cc_array);
                let cc = u64::from_le_bytes(*cc_ref);

                // 截取数组13到最后的位置，这一步分数据就是参数dd
                let dd_array = &rest[13..];
                let dd = String::from(from_utf8(&dd_array).unwrap());

                Self::MultiParamTest {
                    aa,
                    bb,
                    cc,
                    dd
                }
            },
            // 如果参数的第一个字节是其它数字则直接抛出异常
            _ => return Err(BasicExampleError::InvalidInstruction.into()),
        })
    }


    /**
     * 编码调用合约的参数数据（参数是我们定义的枚举类型，返回值是u8数组）
     */
    pub fn pack(&self) -> Vec<u8> {
        // 获取当前枚举对象的数据长度
        let self_len = size_of::<Self>();
        // 初始化一个指定大小的数组（注意：为什么长度要加1是因为我们要多写入一个标识位置）
        let mut res = Vec::with_capacity(self_len + 1);
        match self {
            Self::Initialize => {
                // 往数组的第一个位置写入数字0
                res.push(0);
            },
            Self::Update { ref data} => {
                // 往数组的一个位置写入数字1
                res.push(1);
                // 往数组后面的位置写入data数据（注意：这个填充不是从空位置开始而是从数组的长度位置开始，它会扩张数组的长度）
                res.extend_from_slice(data.as_bytes());
            },
            Self::Delete => {
                // 往数组的第一个位置写入数字2
                res.push(2);
            },
            Self::MultiParamTest {aa,bb,cc,dd} => {
                // 往数组的第一个位置写入数字3
                res.push(3);
                // 往数组后面的位置写入data数据（注意：这个填充不是从空位置开始而是从数组的长度位置开始，它会扩张数组的长度）
                res.extend_from_slice(&aa.to_le_bytes());
                res.extend_from_slice(&bb.to_le_bytes());
                res.extend_from_slice(&cc.to_le_bytes());
                res.extend_from_slice(dd.as_bytes());
            }
        }
        return res;
    }
}

#[cfg(test)]
mod test {
    use solana_program::msg;
    use crate::instruction::BasicExampleInstruction;

    #[test]
    fn test_unpack() {
        let mut input: Vec<u8> = Vec::new();
        input.push(0);
        input.extend_from_slice("0".as_bytes());
        let instruction = BasicExampleInstruction::unpack(&input).unwrap();
        msg!("Instruction枚举类型解码成功：{:#?}",instruction);
    }

    #[test]
    fn test_pack() {
        let instruction = BasicExampleInstruction::Initialize;
        let instruction_bytes = instruction.pack();
        msg!("Instruction枚举类型编码成功：{:#?}",instruction_bytes);
    }

    #[test]
    fn test_pack_multi_param() {
        let instruction = BasicExampleInstruction::MultiParamTest {
            aa: 10,
            bb: 255,
            cc: 54515151,
            dd: String::from("ddccbbaa")
        };
        let instruction_bytes = instruction.pack();
        msg!("Instruction枚举类型编码成功：{:#?}",instruction_bytes);
    }

    #[test]
    fn test_unpack_multi_param() {
        let mut input: Vec<u8> = Vec::new();
        input.push(3);

        let aa: u32 = 33;
        let bb: u8 = 3;
        let cc: u64 = 414455;
        let dd = String::from("aabbccdd");

        input.extend_from_slice(&aa.to_le_bytes());
        input.extend_from_slice(&bb.to_le_bytes());
        input.extend_from_slice(&cc.to_le_bytes());
        input.extend_from_slice(&dd.as_bytes());

        let instruction = BasicExampleInstruction::unpack(&input).unwrap();
        msg!("Instruction枚举类型解码成功：{:#?}",instruction);

    }

}

