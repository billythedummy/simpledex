use solana_program::account_info::AccountInfo;

pub struct Account<'a: 'me, 'me, T> {
    // `account_info` reference has same lifetime as me,
    // AccountInfo needs to have lifetime at least as long as me
    pub account_info: &'me AccountInfo<'a>,
    pub data: T,
}
