use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{ Addr };
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct InstantiateMsg {
    pub nft_collections: Vec<NFTCollectionResp>,
    pub offers: Vec<OfferResp>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum ExecuteMsg {
    Lend { amount: u128, collection_id: u16, contract_address: Addr },
    CancelOffer { offer_id: u16 },
    Borrow { offer_id: u16, token_id: u16, contract_address: Addr },
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum QueryMsg {
    OfferList {}
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct OfferListResp {
   pub offers: Vec<OfferResp>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct NFTCollectionListResp {
   pub nftcollections: Vec<NFTCollectionResp>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct OfferResp {
    pub offer_id: u16,
    pub owner: Addr,
    pub amount: u128,
    pub nft_collection: String,
    pub nft_contract: String,
    pub token_id: u16,
    pub apy_collection: u16,
    pub max_time: i64,
    pub accepted: bool, 
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct NFTCollectionResp {
    pub collection_id: u16,
    pub collection: String,
    pub contract: String,
    pub apy: u16,
    pub max_time: i64,
}