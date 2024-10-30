// use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{ Addr };
use serde::{Deserialize, Serialize};
use cw_storage_plus::{UniqueIndex/*,MultiIndex*/, IndexedMap, Index, IndexList};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct InstantiateMsg {
    pub nft_collections: Vec<NFTCollectionResp>,
    // pub offers: Vec<OfferResp>,
    pub admin: Addr, 
    pub interest: u128,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum ExecuteMsg {
    Lend { amount: u128, collection_id: u16 },
    CancelOffer { offer_id: u16 },
    Borrow { owner: Addr, offer_id: u16, token_id: String },
    UpdateFloorPrice { collection_id: u16, new_floor_price: u128 },
    AddNFTCollection { collection: NFTCollectionResp },
    UpdateAdmin { new_admin: Addr },
    UpdateInterest { interest: u128 },
    Repay {owner: Addr, offer_id: u16},
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum QueryMsg {
    OfferList { page_size: u16, page_num: u16},
    OfferByID {offer_id: u16},
    OffersByOwner {owner: Addr, page_size: u16, page_num: u16}, 

    OffersAcceptByBorrow {borrower: Addr ,page_size: u16, page_num: u16}, 
    OffersByPrice {page:u16, page_size:u16, limit: u128, sort: bool},
    CollectionByID { collection_id: u16 },
    QueryAdmin {},
}


#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct NFTCollectionListResp {
   pub nftcollections: Vec<NFTCollectionResp>,
}


#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct OfferListResp {
   pub offers: Vec<OfferResp>,
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

pub struct OfferRespIndexes<'a> {
    pub id: UniqueIndex<'a, u16, OfferResp, (&'a Addr, u16)>,
    // pub borrow: MultiIndex<'a, u16, OfferResp, (&'a Addr, u16)>,
}

impl IndexList<OfferResp> for OfferRespIndexes<'_> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<OfferResp>> + '_> {
        let v: Vec<&dyn Index<OfferResp>> =
            vec![&self.id/*, &self.borrow*/];
        Box::new(v.into_iter())
    }
}

// offer_resps() is the storage access function.
pub fn offer_resps<'a>() -> IndexedMap<'a,(&'a Addr, u16), OfferResp, OfferRespIndexes<'a>> {
    let indexes = OfferRespIndexes {
      id: UniqueIndex::new(|a_offer| a_offer.offer_id, "offer__id"),
    //   borrow: MultiIndex::new(
    //     |_pk, a_offer| a_offer.offer_id,
    //     "offers_im",
    //     "offers_borrow",
    //   )
    };
    IndexedMap::new("offers_im", indexes)
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
    pub interest: u128,
}
