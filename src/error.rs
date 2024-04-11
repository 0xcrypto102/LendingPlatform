use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Not Owner")]
    Unauthorized,

    #[error("Invalid NFT Owner")]
    InvalidNftOwner,

    #[error("Invalid Offer Id")]
    InvalidOfferId,

    #[error("Invalid Borrower")]
    InvalidBorrow,

    #[error("Lend amount should be less than floor price of collection")]
    TooMuchLendAmount,
    
    #[error("Invalid Offer Owner")]
    InvalidOfferOwner,

    #[error("Collection not found")]
    CollectionNotFound,

    #[error("Collections Loading Fail")]
    CollectionLoadFail,

    #[error("DepositFail")]
    DepositFail,

    #[error("Not Exact amount to lend")]
    NotExactAmount,

    #[error("Offer not found")]
    OfferNotFound,

    #[error("Offer already accepted")]
    OfferAlreadyAccepted,

    #[error("Offer Not accepted")]
    OfferNotAccepted,

}
