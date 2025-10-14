use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;
use spl_tlv_account_resolution::{
    account::ExtraAccountMeta, 
    seeds::Seed, 
    state::ExtraAccountMetaList
};

#[derive(Accounts)]
pub struct InitializeExtraAccountMetaList<'info> {
    #[account(mut)]
    payer: Signer<'info>,
    /// CHECK:  unsafe
    #[account(
        init,
        seeds = [b"extra-account-metas", mint.key().as_ref()],
        bump,
        space = ExtraAccountMetaList::size_of(
            InitializeExtraAccountMetaList::extra_account_metas()?.len()
        )?,
        payer = payer
    )]
    pub extra_account_meta_list: AccountInfo<'info>,

    pub mint: InterfaceAccount<'info, Mint>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeExtraAccountMetaList<'info> {
    pub fn extra_account_metas() -> Result<Vec<ExtraAccountMeta>> {
        Ok(
            vec![
                ExtraAccountMeta::new_with_seeds(
                    &[
                        Seed::Literal {
                            bytes: b"restricted_account".to_vec(),
                        },
                        Seed::AccountData { 
                            account_index: 2,
                            data_index: 32,
                            length: 32,
                        },  // Should refer to vault pubkey on deposit
                    ],
                    false, // is_signer
                    false // is_writable
                )?,
                ExtraAccountMeta::new_with_seeds(
                    &[
                        Seed::Literal {
                            bytes: b"restricted_account".to_vec(),
                        },
                        Seed::AccountKey { 
                            index: 3,
                        },  // Should refer to depositor PubKey on deposit 
                    ],
                    false, // is_signer
                    false // is_writable
                )?
            ]
        )
    }

}
