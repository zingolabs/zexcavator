# ZExCavator

> ⚠️ This project is in active development and is not yet ready for production use.
> Check [the GitHub project](https://github.com/orgs/zingolabs/projects/9) to track progress.

**ZExCavator** is a universal Zcash wallet parser designed to read wallet files and extract addresses, keys, and seeds
in order to facilitate migration and interoperability between different wallet implementations. It builds on top of
ZeWIF (Zcash Extensible Wallet Interchange Format) and is currenlty focused on **zcashd**, **ZecWallet Lite** and **YWallet**,
though the architecture is extensible and can be extended to support additional wallets in the future.

---

## Features

- Parses ZecWallet Lite (version 25) and YWallet files partially.
- Extracts wallet accounts, seeds, keys, and addresses.
- Outputs parsed data into a standardized Rust struct for future interoperability.

---

## How to Run (WIP)

```bash
cargo run -- <filename>
```

This command parses the example wallet files included in the project and prints the output to `stdout`.

---

## Principles

The primary goal of this project is to enable seamless and lossless migration of wallet data across different wallet implementations.
By reading a wallet file and exporting it into a standardized Rust struct, **ZExCavator** ensures compatibility and ease of future development.

### Core Structs (WIP: Needs to be revisited after documenting issues with zcashd & zecwallet-lite derivation)

#### `Wallet`

Represents the parsed data of a wallet file:

```rs
pub struct Wallet {
    pub wallet_name: String, // Name of the wallet, e.g., "ZecWallet Lite", "YWallet"
    pub version: u64,        // Wallet file version
    pub accounts: Vec<WalletAccount>, // List of wallet accounts
}
```

#### `WalletAccount`

Represents an individual account in the wallet:

```rs
pub struct WalletAccount {
    pub name: String,               // Name of the account (if supported, e.g., YWallet)
    pub seed: Option<Vec<u8>>,      // Seed entropy of the account
    pub birthday: BlockHeight,      // Birthday of the account (used for syncing)
    pub keys: WalletKeys,           // List of wallet keys
}
```

---

## Current Limitations

1. **CLI Functionality**: The CLI is not fully interactive. It simply parses example wallet files and prints the results.

2. **ZecWallet Lite Support**:

   - Only supports wallet version 25 (latest).
   - Does not support encrypted wallets.

3. Does not integrate a sync engine:

   - Currently, the parser reads the data present in the wallet file. It does not try to reconstruct/discover missing or lost funds.

4. **File Documentation**: Refer to the following source files for detailed wallet file structure documentation:

   - `src/zwl.rs` for ZecWallet Lite.
   - `src/ywallet.rs` for YWallet.

---

## Future Plans

- Add sync engine.
  - This engine will be used while taking into account 'know quirks' for each wallet.
- Add proper test vectors!
- Extend support to additional wallet formats.
- Fully implement ZecWallet Lite parsing (including encryption).
- Add a complete CLI interface for user interaction.
- Improve error handling and provide detailed parsing reports.

---

## Contributing

Contributions are welcome! Feel free to open issues, suggest features, or submit pull requests.
