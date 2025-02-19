use cw_storage_plus::{Item, Map,};
use cosmwasm_std::{Addr, Uint128};
use cw_utils::Expiration;
use cosmwasm_schema::cw_serde;


#[cw_serde]
pub struct TokenInfo{
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: Uint128,
    pub mint: Option<MinterData>,
}

#[cw_serde]
pub struct MinterData{
    pub minter: Addr,
    pub cap: Option<Uint128>,
}

pub const TOKEN_INFO: Item<TokenInfo> = Item::new("token_info");
pub const BALANCES: Map<&Addr, Uint128> = Map::new("balance");

#[cw_serde]
#[derive(Default)]
pub struct Allowance{
    pub allowance: Uint128,
    pub expires: Expiration,
}

pub const ALLOWANCES: Map<(&Addr, &Addr), Allowance> = Map::new("allowances");

pub const ALLOWANCES_SPENDER: Map<(&Addr, &Addr), Allowance> = Map::new("allowance_spender");

