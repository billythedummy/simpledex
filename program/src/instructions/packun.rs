use std::io::{Read, Write};

use crate::{
    error::SimpleDexError,
    packun::{DeserializePacked, SerializePacked},
};

use super::{create_offer::CreateOfferArgs, SimpleDexInstruction};

// TODO: all this should just be derived

impl<R: Read> DeserializePacked<R, SimpleDexError> for SimpleDexInstruction {
    fn read_bytes(buf: &mut R) -> Result<Self, SimpleDexError> {
        let tag = u8::read_bytes(buf)?;
        match tag {
            0 => Ok(Self::CreateOffer(CreateOfferArgs::read_bytes(buf)?)),
            1 => Ok(Self::CancelOffer),
            2 => Ok(Self::Match),
            _ => Err(SimpleDexError::PackunError),
        }
    }
}

impl<R: Read> DeserializePacked<R, SimpleDexError> for CreateOfferArgs {
    fn read_bytes(buf: &mut R) -> Result<Self, SimpleDexError> {
        let bump = u8::read_bytes(buf)?;
        let seed = u16::read_bytes(buf)?;
        let offering = u64::read_bytes(buf)?;
        let accept_at_least = u64::read_bytes(buf)?;
        Ok(Self {
            bump,
            seed,
            offering,
            accept_at_least,
        })
    }
}

impl<W: Write> SerializePacked<W, SimpleDexError> for SimpleDexInstruction {
    fn write_bytes(&self, buf: &mut W) -> Result<(), SimpleDexError> {
        match self {
            Self::CreateOffer(args) => {
                0u8.write_bytes(buf)?;
                args.write_bytes(buf)
            }
            Self::CancelOffer => 1u8.write_bytes(buf),
            Self::Match => 2u8.write_bytes(buf),
        }
    }
}

impl<W: Write> SerializePacked<W, SimpleDexError> for CreateOfferArgs {
    fn write_bytes(&self, buf: &mut W) -> Result<(), SimpleDexError> {
        self.bump.write_bytes(buf)?;
        self.seed.write_bytes(buf)?;
        self.offering.write_bytes(buf)?;
        self.accept_at_least.write_bytes(buf)
    }
}
