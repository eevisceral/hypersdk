//! List all perpetual markets grouped by their DEX.
//!
//! Queries the perp dexs endpoint and prints each market's name, index, collateral,
//! growth mode, and aligned quote token along with HIP-3 DEX metadata.

use hypersdk::hypercore;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = hypercore::mainnet();

    let dex_names = hypercore::all_perp_dex_names(&client).await?;
    println!("perpDexs slots: {:?}", dex_names);

    let dexes = client.perp_dexes().await?;
    for dex in dexes {
        println!("\n\nmarkets for {dex}");
        println!("full name: {:?}", dex.full_name());
        println!("deployer: {:?}", dex.deployer());
        println!("oracle updater: {:?}", dex.oracle_updater());
        println!("fee recipient: {:?}", dex.fee_recipient());
        println!("deployer fee scale: {:?}", dex.deployer_fee_scale());
        println!(
            "funding multipliers: {} entries",
            dex.asset_to_funding_multiplier().len()
        );
        println!(
            "streaming OI caps: {} entries",
            dex.asset_to_streaming_oi_cap().len()
        );

        let markets = client.perps_from(dex).await?;
        for market in markets {
            println!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{:?}",
                market.name,
                market.index,
                market.dex.as_deref().unwrap_or("native"),
                market.collateral,
                market.growth_mode,
                market.aligned_quote_token,
                market.dex
            );
        }
    }

    Ok(())
}
