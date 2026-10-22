use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    pub macro_core_path: PathBuf,
    pub worker_js_path: PathBuf,
    pub db_helper_path: PathBuf,
    pub payload_path: PathBuf,
    pub working_directory: PathBuf,
    pub check_commands: Vec<CheckCommand>,

    /// Dev switch. When true, `resolved()` returns paths under `mock_root`
    /// instead of the real ones. Defaults to false. Old settings files
    /// without these fields load fine because of `#[serde(default)]`.
    #[serde(default)]
    pub use_mock_paths: bool,
    #[serde(default)]
    pub mock_root: PathBuf,

    /// Where the guard writes its backup and marker, and where the apply
    /// backup lives. Defaults to ~/.cache/add_new_method_to_wrapper.
    /// Tests override this with a per-test tempdir so they don't collide.
    #[serde(default = "default_cache_root")]
    pub cache_root: PathBuf,

    /// Shell line the "build everything and check" button runs. It runs
    /// in `working_directory` through `sh -c`.
    #[serde(default = "default_build_everything")]
    pub build_everything_command: String,

    /// Shell scripts that run, one after the other, after the "build
    /// everything and check" command exits successfully. Each runs in
    /// `working_directory` through `sh -c`, streamed into the same log.
    #[serde(default)]
    pub post_build_scripts: Vec<PathBuf>,
}

fn default_build_everything() -> String {
    "./build_web_and_android.sh && ./check_everything.sh && cargo check -p web_internal_db"
        .to_string()
}

fn default_cache_root() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".cache/add_new_method_to_wrapper")
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CheckCommand {
    pub label: String,
    pub command_line: String,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct ResolvedPaths {
    pub macro_core_path: PathBuf,
    pub worker_js_path: PathBuf,
    pub db_helper_path: PathBuf,
    pub payload_path: PathBuf,
    pub working_directory: PathBuf,
}

impl Settings {
    pub fn default_for_user() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        let zdb = PathBuf::from(home).join("ProgStuff/z_db");
        Self {
            macro_core_path: zdb.join("db_wrapper/src/macro_core.rs"),
            worker_js_path: zdb.join("db_wrapper/src/web_output/worker.js"),
            db_helper_path: zdb.join("web_internal_db/src/db_helper.rs"),
            payload_path: zdb.join("protocol/src/payload.rs"),
            working_directory: zdb.clone(),
            check_commands: vec![CheckCommand {
                label: "testing mascot".into(),
                command_line:
                    "cargo check -p db_wrapper --no-default-features --features testing".into(),
                enabled: true,
            }],
            use_mock_paths: false,
            mock_root: zdb.join("add_new_method_to_wrapper/mock"),
            cache_root: default_cache_root(),
            build_everything_command: default_build_everything(),
            post_build_scripts: Vec::new(),
        }
    }

    /// The paths the pipeline should actually use. Either the real ones from
    /// the settings, or the mock tree when the dev switch is on.
    pub fn resolved(&self) -> ResolvedPaths {
        if self.use_mock_paths {
            // The four file paths point at the mock tree. The working
            // directory stays whatever it is — cargo has to run inside
            // the real workspace regardless of where the files live.
            let root = &self.mock_root;
            ResolvedPaths {
                macro_core_path: root.join("db_wrapper/src/macro_core.rs"),
                worker_js_path: root.join("db_wrapper/src/web_output/worker.js"),
                db_helper_path: root.join("web_internal_db/src/db_helper.rs"),
                payload_path: root.join("protocol/src/payload.rs"),
                working_directory: self.working_directory.clone(),
            }
        } else {
            ResolvedPaths {
                macro_core_path: self.macro_core_path.clone(),
                worker_js_path: self.worker_js_path.clone(),
                db_helper_path: self.db_helper_path.clone(),
                payload_path: self.payload_path.clone(),
                working_directory: self.working_directory.clone(),
            }
        }
    }
}

pub fn config_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".config/add_new_method_to_wrapper")
}

pub fn config_path() -> PathBuf {
    config_dir().join("settings.json")
}

pub fn load() -> Settings {
    let path = config_path();
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Settings::default_for_user();
    };
    serde_json::from_str(&text).unwrap_or_else(|_| Settings::default_for_user())
}

pub fn save(s: &Settings) -> Result<(), String> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("couldn't create config dir {}: {}", dir.display(), e))?;
    let text =
        serde_json::to_string_pretty(s).map_err(|e| format!("couldn't serialise settings: {}", e))?;
    let path = config_path();
    std::fs::write(&path, text).map_err(|e| format!("couldn't write {}: {}", path.display(), e))?;
    Ok(())
}

/// Copy the four real files into the mock tree, creating any missing folders.
/// Used by the "populate mock" button and by tests that want a fresh mock tree.
pub fn populate_mock_folder(settings: &Settings) -> Result<(), String> {
    let root = &settings.mock_root;
    let pairs: [(&PathBuf, PathBuf); 4] = [
        (
            &settings.macro_core_path,
            root.join("db_wrapper/src/macro_core.rs"),
        ),
        (
            &settings.worker_js_path,
            root.join("db_wrapper/src/web_output/worker.js"),
        ),
        (
            &settings.db_helper_path,
            root.join("web_internal_db/src/db_helper.rs"),
        ),
        (
            &settings.payload_path,
            root.join("protocol/src/payload.rs"),
        ),
    ];
    for (src, dst) in pairs {
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("couldn't create {}: {}", parent.display(), e))?;
        }
        std::fs::copy(src, &dst).map_err(|e| {
            format!(
                "couldn't copy {} -> {}: {}",
                src.display(),
                dst.display(),
                e
            )
        })?;
    }
    Ok(())
}