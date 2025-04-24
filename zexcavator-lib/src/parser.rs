use crate::{ywallet::YWallet, zwl::ZecWalletLite, WalletParser};

pub struct WalletParserFactory {
    pub parser: Box<dyn WalletParser>,
    pub filename: String,
}

impl WalletParserFactory {
    pub fn read(filename: &str) -> Result<Self, &str> {
        if filename.ends_with(".db") {
            Ok(WalletParserFactory {
                filename: filename.to_string(),
                parser: Box::new(YWallet::read(filename).unwrap()),
            })
        } else if filename.ends_with(".dat") {
            Ok(WalletParserFactory {
                filename: filename.to_string(),
                parser: Box::new(ZecWalletLite::read(filename).unwrap()),
            })
        } else {
            Err("Unknown wallet format")
        }
    }
}
