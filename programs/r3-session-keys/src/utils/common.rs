use anchor_lang::prelude::*;

use crate::{state::session::Session, utils::errors::DappError};

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
    // array[start..end].to_vec()
    &array[start..end]
}

pub fn write_array_element(array: &mut [u8], index: usize, element: &[u8]) {
    let start = index * element.len();
    let end = start + element.len();
    array[start..end].copy_from_slice(element);
}
