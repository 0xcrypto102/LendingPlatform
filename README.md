# NFT Lending Platform on SEI

## Creating a new repo from template

Assuming you have a recent version of Rust and Cargo installed
(via [rustup](https://rustup.rs/)),
then the following should get you a new repo to start a contract:

Install [cargo-generate](https://github.com/ashleygwilliams/cargo-generate) and cargo-run-script.
Unless you did that before, run this line now:

```sh
cargo install cargo-generate --features vendored-openssl
cargo install cargo-run-script
```

Now, use it to create your new contract.
Go to the folder in which you want to place it and run:

## Summary

Everything about FoxyLend! 

• Supported wallets : 
Compass Wallet
Leap Wallet


• How does it work: 

- FoxyLend is a place where users can lend or borrow money by using theit NFT as collateral. Anyone with an NFT can borrow money by collateralising his NFT and lenders can make profit through APY by lending money. 

### LEND
Suppose user A has 1000 sei that he want to lend, so he goes to FoxyLend, selects a collection that he wants to lend the sei on, and creates an offer by selecting the amount he wants to lend ( the amount cannot be bigger than the floor of that collection, and lender can choose how much he wants to lend and how many NFTs he wants to lend for) . And the lend offer is created, the amount is deducted from his wallet and goes to the eacrow wallet. Then we’ll show every other lend offers on that collection too in an order (high to low). 

### BORROW
Now suppose user B has an NFT, he goes to FoxyLend and see what offers that collection has. If there’s multiple offers then he choses one and accepts that. Now the NFT he owns goes to escrow wallet, and the loan amount goes to him from the escrow wallet. 

### REPAY
When the borrower feels he now can repay the loan he goes to FoxyLend, selects the order he wants to repay. And pays the Capital+Interest (preset). Now the repay amount goes to escrow wallet and the NFT goes back to the borrower. And then we deduct the fee from the loan amount (20% of the interest earned) and return the rest to the lender. 

### FAILED TO REPAY
Within the selected loan timeline, if the borrower fails to repay the loan, we transfer the NFT from escrow wallet to the lender. 


### KEY POINTS 

• We set an APY and max time of a loan for every collection. 
APY will have 3 tiers. 
160% for Bluechip NFTs 
180% for medium ones 
200% for low cap/risky ones 

• Lender can only choose the amount he wants to lend per NFT. APY and MAX TIME is preset foreach collection. And while creating the offer we show an approximate interest that’ll be generated from that offer. 

• Borrower can only accept offers. While accepting an offer we show them the approximate interest they have to pay.

• We show all the collections with the available lend offers and live orders. And anyone can select a collection and create lend offers for them. 

• We show all the collections with the available borrow offers where anyone with an nft from any available collection can see offers and accept an offer. 

• We show a profile option where one can find their active lend/borrow offers and ongoing orders.

• Interest will be calculated based on APY, Time and capital. 
    


### WORKING EXAMPLE LEND 

User A has 1000 sei to lend. He goes to FoxyLend and selects WEBUMP that has a floor of 1200 SEI, APY is 160% and max lend time is 7 Days. He goes to create an offer of 1000 sei and sees the approximate interest he will earn. And he confirms the order. Now 1000 Sei is transferred to escrow wallet and the offer is now shown on the lend/borrow pages. 


### WORKING EXAMPLE BORROW 

User B has a WEBUMP, he needs some liquidity so he goes to FoxyLend and checks for available offers for WEBUMP. He finds the best offer for him, 1000 sei. He selects the offer and sees how much he’ll have to pay interest. And he accepts the offer. Now the WEBUMP is transferred to the escrow wallet, and the 1000 sei from the escrow wallet goes to the borrower. 
And the order is shown to both lender and the borrower on their active order tab.


### WORKING EXAMPLE REPAY 

Now when User B wants to repay the loan, he comes to FoxyLend, selects the order and repays the 1000 sei + interest of 100 sei (assume the interest is 100 sei). So the 1100 sei goes to the escrow wallet, and the WEBUMP goes back to the borrower. And from the 1100 sei, we deduct 20% of the interest earned that is 20 sei and gives the lender 1080 sei back. 


WORKING EXAMPLE FAILED TO REPAY 

Suppose User B lost the 1000 Sei on gambling and can’t repay back. So after the loan time ends, we send the WEBUMP from the escrow wallet to the lender. So the lender gets the WEBUMP for 1000 sei that had floor of 1200 sei. And the borrower got 1000 sei already. A good deal.
