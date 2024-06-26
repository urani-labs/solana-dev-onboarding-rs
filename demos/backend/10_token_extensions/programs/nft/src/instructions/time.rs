pub use anchor_lang::{
    system_program::{transfer, Transfer},
    prelude::*
};
use crate::{errors::*, state::*};

#[derive(Accounts)]
pub struct ManageTime<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut)]
    pub treasury: SystemAccount<'info>,

    #[account(mut)]
    /// CHECK
    pub membership: UncheckedAccount<'info>,
    
    #[account(
        seeds = [b"nft_rule", rule.seed.to_le_bytes().as_ref()],
        bump,
        has_one = treasury
    )]
    pub rule: Account<'info, NftRule>,
    #[account(
        mut,
        seeds = [b"nft_data", membership.key().as_ref()],
        bump,
        has_one = rule
    )]
    pub data: Account<'info, NftData>,

    pub system_program: Program<'info, System>,
}

impl<'info> ManageTime<'info> {
    pub fn add(
        &mut self,
        time: u64, 
    ) -> Result<()> {

        let mut cost = time * self.rule.price;
        let time: u64 = time.checked_mul(3600).ok_or(NftError::Overflow)?;

        if self.data.expiry < Clock::get()?.unix_timestamp {
            let flat_fee: u64 = 20;
            cost = cost.checked_add(flat_fee.checked_mul(self.rule.price).ok_or(NftError::Overflow)?).ok_or(NftError::Overflow)?;
        } else if self.payer.key() == self.rule.creator {
            cost = 0;
        }

        self.data.expiry = self.data.expiry.checked_add(time as i64).ok_or(NftError::Overflow)?;

        transfer(
            CpiContext::new(
                self.system_program.to_account_info(), 
                Transfer {
                    from: self.payer.to_account_info(),
                    to: self.treasury.to_account_info(),
                }),
            cost
        )?;

        Ok(())
    }

    pub fn remove(
        &mut self,
        time: u64, 
    ) -> Result<()> {

        require!(self.data.expiry > Clock::get()?.unix_timestamp, NftError::Expired);
        require!(self.payer.key() == self.rule.creator, NftError::NotAuthorized);

        let time: u64 = time.checked_mul(3600).ok_or(NftError::Overflow)?;
        self.data.expiry = self.data.expiry.checked_sub(time as i64).ok_or(NftError::Overflow)?;

        Ok(())
    }
}