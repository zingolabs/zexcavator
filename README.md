# ZExCavator

> ⚠️ Use the "Zingolib" export option for fund recovery. This is temporary until full ZeWIF support is added.
> Parsing has moved to a different repository. Check https://github.com/BlockchainCommons/zmigrate for more information.

> Check [the GitHub project](https://github.com/orgs/zingolabs/projects/9) to track progress.

**ZExCavator** is a tool that recovers (excavates!) _possibly_ lost ZEC.
It builds on top of [ZeWIF (Zcash Extensible Wallet Interchange Format)](https://github.com/BlockchainCommons/zewif) and is currenlty focused on **ZecWallet Lite** wallets, though the architecture is extensible and can be extended to support additional wallets in the future. It parses wallet files into an in-memory representation, uses **zingolib** and **pepper-sync** for fund recovery and syncing, and the **ZeWIF** specification for wallet export (WIP: Currently a minimal export is supported).

---

## Roadmap (in no particular order)

- Automatically discovers wallet files in your system.
- Handles both encrypted and unencrypted wallets.
- Supports ZecWallet and zcashd wallets.
- Configurable & interactive syncing and recovery.
- TUI interfaces.
- Supports ZeWIF and sweeping funds.
- Extensible architecture.

---

## How to Run The Terminal User Interface (WIP)

This command opens the interactive TUI.

```bash
cargo run
```

---

## Principles

The primary goal of this project is to enable seamless and lossless migration of wallet data across different wallet implementations.
By reading a wallet file and exporting it into a standardized format (ZeWIF), **ZExCavator** ensures compatibility and ease of future development.

### Core Structs (WIP: Needs to be revisited after documenting issues with zcashd & zecwallet-lite)

Head to https://github.com/BlockchainCommons/zewif for more info.

---

## Contributing

Contributions are welcome! Feel free to open issues, suggest features, or submit pull requests.
