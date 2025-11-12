# Bulk ERC20 Approve & Swap Tool (Rust)

This repository contains a Rust program that derives Ethereum addresses from a BIP-39 mnemonic, then performs ERC-20 approvals and executes UniswapV2-style swapTokensForExactTokens calls for multiple derived accounts in parallel.

Important: this tool controls private keys derived from your mnemonic. Only run on testnets or a local fork until you have fully audited and tested it. Do NOT use a real/mainnet mnemonic with real funds unless you understand the risks.

## Features
- Derives multiple Ethereum addresses from a mnemonic (BIP-39, derivation path m/44'/60'/0'/0/i).
- For each derived address:
    - Optionally checks and sends ERC-20 approve to a router contract.
    - Sends swapTokensForExactTokens to swap token_in → token_out.
- Parallel execution with configurable concurrency.
- Option B: two-phase mode — fire-and-forget sends then wait-for-confirmation for higher throughput.
- Uses ethers-rs for provider, signing and contract interactions.

## Requirements
- Rust (stable)
- An Ethereum JSON-RPC endpoint (Infura/Alchemy/local node)
- A mnemonic (BIP-39)
- Token and router contract addresses compatible with UniswapV2-style router ABI
- Basic familiarity with Ethereum, gas, nonces and token approvals

## Dependencies (example)
Include these in Cargo.toml (adjust versions as needed):
- tokio
- ethers (enable signers, providers, contract, abigen features)
- dotenv
- anyhow
- futures

## Example .env
- Create a `.env` in the project root with values similar to:

- RPC_URL=https://rpc.testnet.example
- MNEMONIC="test test test test test test test test test test test junk"
- CHAIN_ID=5
- TOKEN_IN=0xTokenInContractAddressHere
- TOKEN_OUT=0xTokenOutContractAddressHere
- ROUTER=0xRouterContractAddressHere
- AMOUNT_OUT=1000000000000000000    # desired output amount in token minimal units
- AMOUNT_IN_MAX=2000000000000000000 # maximum input amount in token minimal units
- TO=0xYourReceivingAddressHere
- START_INDEX=0
- COUNT=1000
- CONCURRENCY=20
- DEADLINE_OFFSET_SECS=300

## Usage
1. Fill `.env` with correct values for your testnet environment.
2. Build and run:
   cargo run --release

3. Program flow:
    - Derive `COUNT` addresses starting at `START_INDEX`.
    - For each address, spawn a task that:
        - Optionally approve router (if allowance insufficient).
        - Send swapTokensForExactTokens.
    - Optionally send swaps in two phases: send all, then wait for confirmations.

## Modes & Options
- Sequential vs parallel: concurrency is configurable via `CONCURRENCY`.
- Approve behavior: per-address check & approve before swap; you can also pre-run a batch approve step.
- Fire-and-forget sending: collect PendingTransaction objects and await confirmations in a second phase for higher throughput.

## Safety & Caveats
- Each derived account is an independent wallet — nonce management is handled by the signer middleware per wallet.
- RPC provider rate limits and network congestion can cause failures — tune concurrency and add retry/backoff.
- Some ERC-20 tokens require allowance to be set to 0 before re-approving — handle token-specific rules if needed.
- Slippage, pool liquidity, or routing errors may cause swaps to revert — handle errors and optionally log failed indices for retry.
- Test on a local fork or testnet extensively before any mainnet usage.

## Extending / Customizing
- Support other router ABIs (e.g., UniswapV3) by changing the abigen! contract interface and call parameters.
- Replace wallet derivation method if you require alternative derivation paths or hardware wallet signing.
- Add CSV or JSON output of tx hashes and statuses.
- Add gas price strategy, dynamic nonce handling, and exponential backoff on failures.

## Example Integrations
- Use with Ganache or Anvil (foundry) for large-scale dry runs.
- Integrate with monitoring to retry failed swaps or alert on large error rates.

## Contact / Support
If you need help adapting the code to your ethers version or want a ready-to-run main.rs that matches your Cargo.toml, provide:
- Your Cargo.toml ethers version (e.g., ethers = "1.0" or ethers = { version = "2.0", features = [...] })
- Whether you want approve and swap in the same task or two-phase (approve all, then swap)
- Target network (testnet/mainnet/local fork)

License: MIT (or choose appropriate license).
