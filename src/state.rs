use cw_storage_plus::Item;

pub const NFT_COLLECTIONS: Item<Vec<NFTCollectionResp>> = Item::new();
pub const OFFERS: Item<Vec<OfferResp>> = Item::new();
pub const OFFER_INDEX: Item<u16> = Item::new(0);
pub const LEND_DENOM: Item<String> = Item::new("sei");
