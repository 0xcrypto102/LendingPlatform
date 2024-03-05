
use cw_storage_plus::{ Item, Map };
use crate::msg::NFTCollectionResp;
use crate::msg::OfferResp;

pub const NFT_COLLECTIONS: Item<Vec<NFTCollectionResp>> = Item::new("nft_collections");
pub const OFFERS: Map<u16, OfferResp> = Map::new("offers");
pub const LAST_OFFER_INDEX: Item<u16> = Item::new("0");
pub const LEND_DENOM: Item<String> = Item::new("SEI");
pub const DEFAULT_LIMIT: u16 = 10;