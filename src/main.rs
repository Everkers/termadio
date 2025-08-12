use anyhow::Result;
use clap::{Parser, Subcommand};

mod api;
mod commands;
pub mod config;
mod preferences;
mod ui;
mod player;
mod favorites;

use commands::{hello, info, search, country, radio};

#[derive(Parser)]
#[command(name = "termadio")]
#[command(about = "A simple CLI application")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch interactive radio terminal (default)
    Radio,
    /// Search for radio stations or countries
    Search {
        /// Search query (station or country name)
        query: String,
    },
    /// List stations for a specific country
    Country {
        /// Country ID from search results
        id: String,
    },
    /// Say hello to someone
    Hello {
        /// Name of the person to greet
        #[arg(short, long, default_value = "World")]
        name: String,
    },
    Preferences {
        #[arg(short, long)]
        country_id: String,
    },
    /// Manage favorite countries and stations
    Favorites {
        #[command(subcommand)]
        action: FavoritesAction,
    },
    /// Show system information
    Info,
}

#[derive(Subcommand)]
enum FavoritesAction {
    /// Add a country to favorites
    AddCountry {
        /// Country ID
        id: String,
        /// Country name
        name: String,
    },
    /// Add a station to favorites
    AddStation {
        /// Station ID
        id: String,
        /// Station name
        name: String,
        /// Country (optional)
        #[arg(short, long)]
        country: Option<String>,
    },
    /// List all favorites
    List,
    /// Remove a country from favorites
    RemoveCountry {
        /// Country ID
        id: String,
    },
    /// Remove a station from favorites
    RemoveStation {
        /// Station ID
        id: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::Radio) => {
            radio::run().await?;
        }
        Some(Commands::Search { query }) => {
            search::run(query, cli.verbose).await?;
        }
        Some(Commands::Country { id }) => {
            country::run(id, cli.verbose).await?;
        }
        Some(Commands::Hello { name }) => {
            hello::run(name, cli.verbose)?;
        }
        Some(Commands::Info) => {
            info::run(cli.verbose)?;
        }
        Some(Commands::Preferences { country_id }) => {
            println!("Country ID: {}", country_id);
            let storage =
                preferences::storage::PreferencesStorage::new("preferences.json".to_string());
            let user_prefs = preferences::storage::UserPreferences {
                countryId: Some(country_id.clone()),
                favoriteChannl: None,
            };
            storage.save_preferences(user_prefs)?;
        }
        Some(Commands::Favorites { action }) => {
            match action {
                FavoritesAction::AddCountry { id, name } => {
                    commands::favorites::add_country(id, name)?;
                }
                FavoritesAction::AddStation { id, name, country } => {
                    commands::favorites::add_station(id, name, country.as_deref())?;
                }
                FavoritesAction::List => {
                    commands::favorites::list()?;
                }
                FavoritesAction::RemoveCountry { id } => {
                    commands::favorites::remove_country(id)?;
                }
                FavoritesAction::RemoveStation { id } => {
                    commands::favorites::remove_station(id)?;
                }
            }
        }
        None => {
            radio::run().await?;
        }
    }

    Ok(())
}
