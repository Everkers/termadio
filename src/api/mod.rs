use anyhow::Result;
use serde::{Deserialize, Serialize};

const BASE_URL: &str = "https://radio.garden/api";

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SearchResult {
    pub hits: Hits,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Hits {
    pub hits: Vec<Hit>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Hit {
    #[serde(rename = "_source")]
    pub source: Source,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Source {
    #[serde(rename = "type")]
    pub result_type: String,
    pub title: String,
    pub country: Option<Country>,
    pub page: Option<Page>,
    pub url: Option<String>,
    pub code: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Country {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Page {
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CountryPage {
    pub data: CountryData,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CountryData {
    pub content: Vec<ContentItem>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ContentItem {
    pub items: Option<Vec<StationItem>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StationItem {
    pub page: StationPage,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StationPage {
    pub url: String,
    pub title: String,
    pub stream: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Station {
    pub title: String,
    pub page: Page,
}

pub struct RadioClient {
    client: reqwest::Client,
}

impl RadioClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn search(&self, query: &str) -> Result<SearchResult> {
        let url = format!("{}/search/secure?q={}", BASE_URL, query);
        let response = self.client.get(&url).send().await?;
        let result = response.json::<SearchResult>().await?;
        Ok(result)
    }

    pub async fn get_country_stations(&self, country_id: &str) -> Result<CountryPage> {
        let url = format!("{}/ara/content/secure/page/{}", BASE_URL, country_id);
        let response = self.client.get(&url).send().await?;
        let result = response.json::<CountryPage>().await?;
        Ok(result)
    }

    pub async fn get_actual_stream_url(&self, station_id: &str) -> Result<String> {
        let url = format!("{}/ara/content/listen/{}/channel.mp3", BASE_URL, station_id);
        let response = self.client.head(&url).send().await?;
        Ok(response.url().to_string())
    }

    pub fn get_stream_url(&self, station_id: &str) -> String {
        format!("{}/ara/content/listen/{}/channel.mp3", BASE_URL, station_id)
    }
}

impl Default for RadioClient {
    fn default() -> Self {
        Self::new()
    }
}