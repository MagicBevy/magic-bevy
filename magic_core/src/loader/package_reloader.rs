use std::fs;
use std::path::{Path, PathBuf};
use crate::status::CompilerStateManager;

pub struct PackageReloader {
    packages_root: PathBuf,
}

impl PackageReloader {
    pub fn new<P: AsRef<Path>>(packages_root: P) -> Self {
        Self {
            packages_root: packages_root.as_ref().to_path_buf(),
        }
    }

    pub fn sync_profiles(&self, status_manager: &mut CompilerStateManager) -> Result<(), String> {
        let entries = fs::read_dir(&self.packages_root)
            .map_err(|e| format!("Failed to read packages root: {}", e))?;

        let mut count = 0;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let manifest_path = path.join("package.json");
                if manifest_path.exists() {
                    count += 1;
                    let package_id = path.file_name().unwrap().to_string_lossy().into_owned();
                    
                    status_manager.set_compiling(&package_id, count);

                    if path.join("editor").exists() {
                        self.create_editor_profile(&package_id, &path)?;
                    }

                    if path.join("runtime").exists() {
                        self.create_engine_profile(&package_id, &path)?;
                    }
                }
            }
        }

        status_manager.set_success();
        Ok(())
    }

    fn create_editor_profile(&self, package_id: &str, package_path: &Path) -> Result<(), String> {
        let profile_dir = Path::new("editor/editor_profiles").join(package_id);
        fs::create_dir_all(&profile_dir).map_err(|e| e.to_string())?;

        let cargo_toml_content = format!(
            "[package]\nname = \"profile_editor_{}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\n{}_editor = {{ path = \"{}\" }}",
            package_id,
            package_id,
            package_path.join("editor").canonicalize().unwrap().display().to_string().replace("\\\\?\\", "")
        );

        fs::write(profile_dir.join("Cargo.toml"), cargo_toml_content).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn create_engine_profile(&self, package_id: &str, package_path: &Path) -> Result<(), String> {
        let profile_dir = Path::new("engine/engine_profiles").join(package_id);
        fs::create_dir_all(&profile_dir).map_err(|e| e.to_string())?;

        let cargo_toml_content = format!(
            "[package]\nname = \"profile_engine_{}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\n{}_runtime = {{ path = \"{}\" }}",
            package_id,
            package_id,
            package_path.join("runtime").canonicalize().unwrap().display().to_string().replace("\\\\?\\", "")
        );

        fs::write(profile_dir.join("Cargo.toml"), cargo_toml_content).map_err(|e| e.to_string())?;
        Ok(())
    }
}