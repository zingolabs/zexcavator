use zwl_parser::Wallet;

fn main() {
    // let wallet = Wallet::parse("zecwallet-light-wallet.dat").map_err(|_|"Cannot open file").unwrap();
    match Wallet::parse("zecwallet-light-wallet.dat") {
        Ok(w) => println!("{:?}", w),
        Err(e) => println!("{}", e.to_string())
    }
    // println!("{:?}", wallet);
}