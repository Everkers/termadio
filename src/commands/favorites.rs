use anyhow::Result;
use crate::favorites::{Favorites, FavoriteCountry, FavoriteStation};

pub fn add_country(id: &str, name: &str) -> Result<()> {
    let mut favorites = Favorites::load()?;
    let country = FavoriteCountry {
        id: id.to_string(),
        title: name.to_string(),
    };
    favorites.add_country(country)?;
    println!("✅ Added '{}' to favorite countries", name);
    Ok(())
}

pub fn add_station(id: &str, name: &str, country: Option<&str>) -> Result<()> {
    let mut favorites = Favorites::load()?;
    let station = FavoriteStation {
        id: id.to_string(),
        title: name.to_string(),
        country: country.map(|c| c.to_string()),
    };
    favorites.add_station(station)?;
    println!("✅ Added '{}' to favorite stations", name);
    Ok(())
}

pub fn list() -> Result<()> {
    let favorites = Favorites::load()?;
    
    println!("⭐ Your Favorites:");
    println!("━━━━━━━━━━━━━━━━━━");
    
    if !favorites.countries.is_empty() {
        println!("\n🌍 Countries:");
        for country in &favorites.countries {
            println!("  • {} (ID: {})", country.title, country.id);
        }
    }
    
    if !favorites.stations.is_empty() {
        println!("\n📻 Stations:");
        for station in &favorites.stations {
            let country_info = station.country.as_ref()
                .map(|c| format!(" - {}", c))
                .unwrap_or_default();
            println!("  • {}{} (ID: {})", station.title, country_info, station.id);
        }
    }
    
    if favorites.countries.is_empty() && favorites.stations.is_empty() {
        println!("No favorites yet. Add some with 'termadio favorites add-country' or 'termadio favorites add-station'");
    }
    
    Ok(())
}

pub fn remove_country(id: &str) -> Result<()> {
    let mut favorites = Favorites::load()?;
    favorites.remove_country(id)?;
    println!("🗑️  Removed country from favorites");
    Ok(())
}

pub fn remove_station(id: &str) -> Result<()> {
    let mut favorites = Favorites::load()?;
    favorites.remove_station(id)?;
    println!("🗑️  Removed station from favorites");
    Ok(())
}