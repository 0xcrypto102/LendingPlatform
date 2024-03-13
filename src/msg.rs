// use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{ Addr };
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct InstantiateMsg {
    pub nft_collections: Vec<NFTCollectionResp>,
    // pub offers: Vec<OfferResp>,
    pub admin: Addr, 
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum ExecuteMsg {
    Lend { amount: u128, collection_id: u16 },
    CancelOffer { offer_id: u16 },
    Borrow { offer_id: u16, token_id: String },
    UpdateFloorPrice { collection_id: u16, new_floor_price: u128 },
    AddNFTCollection { collection: NFTCollectionResp },
    UpdateAdmin { new_admin: Addr },
    RepaySuccess {offer_id: u16},
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum QueryMsg {
    OfferList { limit: Option<u32>, start_after: Option<u16> },
    // OfferListByOwner { owner: Addr },
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
    pub start_time: u64,
    pub collection_id: u16,
    pub token_id: String,
    pub accepted: bool, 
    pub borrower: Addr,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct NFTCollectionResp {
    pub collection_id: u16,
    pub collection: String,
    pub floor_price: u128,
    pub contract: Addr,
    pub apy: u16,
    pub max_time: u64,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ContractConfig {
    pub admin: Addr,
}