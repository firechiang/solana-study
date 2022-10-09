use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

// 该文件定义实际要存储数据的结构体

/// auction data.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Auction {
    pub seller: Pubkey,         // owner of this auction
    pub item: Pubkey,           // token mint about which item sold
    pub item_holder: Pubkey,    // a custodian account of the item (controlled by pda)
    pub currency: Pubkey,       // token mint about using which type of token to bid
    pub money_holder: Pubkey,   // a custodian account of the bidding money (controlled by pda)
    pub bidder: Pubkey,         // the highest price bidder
    pub refund_address: Pubkey, // if someone bid a higher price, the previous price will return back to this address
    pub price: u64,             // price for the item now
}

impl Sealed for Auction {}
impl IsInitialized for Auction {
    fn is_initialized(&self) -> bool {
        self.seller != Pubkey::default()
    }
}

impl Pack for Auction {
    const LEN: usize = 232;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, 232];
        let (seller, item, item_holder, currency, money_holder, bidder, refund_address, price) =
            array_refs![src, 32, 32, 32, 32, 32, 32, 32, 8];
        Ok(Auction {
            seller: Pubkey::new_from_array(*seller),
            item: Pubkey::new_from_array(*item),
            item_holder: Pubkey::new_from_array(*item_holder),
            currency: Pubkey::new_from_array(*currency),
            money_holder: Pubkey::new_from_array(*money_holder),
            bidder: Pubkey::new_from_array(*bidder),
            refund_address: Pubkey::new_from_array(*refund_address),
            price: u64::from_le_bytes(*price),
        })
    }
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, 232];
        let (
            seller_dst,
            item_dst,
            item_holder_dst,
            currency_dst,
            money_holder_dst,
            bidder_dst,
            refund_address_dst,
            price_dst,
        ) = mut_array_refs![dst, 32, 32, 32, 32, 32, 32, 32, 8];
        let &Auction {
            ref seller,
            ref item,
            ref item_holder,
            ref currency,
            ref money_holder,
            ref bidder,
            ref refund_address,
            price,
        } = self;
        seller_dst.copy_from_slice(seller.as_ref());
        item_dst.copy_from_slice(item.as_ref());
        item_holder_dst.copy_from_slice(item_holder.as_ref());
        currency_dst.copy_from_slice(currency.as_ref());
        money_holder_dst.copy_from_slice(money_holder.as_ref());
        bidder_dst.copy_from_slice(bidder.as_ref());
        refund_address_dst.copy_from_slice(refund_address.as_ref());
        *price_dst = price.to_le_bytes();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn unpack_auction() {
        let auction = Auction::unpack_from_slice(&[
            155, 223, 245, 186, 87, 123, 202, 87, 3, 244, 165, 72, 147, 80, 16, 41, 197, 14, 57,
            96, 239, 172, 90, 101, 168, 211, 132, 86, 70, 172, 14, 121, 37, 198, 132, 32, 63, 71,
            81, 180, 24, 168, 70, 238, 174, 211, 132, 126, 37, 33, 74, 50, 227, 200, 7, 48, 254,
            34, 67, 22, 161, 74, 224, 249, 222, 164, 160, 26, 92, 76, 214, 250, 120, 86, 178, 67,
            175, 216, 46, 116, 249, 205, 211, 136, 175, 117, 66, 172, 221, 213, 200, 61, 173, 67,
            91, 37, 159, 113, 69, 97, 71, 173, 95, 254, 20, 37, 26, 26, 210, 82, 18, 209, 68, 108,
            167, 252, 243, 16, 244, 30, 114, 202, 132, 223, 149, 107, 227, 21, 222, 33, 94, 41, 10,
            10, 118, 49, 101, 52, 192, 78, 171, 210, 237, 138, 151, 36, 123, 100, 14, 26, 162, 56,
            124, 99, 177, 76, 51, 215, 112, 210, 213, 12, 76, 221, 151, 56, 247, 250, 170, 122,
            211, 60, 235, 243, 101, 71, 225, 0, 169, 229, 25, 3, 152, 105, 127, 221, 19, 72, 200,
            91, 152, 253, 191, 213, 97, 49, 159, 191, 3, 181, 236, 255, 53, 161, 134, 235, 33, 246,
            120, 249, 122, 239, 68, 147, 37, 246, 205, 227, 113, 119, 200, 114, 234, 239, 50, 0, 0,
            0, 0, 0, 0, 0,
        ])
            .unwrap();

        let expected_auction = Auction {
            seller: Pubkey::from_str("BVUGLStgsasiAQHjT8tc2SJMUVgxpFopwchV9PeoBCkY").unwrap(),
            item: Pubkey::from_str("3YTeuJHrQt5e9ggtVE69kWP6f4rc8damLhfZ9yeBH7XN").unwrap(),
            item_holder: Pubkey::from_str("Fz774M8wRQAfHBFQvL1s32GnXPNsiV6LrA9ictCFwh5n").unwrap(),
            currency: Pubkey::from_str("BjQ15j2gyAFLBTDB1MDU6DCGWcwR4Y9PgDCycVygRfY8").unwrap(),
            money_holder: Pubkey::from_str("Fx728pgJGWqqdzjyCVN3ZdybMhrcAK5m4wxiKugmuDbo").unwrap(),
            bidder: Pubkey::from_str("FLeieTgGK8PMQ89n4NbtdQZJMm8vW98M6Q7hSfAzXy6U").unwrap(),
            refund_address: Pubkey::from_str("DuqatgVfG5qKZVtxaXCP4U5BRyPXM7w53DJjFmS1ZWAr")
                .unwrap(),
            price: 50,
        };

        assert_eq!(auction, expected_auction);
    }

    #[test]
    fn pack_auction() {
        let auction = Auction {
            seller: Pubkey::from_str("BVUGLStgsasiAQHjT8tc2SJMUVgxpFopwchV9PeoBCkY").unwrap(),
            item: Pubkey::from_str("3YTeuJHrQt5e9ggtVE69kWP6f4rc8damLhfZ9yeBH7XN").unwrap(),
            item_holder: Pubkey::from_str("Fz774M8wRQAfHBFQvL1s32GnXPNsiV6LrA9ictCFwh5n").unwrap(),
            currency: Pubkey::from_str("BjQ15j2gyAFLBTDB1MDU6DCGWcwR4Y9PgDCycVygRfY8").unwrap(),
            money_holder: Pubkey::from_str("Fx728pgJGWqqdzjyCVN3ZdybMhrcAK5m4wxiKugmuDbo").unwrap(),
            bidder: Pubkey::from_str("FLeieTgGK8PMQ89n4NbtdQZJMm8vW98M6Q7hSfAzXy6U").unwrap(),
            refund_address: Pubkey::from_str("DuqatgVfG5qKZVtxaXCP4U5BRyPXM7w53DJjFmS1ZWAr")
                .unwrap(),
            price: 50,
        };

        let mut data_dst = vec![0x00; Auction::LEN];
        auction.pack_into_slice(&mut data_dst);

        let expected_data = [
            155, 223, 245, 186, 87, 123, 202, 87, 3, 244, 165, 72, 147, 80, 16, 41, 197, 14, 57,
            96, 239, 172, 90, 101, 168, 211, 132, 86, 70, 172, 14, 121, 37, 198, 132, 32, 63, 71,
            81, 180, 24, 168, 70, 238, 174, 211, 132, 126, 37, 33, 74, 50, 227, 200, 7, 48, 254,
            34, 67, 22, 161, 74, 224, 249, 222, 164, 160, 26, 92, 76, 214, 250, 120, 86, 178, 67,
            175, 216, 46, 116, 249, 205, 211, 136, 175, 117, 66, 172, 221, 213, 200, 61, 173, 67,
            91, 37, 159, 113, 69, 97, 71, 173, 95, 254, 20, 37, 26, 26, 210, 82, 18, 209, 68, 108,
            167, 252, 243, 16, 244, 30, 114, 202, 132, 223, 149, 107, 227, 21, 222, 33, 94, 41, 10,
            10, 118, 49, 101, 52, 192, 78, 171, 210, 237, 138, 151, 36, 123, 100, 14, 26, 162, 56,
            124, 99, 177, 76, 51, 215, 112, 210, 213, 12, 76, 221, 151, 56, 247, 250, 170, 122,
            211, 60, 235, 243, 101, 71, 225, 0, 169, 229, 25, 3, 152, 105, 127, 221, 19, 72, 200,
            91, 152, 253, 191, 213, 97, 49, 159, 191, 3, 181, 236, 255, 53, 161, 134, 235, 33, 246,
            120, 249, 122, 239, 68, 147, 37, 246, 205, 227, 113, 119, 200, 114, 234, 239, 50, 0, 0,
            0, 0, 0, 0, 0,
        ];

        assert_eq!(data_dst, expected_data);
    }
}