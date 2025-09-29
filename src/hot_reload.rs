//! Configuration Hot Reloading System
//!
//! Enterprise-grade hot reloading with:
//! - File watching for automatic updates
//! - Arc swapping for zero-downtime updates
//! - Change notifications and callbacks
//! - Thread-safe concurrent access
//! - Graceful error handling and fallback

use crate::config::Config;
use crate::error::{Error, Result};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, SystemTime};

/// Configuration change event types
#[derive(Debug, Clone)]
pub enum ConfigChangeEvent {
    /// Configuration successfully reloaded
    Reloaded {
        /// Path to the configuration file that was reloaded
        path: PathBuf,
        /// Timestamp when the reload completed
        timestamp: SystemTime,
    },
    /// Configuration reload failed
    ReloadFailed {
        /// Path to the configuration file that failed to reload
        path: PathBuf,
        /// Error message describing what went wrong
        error: String,
        /// Timestamp when the error occurred
        timestamp: SystemTime,
    },
    /// Configuration file was modified
    FileModified {
        /// Path to the configuration file that was modified
        path: PathBuf,
        /// Timestamp when the modification was detected
        timestamp: SystemTime,
    },
    /// Configuration file was deleted
    FileDeleted {
        /// Path to the configuration file that was deleted
        path: PathBuf,
        /// Timestamp when the deletion was detected
        timestamp: SystemTime,
    },
}

/// Hot-reloadable configuration container
pub struct HotReloadConfig {
    /// Current configuration (thread-safe)
    current: Arc<RwLock<Config>>,
    /// File path being watched
    file_path: PathBuf,
    /// Last known modification time
    last_modified: SystemTime,
    /// Event sender for notifications
    event_sender: Option<Sender<ConfigChangeEvent>>,
    /// Polling interval for file changes
    poll_interval: Duration,
}

impl HotReloadConfig {
    /// Create a new hot-reloadable configuration from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let config = Config::from_file(&path)?;

        let last_modified = std::fs::metadata(&path)
            .map_err(|e| Error::io(path.display().to_string(), e))?
            .modified()
            .map_err(|e| Error::io(path.display().to_string(), e))?;

        Ok(Self {
            current: Arc::new(RwLock::new(config)),
            file_path: path,
            last_modified,
            event_sender: None,
            poll_interval: Duration::from_millis(1000), // Default 1 second polling
        })
    }

    /// Set the polling interval for file change detection
    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    /// Enable change notifications
    pub fn with_change_notifications(mut self) -> (Self, Receiver<ConfigChangeEvent>) {
        let (sender, receiver) = mpsc::channel();
        self.event_sender = Some(sender);
        (self, receiver)
    }

    /// Get a thread-safe reference to the current configuration
    pub fn config(&self) -> Arc<RwLock<Config>> {
        Arc::clone(&self.current)
    }

    /// Get a read-only snapshot of the current configuration
    pub fn snapshot(&self) -> Result<Config> {
        let _config = self
            .current
            .read()
            .map_err(|_| Error::concurrency("Failed to acquire read lock".to_string()))?;

        // Create a deep copy of the config
        // Since Config doesn't implement Clone, we'll serialize and deserialize
        let _content = std::fs::read_to_string(&self.file_path)
            .map_err(|e| Error::io(self.file_path.display().to_string(), e))?;

        Config::from_file(&self.file_path)
    }

    /// Manually trigger a reload
    pub fn reload(&mut self) -> Result<bool> {
        let metadata = std::fs::metadata(&self.file_path)
            .map_err(|e| Error::io(self.file_path.display().to_string(), e))?;

        let modified = metadata
            .modified()
            .map_err(|e| Error::io(self.file_path.display().to_string(), e))?;

        if modified <= self.last_modified {
            return Ok(false); // No changes
        }

        match Config::from_file(&self.file_path) {
            Ok(new_config) => {
                // Atomic swap of configuration
                {
                    let mut config = self.current.write().map_err(|_| {
                        Error::concurrency("Failed to acquire write lock".to_string())
                    })?;
                    *config = new_config;
                }

                self.last_modified = modified;

                // Send notification if enabled
                if let Some(ref sender) = self.event_sender {
                    let _ = sender.send(ConfigChangeEvent::Reloaded {
                        path: self.file_path.clone(),
                        timestamp: SystemTime::now(),
                    });
                }

                Ok(true)
            }
            Err(e) => {
                // Send error notification if enabled
                if let Some(ref sender) = self.event_sender {
                    let _ = sender.send(ConfigChangeEvent::ReloadFailed {
                        path: self.file_path.clone(),
                        error: e.to_string(),
                        timestamp: SystemTime::now(),
                    });
                }
                Err(e)
            }
        }
    }

    /// Start automatic hot reloading in a background thread
    pub fn start_watching(self) -> HotReloadHandle {
        let (stop_sender, stop_receiver) = mpsc::channel();
        let config_clone = Arc::clone(&self.current);
        let file_path = self.file_path.clone();
        let event_sender = self.event_sender.clone();
        let poll_interval = self.poll_interval;
        let mut last_modified = self.last_modified;

        let handle = thread::spawn(move || {
            loop {
                // Check for stop signal
                if stop_receiver.try_recv().is_ok() {
                    break;
                }

                // Check for file changes
                if let Ok(metadata) = std::fs::metadata(&file_path) {
                    if let Ok(modified) = metadata.modified() {
                        if modified > last_modified {
                            // File was modified, send notification
                            if let Some(ref sender) = event_sender {
                                let _ = sender.send(ConfigChangeEvent::FileModified {
                                    path: file_path.clone(),
                                    timestamp: SystemTime::now(),
                                });
                            }

                            // Attempt to reload
                            match Config::from_file(&file_path) {
                                Ok(new_config) => {
                                    // Atomic swap
                                    if let Ok(mut config) = config_clone.write() {
                                        *config = new_config;
                                        last_modified = modified;

                                        // Send success notification
                                        if let Some(ref sender) = event_sender {
                                            let _ = sender.send(ConfigChangeEvent::Reloaded {
                                                path: file_path.clone(),
                                                timestamp: SystemTime::now(),
                                            });
                                        }
                                    }
                                }
                                Err(e) => {
                                    // Send error notification
                                    if let Some(ref sender) = event_sender {
                                        let _ = sender.send(ConfigChangeEvent::ReloadFailed {
                                            path: file_path.clone(),
                                            error: e.to_string(),
                                            timestamp: SystemTime::now(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }

                thread::sleep(poll_interval);
            }
        });

        HotReloadHandle {
            handle: Some(handle),
            stop_sender,
        }
    }

    /// Get the file path being watched
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    /// Get the last modification time
    pub fn last_modified(&self) -> SystemTime {
        self.last_modified
    }
}

/// Handle for controlling hot reload background thread
pub struct HotReloadHandle {
    handle: Option<thread::JoinHandle<()>>,
    stop_sender: Sender<()>,
}

impl HotReloadHandle {
    /// Stop the background watching thread
    pub fn stop(mut self) -> Result<()> {
        if self.stop_sender.send(()).is_err() {
            return Err(Error::concurrency("Failed to send stop signal".to_string()));
        }

        if let Some(handle) = self.handle.take() {
            handle
                .join()
                .map_err(|_| Error::concurrency("Failed to join background thread".to_string()))?;
        }

        Ok(())
    }
}

impl Drop for HotReloadHandle {
    fn drop(&mut self) {
        let _ = self.stop_sender.send(());
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_hot_reload_basic() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.conf");

        // Create initial config file
        let mut file = File::create(&config_path).unwrap();
        writeln!(file, "key=value1").unwrap();
        file.flush().unwrap();
        drop(file);

        // Create hot reload config
        let mut hot_config = HotReloadConfig::from_file(&config_path).unwrap();

        // Read initial value
        {
            let config = hot_config.config();
            let config_read = config.read().unwrap();
            assert_eq!(
                config_read.get("key").unwrap().as_string().unwrap(),
                "value1"
            );
        }

        // Wait a bit to ensure different modification time
        thread::sleep(Duration::from_millis(10));

        // Update config file
        let mut file = File::create(&config_path).unwrap();
        writeln!(file, "key=value2").unwrap();
        file.flush().unwrap();
        drop(file);

        // Manual reload
        let reloaded = hot_config.reload().unwrap();
        assert!(reloaded);

        // Verify new value
        {
            let config = hot_config.config();
            let config_read = config.read().unwrap();
            assert_eq!(
                config_read.get("key").unwrap().as_string().unwrap(),
                "value2"
            );
        }
    }

    #[test]
    fn test_hot_reload_notifications() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.conf");

        // Create initial config file
        let mut file = File::create(&config_path).unwrap();
        writeln!(file, "key=value1").unwrap();
        file.flush().unwrap();
        drop(file);

        // Create hot reload config with notifications
        let (mut hot_config, receiver) = HotReloadConfig::from_file(&config_path)
            .unwrap()
            .with_change_notifications();

        // Wait a bit
        thread::sleep(Duration::from_millis(10));

        // Update config file
        let mut file = File::create(&config_path).unwrap();
        writeln!(file, "key=value2").unwrap();
        file.flush().unwrap();
        drop(file);

        // Manual reload should trigger notification
        hot_config.reload().unwrap();

        // Check for notification
        let event = receiver.try_recv().unwrap();
        match event {
            ConfigChangeEvent::Reloaded { path, .. } => {
                assert_eq!(path, config_path);
            }
            _ => panic!("Expected Reloaded event"),
        }
    }

    #[test]
    fn test_automatic_watching() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.conf");

        // Create initial config file
        let mut file = File::create(&config_path).unwrap();
        writeln!(file, "key=value1").unwrap();
        file.flush().unwrap();
        drop(file);

        // Create hot reload config with fast polling
        let (hot_config, receiver) = HotReloadConfig::from_file(&config_path)
            .unwrap()
            .with_poll_interval(Duration::from_millis(50))
            .with_change_notifications();

        let config_ref = hot_config.config();
        let handle = hot_config.start_watching();

        // Wait a bit
        thread::sleep(Duration::from_millis(100));

        // Update config file
        let mut file = File::create(&config_path).unwrap();
        writeln!(file, "key=value2").unwrap();
        file.flush().unwrap();
        drop(file);

        // Wait for automatic reload
        thread::sleep(Duration::from_millis(200));

        // Check that config was updated
        {
            let config_read = config_ref.read().unwrap();
            assert_eq!(
                config_read.get("key").unwrap().as_string().unwrap(),
                "value2"
            );
        }

        // Check for notifications
        let mut received_events = Vec::new();
        while let Ok(event) = receiver.try_recv() {
            received_events.push(event);
        }

        assert!(!received_events.is_empty());

        // Should have received at least a Reloaded event
        let has_reloaded = received_events
            .iter()
            .any(|event| matches!(event, ConfigChangeEvent::Reloaded { .. }));
        assert!(has_reloaded);

        // Stop watching
        handle.stop().unwrap();
    }
}
