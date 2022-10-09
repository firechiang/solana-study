use crate::{instruction::AuctionInstruction, state::Auction};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
};

use spl_token;

pub struct Processor {}

// 该文件是合约里面各个函数的逻辑实现
impl Processor {

    pub fn process_create_auction(
        program_id: Pubkey,
        accounts: &[AccountInfo],
        start_price: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let auction_info = next_account_info(account_info_iter)?;
        let mut auction = Auction::unpack_from_slice(&auction_info.data.borrow())?;

        let seller = next_account_info(account_info_iter)?;

        let (pda, _) = Pubkey::find_program_address(&[&seller.key.to_bytes()], &program_id);

        let item = next_account_info(account_info_iter)?;
        let item_holder_info = next_account_info(account_info_iter)?;
        let item_holder = spl_token::state::Account::unpack(&item_holder_info.data.borrow())?;
        if item_holder.mint != *item.key {
            msg!("item holder mint mismatch");
            return Err(ProgramError::InvalidAccountData);
        }
        if item_holder.owner != pda {
            msg!("item holder owner mismatch");
            return Err(ProgramError::InvalidAccountData);
        }

        let currency = next_account_info(account_info_iter)?;
        let money_holder_info = next_account_info(account_info_iter)?;
        let money_holder = spl_token::state::Account::unpack(&money_holder_info.data.borrow())?;
        if money_holder.mint != *currency.key {
            msg!("money holder mint mismatch");
            return Err(ProgramError::InvalidAccountData);
        }
        if money_holder.owner != pda {
            msg!("money holder owner mismatch");
            return Err(ProgramError::InvalidAccountData);
        }

        let bidder = next_account_info(account_info_iter)?;
        let refund_address = next_account_info(account_info_iter)?;

        let send_currency_instruction = spl_token::instruction::transfer(
            &spl_token::id(),
            &refund_address.key,
            &money_holder_info.key,
            &seller.key,
            &[&seller.key],
            start_price,
        )?;
        invoke(&send_currency_instruction, accounts)?;

        let send_item_instruction = spl_token::instruction::transfer(
            &spl_token::id(),
            &bidder.key,
            &item_holder_info.key,
            &seller.key,
            &[&seller.key],
            1,
        )?;
        invoke(&send_item_instruction, accounts)?;

        auction.seller = *seller.key;
        auction.item = *item.key;
        auction.item_holder = *item_holder_info.key;
        auction.currency = *currency.key;
        auction.money_holder = *money_holder_info.key;
        auction.bidder = *bidder.key;
        auction.refund_address = *refund_address.key;
        auction.price = start_price;
        Auction::pack(auction, &mut auction_info.data.borrow_mut())?;
        Ok(())
    }

    pub fn process_bidding(
        program_id: Pubkey,
        accounts: &[AccountInfo],
        price: u64,
        decimals: u8,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let auction_info = next_account_info(account_info_iter)?;
        let mut auction = Auction::unpack(&auction_info.data.borrow())?;

        let bidder = next_account_info(account_info_iter)?;

        let item_receiver_info = next_account_info(account_info_iter)?;
        let item_receiver = spl_token::state::Account::unpack(&item_receiver_info.data.borrow())?;
        if !item_receiver.is_initialized() {
            msg!("item receiver doesn't init");
            return Err(ProgramError::InvalidAccountData);
        }
        if item_receiver.mint != auction.item {
            msg!("item receiver mismatch");
            return Err(ProgramError::InvalidAccountData);
        }

        let refund_address_info = next_account_info(account_info_iter)?;
        let refund_address = spl_token::state::Account::unpack(&refund_address_info.data.borrow())?;
        if !refund_address.is_initialized() {
            msg!("item receiver doesn't init");
            return Err(ProgramError::InvalidAccountData);
        }
        if refund_address.mint != auction.currency {
            msg!("refund address mismatch");
            return Err(ProgramError::InvalidAccountData);
        }

        if price <= auction.price {
            msg!("bid price too low");
            return Err(ProgramError::InvalidAccountData);
        }

        let (pda, seed) = Pubkey::find_program_address(&[&auction.seller.to_bytes()], &program_id);
        let refund_instruction = spl_token::instruction::transfer_checked(
            &spl_token::id(),
            &auction.money_holder,
            &auction.currency,
            &auction.refund_address,
            &pda,
            &[&pda],
            auction.price,
            decimals,
        )?;

        invoke_signed(
            &refund_instruction,
            accounts,
            &[&[&auction.seller.to_bytes(), &[seed]]],
        )?;

        let payer = next_account_info(account_info_iter)?;
        let pay_for_bidding_instruction = spl_token::instruction::transfer_checked(
            &spl_token::id(),
            payer.key,
            &auction.currency,
            &auction.money_holder,
            &bidder.key,
            &[&bidder.key],
            price,
            decimals,
        )?;
        invoke_signed(
            &pay_for_bidding_instruction,
            accounts,
            &[&[&auction.seller.to_bytes(), &[seed]]],
        )?;

        auction.bidder = *item_receiver_info.key;
        auction.refund_address = *refund_address_info.key;
        auction.price = price;
        Auction::pack(auction, &mut auction_info.data.borrow_mut())?;
        Ok(())
    }

    pub fn process_close_auction(program_id: Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let auction_info = next_account_info(account_info_iter)?;
        let auction = Auction::unpack(&auction_info.data.borrow())?;

        let seller = next_account_info(account_info_iter)?;
        if *seller.key != auction.seller {
            msg!("seller mismatch");
            return Err(ProgramError::InvalidAccountData.into());
        }
        if !seller.is_signer {
            msg!("seller need sign");
            return Err(ProgramError::InvalidAccountData.into());
        }

        let moeny_receiver = next_account_info(account_info_iter)?;

        let (pda, seed) = Pubkey::find_program_address(&[&auction.seller.to_bytes()], &program_id);
        let receive_money_instruction = spl_token::instruction::transfer(
            &spl_token::id(),
            &auction.money_holder,
            moeny_receiver.key,
            &pda,
            &[&pda],
            auction.price,
        )?;
        invoke_signed(
            &receive_money_instruction,
            accounts,
            &[&[&auction.seller.to_bytes(), &[seed]]],
        )?;

        let send_item_instruction = spl_token::instruction::transfer(
            &spl_token::id(),
            &auction.item_holder,
            &auction.bidder,
            &pda,
            &[&pda],
            1,
        )?;
        invoke_signed(
            &send_item_instruction,
            accounts,
            &[&[&auction.seller.to_bytes(), &[seed]]],
        )?;

        // TODO add close status to auction

        Ok(())
    }

    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = AuctionInstruction::unpack(input)?;

        match instruction {
            AuctionInstruction::CreateAuction { start_price } => {
                msg!("Instruction: CreateAuction");
                Self::process_create_auction(*program_id, accounts, start_price)
            }
            AuctionInstruction::Bidding { price, decimals } => {
                msg!("Instruction: Bidding");
                Self::process_bidding(*program_id, accounts, price, decimals)
            }
            AuctionInstruction::CloseAuction => {
                msg!("Instruction: CloseAuction");
                Self::process_close_auction(*program_id, accounts)
            }
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}