use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use msg::{ ExecuteMsg, InstantiateMsg, QueryMsg };

pub mod contract;
mod error;
pub mod msg;
pub mod state;
pub mod integration_tests;
pub mod helpers;

pub use crate::error::ContractError;


#[entry_point]
pub fn instantiate(deps: DepsMut, _env: Env, _info: MessageInfo, msg: InstantiateMsg)
  -> StdResult<Response>
{
    contract::instantiate(deps, _env, _info, msg)
}

#[entry_point]
pub fn execute(deps: DepsMut, _env: Env, info: MessageInfo, msg: ExecuteMsg) -> Result<Response, ContractError>
{
  contract::execute(deps, _env, info, msg)
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg)
  -> StdResult<Binary>
{
    contract::query(deps, _env, msg)
}
