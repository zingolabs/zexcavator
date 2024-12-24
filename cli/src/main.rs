use zwl_parser::{Wallet, zwl::ZecWalletLite, ywallet::YWallet};

fn main() {
    // match Wallet::parse::<ZecWalletLite>("zecwallet-light-wallet.dat") {
    //     Ok(w) => println!("{:?}", w),
    //     Err(e) => println!("{}", e.to_string())
    // }

    match Wallet::parse::<YWallet>("../zec.db") {
        Ok(w) => println!("{:?}", w),
        Err(e) => println!("{}", e.to_string())
    }
    
}