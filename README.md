# ZExCavator

> ⚠️ Parsing has moved to a different repository. Check https://github.com/BlockchainCommons/zmigrate and https://github.com/zingolabs/zmigrate for more information.

> ⚠️ This project is scheduled to start development in April 2025 and is not yet ready for production use.
> Check [the GitHub project](https://github.com/orgs/zingolabs/projects/9) to track progress.

**ZExCavator** is a tool that recovers (excavates!) lost ZEC.
It builds on top of ZeWIF (Zcash Extensible Wallet Interchange Format) and is currenlty focused on **ZecWallet Lite** wallets, though the architecture is extensible and can be extended to support additional wallets in the future. Under the hood, it uses **zmigrate** to parse
wallet files into an in-memory representation, **zingolib** for fund recovery and syncing, and the **ZeWIF** specification for wallet export (this is WIP).

---

## Roadmap (in no particular order)

- Automatically discovers wallet files in your system.
- Handles both encrypted and unencrypted wallets.
- Supports ZecWallet and zcashd wallets.
- Configurable & interactive syncing and recovery.
- CLI & TUI interfaces.
- Supports ZeWIF and sweeping funds.
- Extensible architecture.

---

## How to Run (WIP)

```bash
cargo run -- <filename>
```

This command parses the example wallet files included in the project and prints the output to `stdout`.

---

## Principles

The primary goal of this project is to enable seamless and lossless migration of wallet data across different wallet implementations.
By reading a wallet file and exporting it into a standardized format (ZeWIF), **ZExCavator** ensures compatibility and ease of future development.

### Core Structs (WIP: Needs to be revisited after documenting issues with zcashd & zecwallet-lite)

Head to https://github.com/BlockchainCommons/zmigrate for more info.

---

## Contributing

Contributions are welcome! Feel free to open issues, suggest features, or submit pull requests.
