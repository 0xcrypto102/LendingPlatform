#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary,to_binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr, BankMsg, CosmosMsg, WasmMsg, Order};
use cw721::{ Cw721ExecuteMsg };
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, OfferResp, OfferListResp};
use crate::state::{ DEFAULT_LIMIT,LEND_DENOM, OFFERS, NFT_COLLECTIONS, LAST_OFFER_INDEX };
use cw_paginate_storage::paginate_map;      
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
) -> StdResult<Response> {
    let nft_collections = msg.nft_collections;

    NFT_COLLECTIONS.save(deps.storage, &nft_collections)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    use ExecuteMsg::*;

    match msg {
        Lend { amount, collection_id, contract_address } => exec::lend(
            deps, 
            env,
            info, 
            amount,
            collection_id,
            contract_address, 
        ),
        CancelOffer { offer_id } => exec::cancel_offer(
            deps,
            info,
            offer_id
        ),
        Borrow { offer_id, token_id, contract_address} => exec::borrow (
            deps,
            info,
            offer_id,
            token_id,
            contract_address
        ),
    }
}

mod exec {
    use super::*;

    pub fn lend(
        deps: DepsMut, 
        env: Env,
        info: MessageInfo,
        amount: u128,
        collection_id: u16,
        contract_address: Addr,
    ) -> Result<Response, ContractError> {
        let denom = LEND_DENOM.load(deps.storage)?;
        let offer_index = LAST_OFFER_INDEX.load(deps.storage)?;
    
        let start_time = env.block.time.seconds();
    
        let offer = OfferResp {
            offer_id: offer_index + 1,
            owner: info.sender.clone(),
            amount,
            start_time,
            collection_id,
            token_id: "".to_string(), // Adjust the type according to your token identifier type
            accepted: false,
        };

        // Create BankMsg::Send message with the desired lending amount
        let message = BankMsg::Send {
            to_address: contract_address.into_string(),
            amount: vec![Coin {
                denom: denom.to_string(), // Denomination of the payment amount
                amount: amount.into(),    // Payment amount
            }],
        };
    
        // Save the offer and update the last offer index
        OFFERS.save(deps.storage, offer.offer_id, &offer)?;
        LAST_OFFER_INDEX.save(deps.storage, &(offer_index + 1))?;
            
        // Return the BankMsg::Send message as a response
        Ok(Response::new()
            .add_message(message)
            .add_attribute("action", "lend"))
        
    }

    pub fn cancel_offer(
        deps: DepsMut,
        info: MessageInfo,
        offer_id: u16
    ) -> Result<Response, ContractError> {
        // Load the denom
        let denom = LEND_DENOM.load(deps.storage)?;

        // Load the offer from storage
        let offer = match OFFERS.may_load(deps.storage, offer_id)? {
            Some(offer) => offer,
            None => return Err(ContractError::OfferNotFound), // Return error if offer does not exist
        };

        // Check if the sender is the owner of the offer
        if offer.owner != info.sender {
            return Err(ContractError::InvalidOfferOwner);
        }

     
        // Repay the amount to the sender
        let message = BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![Coin {
                denom: denom.to_string(),
                amount: offer.amount.into(),
            }],
        };

        // Remove the offer from storage
        OFFERS.remove(deps.storage, offer_id);
            
        // Return a response with the repayment message
        Ok(Response::new()
            .add_message(message)
            .add_attribute("action", "cancel_offer")
            .add_attribute("amount", offer.amount.to_string())
            .add_attribute("denom", denom))
    }

    pub fn borrow(
        deps: DepsMut,
        info: MessageInfo,
        offer_id: u16,
        token_id: String,
        contract_address: Addr
    ) -> Result<Response, ContractError> {
        // Load the offer from storage
        let mut offer = match OFFERS.may_load(deps.storage, offer_id)? {
            Some(offer) => offer,
            None => return Err(ContractError::OfferNotFound), // Return error if offer does not exist
        };

        
        // Check if the offer is not already accepted
        if offer.accepted {
            return Err(ContractError::OfferAlreadyAccepted);
        }

        // Get the collection associated with the offer
        let collections_option = NFT_COLLECTIONS.may_load(deps.storage)?;

        // Check if loading collections was successful
        if let Some(collections) = collections_option {
            // Find the collection with the specified collection_id
            let collection_option = collections.iter().find(|collection| collection.collection_id == offer.collection_id);

            // Check if the collection with the specified collection_id exists
            if let Some(collection) = collection_option {
                // Send the NFT to the contract address
                let msg = Cw721ExecuteMsg::TransferNft {
                    recipient: contract_address.to_string(),
                    token_id: token_id.to_string(),
                };

                let execute_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: collection.contract.clone(),
                    msg: to_binary(&msg)?,
                    funds: vec![],
                });
                
                let messages: Vec<CosmosMsg> = vec![execute_msg];
                let response = Response::new().add_messages(messages);

                
                // Update the offer's token_id and accepted fields
                offer.token_id = token_id;
                offer.accepted = true;

                // Save the updated offer back to storage
                OFFERS.save(deps.storage, offer_id, &offer)?;

                // Return success response
                Ok(Response::new())
            } else {
                // Collection with the specified collection_id not found
                return Err(ContractError::CollectionNotFound);
            }
        } else {
            // Handle the case where loading collections failed
            return Err(ContractError::CollectionLoadFail);
        }

       
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;

    match msg {
        OfferList {limit, start_after} => query::offer_list(deps, limit, start_after),
    }
}

mod query {
    use super::*;

   // Implement the offer_list function to query all offers with pagination
    pub fn offer_list(deps: Deps, limit: Option<u32>, start_after: Option<u16>) -> StdResult<Binary> {
        to_binary(&paginate_map(deps, &OFFERS, start_after, limit, cosmwasm_std::Order::Descending,)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::coins;

    // Define a unit test to test the instantiate function
    // #[test]
    // fn test_instantiate() {
    //     let mut deps = mock_dependencies(&[]);
    //     let env = mock_env();
    //     let info = mock_info("creator", &coins(1000, "earth"));

    //     // Sample NFT collections data
    //     let nft_collections = vec![
    //         NFTCollectionResp {
    //             collection_id: 1,
    //             collection: "Collection 1".to_string(),
    //             contract: "Contract 1".to_string(),
    //             apy: 5,
    //             max_time: 100,
    //         },
    //         NFTCollectionResp {
    //             collection_id: 2,
    //             collection: "Collection 2".to_string(),
    //             contract: "Contract 2".to_string(),
    //             apy: 7,
    //             max_time: 150,
    //         },
    //     ];

    //     // Instantiate the contract with sample NFT collections data
    //     let msg = InstantiateMsg {
    //         nft_collections: nft_collections.clone(),
    //         offers: vec![],
    //     };
    //     let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    //     // Ensure no error in response
    //     assert_eq!(0, res.messages.len());
    //     assert_eq!(0, res.attributes.len());

    //     // Ensure NFT collections are stored
    //     let collections = NFT_COLLECTIONS.load(deps.as_ref().storage).unwrap();
    //     assert_eq!(2, collections.len());
    // }

    // // Define a unit test to test the execute function for Lend variant
    // #[test]
    // fn test_execute_lend() {
    //     let (mut deps, env, info) = mock_dependencies_with_custom_querier_and_instantiate(vec![]);
    //     let amount = 100;
    //     let collection_id = 1;
    //     let contract_address = Addr::unchecked("contract");

    //     // Call execute with Lend variant
    //     let msg = ExecuteMsg::Lend {
    //         amount,
    //         collection_id,
    //         contract_address: contract_address.clone(),
    //     };
    //     let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    //     // Ensure correct response with no messages or attributes
    //     assert_eq!(0, res.messages.len());
    //     assert_eq!(0, res.attributes.len());

    //     // Ensure the offer is stored
    //     let offers = OFFERS
    //         .range(None, None, Order::Ascending)
    //         .unwrap()
    //         .collect::<Vec<_>>();
    //     assert_eq!(1, offers.len());

    //     // Ensure the last offer index is updated
    //     let last_offer_index = LAST_OFFER_INDEX.load(deps.as_ref().storage).unwrap();
    //     assert_eq!(1, last_offer_index);
    // }

    // // Define a unit test to test the execute function for CancelOffer variant
    // #[test]
    // fn test_execute_cancel_offer() {
    //     let (mut deps, env, info) = mock_dependencies_with_custom_querier_and_instantiate(vec![]);
    //     let sender = String::from("sender");

    //     // Create an offer
    //     let lend_msg = ExecuteMsg::Lend {
    //         amount: 100,
    //         collection_id: 1,
    //         contract_address: Addr::unchecked("contract"),
    //     };
    //     execute(deps.as_mut(), env.clone(), info.clone(), lend_msg).unwrap();

    //     // Get the offer ID
    //     let offer_id = 1;

    //     // Call execute with CancelOffer variant
    //     let cancel_offer_msg = ExecuteMsg::CancelOffer { offer_id };
    //     let res = execute(deps.as_mut(), env.clone(), info.clone(), cancel_offer_msg).unwrap();

    //     // Ensure correct response with no messages or attributes
    //     assert_eq!(0, res.messages.len());
    //     assert_eq!(0, res.attributes.len());

    //     // Ensure the offer is removed
    //     assert!(OFFERS.may_load(deps.as_ref().storage, offer_id).unwrap().is_none());
    // }

    // // Define a unit test to test the query function for OfferList variant
    // #[test]
    // fn test_query_offer_list() {
    //     let (mut deps, env, info) = mock_dependencies_with_custom_querier_and_instantiate(vec![]);

    //     // Call query with OfferList variant
    //     let query_msg = QueryMsg::OfferList {};
    //     let res: OfferListResp = query(deps.as_ref(), env.clone(), query_msg).unwrap();

    //     // Ensure empty offers list initially
    //     assert_eq!(0, res.offers.len());
    // }

}
