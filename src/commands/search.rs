use anyhow::Result;
use crate::api::RadioClient;

pub async fn run(query: &str, verbose: bool) -> Result<()> {
    if verbose {
        println!("Searching for: {}", query);
    }

    let client = RadioClient::new();
    let results = client.search(query).await?;
    let favorites = crate::favorites::Favorites::load().unwrap_or_default();

    println!("🔍 Search Results for '{}':", query);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    for hit in results.hits.hits {
        let source = hit.source;
        match source.result_type.as_str() {
            "country" => {
                if let Some(country) = source.country {
                    let is_favorite = favorites.countries.iter().any(|c| c.id == country.id);
                    let star = if is_favorite { "⭐" } else { "🌍" };
                    println!("{} Country: {} (ID: {})", star, country.title, country.id);
                }
            }
            "channel" => {
                println!("📻 Station: {}", source.title);
                if let Some(page) = source.page {
                    // Extract station ID from URL like "/listen/station-name/stationId"
                    if let Some(station_id) = page.url.split('/').last() {
                        println!("   Stream: {}", client.get_stream_url(station_id));
                    }
                }
                if let Some(country) = source.country {
                    println!("   Country: {}", country.title);
                }
            }
            _ => {
                println!("❓ {}: {}", source.result_type, source.title);
            }
        }
        println!();
    }

    Ok(())
}