#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary,to_binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr, BankMsg, CosmosMsg, WasmMsg};
use cw721::{ Cw721ExecuteMsg };
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, OfferResp, ContractConfig, NFTCollectionResp };
use crate::state::{ LEND_DENOM, OFFERS, NFT_COLLECTIONS, LAST_OFFER_INDEX, CONFIG };
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

    let config = ContractConfig { admin: msg.admin };
    CONFIG.save(deps.storage, &config)?;

    LEND_DENOM.save(deps.storage, &"SEI".to_string())?;
    LAST_OFFER_INDEX.save(deps.storage, &0)?;

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
        Lend { amount, collection_id } => exec::lend(
            deps, 
            env,
            info, 
            amount,
            collection_id
        ),
        CancelOffer { offer_id } => exec::cancel_offer(
            deps,
            info,
            offer_id
        ),
        Borrow { offer_id, token_id} => exec::borrow (
            deps,
            env,
            info,
            offer_id,
            token_id
        ),
        UpdateFloorPrice{ collection_id, new_floor_price } => exec::update_floor_price (
            deps,
            info,
            collection_id,
            new_floor_price
        ),
        AddNFTCollection {collection } => exec::add_nft_collection(
            deps,
            info,
            collection
        ),
        UpdateAdmin { new_admin } => exec::update_admin (
            deps,
            info,
            new_admin
        ),
        RepaySuccess { offer_id } => exec::repay_success (
            deps,
            info,
            env,
            offer_id
        )
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
    ) -> Result<Response, ContractError> {
        let denom = LEND_DENOM.load(deps.storage)?;
        let offer_index = LAST_OFFER_INDEX.load(deps.storage)?; 
        let contract_address = env.contract.address.clone();

        // Get the collection associated with the offer
        let collections_option = NFT_COLLECTIONS.may_load(deps.storage)?;

        // Check if loading collections was successful
        if let Some(collections) = collections_option {
            // Find the collection with the specified collection_id
            let collection_option = collections.iter().find(|collection| collection.collection_id == collection_id);
 
            // Check if the collection with the specified collection_id exists
            if let Some(collection) = collection_option {
                if collection.floor_price < amount {
                    return Err(ContractError::TooMuchLendAmount)
                }
            } else {
                // Collection with the specified collection_id not found
                return Err(ContractError::CollectionNotFound);
            }
        } else {
            // Handle the case where loading collections failed
            return Err(ContractError::CollectionLoadFail);
        }
    
        let start_time = env.block.time.seconds();

        let offer = OfferResp {
            offer_id: offer_index + 1,
            owner: info.sender.clone(),
            amount,
            start_time,
            collection_id,
            token_id: "".to_string(), // Adjust the type according to your token identifier type
            accepted: false,
            borrower: Addr::unchecked("none"),
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
        let config = CONFIG.load(deps.storage)?;

        // Load the offer from storage
        let offer = match OFFERS.may_load(deps.storage, offer_id)? {
            Some(offer) => offer,
            None => return Err(ContractError::OfferNotFound), // Return error if offer does not exist
        };

        // Check if the sender is the owner of the offer
        if offer.owner != info.sender {
            if config.admin != info.sender {
                return Err(ContractError::InvalidOfferOwner);
            }
        }

        if offer.accepted {
            return Err(ContractError::OfferAlreadyAccepted);
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
        env: Env,
        info: MessageInfo,
        offer_id: u16,
        token_id: String,
    ) -> Result<Response, ContractError> {
        // Load the offer from storage
        let mut offer = match OFFERS.may_load(deps.storage, offer_id)? {
            Some(offer) => offer,
            None => return Err(ContractError::OfferNotFound), // Return error if offer does not exist
        };
        let contract_address = env.contract.address.clone();
        
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
                    contract_addr: collection.contract.to_string(),
                    msg: to_binary(&msg)?,
                    funds: vec![],
                });
                
                let messages: Vec<CosmosMsg> = vec![execute_msg];
                let _response = Response::new().add_messages(messages);
                
                // Update the offer's token_id and accepted fields
                offer.token_id = token_id;
                offer.accepted = true;
                offer.borrower = info.sender;

                // Save the updated offer back to storage
                OFFERS.save(deps.storage, offer_id, &offer)?;

                // Return success response
                Ok(Response::new()
                    .add_attribute("action", "borrow"))
            } else {
                // Collection with the specified collection_id not found
                return Err(ContractError::CollectionNotFound);
            }
        } else {
            // Handle the case where loading collections failed
            return Err(ContractError::CollectionLoadFail);
        }

       
    }

    pub fn update_floor_price(
        deps: DepsMut,
        info: MessageInfo,
        collection_id: u16,
        new_floor_price: u128
    ) -> Result<Response, ContractError> {
        let config = CONFIG.load(deps.storage)?;

        if config.admin != info.sender {
            return Err(ContractError::Unauthorized);
        }
        // Get the collection associated with the offer
        let mut collections = NFT_COLLECTIONS.load(deps.storage)?;

        // Find the collection with the specified collection_id
        if let Some(collection) = collections.iter_mut().find(|collection| collection.collection_id == collection_id) {
            // Update the floor price of the collection
            collection.floor_price = new_floor_price;

            // Save the updated collection back to storage
            NFT_COLLECTIONS.save(deps.storage, &collections)?;

            Ok(Response::new()
                .add_attribute("action", "update_floor_price"))
        } else {
            // Collection with the specified collection_id not found
            Err(ContractError::CollectionNotFound)
        }
    }

    pub fn add_nft_collection(
        deps: DepsMut,
        info: MessageInfo,
        collection: NFTCollectionResp,
    ) -> Result<Response, ContractError> {
        let config = CONFIG.load(deps.storage)?;

        if config.admin != info.sender {
            return Err(ContractError::Unauthorized);
        }

        let mut collections = NFT_COLLECTIONS.load(deps.storage)?;
        collections.push(collection);

        let _= NFT_COLLECTIONS.save(deps.storage, &collections);

        Ok(Response::new()
                .add_attribute("action", "add_nft_collection"))
    }

    pub fn update_admin(
        deps: DepsMut,
        info: MessageInfo,
        new_admin: Addr
    ) -> Result<Response, ContractError> {
        let mut config = CONFIG.load(deps.storage)?;

        if config.admin != info.sender {
            return Err(ContractError::Unauthorized);
        }
    
        config.admin = new_admin;
        CONFIG.save(deps.storage, &config)?;
        Ok(Response::new()
                .add_attribute("action", "update_admin"))
    }

    pub fn repay_success(
        deps: DepsMut,
        info: MessageInfo,
        env: Env,
        offer_id: u16,
    ) -> Result<Response, ContractError>  {
        // Load the denom
        let denom = LEND_DENOM.load(deps.storage)?;
        // Load the config
        let config = CONFIG.load(deps.storage)?;
        // Load the offer from storage
        let offer = match OFFERS.may_load(deps.storage, offer_id)? {
            Some(offer) => offer,
            None => return Err(ContractError::OfferNotFound), // Return error if offer does not exist
        };
        // Check if the sender is the owner of the offer
        if offer.borrower != info.sender {
            return Err(ContractError::InvalidBorrow);
        }

        // Check if the offer was accepted
        if !offer.accepted {
            return Err(ContractError::OfferNotAccepted);
        }

        // Get the collection associated with the offer
        let collections_option = NFT_COLLECTIONS.may_load(deps.storage)?;

        // Check if loading collections was successful
        if let Some(collections) = collections_option {
            // Find the collection with the specified collection_id
            let collection_option = collections.iter().find(|collection| collection.collection_id == offer.collection_id);

            // Check if the collection with the specified collection_id exists
            if let Some(collection) = collection_option {

                let current_time = env.block.time.seconds();
                if (offer.start_time + collection.max_time) < current_time {
                    //  Send the NFT to the contract address
                    let msg = Cw721ExecuteMsg::TransferNft {
                        recipient: config.admin.to_string(),
                        token_id: offer.token_id.to_string(),
                    };

                    let execute_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: collection.contract.to_string(),
                        msg: to_binary(&msg)?,
                        funds: vec![],
                    });
                    
                    let messages: Vec<CosmosMsg> = vec![execute_msg];
                    Ok(Response::new().add_messages(messages)
                        .add_attribute("action","repay_fail"))
                } else {
                    // Calculate reward
                    let reward = calculate_reward(offer.start_time, collection.apy, current_time, offer.amount);

                    // Send the NFT to the borrower
                    let msg = Cw721ExecuteMsg::TransferNft {
                        recipient: offer.borrower.into(),
                        token_id: offer.token_id.into(),
                    };
                    let execute_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: collection.contract.clone().into(),
                        msg: to_binary(&msg)?,
                        funds: vec![],
                    });

                    // Send the repayment amount (loan amount + reward) to the offer owner
                    let payment_amount = offer.amount + reward;

                    let payment_coin = Coin {
                        denom: LEND_DENOM.load(deps.storage)?,
                        amount: payment_amount.into(),
                    };
                    let payment_msg = BankMsg::Send {
                        to_address: offer.owner.into(),
                        amount: vec![payment_coin],
                    };

                    // Remove the offer from storage
                    OFFERS.remove(deps.storage, offer_id);

                    // Construct and return the response
                    let messages: Vec<CosmosMsg> = vec![execute_msg, CosmosMsg::Bank(payment_msg)];
                    Ok(Response::new().add_messages(messages).add_attribute("action", "repay_success"))
                }
               
            } else {
                // Collection with the specified collection_id not found
                return Err(ContractError::CollectionNotFound);
            }
        } else {
            // Handle the case where loading collections failed
            return Err(ContractError::CollectionLoadFail);
        }
    }

    // Function to calculate reward
    fn calculate_reward(start_time: u64, apy: u16, current_time: u64, amount: u128) -> u128 {
        // Calculate elapsed time in seconds
        let elapsed_time_seconds = current_time - start_time;

        let reward = amount * elapsed_time_seconds as u128 * apy as u128 / (365 * 24 * 60 * 60) as u128;

        reward
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;

    match msg {
        OfferList {limit, start_after} => query::offer_list(deps, limit, start_after),
        // OfferListByOwner { owner } => query::offer_list_by_owner(deps, owner),
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
    use crate::contract::exec::{lend, update_floor_price};
    use crate::contract::{execute, instantiate, query};
    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{ to_binary, Addr };
    use cosmwasm_std::coins;
    // const admin: Addr = Addr::unchecked("owner");

    // Define a unit test to test the instantiate function
    #[test]
    fn test_all_functions() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let admin = Addr::unchecked("creator");
        let info = mock_info("creator", &coins(1000, "SEI"));

        let user = Addr::unchecked("user");
        let user_info = mock_info("user", &coins(500, "SEI")); // User with 500 SEI tokens

        // Sample NFT collections data
        let nft_collections = vec![
            NFTCollectionResp {
                collection_id: 1,
                collection: "Collection 1".to_string(),
                floor_price: 100,
                contract: Addr::unchecked("Contract 1"),
                apy: 5,
                max_time: 100,
            },
            NFTCollectionResp {
                collection_id: 2,
                collection: "Collection 2".to_string(),
                floor_price: 150,
                contract: Addr::unchecked("Contract 2"),
                apy: 7,
                max_time: 130,
            },
        ];

        // Instantiate the contract with sample NFT collections data
        let msg = InstantiateMsg {
            nft_collections: nft_collections.clone(),
            admin: admin.clone(),
        };
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Ensure no error in response
        assert_eq!(0, res.messages.len());
        assert_eq!(0, res.attributes.len());

        // Ensure NFT collections are stored
        let collections = NFT_COLLECTIONS.load(deps.as_ref().storage).unwrap();
        assert_eq!(2, collections.len());

      

        // *************************************************

        // Call the lend function

        // Define parameters for the lend function
        let amount1: u128 = 50;
        let amount2: u128 = 80;

        let collection_id1: u16 = 1;
        let collection_id2: u16 = 2;

        let contract_address = Addr::unchecked("contract");
        
        let res: Response = lend(deps.as_mut(), env.clone(), user_info.clone(), amount1, collection_id1).unwrap();

        let res: Response = lend(deps.as_mut(), env.clone(), user_info.clone(), amount2, collection_id2).unwrap();


        // Verify the state changes
        let offer = OFFERS.may_load(&deps.storage, 1);
        assert_eq!(offer.unwrap().unwrap().offer_id, 1); // Offer ID starts from 1

        // *************************************************
        
        // Update floor Price Test
        let new_floor_price: u128 = 150;
        let response =
         update_floor_price(deps.as_mut(), info.clone(), collection_id1, new_floor_price).unwrap();

        // Verify the response
        assert_eq!(0, response.messages.len());
        assert_eq!(1, response.attributes.len());
        assert_eq!(response.attributes[0], ("action", "update_floor_price"));

        // Verify the updated collection in storage
        let updated_collections = NFT_COLLECTIONS.load(deps.as_ref().storage).unwrap();
        assert_eq!(updated_collections[0].floor_price, new_floor_price);

        
        // *************************************************

        // cancel offer by user
        let offer_id = 1;
        let cancel_offer_msg = ExecuteMsg::CancelOffer { offer_id };
        let res = execute(deps.as_mut(), env.clone(), user_info.clone(), cancel_offer_msg).unwrap();
        assert!(OFFERS.may_load(deps.as_ref().storage, offer_id).unwrap().is_none());

        // cancel offer by owner
         let offer_id = 2;
         let cancel_offer_msg = ExecuteMsg::CancelOffer { offer_id };
         let res = execute(deps.as_mut(), env.clone(), info.clone(), cancel_offer_msg).unwrap();
         assert!(OFFERS.may_load(deps.as_ref().storage, offer_id).unwrap().is_none());
    
        // *************************************************

        // Call the lend function again 

        // Define parameters for the lend function
        let amount: u128 = 50;

        let collection_id: u16 = 1;

        let contract_address = Addr::unchecked("contract");
        
        let res: Response = lend(deps.as_mut(), env.clone(), user_info.clone(), amount, collection_id).unwrap();
        // ************************************************

        // Bororw function
        // Set up mock environment and info
        let borrow_info = mock_info("borrower", &[]);
        let offer_id: u16 = 3;
        let token_id = "token123".to_string();
        let contract_address = Addr::unchecked("contract");

        let borrow_msg = ExecuteMsg::Borrow { offer_id, token_id: token_id.clone() };

        let res = execute(deps.as_mut(), env.clone(), borrow_info.clone(), borrow_msg).unwrap();

        let updated_offer = OFFERS
        .load(&deps.storage, offer_id)
        .expect("failed to load offer");
        assert_eq!(updated_offer.token_id, token_id);
        assert_eq!(updated_offer.accepted, true);
        assert_eq!(updated_offer.borrower, borrow_info.sender);
        // ************************************************

        // Repay function
        let repay_msg = ExecuteMsg::RepaySuccess { offer_id: offer_id.clone() };
        let res = execute(deps.as_mut(), env.clone(), borrow_info.clone(), repay_msg).unwrap();

        // ************************************************

        // add nft collection function
        let collection = NFTCollectionResp {
            collection_id: 3,
            collection: "Collection 3".to_string(),
            floor_price: 200,
            contract: Addr::unchecked("Contract 3"),
            apy: 5,
            max_time: 100,
        };
        let add_collection_msg = ExecuteMsg::AddNFTCollection { collection: collection.clone() };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), add_collection_msg).unwrap();
        
        let collections = NFT_COLLECTIONS.load(deps.as_ref().storage).unwrap();
        assert_eq!(collections.len(), 3);
        assert_eq!(collections[2], collection);

        // ************************************************

        // add new admin
        let new_admin = Addr::unchecked("new_admin");
        let update_admin_msg = ExecuteMsg::UpdateAdmin { new_admin: new_admin.clone() };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), update_admin_msg).unwrap();
        let config = CONFIG.load(deps.as_ref().storage).unwrap();
        assert_eq!(config.admin, new_admin);

        
    }


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
