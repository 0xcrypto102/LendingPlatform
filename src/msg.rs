use cosmwasm_schema::{cw_serde, QueryResponses};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct InstantiateMsg {
    pub nft_collections: Vec<String>,
    pub apy_collections: Vec<u16>,
    pub max_time_collections: Vec<i64>,
    pub offers: Vec<OfferResp>,
    pub offer_index: u16;
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum ExecuteMsg {
    Lend { amount: u128, collection: String, apy: u16,contract_address: Addr }
    // CancelLend { offer_id: u16 }
    Borrow { collection: String }
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
pub struct OfferResp {
    pub offer_id: u16,
    pub amount: u128,
    pub nft_collection: String,
    pub apy_collection: u16,
    pub accepted: bool, 
}