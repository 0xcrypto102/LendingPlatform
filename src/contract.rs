#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary,to_binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr, BankMsg, CosmosMsg, WasmMsg};
use cw721::{ Cw721ExecuteMsg };
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, OfferResp, OfferListResp};
use crate::state::{ LEND_DENOM, OFFERS, NFT_COLLECTIONS, LAST_OFFER_INDEX };

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
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    use ExecuteMsg::*;

    match msg {
        Lend { amount, collection_id, contract_address } => exec::lend(
            deps, 
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
        info: MessageInfo,
        amount: u128,
        collection_id: u16,
        contract_address: Addr,
    ) -> Result<Response, ContractError> {
        let denom = LEND_DENOM.load(deps.storage)?;
        let mut offers = OFFERS.load(deps.storage)?;
        let mut collections = NFT_COLLECTIONS.load(deps.storage)?;
        let offer_index = LAST_OFFER_INDEX.load(deps.storage)?;

        // Find the collection by collection_id
        let collection_index = collections.iter().position(|collection| collection.collection_id == collection_id);

        let current_collection = match collection_index {
            Some(collection_index) => &mut collections[collection_index],
            None => { return Err(ContractError::CollectionNotFound) },
        };

        // Create BankMsg::Send message with the desired lending amount
        let message = BankMsg::Send {
            to_address: contract_address.into_string(),
            amount: vec![Coin {
                denom: denom.to_string(), // Denomination of the payment amount
                amount: amount.into(),    // payment amount
            }],
        };

        let offer = OfferResp {
            offer_id: offer_index +1,
            owner: info.sender,
            amount: amount,
            nft_collection: current_collection.collection.clone(),
            nft_contract: current_collection.contract.clone(),
            token_id:0,
            apy_collection: current_collection.apy,
            max_time: current_collection.max_time,
            accepted: false
        };

        LAST_OFFER_INDEX.save(deps.storage, &(offer_index +1))?;

        offers.push(offer);
        OFFERS.save(deps.storage, &offers)?;
        
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

        // Load all offers
        let mut offers = OFFERS.load(deps.storage)?;

        // Find the offer with the given offer_id
        let offer_index = offers.iter().position(|offer| offer.offer_id == offer_id);


        match offer_index {
            Some(index) => {
                let offer = offers.remove(index);
    
                // Check if the sender is the owner of the offer
                if offer.owner != info.sender {
                    return Err(ContractError::Unauthorized {});
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
                OFFERS.save(deps.storage, &offers)?;
    
                // Return a response with the repayment message
                Ok(Response::new()
                    .add_message(message)
                    .add_attribute("action", "cancel_offer")
                    .add_attribute("amount", offer.amount.to_string())
                    .add_attribute("denom", denom))
            }
            None => Err(ContractError::OfferNotFound {}),
        }
    }

    pub fn borrow(
        deps: DepsMut,
        info: MessageInfo,
        offer_id: u16,
        token_id: u16,
        contract_address: Addr
    ) -> Result<Response, ContractError> {
        // Load the offers from storage
        let mut offers = OFFERS.load(deps.storage)?;

        // Find the offer with the given offer_id
        let offer_index = offers.iter().position(|offer| offer.offer_id == offer_id);

        match offer_index {
            Some(index) => {
                // Check if the offer is not already accepted
                if offers[index].accepted {
                    return Err(ContractError::OfferAlreadyAccepted);
                }
    
                // Send the NFT to the contract address
                let msg = Cw721ExecuteMsg::TransferNft {
                    recipient: contract_address.to_string(),
                    token_id: token_id.to_string(),
                };

                let execute_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: offers[index].nft_contract.clone().into(),
                    msg: to_binary(&msg)?,
                    funds: vec![],
                });
                
                let messages: Vec<CosmosMsg> = vec![execute_msg];
                let response = Response::new().add_messages(messages);
    
                // Update the offer
                offers[index].token_id = token_id.into();
                offers[index].accepted = true;
    
                // Save the updated offers
                OFFERS.save(deps.storage, &offers)?;
    
                Ok(response)
            }
            None => Err(ContractError::OfferNotFound),
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;

    match msg {
        OfferList {} => to_binary(&query::offer_list(deps)?),
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
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::coins;

    // Test instantiate function
    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies(&[]);
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        // Sample NFT collections data
        let nft_collections = vec![
            NFTCollectionResp {
                collection_id: 1,
                collection: "Collection 1".to_string(),
                contract: "Contract 1".to_string(),
                apy: 5,
                max_time: 100,
            },
            NFTCollectionResp {
                collection_id: 2,
                collection: "Collection 2".to_string(),
                contract: "Contract 2".to_string(),
                apy: 7,
                max_time: 150,
            },
        ];

        // Instantiate the contract with sample NFT collections data
        let msg = InstantiateMsg {
            nft_collections: nft_collections.clone(),
            offers: vec![],
        };
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Ensure no error in response
        assert_eq!(0, res.messages.len());
        assert_eq!(0, res.attributes.len());

        // Ensure NFT collections are stored
        let collections = NFT_COLLECTIONS.load(deps.as_ref().storage).unwrap();
        assert_eq!(0, collections.len());
    }

    // Test execute function for Lend variant
    #[test]
    fn test_execute_lend() {
        let mut deps = mock_dependencies(&[]);
        let env = mock_env();
        let info = mock_info("sender", &coins(1000, "earth"));

        // Instantiate the contract
        let msg = InstantiateMsg {
            nft_collections: vec![],
            offers: vec![],
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Call execute with Lend variant
        let msg = ExecuteMsg::Lend {
            amount: 100,
            collection_id: 1,
            contract_address: Addr::unchecked("contract"),
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Ensure correct response with no messages or attributes
        assert_eq!(0, res.messages.len());
        assert_eq!(0, res.attributes.len());
    }

    // Test query function for OfferList variant
    #[test]
    fn test_query_offer_list() {
        let mut deps = mock_dependencies(&[]);
        let env = mock_env();
        let info = mock_info("anyone", &coins(1000, "earth"));

        // Instantiate the contract
        let msg = InstantiateMsg {
            nft_collections: vec![],
            offers: vec![],
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Call query with OfferList variant
        let query_msg = QueryMsg::OfferList {};
        let res: OfferListResp = query(deps.as_ref(), env.clone(), query_msg).unwrap();

        // Ensure empty offers list initially
        assert_eq!(0, res.offers.len());
    }
        // Test execute function for CancelOffer variant
    #[test]
    fn test_execute_cancel_offer() {
        let mut deps = mock_dependencies(&[]);
        let env = mock_env();
        let sender = String::from("sender");
        let info = mock_info(&sender, &coins(1000, "earth"));

        // Instantiate the contract
        let msg = InstantiateMsg {
            nft_collections: vec![],
            offers: vec![],
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create an offer
        let lend_msg = ExecuteMsg::Lend {
            amount: 100,
            collection_id: 1,
            contract_address: Addr::unchecked("contract"),
        };
        execute(deps.as_mut(), env.clone(), info.clone(), lend_msg).unwrap();

        // Get the offer ID
        let offer_id = 1;

        // Call execute with CancelOffer variant
        let cancel_offer_msg = ExecuteMsg::CancelOffer { offer_id };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), cancel_offer_msg).unwrap();

        // Ensure correct response with no messages or attributes
        assert_eq!(0, res.messages.len());
        assert_eq!(0, res.attributes.len());
    }

}
