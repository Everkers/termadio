use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::io;

use crate::api::{RadioClient, Hit, Station, Page};
use crate::player::AudioPlayer;
use crate::favorites::{Favorites, FavoriteStation, FavoriteCountry};

pub struct App {
    client: RadioClient,
    player: AudioPlayer,
    search_input: String,
    search_results: Vec<Hit>,
    stations: Vec<Station>,
    current_view: View,
    list_state: ListState,
    current_station: Option<String>,
    status_message: String,
    favorites: Favorites,
}

#[derive(PartialEq)]
enum View {
    Search,
    Results,
    Stations,
    Favorites,
    FavoriteCountries,
}

impl App {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: RadioClient::new(),
            player: AudioPlayer::new()?,
            search_input: String::new(),
            search_results: Vec::new(),
            stations: Vec::new(),
            current_view: View::Search,
            list_state: ListState::default(),
            current_station: None,
            status_message: "Controls: Ctrl+s=search, Ctrl+f=favorites, Ctrl+c=countries, 'a'=favorite, SPACE=pause/play, 'x'=stop, 'q'=quit".to_string(),
            favorites: Favorites::load().unwrap_or_default(),
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.run_app(&mut terminal).await;

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    async fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('s') => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) || self.current_view != View::Search {
                            self.current_view = View::Search;
                            self.search_input.clear();
                        } else if self.current_view == View::Search {
                            self.search_input.push('s');
                        }
                    }
                    KeyCode::Char('f') => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) || self.current_view != View::Search {
                            self.current_view = View::Favorites;
                            self.list_state.select(Some(0));
                        } else if self.current_view == View::Search {
                            self.search_input.push('f');
                        }
                    }
                    KeyCode::Char('c') => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) || self.current_view != View::Search {
                            self.current_view = View::FavoriteCountries;
                            self.list_state.select(Some(0));
                        } else if self.current_view == View::Search {
                            self.search_input.push('c');
                        }
                    }
                    KeyCode::Enter => {
                        match self.current_view {
                            View::Search => {
                                if !self.search_input.is_empty() {
                                    self.search().await?;
                                }
                            }
                    View::Results => {
                        if let Some(selected) = self.list_state.selected() {
                            if let Some(hit) = self.search_results.get(selected) {
                                if hit.source.result_type == "country" {
                                    if let Some(url) = &hit.source.url {
                                        if let Some(country_id) = url.split('/').last() {
                                            let country_id = country_id.to_string();
                                            self.load_country_stations(&country_id).await?;
                                        }
                                    }
                                } else if hit.source.result_type == "channel" {
                                    if let Some(page) = &hit.source.page {
                                        if let Some(station_id) = page.url.split('/').last() {
                                            let station_id = station_id.to_string();
                                            let title = hit.source.title.clone();
                                            self.play_station(&station_id, &title)?;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    View::Stations => {
                        if let Some(selected) = self.list_state.selected() {
                            if let Some(station) = self.stations.get(selected) {
                                if let Some(station_id) = station.page.url.split('/').last() {
                                    let station_id = station_id.to_string();
                                    let title = station.title.clone();
                                    self.play_station(&station_id, &title)?;
                                }
                            }
                        }
                    }
                    View::Favorites => {
                        if let Some(selected) = self.list_state.selected() {
                            let countries_count = self.favorites.countries.len();
                            if selected < countries_count {
                                // Selected a country - load its stations
                                if let Some(country) = self.favorites.countries.get(selected) {
                                    let country_id = country.id.clone();
                                    self.load_country_stations(&country_id).await?;
                                }
                            } else {
                                // Selected a station - play it
                                let station_index = selected - countries_count;
                                if let Some(station) = self.favorites.stations.get(station_index) {
                                    let station_id = station.id.clone();
                                    let station_title = station.title.clone();
                                    self.play_station(&station_id, &station_title)?;
                                }
                            }
                        }
                    }
                    View::FavoriteCountries => {
                        if let Some(selected) = self.list_state.selected() {
                            if let Some(country) = self.favorites.countries.get(selected) {
                                let country_id = country.id.clone();
                                self.load_country_stations(&country_id).await?;
                            }
                        }
                    }
                }
            }
            KeyCode::Up => {
                if self.current_view != View::Search {
                    let len = match self.current_view {
                        View::Results => self.search_results.len(),
                        View::Stations => self.stations.len(),
                        View::Favorites => self.favorites.countries.len() + self.favorites.stations.len(),
                        View::FavoriteCountries => self.favorites.countries.len(),
                        _ => 0,
                    };
                    if len > 0 {
                        let selected = self.list_state.selected().unwrap_or(0);
                        self.list_state.select(Some(if selected == 0 { len - 1 } else { selected - 1 }));
                    }
                }
            }
            KeyCode::Down => {
                if self.current_view != View::Search {
                    let len = match self.current_view {
                        View::Results => self.search_results.len(),
                        View::Stations => self.stations.len(),
                        View::Favorites => self.favorites.countries.len() + self.favorites.stations.len(),
                        View::FavoriteCountries => self.favorites.countries.len(),
                        _ => 0,
                    };
                    if len > 0 {
                        let selected = self.list_state.selected().unwrap_or(0);
                        self.list_state.select(Some((selected + 1) % len));
                    }
                }
            }
                    KeyCode::Char(' ') => {
                        if self.current_view == View::Search {
                            self.search_input.push(' ');
                        } else {
                            if self.player.is_paused() {
                                self.player.resume();
                                self.status_message = "▶️ Resumed playback".to_string();
                            } else {
                                self.player.pause();
                                self.status_message = "⏸️ Paused playback".to_string();
                            }
                        }
                    }
                    KeyCode::Char(c) => {
                        if self.current_view == View::Search {
                            self.search_input.push(c);
                        } else {
                            match c {
                                'a' => self.add_to_favorites(),
                                'x' => {
                                    self.player.stop();
                                    self.current_station = None;
                                    self.status_message = "⏹️ Stopped playback".to_string();
                                }
                                _ => {}
                            }
                        }
                    }
                    KeyCode::Backspace => {
                        if self.current_view == View::Search {
                            self.search_input.pop();
                        }
                    }
            KeyCode::Esc => {
                match self.current_view {
                    View::Stations => self.current_view = View::Results,
                    View::Results => self.current_view = View::Search,
                    View::Favorites | View::FavoriteCountries => self.current_view = View::Search,
                    _ => {}
                }
            }
                    _ => {}
                }
            }
        }
    }

    async fn search(&mut self) -> Result<()> {
        self.status_message = format!("Searching for '{}'...", self.search_input);
        match self.client.search(&self.search_input).await {
            Ok(results) => {
                self.search_results = results.hits.hits;
                self.current_view = View::Results;
                self.list_state.select(Some(0));
                self.status_message = "Controls: Ctrl+s=search, Ctrl+f=favorites, Ctrl+c=countries, 'a'=favorite, SPACE=pause/play, 'x'=stop, 'q'=quit".to_string();
            }
            Err(e) => {
                self.status_message = format!("Search failed: {}", e);
            }
        }
        Ok(())
    }

    async fn load_country_stations(&mut self, country_id: &str) -> Result<()> {
        self.status_message = format!("Loading stations for ID: {}...", country_id);
        match self.client.get_country_stations(country_id).await {
            Ok(country_page) => {
                self.stations.clear();
                for content_item in country_page.data.content {
                    if let Some(station_items) = content_item.items {
                        for station_item in station_items {
                            let station = Station {
                                title: station_item.page.title.clone(),
                                page: Page {
                                    url: station_item.page.url.clone(),
                                },
                            };
                            self.stations.push(station);
                        }
                    }
                }
                self.current_view = View::Stations;
                self.list_state.select(Some(0));
                self.status_message = format!("Loaded {} stations", self.stations.len());
            }
            Err(e) => {
                self.status_message = format!("Failed to load stations for {}: {}", country_id, e);
            }
        }
        Ok(())
    }

    fn play_station(&mut self, station_id: &str, title: &str) -> Result<()> {
        let stream_url = self.client.get_stream_url(station_id);
        self.player.play_url(stream_url)?;
        self.current_station = Some(title.to_string());
        self.status_message = format!("♪ Playing: {} (Press 'a' to favorite)", title);
        Ok(())
    }

    fn add_to_favorites(&mut self) {
        match self.current_view {
            View::Results => {
                if let Some(selected) = self.list_state.selected() {
                    if let Some(hit) = self.search_results.get(selected) {
                        if hit.source.result_type == "country" {
                            if let Some(url) = &hit.source.url {
                                if let Some(country_id) = url.split('/').last() {
                                    let country_title = hit.source.title.clone();
                                    if self.is_country_favorite(country_id) {
                                        if self.favorites.remove_country(country_id).is_ok() {
                                            self.status_message = format!("❌ Removed {} from favorites", country_title);
                                            self.favorites = Favorites::load().unwrap_or_default();
                                        }
                                    } else {
                                        let fav_country = FavoriteCountry {
                                            id: country_id.to_string(),
                                            title: country_title.clone(),
                                        };
                                        if self.favorites.add_country(fav_country).is_ok() {
                                            self.status_message = format!("⭐ Added {} to favorites", country_title);
                                            self.favorites = Favorites::load().unwrap_or_default();
                                        }
                                    }
                                }
                            }
                        } else if hit.source.result_type == "channel" {
                            if let Some(page) = &hit.source.page {
                                if let Some(station_id) = page.url.split('/').last() {
                                    if self.is_station_favorite(station_id) {
                                        if self.favorites.remove_station(station_id).is_ok() {
                                            self.status_message = format!("❌ Removed {} from favorites", hit.source.title);
                                            self.favorites = Favorites::load().unwrap_or_default();
                                        }
                                    } else {
                                        let fav_station = FavoriteStation {
                                            id: station_id.to_string(),
                                            title: hit.source.title.clone(),
                                            country: hit.source.country.as_ref().map(|c| c.title.clone()),
                                        };
                                        if self.favorites.add_station(fav_station).is_ok() {
                                            self.status_message = format!("⭐ Added {} to favorites", hit.source.title);
                                            self.favorites = Favorites::load().unwrap_or_default();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            View::Stations => {
                if let Some(selected) = self.list_state.selected() {
                    if let Some(station) = self.stations.get(selected) {
                        if let Some(station_id) = station.page.url.split('/').last() {
                            if self.is_station_favorite(station_id) {
                                if self.favorites.remove_station(station_id).is_ok() {
                                    self.status_message = format!("❌ Removed {} from favorites", station.title);
                                    self.favorites = Favorites::load().unwrap_or_default();
                                }
                            } else {
                                let fav_station = FavoriteStation {
                                    id: station_id.to_string(),
                                    title: station.title.clone(),
                                    country: None,
                                };
                                if self.favorites.add_station(fav_station).is_ok() {
                                    self.status_message = format!("⭐ Added {} to favorites", station.title);
                                    self.favorites = Favorites::load().unwrap_or_default();
                                }
                            }
                        }
                    }
                }
            }
            View::Favorites => {
                if let Some(selected) = self.list_state.selected() {
                    let countries_count = self.favorites.countries.len();
                    if selected < countries_count {
                        // Selected a country - remove it
                        if let Some(country) = self.favorites.countries.get(selected) {
                            let id = country.id.clone();
                            let title = country.title.clone();
                            if self.favorites.remove_country(&id).is_ok() {
                                self.status_message = format!("❌ Removed {} from favorites", title);
                                self.favorites = Favorites::load().unwrap_or_default();
                            }
                        }
                    } else {
                        // Selected a station - remove it
                        let station_index = selected - countries_count;
                        if let Some(station) = self.favorites.stations.get(station_index) {
                            let id = station.id.clone();
                            let title = station.title.clone();
                            if self.favorites.remove_station(&id).is_ok() {
                                self.status_message = format!("❌ Removed {} from favorites", title);
                                self.favorites = Favorites::load().unwrap_or_default();
                            }
                        }
                    }
                }
            }
            View::FavoriteCountries => {
                if let Some(selected) = self.list_state.selected() {
                    if let Some(country) = self.favorites.countries.get(selected) {
                        let id = country.id.clone();
                        let title = country.title.clone();
                        if self.favorites.remove_country(&id).is_ok() {
                            self.status_message = format!("❌ Removed {} from favorites", title);
                            self.favorites = Favorites::load().unwrap_or_default();
                        }
                    }
                }
            }
            _ => {
                self.status_message = "Cannot toggle favorites from this view".to_string();
            }
        }
    }

    fn is_station_favorite(&self, station_id: &str) -> bool {
        self.favorites.stations.iter().any(|s| s.id == station_id)
    }

    fn is_country_favorite(&self, country_id: &str) -> bool {
        self.favorites.countries.iter().any(|c| c.id == country_id)
    }

    fn ui(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
            .split(f.size());

        // Header
        let header = Paragraph::new("🎵 Termadio - Terminal Radio")
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(header, chunks[0]);

        // Main content
        match self.current_view {
            View::Search => {
                let input = Paragraph::new(self.search_input.as_str())
                    .style(Style::default().fg(Color::Yellow))
                    .block(Block::default().borders(Borders::ALL).title("Search"));
                f.render_widget(input, chunks[1]);
            }
            View::Results => {
                let items: Vec<ListItem> = self.search_results
                    .iter()
                    .map(|hit| {
                        let (icon, _is_fav) = match hit.source.result_type.as_str() {
                            "country" => {
                                let is_favorite = hit.source.url.as_ref()
                                    .and_then(|url| url.split('/').last())
                                    .map(|id| self.is_country_favorite(id))
                                    .unwrap_or(false);
                                (if is_favorite { "⭐🌍" } else { "🌍" }, is_favorite)
                            },
                            "channel" => {
                                let is_favorite = hit.source.page.as_ref()
                                    .and_then(|p| p.url.split('/').last())
                                    .map(|id| self.is_station_favorite(id))
                                    .unwrap_or(false);
                                (if is_favorite { "⭐📻" } else { "📻" }, is_favorite)
                            },
                            _ => ("❓", false),
                        };
                        ListItem::new(format!("{} {}", icon, hit.source.title))
                    })
                    .collect();

                let list = List::new(items)
                    .block(Block::default().borders(Borders::ALL).title("Search Results"))
                    .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
                f.render_stateful_widget(list, chunks[1], &mut self.list_state);
            }
            View::Stations => {
                let items: Vec<ListItem> = self.stations
                    .iter()
                    .map(|station| {
                        let is_favorite = station.page.url.split('/').last()
                            .map(|id| self.is_station_favorite(id))
                            .unwrap_or(false);
                        let icon = if is_favorite { "⭐📻" } else { "📻" };
                        ListItem::new(format!("{} {}", icon, station.title))
                    })
                    .collect();

                let list = List::new(items)
                    .block(Block::default().borders(Borders::ALL).title("Radio Stations"))
                    .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
                f.render_stateful_widget(list, chunks[1], &mut self.list_state);
            }
            View::Favorites => {
                let mut items: Vec<ListItem> = Vec::new();
                
                // Add favorite countries first
                for country in &self.favorites.countries {
                    items.push(ListItem::new(format!("⭐🌍 {}", country.title)));
                }
                
                // Add favorite stations
                for station in &self.favorites.stations {
                    let country = station.country.as_deref().unwrap_or("Unknown");
                    items.push(ListItem::new(format!("⭐📻 {} ({})", station.title, country)));
                }

                let list = List::new(items)
                    .block(Block::default().borders(Borders::ALL).title("All Favorites"))
                    .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
                f.render_stateful_widget(list, chunks[1], &mut self.list_state);
            }
            View::FavoriteCountries => {
                let items: Vec<ListItem> = self.favorites.countries
                    .iter()
                    .map(|country| ListItem::new(format!("⭐🌍 {}", country.title)))
                    .collect();

                let list = List::new(items)
                    .block(Block::default().borders(Borders::ALL).title("Favorite Countries"))
                    .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
                f.render_stateful_widget(list, chunks[1], &mut self.list_state);
            }
        }

        // Status bar
        let status = Paragraph::new(self.status_message.as_str())
            .style(Style::default().fg(Color::Green))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(status, chunks[2]);
    }
}