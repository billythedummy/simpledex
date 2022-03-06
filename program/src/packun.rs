//! Packing and unpacking (de)serialization of data

use std::{
    error::Error,
    io::{Read, Write},
};

use solana_program::pubkey::Pubkey;

use crate::error::SimpleDexError;

pub trait DeserializePacked<R: Read, E: Error> {
    fn read_bytes(buf: &mut R) -> Result<Self, E>
    where
        Self: Sized;
}

fn try_read<R: Read, const S: usize>(buf: &mut R) -> Result<[u8; S], SimpleDexError> {
    let mut into = [0; S];
    match buf.read(&mut into) {
        Ok(l) => {
            if l != S {
                Err(SimpleDexError::PackunError)
            } else {
                Ok(into)
            }
        }
        Err(_) => Err(SimpleDexError::PackunError),
    }
}

macro_rules! impl_deserialize_packed_le_primitive {
    ($prim: ty) => {
        impl<R: Read> DeserializePacked<R, SimpleDexError> for $prim {
            fn read_bytes(buf: &mut R) -> Result<Self, SimpleDexError> {
                let bytes = try_read(buf)?;
                Ok(Self::from_le_bytes(bytes))
            }
        }
    };
}

impl_deserialize_packed_le_primitive!(u8);
impl_deserialize_packed_le_primitive!(u16);
impl_deserialize_packed_le_primitive!(u64);

impl<R: Read> DeserializePacked<R, SimpleDexError> for Pubkey {
    fn read_bytes(buf: &mut R) -> Result<Self, SimpleDexError>
    where
        Self: Sized,
    {
        Ok(Self::new_from_array(try_read(buf)?))
    }
}

pub trait SerializePacked<W: Write, E: Error> {
    fn write_bytes(&self, buf: &mut W) -> Result<(), E>;
}

fn try_write<W: Write, S: AsRef<[u8]>>(src: S, buf: &mut W) -> Result<(), SimpleDexError> {
    let src_ref = src.as_ref();
    match buf.write(src_ref) {
        Ok(l) => {
            if l != src_ref.len() {
                Err(SimpleDexError::PackunError)
            } else {
                Ok(())
            }
        }
        Err(_) => Err(SimpleDexError::PackunError),
    }
}

macro_rules! impl_serialize_packed_le_primitive {
    ($prim: ty) => {
        impl<W: Write> SerializePacked<W, SimpleDexError> for $prim {
            fn write_bytes(&self, buf: &mut W) -> Result<(), SimpleDexError> {
                try_write(self.to_le_bytes(), buf)
            }
        }
    };
}

impl_serialize_packed_le_primitive!(u8);
impl_serialize_packed_le_primitive!(u16);
impl_serialize_packed_le_primitive!(u64);

impl<W: Write> SerializePacked<W, SimpleDexError> for Pubkey {
    fn write_bytes(&self, buf: &mut W) -> Result<(), SimpleDexError> {
        try_write(self.as_ref(), buf)
    }
}
