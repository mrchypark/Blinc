//! HarmonyOS asset loading
//!
//! Loads assets from rawfile directory in HarmonyOS packages.

use blinc_platform::assets::{AssetLoader, AssetPath};
use blinc_platform::{PlatformError, Result};

/// HarmonyOS asset loader
///
/// Loads assets from the app's rawfile directory.
pub struct HarmonyAssetLoader {
    /// Resource manager pointer (OH_ResourceManager*)
    #[allow(dead_code)]
    resource_manager: *mut std::ffi::c_void,
}

impl HarmonyAssetLoader {
    /// Create a new HarmonyOS asset loader
    pub fn new() -> Self {
        Self {
            resource_manager: std::ptr::null_mut(),
        }
    }

    /// Create with a resource manager from N-API
    pub fn with_resource_manager(resource_manager: *mut std::ffi::c_void) -> Self {
        Self { resource_manager }
    }

    fn get_path_string(&self, path: &AssetPath) -> String {
        match path {
            AssetPath::Relative(rel) => rel.clone(),
            AssetPath::Absolute(abs) => abs.clone(),
            AssetPath::Embedded(name) => name.to_string(),
        }
    }
}

impl Default for HarmonyAssetLoader {
    fn default() -> Self {
        Self::new()
    }
}

// SAFETY: HarmonyAssetLoader is only accessed from the main thread
unsafe impl Send for HarmonyAssetLoader {}
unsafe impl Sync for HarmonyAssetLoader {}

impl AssetLoader for HarmonyAssetLoader {
    fn load(&self, path: &AssetPath) -> Result<Vec<u8>> {
        let path_str = self.get_path_string(path);

        // In HarmonyOS, assets are loaded via OH_ResourceManager_OpenRawFile
        // and OH_ResourceManager_ReadRawFile APIs

        if self.resource_manager.is_null() {
            return Err(PlatformError::AssetLoad(format!(
                "Resource manager not initialized, cannot load: {}",
                path_str
            )));
        }

        // TODO: Implement actual asset loading via OHOS resource manager
        // let raw_file = unsafe {
        //     OH_ResourceManager_OpenRawFile(self.resource_manager, path_cstr)
        // };
        // if raw_file.is_null() {
        //     return Err(PlatformError::AssetLoad(...));
        // }
        // let length = unsafe { OH_ResourceManager_GetRawFileSize(raw_file) };
        // let mut buffer = vec![0u8; length];
        // unsafe { OH_ResourceManager_ReadRawFile(raw_file, buffer.as_mut_ptr(), length) };
        // unsafe { OH_ResourceManager_CloseRawFile(raw_file) };
        // Ok(buffer)

        Err(PlatformError::AssetLoad(format!(
            "HarmonyOS asset loading not yet implemented: {}",
            path_str
        )))
    }

    fn exists(&self, path: &AssetPath) -> bool {
        if self.resource_manager.is_null() {
            return false;
        }

        // TODO: Check via OH_ResourceManager_OpenRawFile
        let _ = self.get_path_string(path);
        false
    }

    fn platform_name(&self) -> &'static str {
        "harmony"
    }
}
