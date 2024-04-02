#[cfg(test)]
mod tests {
    use crate::helpers::CwTemplateContract;
    use crate::msg::InstantiateMsg;
    use crate::msg::NFTCollectionResp;
    use crate::error::*;
    use cosmwasm_std::{Addr, Coin, Empty, Uint128, coins, Timestamp, BlockInfo};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
    
    pub fn contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    const USER: &str = "user";
    const ANOTHER_USER: &str = "another_user";
    const ADMIN: &str = "admin";
    const DENOM: &str = "SEI";
    const INTEREST: u128 = 80;

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(USER),
                    vec![Coin {
                        denom: DENOM.to_string(),
                        amount: Uint128::new(10000000),
                    },
                    ],
                )
                .unwrap();

            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(ANOTHER_USER),
                    vec![Coin {
                        denom: DENOM.to_string(),
                        amount: Uint128::new(10000),
                    },
                    ],
                )
                .unwrap();


              // Initialize balances for borrow
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked("borrow"),  // Assuming "borrow" is the borrowing contract address
                    vec![
                        Coin {
                            denom: "SEI".to_string(),  // Assuming SEI is the token required for borrowing
                            amount: Uint128::new(10000),  // Initial balance of SEI token for borrow contract
                        },
                    ],
                )
                .unwrap();
        })
    }

    fn proper_instantiate() -> (App, CwTemplateContract) {
        let mut app = mock_app();
        let cw_template_id = app.store_code(contract_template());

        let nft_collections = vec![
            NFTCollectionResp {
                collection_id: 1,
                collection: "Collection1".to_string(),
                floor_price: 100,
                contract: Addr::unchecked("contract1"),
                apy: 5,
                max_time: 3600 * 24 * 365,
            },
            NFTCollectionResp {
                collection_id: 2,
                collection: "Collection2".to_string(),
                floor_price: 150,
                contract: Addr::unchecked("contract2"),
                apy: 7,
                max_time: 130,
            },
        ];

        // Instantiate the contract with sample NFT collections data
        let msg = InstantiateMsg {
            nft_collections: nft_collections.clone(),
            admin: Addr::unchecked(ADMIN).clone(),
            interest: INTEREST.clone(),
        };

        let cw_template_contract_addr = app
            .instantiate_contract(
                cw_template_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "Contract",
                None,
            )
            .unwrap();
        let cw_template_contract = CwTemplateContract(cw_template_contract_addr);
        (app, cw_template_contract)
    }

    mod execute {
        use super::*;
        use crate::msg::{ExecuteMsg, QueryMsg, OfferResp, OfferListResp, ContractConfig, NFTCollectionResp };

        #[test]
    //     fn lend() {
    //         let (mut app, cw_template_contract) = proper_instantiate();
    //         // Set amount and collection id to make offer
    //         let amount: u128 = 50;

            let collection_id: u16 = 1;

            let msg = ExecuteMsg::Lend {amount: amount, collection_id: collection_id } ;
            let funds_sent = Coin::new(50u128, "SEI".to_string());
            let cosmos_msg = cw_template_contract.call(msg, funds_sent).unwrap();
            let res = app.execute(Addr::unchecked(USER), cosmos_msg).unwrap(); 
            // get the offer by offer_id
            let resp: OfferResp = app
                .wrap()
                .query_wasm_smart(cw_template_contract.addr(), &QueryMsg::OfferByID {offer_id: 1})
                .unwrap();

            assert_eq!(
                resp,
                OfferResp {
                    offer_id: 1,
                    owner: Addr::unchecked("user"),
                    amount: 50,
                    start_time: resp.start_time,
                    collection_id: 1,
                    token_id: "".to_string(),
                    accepted: false,
                    borrower: Addr::unchecked("none")
                }
            );
        }

        #[test]
        fn cancel_offer() {
            let (mut app, cw_template_contract) = proper_instantiate();
            // Set amount and collection id to make offer
            let amount: u128 = 50;
            let collection_id: u16 = 1;
            let offer_id: u16 = 1;

            let msg = ExecuteMsg::Lend {amount: amount, collection_id: collection_id } ;
            let funds_sent = Coin::new(50u128, "SEI".to_string());
            let cosmos_msg = cw_template_contract.call(msg, funds_sent).unwrap();
            app.execute(Addr::unchecked(USER), cosmos_msg).unwrap(); 
            // create offer
            let resp: OfferResp = app
                .wrap()
                .query_wasm_smart(cw_template_contract.addr(), &QueryMsg::OfferByID {offer_id: 1})
                .unwrap();
            // cancel offer
            let balance = app.wrap().query_balance("user","SEI").unwrap();
            println!("SEI Token amount before cancel offer:-> {:?}", balance);

            let res = app.execute_contract(
                Addr::unchecked("user"),
                cw_template_contract.addr().clone(),
                &ExecuteMsg::CancelOffer { offer_id: offer_id },
                &[]
            ).unwrap();

            // get the balace of user after lend sei token
            let balance = app.wrap().query_balance("user","SEI").unwrap();
            println!("SEI Token amount after cancel offer:-> {:?}", balance);
        }
        
        #[test]
        fn test_update_floor_price() {
            let (mut app, cw_template_contract) = proper_instantiate();

            app.execute_contract(
                Addr::unchecked("admin"),
                cw_template_contract.addr().clone(),
                &ExecuteMsg::UpdateFloorPrice {collection_id: 1, new_floor_price: 120 },
                &[],
            ).unwrap();

            let resp: NFTCollectionResp = app
                .wrap()
                .query_wasm_smart(cw_template_contract.addr(), &QueryMsg::CollectionByID {collection_id:1})
                .unwrap();

            assert_eq!(
                resp,
                NFTCollectionResp {
                    collection_id: 1,
                    collection: "Collection1".to_string(),
                    floor_price: 120,
                    contract: Addr::unchecked("contract1"),
                    apy: 5,
                    max_time: 31536000
                }
            );
        }

        #[test]
        fn test_add_new_admin() {
            let (mut app, cw_template_contract) = proper_instantiate();
            let new_admin = Addr::unchecked("UpdateAdmin");

            app.execute_contract(
                Addr::unchecked("admin"),
                cw_template_contract.addr().clone(),
                &ExecuteMsg::UpdateAdmin {new_admin: new_admin.clone() },
                &[],
            ).unwrap();

            let resp: ContractConfig = app
                .wrap()
                .query_wasm_smart(cw_template_contract.addr(), &QueryMsg::QueryAdmin {})
                .unwrap();

            assert_eq!(
                resp,
                ContractConfig {
                    admin: Addr::unchecked("UpdateAdmin"),
                    interest: 80,
                }
            );
        }
        
        #[test]
        fn test_update_interest() {
            let (mut app, cw_template_contract) = proper_instantiate();

            app.execute_contract(
                Addr::unchecked("admin"),
                cw_template_contract.addr().clone(),
                &ExecuteMsg::UpdateInterest {interest: 85 },
                &[],
            ).unwrap();

            let resp: ContractConfig = app
                .wrap()
                .query_wasm_smart(cw_template_contract.addr(), &QueryMsg::QueryAdmin {})
                .unwrap();

            assert_eq!(
                resp,
                ContractConfig {
                    admin: Addr::unchecked("admin"),
                    interest: 85,
                }
            );
        }
        
        #[test]
        fn borrow() {
            let (mut app, cw_template_contract) = proper_instantiate();
            // Set amount and collection id to make offer
            let amount: u128 = 50;
            let collection_id: u16 = 1;

            // create the offer
            let msg = ExecuteMsg::Lend {amount: amount, collection_id: collection_id } ;
            let funds_sent = Coin::new(50u128, "SEI".to_string());
            let cosmos_msg = cw_template_contract.call(msg, funds_sent).unwrap();
            app.execute(Addr::unchecked(USER), cosmos_msg).unwrap(); 
            
            // borrow nft
            let token_id = "token123".to_string();

            app.execute_contract(
                Addr::unchecked("borrow"),
                cw_template_contract.addr().clone(),
                &ExecuteMsg::Borrow {offer_id: 1, token_id: token_id.clone() },
                &[],
            ).unwrap();

            let resp: OfferResp = app
                .wrap()
                .query_wasm_smart(cw_template_contract.addr().clone(), &QueryMsg::OfferByID {offer_id: 1})
                .unwrap();

            println!("{:?}", resp);
        }

        #[test]
        fn repay() {
            let (mut app, cw_template_contract) = proper_instantiate();
            // Set amount and collection id to make offer
            let amount: u128 = 50;
            let collection_id: u16 = 1;

            // create the offer
            let msg = ExecuteMsg::Lend {amount: amount, collection_id: collection_id } ;
            let funds_sent = Coin::new(50u128, "SEI".to_string());
            let cosmos_msg = cw_template_contract.call(msg, funds_sent).unwrap();
            app.execute(Addr::unchecked(USER), cosmos_msg).unwrap(); 
            
            // borrow nft
            let token_id = "token123".to_string();
            
            app.execute_contract(
                Addr::unchecked("borrow"),
                cw_template_contract.addr().clone(),
                &ExecuteMsg::Borrow {offer_id: 1, token_id: token_id.clone() },
                &[],
            ).unwrap();

            let resp: OfferResp = app
                .wrap()
                .query_wasm_smart(cw_template_contract.addr().clone(), &QueryMsg::OfferByID {offer_id: 1})
                .unwrap();

            println!("{:?}", resp);
            let balance = app.wrap().query_balance("user","SEI").unwrap();
            println!("balance of offer after make an offer {:?}", balance);
            //  update the block_timestamp
            let block = app.block_info();
            println!("{:?}", block);
            app.set_block(BlockInfo {
                height: 12345u64,
                time: Timestamp::from_seconds(block.time.seconds() + 3600 * 24 * 180),
                chain_id: block.chain_id,
            });
            // repay function
            let msg = ExecuteMsg::Repay {offer_id: 1 } ;
            let funds_sent = Coin::new(172u128, "SEI".to_string());
            let cosmos_msg = cw_template_contract.call(msg, funds_sent).unwrap();
            let res = app.execute(Addr::unchecked("borrow"), cosmos_msg).unwrap(); 

            println!("{:?}", res);
        }
    }

    mod execute_fail {
        use super::*;
        use crate::msg::{ExecuteMsg, QueryMsg, OfferResp, OfferListResp, ContractConfig, NFTCollectionResp };

        #[test]
        fn lend_fail_over_floor_price() {
            let (mut app, cw_template_contract) = proper_instantiate();
            // Set amount and collection id to make offer
            let amount: u128 = 200;
            let collection_id: u16 = 1;
            // the floor price of collection id  = 1 is 100
            let msg = ExecuteMsg::Lend {amount: amount, collection_id: collection_id } ;
            let funds_sent = Coin::new(200u128, "SEI".to_string());
            let cosmos_msg = cw_template_contract.call(msg, funds_sent).unwrap();
            let res = app.execute(Addr::unchecked(USER), cosmos_msg); 
            // println!("{:?}", res);
        }

        #[test]
        fn cancel_offer_fail_by_another() {
            let (mut app, cw_template_contract) = proper_instantiate();
            // Set amount and collection id to make offer
            let amount: u128 = 50;
            let collection_id: u16 = 1;
            let offer_id: u16 = 1;

            let msg = ExecuteMsg::Lend {amount: amount, collection_id: collection_id } ;
            let funds_sent = Coin::new(50u128, "SEI".to_string());
            let cosmos_msg = cw_template_contract.call(msg, funds_sent).unwrap();
            app.execute(Addr::unchecked(USER), cosmos_msg).unwrap(); 
            // create offer
            let resp: OfferResp = app
                .wrap()
                .query_wasm_smart(cw_template_contract.addr(), &QueryMsg::OfferByID {offer_id: 1})
                .unwrap();

            // cancel offer
            let res = app.execute_contract(
                Addr::unchecked("another_user"),
                cw_template_contract.addr().clone(),
                &ExecuteMsg::CancelOffer { offer_id: offer_id },
                &[]
            );
            // println!("{:?}", res);
        }
        
        #[test]
        fn borrow_fail_about_accpeted_offer() {
            let (mut app, cw_template_contract) = proper_instantiate();
            // Set amount and collection id to make offer
            let amount: u128 = 50;
            let collection_id: u16 = 1;

            // create the offer
            let msg = ExecuteMsg::Lend {amount: amount, collection_id: collection_id } ;
            let funds_sent = Coin::new(50u128, "SEI".to_string());
            let cosmos_msg = cw_template_contract.call(msg, funds_sent).unwrap();
            app.execute(Addr::unchecked(USER), cosmos_msg).unwrap(); 
            
            // borrow nft
            let token_id = "token123".to_string();

            app.execute_contract(
                Addr::unchecked("borrow"),
                cw_template_contract.addr().clone(),
                &ExecuteMsg::Borrow {offer_id: 1, token_id: token_id.clone() },
                &[],
            ).unwrap();
            // fail because this was already accepted
            let res = app.execute_contract(
                Addr::unchecked("another_user"),
                cw_template_contract.addr().clone(),
                &ExecuteMsg::Borrow {offer_id: 1, token_id: token_id.clone() },
                &[],
            );

            // println!("{:?}", res);
        }

        #[test]
        fn update_floor_price_fail_cause_not_admin() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let res = app.execute_contract(
                Addr::unchecked("user"),
                cw_template_contract.addr().clone(),
                &ExecuteMsg::UpdateFloorPrice {collection_id: 1, new_floor_price: 120 },
                &[],
            );

            println!("{:?}", res);
        }
        
        #[test]
        fn add_new_admin_fail_cause_not_admin() {
            let (mut app, cw_template_contract) = proper_instantiate();
            let new_admin = Addr::unchecked("UpdateAdmin");

            let res = app.execute_contract(
                Addr::unchecked("user"),
                cw_template_contract.addr().clone(),
                &ExecuteMsg::UpdateAdmin {new_admin: new_admin.clone() },
                &[],
            );
            // println!("{:?}", res)
        }
        
        #[test]
        fn update_interest_fail_cause_not_admin() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let res = app.execute_contract(
                Addr::unchecked("user"),
                cw_template_contract.addr().clone(),
                &ExecuteMsg::UpdateInterest {interest: 85 },
                &[],
            );
            // println!("{:?}", res)
        }
        

        #[test]
        fn repay_fail_cause_max_time() {
            let (mut app, cw_template_contract) = proper_instantiate();
            // Set amount and collection id to make offer
            let amount: u128 = 50;
            let collection_id: u16 = 1;

            // create the offer
            let msg = ExecuteMsg::Lend {amount: amount, collection_id: collection_id } ;
            let funds_sent = Coin::new(50u128, "SEI".to_string());
            let cosmos_msg = cw_template_contract.call(msg, funds_sent).unwrap();
            app.execute(Addr::unchecked(USER), cosmos_msg).unwrap(); 
            
            // borrow nft
            let token_id = "token123".to_string();
            
            app.execute_contract(
                Addr::unchecked("borrow"),
                cw_template_contract.addr().clone(),
                &ExecuteMsg::Borrow {offer_id: 1, token_id: token_id.clone() },
                &[],
            ).unwrap();

            //  update the block_timestamp
            let block = app.block_info();
            // println!("{:?}", block);
            app.set_block(BlockInfo {
                height: 12345u64,
                time: Timestamp::from_seconds(block.time.seconds() + 3600 * 24 * 400),
                chain_id: block.chain_id,
            });
            // repay function
            let msg = ExecuteMsg::Repay {offer_id: 1 } ;
            let funds_sent = Coin::new(172u128, "SEI".to_string());
            let cosmos_msg = cw_template_contract.call(msg, funds_sent).unwrap();
            let res = app.execute(Addr::unchecked("borrow"), cosmos_msg); 

            println!("{:?}", res);
        }
    }

    mod query {
        use super::*;
        use crate::msg::{ExecuteMsg, QueryMsg, OfferResp, OfferListResp, ContractConfig, NFTCollectionResp };

        fn query_by_id() {
            let (mut app, cw_template_contract) = proper_instantiate();
            // Set amount and collection id to make offer
            let amount: u128 = 50;

            let collection_id: u16 = 1;

            let msg = ExecuteMsg::Lend {amount: amount, collection_id: collection_id } ;
            let funds_sent = Coin::new(50u128, "SEI".to_string());
            let cosmos_msg = cw_template_contract.call(msg, funds_sent).unwrap();
            let res = app.execute(Addr::unchecked(USER), cosmos_msg).unwrap(); 
            // get the offer by offer_id
            let resp: OfferResp = app
                .wrap()
                .query_wasm_smart(cw_template_contract.addr(), &QueryMsg::OfferByID {offer_id: 1})
                .unwrap();

            assert_eq!(
                resp,
                OfferResp {
                    offer_id: 1,
                    owner: Addr::unchecked("user"),
                    amount: 50,
                    start_time: resp.start_time,
                    collection_id: 1,
                    token_id: "".to_string(),
                    accepted: false,
                    borrower: Addr::unchecked("none")
                }
            );
        }

        #[test]
        fn query_offers_by_pagination() {
            let (mut app, cw_template_contract) = proper_instantiate();
            // Set amount and collection id to make offer
            let amount: u128 = 50;
            let collection_id: u16 = 1;
            // create offer 1
            let msg = ExecuteMsg::Lend {amount: amount, collection_id: collection_id } ;
            let funds_sent = Coin::new(50u128, "SEI".to_string());
            let cosmos_msg = cw_template_contract.call(msg, funds_sent).unwrap();
            app.execute(Addr::unchecked(USER), cosmos_msg).unwrap(); 

            // Set amount and collection id to make offer
            let amount: u128 = 90;
            let collection_id: u16 = 1;
            // create offer 1
            let msg = ExecuteMsg::Lend {amount: amount, collection_id: collection_id } ;
            let funds_sent = Coin::new(90u128, "SEI".to_string());
            let cosmos_msg = cw_template_contract.call(msg, funds_sent).unwrap();
            app.execute(Addr::unchecked(USER), cosmos_msg).unwrap(); 

            // Set amount and collection id to make offer
            let amount: u128 = 100;
            let collection_id: u16 = 2;
            // create offer 1
            let msg = ExecuteMsg::Lend {amount: amount, collection_id: collection_id } ;
            let funds_sent = Coin::new(100u128, "SEI".to_string());
            let cosmos_msg = cw_template_contract.call(msg, funds_sent).unwrap();
            app.execute(Addr::unchecked(USER), cosmos_msg).unwrap(); 

            // Set amount and collection id to make offer
            let amount: u128 = 120;
            let collection_id: u16 = 2;
            // create offer 4
            let msg = ExecuteMsg::Lend {amount: amount, collection_id: collection_id } ;
            let funds_sent = Coin::new(120u128, "SEI".to_string());
            let cosmos_msg = cw_template_contract.call(msg, funds_sent).unwrap();
            app.execute(Addr::unchecked(USER), cosmos_msg).unwrap(); 
          
            let start: u16 = 1;
            let stop: u16 = 2;
            let resp: Vec<OfferResp> = app
                .wrap()
                .query_wasm_smart(cw_template_contract.addr(), &QueryMsg::OfferList {start: start, stop: stop})
                .unwrap();
            assert_eq!(
                resp,
                [
                    OfferResp {
                        offer_id: 1,
                        owner: Addr::unchecked("user"),
                        amount: 50,
                        start_time: resp[0].start_time,
                        collection_id: 1,
                        token_id: "".to_string(),
                        accepted: false,
                        borrower: Addr::unchecked("none")
                    },
                    OfferResp {
                        offer_id: 2,
                        owner: Addr::unchecked("user"),
                        amount: 90,
                        start_time: resp[1].start_time,
                        collection_id: 1,
                        token_id: "".to_string(),
                        accepted: false,
                        borrower: Addr::unchecked("none")
                    }
                ]
            )
        }
        
        #[test]
        fn query_offers_by_owner() {
            let (mut app, cw_template_contract) = proper_instantiate();
            // Set amount and collection id to make offer
            let amount: u128 = 50;
            let collection_id: u16 = 1;
            // create offer 1
            let msg = ExecuteMsg::Lend {amount: amount, collection_id: collection_id } ;
            let funds_sent = Coin::new(50u128, "SEI".to_string());
            let cosmos_msg = cw_template_contract.call(msg, funds_sent).unwrap();
            app.execute(Addr::unchecked(USER), cosmos_msg).unwrap(); 

            // Set amount and collection id to make offer
            let amount: u128 = 90;
            let collection_id: u16 = 1;
            // create offer 1
            let msg = ExecuteMsg::Lend {amount: amount, collection_id: collection_id } ;
            let funds_sent = Coin::new(90u128, "SEI".to_string());
            let cosmos_msg = cw_template_contract.call(msg, funds_sent).unwrap();
            app.execute(Addr::unchecked(USER), cosmos_msg).unwrap(); 

            // Set amount and collection id to make offer
            let amount: u128 = 100;
            let collection_id: u16 = 2;
            // create offer 1
            let msg = ExecuteMsg::Lend {amount: amount, collection_id: collection_id } ;
            let funds_sent = Coin::new(100u128, "SEI".to_string());
            let cosmos_msg = cw_template_contract.call(msg, funds_sent).unwrap();
            app.execute(Addr::unchecked(ANOTHER_USER), cosmos_msg).unwrap(); 

            // Set amount and collection id to make offer
            let amount: u128 = 120;
            let collection_id: u16 = 2;
            // create offer 4
            let msg = ExecuteMsg::Lend {amount: amount, collection_id: collection_id } ;
            let funds_sent = Coin::new(120u128, "SEI".to_string());
            let cosmos_msg = cw_template_contract.call(msg, funds_sent).unwrap();
            app.execute(Addr::unchecked(ANOTHER_USER), cosmos_msg).unwrap(); 
            
            let resp: Vec<OfferResp> = app
                .wrap()
                .query_wasm_smart(cw_template_contract.addr(), &QueryMsg::OffersByOwner {owner: Addr::unchecked(ANOTHER_USER)})
                .unwrap();
            assert_eq!(
                resp,
                [
                    OfferResp {
                        offer_id: 3,
                        owner: Addr::unchecked("another_user"),
                        amount: 100,
                        start_time: resp[0].start_time,
                        collection_id: 2,
                        token_id: "".to_string(),
                        accepted: false,
                        borrower: Addr::unchecked("none")
                    },
                    OfferResp {
                        offer_id: 4,
                        owner: Addr::unchecked("another_user"),
                        amount: 120,
                        start_time: resp[1].start_time,
                        collection_id: 2,
                        token_id: "".to_string(),
                        accepted: false,
                        borrower: Addr::unchecked("none")
                    }
                ]
            )
        }
    
        #[test]
        fn query_offers_accept() {
            let (mut app, cw_template_contract) = proper_instantiate();
            // Set amount and collection id to make offer
            let amount: u128 = 50;
            let collection_id: u16 = 1;
            // create offer 1
            let msg = ExecuteMsg::Lend {amount: amount, collection_id: collection_id } ;
            let funds_sent = Coin::new(50u128, "SEI".to_string());
            let cosmos_msg = cw_template_contract.call(msg, funds_sent).unwrap();
            app.execute(Addr::unchecked(USER), cosmos_msg).unwrap(); 

            // Set amount and collection id to make offer
            let amount: u128 = 90;
            let collection_id: u16 = 1;
            // create offer 1
            let msg = ExecuteMsg::Lend {amount: amount, collection_id: collection_id } ;
            let funds_sent = Coin::new(90u128, "SEI".to_string());
            let cosmos_msg = cw_template_contract.call(msg, funds_sent).unwrap();
            app.execute(Addr::unchecked(USER), cosmos_msg).unwrap(); 

            // Set amount and collection id to make offer
            let amount: u128 = 100;
            let collection_id: u16 = 2;
            // create offer 1
            let msg = ExecuteMsg::Lend {amount: amount, collection_id: collection_id } ;
            let funds_sent = Coin::new(100u128, "SEI".to_string());
            let cosmos_msg = cw_template_contract.call(msg, funds_sent).unwrap();
            app.execute(Addr::unchecked(ANOTHER_USER), cosmos_msg).unwrap(); 

            // Set amount and collection id to make offer
            let amount: u128 = 120;
            let collection_id: u16 = 2;
            // create offer 4
            let msg = ExecuteMsg::Lend {amount: amount, collection_id: collection_id } ;
            let funds_sent = Coin::new(120u128, "SEI".to_string());
            let cosmos_msg = cw_template_contract.call(msg, funds_sent).unwrap();
            app.execute(Addr::unchecked(ANOTHER_USER), cosmos_msg).unwrap(); 

            let token_id = "token123".to_string();
            app.execute_contract(
                Addr::unchecked("borrow"),
                cw_template_contract.addr().clone(),
                &ExecuteMsg::Borrow {offer_id: 1, token_id: token_id.clone() },
                &[],
            ).unwrap();

            let token_id = "token13".to_string();
            app.execute_contract(
                Addr::unchecked("borrow"),
                cw_template_contract.addr().clone(),
                &ExecuteMsg::Borrow {offer_id: 4, token_id: token_id.clone() },
                &[],
            ).unwrap();
            
            let resp: Vec<OfferResp> = app
                .wrap()
                .query_wasm_smart(cw_template_contract.addr(), &QueryMsg::OffersAcceptByBorrow {borrower: Addr::unchecked("borrow")})
                .unwrap();

            assert_eq!(
                resp,
                [
                    OfferResp {
                        offer_id: 1,
                        owner: Addr::unchecked("user"),
                        amount: 50,
                        start_time: resp[0].start_time,
                        collection_id: 1,
                        token_id: "token123".to_string(),
                        accepted: true,
                        borrower: Addr::unchecked("borrow")
                    },
                    OfferResp {
                        offer_id: 4,
                        owner: Addr::unchecked("another_user"),
                        amount: 120,
                        start_time: resp[1].start_time,
                        collection_id: 2,
                        token_id: "token13".to_string(),
                        accepted: true,
                        borrower: Addr::unchecked("borrow")
                    }
                ]
            )
        }
    
        #[test]
        fn get_offers_by_price() {
            let (mut app, cw_template_contract) = proper_instantiate();
            // Set amount and collection id to make offer
            // Simulate 1000 offers
            for i in 0..10000 {
                let amount: u128 = (i % 100 + 1) ; // Varying amount
                let collection_id: u16 = ((i % 2) + 1).try_into().unwrap();   // Varying collection_id
                let msg = ExecuteMsg::Lend { amount, collection_id };
                let funds_sent = Coin::new(amount, "SEI".to_string());
                let cosmos_msg = cw_template_contract.call(msg, funds_sent).unwrap();
                app.execute(Addr::unchecked(USER), cosmos_msg).unwrap(); 
            }

            let resp: Vec<OfferResp> = app
                .wrap()
                .query_wasm_smart(cw_template_contract.addr(), &QueryMsg::OffersByPrice {page:10, page_size: 10,limit: 85, sort: true})
                .unwrap();
            println!("{:?}", resp);
        }
    
    }
}
