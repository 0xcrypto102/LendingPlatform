#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Api, Binary, CanonicalAddr, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Storage,BankMsg
};
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:foxy-lend";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let nft_collections: StdResult<Vec<_>> = msg
        .nft_collections
        .into_iter()
        .collect();

    let apy_collections: StdResult<Vec<_>> = msg
        .apy_collections
        .into_iter()
        .collect();

    let max_time_collections: StdResult<Vec<_>> = msg
        .max_time_collections
        .into_iter()
        .collect()

    let offers: StdResult<Vec<_>> = msg
        .offers
        .into_iter()
        .collect()

    NFT_COLLECTIONS.save(dps.storage, &nft_collections)?;
    APY_COLLECTIONS.save(deps.storage, &apy_collections)?;
    MAX_TIME_COLLECTIONS.save(deps.storage, &max_time_collections)?;
    OFFERS.save(deps.storage, &offers)?;
    OFFER_INDEX.save(deps.storage, 0)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    use ExecuteMsg::*;

    match msg {
        Lend { amount: u128, collection: String, contract_address: Addr } => exec::lend(
            deps, 
            info, 
            amount,
            contact_address, 
            collection
        ),
    }
}

mod exec {
    use super::*;
    
    pub fn lend(
        deps: DepsMut, 
        info: MessageInfo,
        amount: u128,
        collection: String,
        apy: u16,
        contact_address: Addr,
    ) -> Result<Response, ContractError> {
        let denom = LEND_DENOM.load(deps.storage)?;
        let current_offers = OFFERS.load(deps.storage)?;
        let offer_index = OFFER_INDEX.load(deps.storage)?;

        // Ensure the sender included the required lending amount
        let actual_amount = cw_utils::must_pay(&info, &denom)
            .map_err(|err| StdError::generic_error(err.to_string()))?
            .u128();

        // Ensure that the actual_amount matches the expected amount
        if actual_amount < amount {
            return Err(ContractError::InvalidAmount {});
        }

        // Create BankMsg::Send message with the desired lending amount
        let message = BankMsg::Send {
            to_address: contract_address.into_string(),
            amount: coins(amount, &denom),
        };

        let offer = OfferResp {
            offer_id: offer_index +1,
            amount: amount,
            nft_collection: collection,
            apy_collection: apy,
            accepted: false
        }

        current_offers.append(&mut offer?);
        OFFERS.save(deps.storage, &current_offers)?;
        
        // Return the BankMsg::Send message as a response
        Ok(Response::new()
            .add_message(message)
            .add_attribute("action", "lend")
            .add_attribute("offer",offer)
            .add_attribute("collection", collection))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;

    match msg {
        OfferList {} => to_binary(&query::offer_list()?),
    }
}

mod query {
    use super::*;

    pub fn offer_list(deps: Deps) -> StdResult<OfferListResp> {
        let offers = OFFERS.load(deps.storage)?;
        let resp = OfferListResp {offers};
        Ok(resp)
    }
}

#[cfg(test)]
mod tests {}
