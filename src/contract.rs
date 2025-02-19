#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{TOKEN_INFO, TokenInfo, BALANCES, MinterData, Allowance, ALLOWANCES, ALLOWANCES_SPENDER};
use cosmwasm_std::{Uint128, StdResult, Deps, Binary, to_json_binary, Order};
use cw20::{Expiration, BalanceResponse, MinterResponse, AllowanceInfo, AllowanceResponse, AllAllowancesResponse, AllAccountsResponse, SpenderAllowanceInfo, AllSpenderAllowancesResponse};
use cw_storage_plus::Bound;
/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:my_cw20_token";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/

// settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let owner = info.sender.clone();
    let token_info = TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        total_supply: msg.total_supply,
        mint: msg.mint.as_ref().map(|minter| MinterData {
            minter: info.sender.clone(),
            cap: minter.cap,
        }),
    };

    TOKEN_INFO.save(deps.storage, &token_info)?;

    BALANCES.save(deps.storage, &owner, &msg.total_supply)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    
    match _msg{
        ExecuteMsg::Transfer{recipient, amount} => {
            let sender = info.sender.clone();
            let sender_balance = BALANCES.may_load(deps.storage, &sender)?.unwrap_or_default();

            if sender_balance < amount.into() {
                return Err(ContractError::InsufficientFunds {});
            }
            
            BALANCES.update(deps.storage, &sender, 
                |balance: Option<Uint128>| -> StdResult<_> {
                    Ok(balance.unwrap_or_default().checked_sub(amount.into())?)
                },
            )?;
            BALANCES.update(deps.storage, &recipient, |balance: Option<Uint128>| -> StdResult<_> {
                Ok(balance.unwrap_or_default().checked_add(amount.into())?)
                },
            )?;
            Ok(Response::new().add_attribute("action", "transfer"))
        }
        ExecuteMsg::Mint{recipient, amount} => {
            let mut token_info = TOKEN_INFO.may_load(deps.storage)?.ok_or(ContractError::Unauthorized {})?;   
            token_info.total_supply += Uint128::from(amount);
            
            let minter_data = token_info.mint.clone().ok_or(ContractError::Unauthorized {})?; 
            if minter_data.minter != info.sender {
                return Err(ContractError::Unauthorized {});
            }

            BALANCES.update(deps.storage, &recipient, |balance: Option<Uint128>| -> StdResult<_> {
                Ok(balance.unwrap_or_default().checked_add(amount.into())?)
                },
            )?;

            TOKEN_INFO.save(deps.storage, &token_info)?;
            Ok(Response::new().add_attribute("action", "mint"))
        }
        ExecuteMsg::Burn{amount} => {
            let mut token_info = TOKEN_INFO.may_load(deps.storage)?.ok_or(ContractError::Unauthorized {})?;   
            let minter_data = token_info.mint.clone().ok_or(ContractError::Unauthorized {})?; 
            if minter_data.minter != info.sender {
                return Err(ContractError::Unauthorized {});
            }
            let total_supply = token_info.total_supply;
            if total_supply < amount.into(){
                return Err(ContractError::InsufficientFunds {});
            }
            token_info.total_supply -= Uint128::from(amount);
            TOKEN_INFO.save(deps.storage, &token_info)?;
            Ok(Response::new().add_attribute("action", "burn"))
            
        }
        ExecuteMsg::Approve{spender, amount,} => {
            let owner = info.sender;
            let allowance = Allowance {
                allowance: Uint128::from(amount),
                expires: Expiration::Never{},
            };
            let owner_balance = BALANCES.may_load(deps.storage, &owner)?.unwrap_or_default();
            if owner_balance < amount.into() {
                return Err(ContractError::InsufficientFunds {});
            }

            ALLOWANCES.save(deps.storage, (&owner, &spender), &allowance)?;
            Ok(Response::new().add_attribute("action", "approve"))
           
        }
        ExecuteMsg::TransferFrom{owner, recipient, amount} => {
            let allowances = ALLOWANCES.may_load(deps.storage, (&owner, &info.sender))?.ok_or(ContractError::Unauthorized {})?;

            if allowances.allowance < amount.into() {
                return Err(ContractError::InsufficientAllowance {});
            }
            if allowances.expires.is_expired(&env.block) {
                return Err(ContractError::AllowanceExpired {});
            }
            BALANCES.update(deps.storage, &owner, 
                |balance: Option<Uint128>| -> StdResult<_> {
                    Ok(balance.unwrap_or_default().checked_sub(amount.into())?)
                },
            )?;
            BALANCES.update(deps.storage, &recipient, |balance: Option<Uint128>| -> StdResult<_> {
                Ok(balance.unwrap_or_default().checked_add(amount.into())?)
                },
            )?;

            ALLOWANCES.remove(deps.storage, (&owner, &info.sender));
            Ok(Response::new().add_attribute("action", "transfer_from"))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Balance { address } => to_json_binary(&query_balance(deps, address)?),
        QueryMsg::TokenInfo {} => to_json_binary(&query_token_info(deps)?),
        QueryMsg::Minter {} => to_json_binary(&query_minter(deps)?),
        QueryMsg::Allowance { owner, spender } => {
            to_json_binary(&query_allowance(deps, owner, spender)?)
        }
        QueryMsg::AllAllowances {
            owner,
            start_after,
            limit,
        } => to_json_binary(&query_owner_allowances(deps, owner, start_after, limit)?),
        QueryMsg::AllSpenderAllowances {
            spender,
            start_after,
            limit,
        } => to_json_binary(&query_spender_allowances(deps, spender, start_after, limit)?),
        QueryMsg::AllAccounts { start_after, limit } => {
            to_json_binary(&query_all_accounts(deps, start_after, limit)?)
        }
    }
}


pub fn query_balance(deps: Deps, address: String) -> StdResult<BalanceResponse> {
    let address = deps.api.addr_validate(&address)?;
    let balance = BALANCES.load(deps.storage, &address)?;
    Ok(BalanceResponse { balance })
}

pub fn query_token_info(deps: Deps) -> StdResult<TokenInfo> {
    let info = TOKEN_INFO.load(deps.storage)?;
    Ok(TokenInfo {
        name: info.name,
        symbol: info.symbol,
        decimals: info.decimals,
        total_supply: info.total_supply,
        mint: info.mint,
    })
}

pub fn query_minter(deps: Deps) -> StdResult<Option<MinterResponse>> {
    let meta = TOKEN_INFO.load(deps.storage)?;
    let minter = match meta.mint {
        Some(m) => Some(MinterResponse {
            minter: m.minter.into(),
            cap: m.cap,
        }),
        None => None,
    };
    Ok(minter)
}

pub fn query_allowance(deps: Deps, owner: String, spender: String) -> StdResult<AllowanceResponse> {
    let owner_addr = deps.api.addr_validate(&owner)?;
    let spender_addr = deps.api.addr_validate(&spender)?;
    let allowance = ALLOWANCES
        .may_load(deps.storage, (&owner_addr, &spender_addr))?
        .unwrap_or_default();
    Ok(AllowanceResponse{
        allowance: allowance.allowance,
        expires: allowance.expires
    })
}


pub fn query_owner_allowances(
    deps: Deps,
    owner: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<AllAllowancesResponse> {
    let owner_addr = deps.api.addr_validate(&owner)?;
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.into_bytes()));

    let allowances = ALLOWANCES
        .prefix(&owner_addr)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            item.map(|(addr, allow)| AllowanceInfo {
                spender: addr.into(),
                allowance: allow.allowance,
                expires: allow.expires,
            })
        })
        .collect::<StdResult<_>>()?;
    Ok(AllAllowancesResponse { allowances })
}


pub fn query_spender_allowances(
    deps: Deps,
    spender: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<AllSpenderAllowancesResponse> {
    let spender_addr = deps.api.addr_validate(&spender)?;
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.into_bytes()));

    let allowances = ALLOWANCES_SPENDER
        .prefix(&spender_addr)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            item.map(|(addr, allow)| SpenderAllowanceInfo {
                owner: addr.into(),
                allowance: allow.allowance,
                expires: allow.expires,
            })
        })
        .collect::<StdResult<_>>()?;
    Ok(AllSpenderAllowancesResponse { allowances })
}

pub fn query_all_accounts(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<AllAccountsResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

    let accounts = BALANCES
        .keys(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(Into::into))
        .collect::<StdResult<_>>()?;

    Ok(AllAccountsResponse { accounts })
}







#[cfg(test)]
mod tests {}
