use anyhow::Result;
use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub struct UserPreferences {
    pub countryId: Option<String>,
    pub favoriteChannl: Option<String>,
}

pub struct PreferencesStorage {
    file_path: String,
}

impl PreferencesStorage {
    pub fn new(file_path: String) -> Self {
        Self { file_path }
    }

    pub fn save_preferences(&self, user_preferences: UserPreferences) -> Result<()> {
        let mut existing_prefs = if Path::new(&self.file_path).exists() {
            let content = fs::read_to_string(&self.file_path)?;
            serde_json::from_str::<Map<String, Value>>(&content).unwrap_or_default()
        } else {
            Map::new()
        };

        if let Some(country_id) = user_preferences.countryId {
            existing_prefs.insert("countryId".to_string(), Value::String(country_id));
        }
        if let Some(favorite_channel) = user_preferences.favoriteChannl {
            existing_prefs.insert(
                "favoriteChannl".to_string(),
                Value::String(favorite_channel),
            );
        }

        print!("{}", &self.file_path);
        let json_content = serde_json::to_string_pretty(&existing_prefs)?;
        fs::write(&self.file_path, json_content)?;
        Ok(())
    }
}
