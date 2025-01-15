use zwl_parser::{ywallet::YWallet, zwl::ZecWalletLite, Wallet, WalletWriter};

fn main() {
    match Wallet::parse::<ZecWalletLite>("zecwallet-light-wallet.dat") {
        Ok(w) => {
            // println!("{:#?}", w);
            let _res = w
                .write::<YWallet>("generated_from_zwl.db")
                .map_err(|_| "Error");
        }
        Err(e) => println!("{}", e.to_string()),
    }

    println!("\n=====\n");

    match Wallet::parse::<YWallet>("zec.db") {
        Ok(_w) => {
            // println!("{:#?}", w);
        }
        Err(e) => println!("{}", e.to_string()),
    }
}
