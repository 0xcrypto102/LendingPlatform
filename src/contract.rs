#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary,to_binary, Empty,  Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr, BankMsg, CosmosMsg, WasmMsg};
use cw721::{ Cw721ExecuteMsg };
use cw_utils::PaymentError;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, OfferResp, OfferListResp, ContractConfig, NFTCollectionResp, NFTCollectionListResp };
use crate::state::{ LEND_DENOM, OFFERS, OFFERS_OWNER, OFFERS_ACCEPT_BORROW, NFT_COLLECTIONS, LAST_OFFER_INDEX, CONFIG };
use cw_paginate_storage::{paginate_map, paginate_map_values};   
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

    for collection_resp in nft_collections {
        NFT_COLLECTIONS.save(deps.storage, collection_resp.collection_id, &collection_resp)?;
    }

    // NFT_COLLECTIONS.save(deps.storage, &nft_collections)?;

    let config = ContractConfig { admin: msg.admin, interest: msg.interest };
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
            env,
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
        UpdateInterest { interest } => exec::update_interest(
            deps,
            info,
            interest
        ),
        Repay { offer_id } => exec::repay (
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
        // Get the collection associated with the offer
        let collection = match NFT_COLLECTIONS.may_load(deps.storage, collection_id)? {
            Some(collection) => collection,
            None => return Err(ContractError::CollectionNotFound),
        };

      
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

        let donation = match cw_utils::must_pay(&info, &denom) {
            Ok(payment) => {
                if payment.u128() != amount {
                    return Err(ContractError::NotExactAmount);
                }
                if collection.floor_price < payment.u128() {
                    return Err(ContractError::TooMuchLendAmount)
                }
            },
            Err(err) => return Err(ContractError::DepositFail),
        };
       
        // Save the offer and update the last offer index
        OFFERS.save(deps.storage, offer.offer_id, &offer)?;
        
        // Load existing offers for the owner, or initialize an empty vector
        let mut offers_owner = match OFFERS_OWNER.may_load(deps.storage, info.sender.clone())? {
            Some(offers_owner) => offers_owner,
            None => Vec::new(), // Initialize an empty vector if no offers exist for the owner
        };
        // Add the new offer ID to the vector
        offers_owner.push(offer_index + 1);
        // Save the updated vector of offer IDs for the owner
        OFFERS_OWNER.save(deps.storage, info.sender.clone(), &offers_owner)?;

        LAST_OFFER_INDEX.save(deps.storage, &(offer_index + 1))?;
        // Return the BankMsg::Send message as a response
        Ok(Response::new()
            .add_attribute("action", "lend"))
    }

    pub fn cancel_offer(
        deps: DepsMut,
        info: MessageInfo,
        env: Env,
        offer_id: u16
    ) -> Result<Response, ContractError> {
        // Load the denom
        let denom = LEND_DENOM.load(deps.storage)?;
        let config = CONFIG.load(deps.storage)?;
        let contract_address = env.contract.address.clone();


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
            to_address: offer.owner.to_string(),
            amount: vec![Coin {
                denom: denom.to_string(),
                amount: offer.amount.into(),
            }],
        };
        
        // Remove the offer from storage
        OFFERS.remove(deps.storage, offer_id);

        // Load existing offers for the owner, or initialize an empty vector
        let mut offers_owner =  OFFERS_OWNER.load(deps.storage, info.sender.clone())?;
        // Add the new offer ID to the vector
        offers_owner.retain(|&index| index != offer_id);
        // Save the updated vector of offer IDs for the owner
        OFFERS_OWNER.save(deps.storage, info.sender.clone(), &offers_owner)?;
            
        // Return a response with the repayment message
        Ok(Response::new()
            .add_message(message)
            .add_attribute("action", "cancel_offer")
            .add_attribute("denom", denom))
    }

    pub fn borrow(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        offer_id: u16,
        token_id: String,
    ) -> Result<Response, ContractError> {
        let denom = LEND_DENOM.load(deps.storage)?;
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
        let collection = match NFT_COLLECTIONS.may_load(deps.storage, offer.collection_id)? {
            Some(collection) => collection,
            None => return Err(ContractError::CollectionNotFound),
        };
        
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

        let fund_msg = BankMsg::Send {
            to_address: info.sender.clone().into_string(),
            amount: vec![Coin {
                denom: denom.to_string(), // Denomination of the payment amount
                amount: offer.amount.into(),    // Payment amount
            }],
        };
    
        let messages: Vec<CosmosMsg> = vec![CosmosMsg::Bank(fund_msg), execute_msg];
        // let messages: Vec<CosmosMsg> = vec![CosmosMsg::Bank(fund_msg)];

        // Update the offer's token_id and accepted fields
        offer.token_id = token_id;
        offer.accepted = true;
        offer.borrower = info.sender.clone();

        // Save the updated offer back to storage
        OFFERS.save(deps.storage, offer_id, &offer)?;

        // Load existing offers for the owner, or initialize an empty vector
        let mut offers_accept_borrow = match OFFERS_ACCEPT_BORROW.may_load(deps.storage, info.sender.clone())? {
            Some(offers_accept_borrow) => offers_accept_borrow,
            None => Vec::new(), // Initialize an empty vector if no offers exist for the owner
        };
        // Add the new offer ID to the vector
        offers_accept_borrow.push(offer_id);
        // Save the updated vector of offer IDs for the owner
        OFFERS_ACCEPT_BORROW.save(deps.storage, info.sender.clone(), &offers_accept_borrow)?;

        // Return success response
        Ok(Response::new()
            .add_messages(messages)
            .add_attribute("action", "borrow"))
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

        let mut collection = match NFT_COLLECTIONS.may_load(deps.storage, collection_id)? {
            Some(collection) => collection,
            None => return Err(ContractError::CollectionNotFound),
        };

        // Update the floor price of the collection
        collection.floor_price = new_floor_price;

        // Save the updated collection back to storage
        NFT_COLLECTIONS.save(deps.storage, collection_id, &collection)?;

        Ok(Response::new()
            .add_attribute("action", "update_floor_price"))
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

        NFT_COLLECTIONS.save(deps.storage,collection.collection_id,  &collection);

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

    pub fn update_interest(
        deps: DepsMut,
        info: MessageInfo,
        interest: u128
    ) -> Result<Response, ContractError> {
        let mut config = CONFIG.load(deps.storage)?;

        if config.admin != info.sender {
            return Err(ContractError::Unauthorized);
        }
    
        config.interest = interest;
        CONFIG.save(deps.storage, &config)?;
        Ok(Response::new()
            .add_attribute("action", "update_interest"))
    }
 
    pub fn repay(
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
        let collection = match NFT_COLLECTIONS.may_load(deps.storage, offer.collection_id)? {
            Some(collection) => collection,
            None => return Err(ContractError::CollectionNotFound),
        };

        let current_time = env.block.time.seconds();
        // this is the case when the borrow couldn't repay fund in time
        if offer.start_time + collection.max_time < current_time {
            //  Send the NFT to the contract address
            let msg = Cw721ExecuteMsg::TransferNft {
                recipient: offer.owner.to_string(),
                token_id: offer.token_id.to_string(),
            };

            let execute_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: collection.contract.to_string(),
                msg: to_binary(&msg)?,
                funds: vec![],
            });
            
            let messages: Vec<CosmosMsg> = vec![execute_msg];
            // Offer remove
            OFFERS.remove(deps.storage, offer_id);

            // Load existing offers for the owner, or initialize an empty vector
            let mut offers_owner = OFFERS_OWNER.load(deps.storage, info.sender.clone())?;

            // Remove the offer index from the vector
            offers_owner.retain(|&index| index != offer_id);
            // Save the updated vector of offer IDs for the owner
            OFFERS_OWNER.save(deps.storage, info.sender.clone(), &offers_owner)?;

            // Load existing offers for the owner, or initialize an empty vector
            let mut offers_accept_borrow =  OFFERS_ACCEPT_BORROW.load(deps.storage, info.sender.clone())?;
            // Add the new offer ID to the vector
            offers_accept_borrow.retain(|&index| index != offer_id);
            // Save the updated vector of offer IDs for the owner
            OFFERS_ACCEPT_BORROW.save(deps.storage, info.sender.clone(), &offers_accept_borrow)?;

            Ok(Response::new().add_messages(messages)
                .add_attribute("action","repay_fail"))
                
            // Ok(Response::new()
            // .add_attribute("action","repay_fail"))
        } else {
            // Calculate reward
            let reward = calculate_reward(offer.start_time, collection.apy, current_time, offer.amount);

            let donation = match cw_utils::must_pay(&info, &denom) {
                Ok(payment) => {
                    if payment.u128() != reward + offer.amount {
                        return Err(ContractError::NotExactAmount);
                    }
                },
                Err(err) => return Err(ContractError::DepositFail),
            };

            // Send the NFT to the borrower
            let msg = Cw721ExecuteMsg::TransferNft {
                recipient: offer.borrower.into(),
                token_id: offer.token_id.into(),
            };
            let execute_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: collection.contract.clone().into(),
                msg: to_binary(&msg)?,
                funds: vec![Coin {
                    denom: LEND_DENOM.load(deps.storage)?,
                    amount: (offer.amount + reward).into(),
                }],
            });
            

            // Send the repayment amount (loan amount + reward) to the offer owner
            let payment_amount = offer.amount + reward * config.interest / 100;

            let payment_coin = Coin {
                denom: LEND_DENOM.load(deps.storage)?,
                amount: payment_amount.into(),
            };
            let payment_msg = BankMsg::Send {
                to_address: offer.owner.into(),
                amount: vec![payment_coin],
            };

            // Send the repayment amount (loan amount + reward) to the admin
            let payment_amount_owner = reward * (100 - config.interest) / 100;

            let payment_coin = Coin {
                denom: LEND_DENOM.load(deps.storage)?,
                amount: payment_amount_owner.into(),
            };

            let payment_msg_owner = BankMsg::Send {
                to_address: config.admin.into(),
                amount: vec![payment_coin],
            };
            // Remove the offer from storage
            OFFERS.remove(deps.storage, offer_id);

            // Load existing offers for the owner, or initialize an empty vector
            let mut offers_owner = OFFERS_OWNER.load(deps.storage, info.sender.clone())?;
            // Remove the offer index from the vector
            offers_owner.retain(|&index| index != offer_id);
            // Save the updated vector of offer IDs for the owner
            OFFERS_OWNER.save(deps.storage, info.sender.clone(), &offers_owner)?;

            // Load existing offers for the owner, or initialize an empty vector
            let mut offers_accept_borrow =  OFFERS_ACCEPT_BORROW.load(deps.storage, info.sender.clone())?;
            // Add the new offer ID to the vector
            offers_accept_borrow.retain(|&index| index != offer_id);
            // Save the updated vector of offer IDs for the owner
            OFFERS_ACCEPT_BORROW.save(deps.storage, info.sender.clone(), &offers_accept_borrow)?;

            // Construct anxs
            Ok(Response::new()
                .add_message(execute_msg)
                .add_message(payment_msg)
                .add_message(payment_msg_owner)
                .add_attribute("action", "repay success")
            )
        }
    }

    // Function to calculate reward
    pub fn calculate_reward(start_time: u64, apy: u16, current_time: u64, amount: u128) -> u128 {
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
        OfferList {start, stop} => query::offer_list(deps,start, stop),
        OfferByID {offer_id} => query::offer_by_id(deps, offer_id),
        OffersByOwner {owner} => query::get_offers_by_owner(deps, owner),
        OffersAcceptByBorrow {borrower} => query::get_offers_accept_by_borrower(deps, borrower),
        OffersByPrice {limit, sort} => query::get_offers_by_price(deps, limit, sort),
        CollectionByID {collection_id} => query::collection_by_id(deps, collection_id),
        QueryAdmin {} => query::query_admin(deps),
    }
}

mod query {
    use super::*;

   // Implement the offer_list function to query all offers with pagination
    pub fn offer_list(deps: Deps, start: u16, stop: u16) -> StdResult<Binary> {
        // let offers = paginate_map_values(deps, &OFFERS, start_after, limit, cosmwasm_std::Order::Descending,)?;
        // let offer_list_resp = OfferListResp { offers };
        
        // Ok(to_binary(&offer_list_resp)?)
        let mut offers: Vec<OfferResp> = Vec::new();

        for i in start..=stop {
            match OFFERS.load(deps.storage, i) {
                Ok(offer) => {
                    offers.push(offer);
                }
                Err(_) => {
                    // Offer not found, continue to the next iteration
                    continue;
                }
            }
        }
        Ok(to_binary(&offers)?)
    }

    pub fn offer_by_id(deps: Deps, offer_id: u16) -> StdResult<Binary> {
        let offer = OFFERS.load(deps.storage, offer_id)?;
        let resp_binary = to_binary(&offer)?;
        Ok(resp_binary)
    }

    // Query offers by owner from OFFERS_OWNER
    pub fn get_offers_by_owner(deps: Deps, owner: Addr) -> StdResult<Binary> {
        let offer_ids = OFFERS_OWNER.load(deps.storage, owner.clone())?;
        let mut offers = Vec::new();
      
        for offer_id in offer_ids {
            let offer = OFFERS.load(deps.storage, offer_id)?;
            offers.push(offer);
        }
      
        let resp_offers = to_binary(&offers)?;
        Ok(resp_offers)
    }

    pub fn get_offers_by_price(deps: Deps, limit: u128, sort: bool) -> StdResult<Binary> {
        let total_offers = LAST_OFFER_INDEX.load(deps.storage)?; 
        let mut resp_offers = Vec::new();
        for i in 1..=total_offers {
            let offer = match OFFERS.may_load(deps.storage, i)? {
                Some(offer) if offer.amount > limit => resp_offers.push(offer),
                _ => continue,
            };
        }

         // Sort offers based on price if `sort` flag is set to true
        if sort {
            resp_offers.sort_by(|a, b| {
                // Compare offers based on their amount (price) in reverse order (high to low)
                b.amount.cmp(&a.amount)
            });
        } else {
            // Sort offers based on price in ascending order (low to high)
            resp_offers.sort_by(|a, b| {
                // Compare offers based on their amount (price)
                a.amount.cmp(&b.amount)
            });
        }
      
        let result = to_binary(&resp_offers)?;
        Ok(result)
    }
    

    pub fn get_offers_accept_by_borrower(deps: Deps, borrower: Addr) -> StdResult<Binary> {
        let offer_ids = OFFERS_ACCEPT_BORROW.load(deps.storage, borrower.clone())?;
        let mut offers = Vec::new();
      
        for offer_id in offer_ids {
            let offer = OFFERS.load(deps.storage, offer_id)?;
            offers.push(offer);
        }
      
        let resp_offers = to_binary(&offers)?;
        Ok(resp_offers)
    }

    pub fn collection_by_id(deps: Deps, collection_id: u16) -> StdResult<Binary> {
        let collection = NFT_COLLECTIONS.load(deps.storage, collection_id)?;
        let resp_binary = to_binary(&collection)?;
        Ok(resp_binary)
    }

    pub fn query_admin(deps: Deps) -> StdResult<Binary> {
        let admin = CONFIG.load(deps.storage)?;
        let resp = ContractConfig { admin: admin.clone().admin, interest: admin.clone().interest };
        let resp_binary = to_binary(&resp)?;
        Ok(resp_binary)
    }
 }


 #[cfg(test)]
 mod tests {
 }