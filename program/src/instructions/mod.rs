use self::create_offer::CreateOfferArgs;

pub mod create_offer;
mod packun;

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub enum SimpleDexInstruction {
    CreateOffer(CreateOfferArgs),
    CancelOffer,
    Match,
}

impl SimpleDexInstruction {
    pub const PACKED_LEN_CREATE_OFFER: usize = 20; // 1 + 19
    pub const PACKED_LEN_CANCEL_OFFER: usize = 1;
    pub const PACKED_LEN_MATCH: usize = 1;
}
