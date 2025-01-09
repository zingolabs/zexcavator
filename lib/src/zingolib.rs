use zcash_client_backend::proto::service::TreeState;

pub struct TxMap {}
pub struct ZingolibWalletOptions {}
pub struct ZecPriceInfo {}

#[allow(dead_code)]
pub struct Zingolib {
    version: u64,
    transaction_metadata_set: TxMap,
    chain: String,
    options: ZingolibWalletOptions,
    birthday: u64,
    verified_tree: Option<TreeState>, // use zcash_encoding::Optional
    price_info: ZecPriceInfo,         // Previous prices
}
