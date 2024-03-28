use cw_storage_plus::{ Item, Map };
use crate::msg::{ NFTCollectionResp, OfferResp, ContractConfig };
use cosmwasm_std::Addr;

// pub const NFT_COLLECTIONS: Item<Vec<NFTCollectionResp>> = Item::new("nft_collections");
pub const NFT_COLLECTIONS: Map<u16, NFTCollectionResp> = Map::new("nft_collections");
pub const OFFERS: Map<u16, OfferResp> = Map::new("offers");
pub const OFFERS_OWNER: Map<Addr, Vec<u16>> = Map::new("owner_offers");
pub const OFFERS_ACCEPT_BORROW: Map<Addr, Vec<u16>> = Map::new("borrow_accept_offers");
pub const LAST_OFFER_INDEX: Item<u16> = Item::new("0");
pub const LEND_DENOM: Item<String> = Item::new("SEI");
pub const CONFIG: Item<ContractConfig> = Item::new("config");
