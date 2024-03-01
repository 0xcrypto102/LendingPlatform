#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coins,Coin, to_binary, Api, Binary, CanonicalAddr, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Storage,BankMsg, Addr, CosmosMsg,WasmMsg
};
use cw721::{ContractInfoResponse,Cw721ReceiveMsg };
use cw_utils::must_pay;
use crate::state::{ NFT_COLLECTIONS, LEND_DENOM, OFFERS, OFFER_INDEX };
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, OfferResp, OfferListResp, NFTCollectionResp };

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
    let nft_collections = msg.nft_collections;

    NFT_COLLECTIONS.save(deps.storage, &nft_collections)?;

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
        Lend { amount, collection,nft_contract,apy, contract_address } => exec::lend(
            deps, 
            info, 
            amount,
            collection,
            nft_contract,
            apy,
            contract_address, 
        ),
        Borrow{ sender, token_id, lend_platform, offer_id } => exec::borrow(
            deps,
            info,
            sender,
            token_id,
            lend_platform,
            offer_id
        ),
        CancelOffer { offer_id } => exec::cancel_offer(
            deps,
            info,
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
        contract_address: Addr,
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
            return StdError::generic_error("No Invalid Amount");
        }

        // Create BankMsg::Send message with the desired lending amount
        let message = BankMsg::Send {
            to_address: contract_address.into_string(),
            amount: coins(amount, &denom),
        };

        let offer = OfferResp {
            offer_id: offer_index +1,
            owner: info.sender,
            amount: amount,
            nft_collection: collection,
            nft_contract: nft_contract,
            token_id:"".to_string(),
            apy_collection: apy,
            accepted: false
        };

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
        token_id: String,
        lend_platform: Addr,
        offer_id: u16
    ) -> Result<Response, ContractError> {
        // Query the storage to retrieve the existing offers
        let mut offers: Vec<OfferResp> = OFFERS.load(deps.storage)?;

        // Find the offer by offer_id
        let offer_index = offers.iter().position(|offer| offer.offer_id == offer_id);
        let offer = match offer_index {
            Some(index) => &mut offers[index],
            None => return StdError::generic_err("Offer not found"),
        };

        let contract_addr = offer.contract_addr;
        let sender = info.sender;

        let msg = Cw721ReceiveMsg::TransferNft {
            sender: info.sender.to_string(),
            recipient: lend_platform,
            token_id: token_id.clone(),
        };

         // Create a CosmosMsg that calls the CW721 contract's `receive` entry point with the transfer message
        let cw721_msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.into(),
            msg: to_binary(&msg)?,
            send: vec![],
        });

        // Check if there are any remaining funds in the message
        if !info.funds.is_empty() {
            return StdError::generic_err("No funds should be sent");
        }

        offer.token_id = token_id.clone();
        offer.accepted = true;

        OFFERS.save(deps.storage, &offers);

        // Create and return a response with the transfer messages
        Ok(Response::new()
            .add_messages(msg)
            .add_attributes(vec![
                ("action", "transfer_nft"),
                ("sender", sender.as_str()),
                ("recipient", lend_platform.as_str()),
                ("token_id", token_id.as_str()),
            ]))
        
    }

    pub fn cancel_offer (
        deps: DepsMut,
        info: MessageInfo,
        offer_id: u16,
    ) -> Result<Response, ContractError> {
        let denom = LEND_DENOM.load(deps.storage)?;
        // Query the storage to retrieve the offer by offer_id
        let offer = OFFERS.may_load(deps.storage, &offer_id)?;

        // Return an error if the offer does not exist
        offer.ok_or_else(|| StdError::generic_err("Offer not found"));
        let sender = offer.owner;

        if sender != info.sender {
            return Err(StdError::Unauthorized { backtrace: None });
        }

        // Create BankMsg::Send message from contract address to sender's address
        let message = BankMsg::Send {
            to_address: info.sender.to_string(), // Sender's address
            amount: vec![Coin {
                denom: denom.to_string(), // Denomination of the repayment amount
                amount: offer.amount.into(),    // Repayment amount
            }],
        };

        let amount = offer.amount;

        // Try to remove the offer from the storage
        let removed_offer = OFFERS.may_remove(deps.storage, offer_id)?;

        // Create and return a response with the repayment message
        Ok(Response::new()
            .add_message(message)
            .add_attribute("action", "cancel_offer")
            .add_attribute("amount", amount.to_string())
            .add_attribute("denom", denom))
    
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
