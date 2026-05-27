use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// A struct that remembers when the user last opened a benchmark.
/// Used for the home screen to sort by most recently opened.
/// Will be saved to disk.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TimeLastOpened {
    benchmarks: HashMap<String, u64>, // (Benchmark name, unix time).
}

impl TimeLastOpened {
    pub fn new() -> Self {
        Self {
            benchmarks: HashMap::new(),
        }
    }

    /// Returns when the the given benchmark was last accessed.
    /// Defaults to the current time if a value is not found to be saved on disk.
    pub fn get_time_used(&self, benchmark_id: &str) -> u64 {
        match self.benchmarks.get(benchmark_id) {
            Some(time) => time.to_owned(),
            None => SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Insert the current time with a benchmark id.
    pub fn insert(&mut self, benchmark_id: String) {
        self.benchmarks.insert(
            benchmark_id,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );

        self.save();
    }

    /// Load the saved_when data from disk. Returns None if failed.
    pub fn load() -> Option<Self> {
        use std::fs::read_to_string;

        let mut save_dir = dirs::data_local_dir()?;
        save_dir.push("stig-view");
        save_dir.push("saved_when.toml");

        let saved_when_str = read_to_string(save_dir).ok()?;

        let saved_when: TimeLastOpened = toml::from_str(&saved_when_str).ok()?;

        Some(saved_when)
    }

    /// Saves the SavedWhen to disk. If errors occur, they are silent.
    /// Not ideal if this has errors, but it doesnt really matter if it does.
    fn save(&self) {
        use std::fs::{File, create_dir_all};
        use std::io::Write;

        let mut save_dir = match dirs::data_local_dir() {
            Some(dir) => dir,
            None => return,
        };

        // Create the dir if it does not exist.
        save_dir.push("stig-view");
        let _ = create_dir_all(&save_dir);

        save_dir.push("saved_when.toml");

        let saved_when_str = match toml::to_string(self) {
            Ok(string) => string,
            Err(_) => return,
        };

        let mut file = match File::create(save_dir) {
            Ok(file) => file,
            Err(_) => return,
        };

        let _ = write!(file, "{}", saved_when_str);
    }
}
