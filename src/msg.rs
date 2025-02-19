use cosmwasm_schema::{cw_serde, QueryResponses};
use cw20::{ TokenInfoResponse, MinterResponse, AllowanceResponse, AllAllowancesResponse, AllSpenderAllowancesResponse, AllAccountsResponse };
use cosmwasm_std::{Addr, Uint128, StdResult, StdError};
use crate::state::MinterData;

#[cw_serde]
pub struct InstantiateMsg {
    pub total_supply: Uint128,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub mint: Option<MinterData>,
}

impl InstantiateMsg{
    pub fn get_cap(&self) -> Option<Uint128> {
        self.mint.as_ref().and_then(|v| v.cap)
    }

    pub fn validate(&self) -> StdResult<()> {
        // Check name, symbol, decimals
        if !self.has_valid_name() {
            return Err(StdError::generic_err(
                "Name is not in the expected format (3-50 UTF-8 bytes)",
            ));
        }
        if !self.has_valid_symbol() {
            return Err(StdError::generic_err(
                "Ticker symbol is not in expected format [a-zA-Z\\-]{3,12}",
            ));
        }
        if self.decimals > 18 {
            return Err(StdError::generic_err("Decimals must not exceed 18"));
        }
        Ok(())
    }

    fn has_valid_name(&self) -> bool {
        let bytes = self.name.as_bytes();
        if bytes.len() < 3 || bytes.len() > 50 {
            return false;
        }
        true
    }

    fn has_valid_symbol(&self) -> bool {
        let bytes = self.symbol.as_bytes();
        if bytes.len() < 3 || bytes.len() > 12 {
            return false;
        }
        for byte in bytes.iter() {
            if (*byte != 45) && (*byte < 65 || *byte > 90) && (*byte < 97 || *byte > 122) {
                return false;
            }
        }
        true
    }
}

#[cw_serde]
pub enum ExecuteMsg {
    Transfer {
        recipient: Addr,
        amount: Uint128,
    },
    Mint {
        recipient: Addr,
        amount: Uint128,
    },
    Burn {
        amount: Uint128,
    },
    Approve {
        spender: Addr,
        amount: Uint128,
    },
    TransferFrom {
        owner: Addr,
        recipient: Addr,
        amount: Uint128,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(cw20::BalanceResponse)]
    Balance{ address: String },

    #[returns(TokenInfoResponse)]
    TokenInfo {},

    #[returns(MinterResponse)]
    Minter {},

    #[returns(AllowanceResponse)]
    Allowance{ owner: String, spender: String },

    #[returns(AllAllowancesResponse)]
    AllAllowances {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },

    #[returns(AllSpenderAllowancesResponse)]
    AllSpenderAllowances {
        spender: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },

    #[returns(AllAccountsResponse)]
    AllAccounts {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}
