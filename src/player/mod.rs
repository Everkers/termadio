use anyhow::Result;
use rodio::{Decoder, OutputStream, Sink};
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use futures_util::StreamExt;

pub struct AudioPlayer {
    _stream: OutputStream,
    sink: Arc<Mutex<Sink>>,
    current_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl AudioPlayer {
    pub fn new() -> Result<Self> {
        let (_stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;
        Ok(Self { 
            _stream, 
            sink: Arc::new(Mutex::new(sink)),
            current_handle: Arc::new(Mutex::new(None)),
        })
    }

    pub fn play_url(&self, url: String) -> Result<()> {
        self.stop();
        
        let sink = Arc::clone(&self.sink);
        let handle_ref = Arc::clone(&self.current_handle);
        
        let handle = tokio::spawn(async move {
            let client = reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
                .build()
                .unwrap();
                
            if let Ok(response) = client.get(&url).send().await {
                let mut stream = response.bytes_stream();
                let mut buffer = Vec::new();
                
                // Collect larger initial buffer for decoder
                while let Some(chunk_result) = stream.next().await {
                    if let Ok(chunk) = chunk_result {
                        buffer.extend_from_slice(&chunk);
                        if buffer.len() >= 131072 { // 128KB initial buffer
                            break;
                        }
                    }
                }
                
                if !buffer.is_empty() {
                    let cursor = Cursor::new(buffer.clone());
                    if let Ok(source) = Decoder::new(cursor) {
                        if let Ok(sink_guard) = sink.lock() {
                            sink_guard.append(source);
                            sink_guard.play();
                        }
                    }
                    
                    // Continue streaming more chunks
                    buffer.clear();
                    while let Some(chunk_result) = stream.next().await {
                        if let Ok(chunk) = chunk_result {
                            buffer.extend_from_slice(&chunk);
                            if buffer.len() >= 65536 { // 64KB chunks
                                let cursor = Cursor::new(buffer.clone());
                                if let Ok(source) = Decoder::new(cursor) {
                                    if let Ok(sink_guard) = sink.lock() {
                                        sink_guard.append(source);
                                    }
                                }
                                buffer.clear();
                            }
                        }
                    }
                }
            }
        });
        
        if let Ok(mut current) = handle_ref.lock() {
            *current = Some(handle);
        }
        
        Ok(())
    }

    pub fn pause(&self) {
        if let Ok(sink) = self.sink.lock() {
            sink.pause();
        }
    }

    pub fn resume(&self) {
        if let Ok(sink) = self.sink.lock() {
            sink.play();
        }
    }

    pub fn stop(&self) {
        // Abort current streaming task
        if let Ok(mut handle) = self.current_handle.lock() {
            if let Some(h) = handle.take() {
                h.abort();
            }
        }
        
        // Stop and clear sink
        if let Ok(sink) = self.sink.lock() {
            sink.stop();
            sink.clear();
        }
    }

    pub fn is_paused(&self) -> bool {
        self.sink.lock().map(|s| s.is_paused()).unwrap_or(false)
    }
}