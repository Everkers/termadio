use anyhow::Result;
use crate::api::RadioClient;

pub async fn run(country_id: &str, verbose: bool) -> Result<()> {
    if verbose {
        println!("Fetching stations for country ID: {}", country_id);
    }

    let client = RadioClient::new();
    let country_page = client.get_country_stations(country_id).await?;

    println!("📻 Radio Stations:");
    println!("━━━━━━━━━━━━━━━━━━━━");

    for content_item in country_page.data.content {
        if let Some(stations) = content_item.items {
            for station in stations {
                println!("🎵 {}", station.page.title);
                
                // Extract station ID from URL like "/listen/station-name/stationId"
                if let Some(station_id) = station.page.url.split('/').last() {
                    let stream_url = client.get_stream_url(station_id);
                    println!("   Stream: {}", stream_url);
                    if verbose {
                        println!("   Station ID: {}", station_id);
                    }
                }
                println!();
            }
        }
    }

    Ok(())
}