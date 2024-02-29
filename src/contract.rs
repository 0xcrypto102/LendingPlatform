#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Api, Binary, CanonicalAddr, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Storage,BankMsg
};
use cw721::{ContractInfoResponse, QueryMsg as CW721QueryMsg, ExecuteMsg as CW721ExecuteMsg};

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

    NFT_COLLECTIONS.save(dps.storage, &nft_collections)?;

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
        Lend { amount: u128, collection: String,nft_contract: Addr,apy: u16, contract_address: Addr } => exec::lend(
            deps, 
            info, 
            amount,
            collection,
            nft_contract,
            apy,
            contact_address, 
        ),
        Borrow{ sender: Addr, offer_id: u16 } => exec::borrow(
            deps,
            info,
            sender,
            offer_id
        )
    }
}

mod exec {
    use super::*;
    
    pub fn lend(
        deps: DepsMut, 
        info: MessageInfo,
        amount: u128,
        collection: String,
        nft_contract: Addr,
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
            nft_contract: nft_contract,
            token_id:"".to_string(),
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

    pub fn borrow(
        deps: DepsMut,
        info: MessageInfo,
        sender: Addr,
        offer_id: u16
    ) -> Result<Response, ContractError> {
         // Query the storage to retrieve the offer by offer_id
        let offer = OFFERS.may_load(deps.storage, &offer_id)?;

        // Return an error if the offer does not exist
        offer.ok_or_else(|| StdError::generic_err("Offer not found"));

        // Verify that sender owns the NFT from the specific NFT contract address
        let owner_nft_info: TokenInfoResponse = deps.querier.query(
            &CW721QueryMsg::TokenInfo {
                contract_addr: offer.nft_contract.to_string(), // Specify the NFT contract address
            },
        )?;

        // Check if the sender owns the NFT and it is not approved
        if owner_nft_info.owner != sender {
            return Err(StdError::Unauthorized { backtrace: None });
        }
        
        let token_id = owner_nft_info.token_id;
        
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
