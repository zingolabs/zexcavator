# zw-parser (Zcash Wallet Parser)

`zw-parser` is a universal Zcash wallet parser designed to read wallet files and extract addresses, keys, and seeds. The goal is to provide a standardized interface for parsing Zcash wallet files, enabling seamless import and export of wallet data across different implementations.

This project is currently in its prototype stage and supports partial parsing of **ZecWallet Lite** and **YWallet** files.

---

## Features

- Parses ZecWallet Lite (version 25) and YWallet files.
- Extracts wallet accounts, seeds, keys, and addresses.
- Outputs parsed data into a standardized Rust struct for future interoperability.

---

## How to Run

```bash
cargo run
```

This command parses the example wallet files included in the project and prints the output to `stdout`. **Note:** The CLI functionality is minimal and not user-interactive at this stage.

---

## Principles

The primary goal of this project is to enable seamless and lossless migration of wallet data across different wallet implementations. By reading a wallet file and exporting it into a standardized Rust struct, `zw-parser` ensures compatibility and ease of future development.

### Core Structs

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
   - Parsing is incomplete; seed and keys are extracted, but account birthday is not yet implemented.

3. **File Documentation**: Refer to the following source files for detailed wallet file structure documentation:

   - `src/zwl.rs` for ZecWallet Lite.
   - `src/ywallet.rs` for YWallet.

---

## Future Plans

- Extend support to additional wallet formats.
- Fully implement ZecWallet Lite parsing (including account birthdays).
- Add a complete CLI interface for user interaction.
- Improve error handling and provide detailed parsing reports.

---

## Contributing

Contributions are welcome! Feel free to open issues, suggest features, or submit pull requests.

