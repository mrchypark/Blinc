//! Android asset loading via NDK AssetManager
//!
//! On Android, app assets are stored in the APK file and accessed
//! through the AssetManager API. This module provides an AssetLoader
//! implementation that wraps the NDK AssetManager.

use blinc_platform::assets::{AssetLoader, AssetPath};
use blinc_platform::{PlatformError, Result};

#[cfg(target_os = "android")]
use android_activity::AndroidApp;

#[cfg(target_os = "android")]
use std::ffi::CString;

/// Android asset loader using NDK AssetManager
///
/// Loads assets from the APK's assets/ folder.
pub struct AndroidAssetLoader {
    #[cfg(target_os = "android")]
    app: AndroidApp,
}

#[cfg(target_os = "android")]
impl AndroidAssetLoader {
    /// Create a new Android asset loader with the given AndroidApp
    pub fn new(app: AndroidApp) -> Self {
        Self { app }
    }

    /// Load an asset by path (relative to assets/ folder in APK)
    fn load_from_assets(&self, path: &str) -> Result<Vec<u8>> {
        use ndk::asset::{Asset, AssetManager};
        use std::io::Read;

        // Get the asset manager from the AndroidApp
        let asset_manager = self.app.asset_manager();

        // Convert path to CString for NDK
        let c_path = CString::new(path)
            .map_err(|e| PlatformError::AssetLoad(format!("Invalid path: {}", e)))?;

        // Open the asset
        let mut asset = asset_manager
            .open(&c_path)
            .ok_or_else(|| PlatformError::AssetLoad(format!("Asset not found: {}", path)))?;

        // Read all bytes
        let mut buffer = Vec::new();
        asset.read_to_end(&mut buffer).map_err(|e| {
            PlatformError::AssetLoad(format!("Failed to read asset '{}': {}", path, e))
        })?;

        Ok(buffer)
    }
}

#[cfg(target_os = "android")]
impl AssetLoader for AndroidAssetLoader {
    fn load(&self, path: &AssetPath) -> Result<Vec<u8>> {
        match path {
            AssetPath::Relative(rel) => self.load_from_assets(rel),
            AssetPath::Absolute(abs) => {
                // On Android, absolute paths still refer to assets
                // Strip any leading slash
                let asset_path = abs.trim_start_matches('/');
                self.load_from_assets(asset_path)
            }
            AssetPath::Embedded(name) => {
                // Embedded assets not supported on Android
                // Try as asset path
                self.load_from_assets(name)
            }
        }
    }

    fn exists(&self, path: &AssetPath) -> bool {
        use ndk::asset::AssetManager;

        let asset_path = match path {
            AssetPath::Relative(rel) => rel.as_str(),
            AssetPath::Absolute(abs) => abs.trim_start_matches('/'),
            AssetPath::Embedded(name) => *name,
        };

        // Try to get the CString
        let Ok(c_path) = CString::new(asset_path) else {
            return false;
        };

        // Try to open the asset
        let asset_manager = self.app.asset_manager();
        asset_manager.open(&c_path).is_some()
    }

    fn platform_name(&self) -> &'static str {
        "android"
    }
}

// Stub implementation for non-Android builds (for cross-compilation checks)
#[cfg(not(target_os = "android"))]
impl AndroidAssetLoader {
    /// Create a placeholder loader (fails on non-Android)
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(not(target_os = "android"))]
impl AssetLoader for AndroidAssetLoader {
    fn load(&self, _path: &AssetPath) -> Result<Vec<u8>> {
        Err(PlatformError::Unsupported(
            "Android asset loading only available on Android".to_string(),
        ))
    }

    fn exists(&self, _path: &AssetPath) -> bool {
        false
    }

    fn platform_name(&self) -> &'static str {
        "android-stub"
    }
}
