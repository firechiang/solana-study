use solana_program::program_error::ProgramError;
use std::mem::size_of;

#[repr(C)]
#[derive(Debug)]
pub enum AuctionInstruction {
    CreateAuction { start_price: u64 },
    Bidding { price: u64, decimals: u8 },
    CloseAuction,
}

// 该文件是解析调用合约入口参数
impl AuctionInstruction {

    // 合约参数解包（解析并转换成实际调用函数和函数的参数）
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&tag, rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;
        Ok(match tag {
            0 => Self::CreateAuction {
                start_price: {
                    let mut fixed_data = [0u8; 8];
                    fixed_data.copy_from_slice(&rest[..8]);
                    u64::from_le_bytes(fixed_data)
                },
            },
            1 => Self::Bidding {
                price: {
                    let mut fixed_data = [0u8; 8];
                    fixed_data.copy_from_slice(&rest[..8]);
                    u64::from_le_bytes(fixed_data)
                },
                decimals: rest[8],
            },
            2 => Self::CloseAuction,
            _ => return Err(ProgramError::InvalidInstructionData.into()),
        })
    }

    // 合约参数打包
    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            &Self::CreateAuction { start_price } => {
                buf.push(0);
                buf.extend_from_slice(&start_price.to_le_bytes());
            }
            &Self::Bidding { price, decimals } => {
                buf.push(1);
                buf.extend_from_slice(&price.to_le_bytes());
                buf.extend_from_slice(&decimals.to_le_bytes());
            }
            Self::CloseAuction => buf.push(2),
        };
        buf
    }
}