use anchor_lang::prelude::*;

use crate::{state::session::Session, utils::errors::DappError};

// To close an account it is enough to remove its lamports; to prevent a revival attack
// we also zeroize the data. The Solana runtime garbage-collects it after the transaction.
pub fn close_account<'a>(receiver: &AccountInfo<'a>, account_to_close: &AccountInfo<'a>) -> Result<()> {
    let all_lamports = account_to_close.lamports();

    **account_to_close.try_borrow_mut_lamports()? = 0;
    **receiver.try_borrow_mut_lamports()? += all_lamports;

    let mut data = account_to_close.data.borrow_mut();
    data.fill(0);

    Ok(())
}

pub fn validate_session_params(
    discriminators: &[u8],
    discriminator_len: u8,
    expires_at: i64,
) -> Result<()> {
    require!(discriminator_len > 0, DappError::InvalidDiscriminatorSize);
    require!(
        !discriminators.is_empty()
            && discriminators.len() % discriminator_len as usize == 0
            && discriminators.len() <= Session::MAX_DISCRIMINATORS_LEN,
        DappError::InvalidDiscriminatorListLength
    );
    require!(
        expires_at > Clock::get()?.unix_timestamp,
        DappError::SessionExpirationInPast
    );
    Ok(())
}

pub fn read_array_element(array: &[u8], index: usize, element_size: usize) -> &[u8] {
    let start = index * element_size;
    let end = start + element_size;
    &array[start..end]
}

pub fn write_array_element(array: &mut [u8], index: usize, element: &[u8]) {
    let start = index * element.len();
    let end = start + element.len();
    array[start..end].copy_from_slice(element);
}
