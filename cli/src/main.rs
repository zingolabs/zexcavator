use zwl_parser::{ywallet::YWallet, zwl::ZecWalletLite, Wallet, WalletWriter};

fn main() {
    let gen = ZecWalletLite::from_seed_phrase("own split raw essay easy ordinary flat glance oil labor pave loan deliver recall kingdom feed toilet moment perfect kid world village liberty honey", 3).unwrap();
    println!("{:#?}", gen);
    // match Wallet::parse::<ZecWalletLite>("zecwallet-light-wallet.dat") {
    //     Ok(w) => {
    //         // println!("{:#?}", w);
    //         let _res = w
    //             .write::<YWallet>("generated_from_zwl.db")
    //             .map_err(|_| "Error");
    //     }
    //     Err(e) => println!("{}", e.to_string()),
    // }

    // println!("\n=====\n");

    // match Wallet::parse::<YWallet>("zec.db") {
    //     Ok(_w) => {
    //         // println!("{:#?}", w);
    //     }
    //     Err(e) => println!("{}", e.to_string()),
    // }
}
