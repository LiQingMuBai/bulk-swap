// Cargo.toml (主要依赖示例)
//
// [dependencies]
// tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
// ethers = { version = "2", features = ["abigen", "contract", "providers", "signers"] }
// dotenv = "0.15"
// anyhow = "1.0"
// bip39 = "1.0"
// futures = "0.3"
// hex = "0.4"

use std::{env, sync::Arc, time::{SystemTime, UNIX_EPOCH}, str::FromStr};
use anyhow::Result;
use dotenv::dotenv;
use ethers::{
    prelude::*,
    signers::{MnemonicBuilder, LocalWallet},
    types::U256,
};
use bip39::Mnemonic;
use ethers::prelude::coins_bip39::English;
use futures::{stream, StreamExt};
// use ethers::signers::MnemonicBuilder;
// use ethers::signers::coins::English; // <- 添加这一行
abigen!(
    ERC20,
    r#"[
        function approve(address spender, uint256 amount) external returns (bool)
        function allowance(address owner, address spender) external view returns (uint256)
    ]"#,
);

abigen!(
    UniswapV2RouterLike,
    r#"[
        function swapTokensForExactTokens(uint amountOut, uint amountInMax, address[] calldata path, address to, uint deadline) external returns (uint[] memory amounts)
    ]"#,
);

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let mnemonic = env::var("MNEMONIC").expect("MNEMONIC must be set");
    let rpc_url = env::var("RPC_URL").expect("RPC_URL must be set");
    let chain_id: u64 = env::var("CHAIN_ID").ok().and_then(|s| s.parse().ok())
        .unwrap_or_else(|| 1u64);
    let token_in: Address = env::var("TOKEN_IN")?.parse()?;
    let token_out: Address = env::var("TOKEN_OUT")?.parse()?;
    let router: Address = env::var("ROUTER")?.parse()?;
    let amount_out: U256 = env::var("AMOUNT_OUT")?.parse::<u128>()?.into();
    let amount_in_max: U256 = env::var("AMOUNT_IN_MAX")?.parse::<u128>()?.into();
    let to: Address = env::var("TO")?.parse()?;
    let start_index: u32 = env::var("START_INDEX").ok().and_then(|s| s.parse().ok()).unwrap_or(0);
    let count: u32 = env::var("COUNT").ok().and_then(|s| s.parse().ok()).unwrap_or(1);
    let concurrency: usize = env::var("CONCURRENCY").ok().and_then(|s| s.parse().ok()).unwrap_or(4);
    let deadline_offset: u64 = env::var("DEADLINE_OFFSET_SECS").ok().and_then(|s| s.parse().ok()).unwrap_or(300);

    let provider = Provider::<Http>::try_from(rpc_url.as_str())?.interval(std::time::Duration::from_millis(200));
    let provider = Arc::new(provider);

    // Validate mnemonic
    // let _ = Mnemonic::from_phrase(&mnemonic, bip39::Language::English)?;

    // iterate indices
    let indices: Vec<u32> = (start_index..start_index.saturating_add(count)).collect();

    stream::iter(indices)
        .map(|i| {
            let mnemonic = mnemonic.clone();
            let provider = provider.clone();
            let token_in = token_in;
            let token_out = token_out;
            let router = router;
            let amount_out = amount_out;
            let amount_in_max = amount_in_max;
            let to = to;
            async move {
                // derive wallet for index i: path m/44'/60'/0'/0/i
                let derivation = format!("m/44'/60'/0'/0/{}", i);
                // mnemonic: String, derivation: &str, chain_id: u64
                let wallet = MnemonicBuilder::<English>::default()
                    .phrase(mnemonic.as_str())
                    .derivation_path(&derivation)?
                    .build()?
                    .with_chain_id(chain_id);

                let address = wallet.address();
                println!("Index {} -> address {:?}", i, address);
                //
                // // Create signer client for this wallet
                // let client = SignerMiddleware::new(provider.clone(), wallet);
                // let client = Arc::new(client);
                //
                // let token_contract = ERC20::new(token_in, client.clone());
                // let router_contract = UniswapV2RouterLike::new(router, client.clone());
                //
                // // check allowance
                // let allowance: U256 = token_contract.allowance(address, router).call().await?;
                // if allowance < amount_in_max {
                //     // approve
                //     let call = token_contract.approve(router, amount_in_max);
                //     let pending = call.send().await?;
                //     let receipt = pending.await?;
                //     println!("approve tx for {}: {:?}", address, receipt.map(|r| r.transaction_hash));
                // } else {
                //     println!("sufficient allowance for {}", address);
                // }

                // prepare path and deadline
                let path = vec![token_in, token_out];
                let deadline = {
                    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
                    U256::from(now + deadline_offset)
                };

                // // swap
                // let call = router_contract.swap_tokens_for_exact_tokens(amount_out, amount_in_max, path, to, deadline);
                // let pending = call.send().await?;
                // let receipt = pending.await?;
                // println!("swap tx for {}: {:?}", address, receipt.map(|r| r.transaction_hash));

                Ok::<(), anyhow::Error>(())
            }
        })
        .buffer_unordered(concurrency)
        .for_each(|res| async {
            if let Err(e) = res {
                eprintln!("task error: {:?}", e);
            }
        })
        .await;

    Ok(())
}
