use crate::{convert::RobtopConvert, error::ValueError};
use base64::{DecodeError, URL_SAFE};
use percent_encoding::percent_decode;
use std::{
    error::Error,
    str::{FromStr, Utf8Error},
};

#[derive(Debug, Clone)]
pub struct SelfZip<I> {
    iter: I,
}

impl<I: Iterator> Iterator for SelfZip<I> {
    type Item = (I::Item, I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        match (self.iter.next(), self.iter.next()) {
            (Some(a), Some(b)) => Some((a, b)),
            _ => None,
        }
    }
}

pub trait SelfZipExt: Iterator {
    fn self_zip(self) -> SelfZip<Self>
    where
        Self: Sized,
    {
        SelfZip { iter: self }
    }
}

impl<I> SelfZipExt for I where I: Iterator {}

pub fn default_to_none<T>(value: T) -> Option<T>
where
    T: Default + PartialEq,
{
    if value == Default::default() {
        None
    } else {
        Some(value)
    }
}

pub fn into_option<T>(value: T) -> Option<T> {
    Some(value)
}

/// Takes a percent-encoded URL and decodes it
///
/// # Errors
/// If the decoded data cannot be put together as UTF8, an [`Utf8Error`] is
/// returned
pub fn decode_url(encoded: &str) -> Result<String, Utf8Error> {
    let utf8_cow = percent_decode(encoded.as_bytes()).decode_utf8()?;

    Ok(utf8_cow.to_string())
}

/// Performs URL-safe base64 decoding on the given [`str`] and returns the
/// decoded bytes
///
/// # Errors
/// If the given string isn't valid URL-safe base64, an [`DecodeError`] is
/// returned
pub fn b64_decode_bytes(encoded: &str) -> Result<Vec<u8>, DecodeError> {
    base64::decode_config(encoded, URL_SAFE)
}

/// Performs URL-safe base64 decoding on the given [`str`] and tries to
/// build a UTF8 String from the resulting bytes.
///
/// # Errors
/// If the given string isn't valid URL-safe base64, a [`DecodeError`] is
/// returned
pub fn b64_decode_string(encoded: &str) -> Result<String, DecodeError> {
    b64_decode_bytes(encoded).map(|bytes| String::from_utf8_lossy(&bytes[..]).to_string())
}

/// Performs robtop's XOR en-/decryption routine on `encrypted` using `key`
///
/// Note that although both `encrypted` and `key` are `str`s, the decryption
/// is done directly on the bytes, and the result of each byte-wise XOR
/// operation is casted to `char`, meaning this function only works for
/// ASCII strings.
pub fn xor_decrypt(encrypted: &str, key: &str) -> String {
    encrypted
        .bytes()
        .zip(key.bytes().cycle())
        .map(|(enc_byte, key_byte)| (enc_byte ^ key_byte) as char)
        .collect()
}

pub fn parse<'a, T>(idx: &'a str, value: &'a str) -> Result<Option<T>, ValueError<'a>>
where
    T: RobtopConvert<String, str>,
{
    if value == "" {
        return Ok(None)
    }

    RobtopConvert::robtop_from(value)
        .map(Some)
        .map_err(|error| ValueError::Parse(idx, value, error))
}
