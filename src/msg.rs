use cosmwasm_schema::{cw_serde, QueryResponses};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct InstantiateMsg {
    pub nft_collections: Vec<NFTCollectionResp>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum ExecuteMsg {
    Lend { amount: u128, collection: String, nft_contract: Addr,apy: u16,contract_address: Addr }
    // CancelLend { offer_id: u16 }
    Borrow { sender: Addr, offer_id: u16  }
    Repay {}
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[derive(QueryResponses)]
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
    pub amount: u128,
    pub nft_collection: String,
    pub nft_contract: Addr,
    pub token_id: String,
    pub apy_collection: u16,
    pub accepted: bool, 
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct NFTCollectionResp {
    pub nft_collection: String,
    pub nft_contract: String,
    pub apy_collection: u16,
    pub max_time: i64,
}