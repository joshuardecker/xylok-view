use std::fs;

/// One-time migration from legacy "stig-view" paths to "xylok-view" paths.
///
/// This is only run when the new cache directory does not yet exist. If it
/// does exist, the app has already run as Xylok View and no migration is
/// needed.
pub fn run() {
    let Some(cache_dir) = dirs::cache_dir() else {
        return;
    };

    let mut new_cache = cache_dir.clone();
    new_cache.push("xylok-view");

    // If the new cache dir already exists, the app has already run under the
    // new name and there is nothing to migrate.
    if new_cache.exists() {
        return;
    }

    let mut old_cache = cache_dir.clone();
    old_cache.push("stig-view");

    // Migrate cache directory.
    if old_cache.exists() {
        let _ = fs::rename(&old_cache, &new_cache);
    }

    // Migrate settings file.
    if let Some(config_dir) = dirs::config_local_dir() {
        let mut old_settings = config_dir.clone();
        old_settings.push("stig-view-settings.toml");

        let mut new_settings = config_dir.clone();
        new_settings.push("xylok-view-settings.toml");

        if old_settings.exists() && !new_settings.exists() {
            let _ = fs::copy(&old_settings, &new_settings);
            let _ = fs::remove_file(&old_settings);
        }
    }

    // Migrate time-opened data.
    if let Some(data_dir) = dirs::data_local_dir() {
        let mut old_data_dir = data_dir.clone();
        old_data_dir.push("stig-view");

        let mut old_data = old_data_dir.clone();
        old_data.push("saved_when.toml");

        let mut new_data_dir = data_dir.clone();
        new_data_dir.push("xylok-view");

        let mut new_data = new_data_dir.clone();
        new_data.push("saved_when.toml");

        if old_data.exists() && !new_data.exists() {
            let _ = fs::create_dir_all(&new_data_dir);
            let _ = fs::copy(&old_data, &new_data);
            let _ = fs::remove_file(&old_data);
            let _ = fs::remove_dir(&old_data_dir);
        }
    }
}
