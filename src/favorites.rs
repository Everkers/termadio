use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FavoriteStation {
    pub id: String,
    pub title: String,
    pub country: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FavoriteCountry {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Favorites {
    pub stations: Vec<FavoriteStation>,
    pub countries: Vec<FavoriteCountry>,
}

impl Favorites {
    pub fn load() -> Result<Self> {
        let path = "favorites.json";
        if Path::new(path).exists() {
            let content = fs::read_to_string(path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write("favorites.json", content)?;
        Ok(())
    }

    pub fn add_station(&mut self, station: FavoriteStation) -> Result<()> {
        if !self.stations.iter().any(|s| s.id == station.id) {
            self.stations.push(station);
            self.save()?;
        }
        Ok(())
    }

    pub fn add_country(&mut self, country: FavoriteCountry) -> Result<()> {
        if !self.countries.iter().any(|c| c.id == country.id) {
            self.countries.push(country);
            self.save()?;
        }
        Ok(())
    }

    pub fn remove_station(&mut self, id: &str) -> Result<()> {
        self.stations.retain(|s| s.id != id);
        self.save()?;
        Ok(())
    }

    pub fn remove_country(&mut self, id: &str) -> Result<()> {
        self.countries.retain(|c| c.id != id);
        self.save()?;
        Ok(())
    }
}