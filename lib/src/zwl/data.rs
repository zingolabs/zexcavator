use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Read};
use zcash_encoding::Optional;

// Struct that tracks the latest and historical price of ZEC in the wallet
#[derive(Clone, Debug)]
pub struct WalletZecPriceInfo {
    // Latest price of ZEC and when it was fetched
    pub zec_price: Option<(u64, f64)>,

    // Wallet's currency. All the prices are in this currency
    pub currency: String,

    // When the last time historical prices were fetched
    pub last_historical_prices_fetched_at: Option<u64>,

    // Historical prices retry count
    pub historical_prices_retry_count: u64,
}

impl WalletZecPriceInfo {
    pub fn new() -> Self {
        Self {
            zec_price: None,
            currency: "USD".to_string(), // Only USD is supported right now.
            last_historical_prices_fetched_at: None,
            historical_prices_retry_count: 0,
        }
    }

    pub fn serialized_version() -> u64 {
        20
    }

    pub fn read<R: Read>(mut reader: R) -> io::Result<Self> {
        let version = reader.read_u64::<LittleEndian>()?;
        if version > Self::serialized_version() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Can't read ZecPriceInfo because of incorrect version",
            ));
        }

        // The "current" zec price is not persisted, since it is almost certainly outdated
        let zec_price = None;

        // Currency is only USD for now
        let currency = "USD".to_string();

        let last_historical_prices_fetched_at =
            Optional::read(&mut reader, |r| r.read_u64::<LittleEndian>())?;
        let historical_prices_retry_count = reader.read_u64::<LittleEndian>()?;

        Ok(Self {
            zec_price,
            currency,
            last_historical_prices_fetched_at,
            historical_prices_retry_count,
        })
    }

    pub fn write<W: WriteBytesExt>(&self, mut writer: W) -> io::Result<()> {
        writer.write_u64::<LittleEndian>(Self::serialized_version())?;

        // We don't write the currency zec price or the currency yet.
        Optional::write(
            &mut writer,
            self.last_historical_prices_fetched_at,
            |w, t| w.write_u64::<LittleEndian>(t),
        )?;
        writer.write_u64::<LittleEndian>(self.historical_prices_retry_count)?;

        Ok(())
    }
}
