use cw_storage_plus::Item;

pub const NFT_COLLECTIONS: Item<Vec<String>> = Item::new("nft collections");
pub const APY_COLLECTIONS: Item<Vec<u16>> = Item::new(0);
pub const MAX_TIME_COLLECTIONS: Item<Vec<u64>> = Item::new(0);
pub const OFFERS: Item<Vec<OfferResp>> = Item::new();
pub const OFFER_INDEX: Item<u16> = Item::new(0);
pub const LEND_DENOM: Item<String> = Item::new("sei");
