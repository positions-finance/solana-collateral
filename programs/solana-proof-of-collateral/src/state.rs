use anchor::prelude::*;

#[account]
pub struct GlobalState {
    admin: Pubkey,
}
