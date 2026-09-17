use eframe::egui;
use notify::{Event, RecursiveMode, Watcher};
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use magic_core::*;

enum CompilerMessage {
    Indexing(String, usize),
    CompileSuccess,
    CompileError(String),
}

struct MagicBevyApp {
    menu_registry: MenuRegistry,
    compiler_manager: CompilerStateManager,
    rx: Receiver<CompilerMessage>,
    _watcher: notify::RecommendedWatcher,
    current_lib: Option<libloading::Library>,
}

impl MagicBevyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());

        let (tx, rx) = channel();
        let is_compiling = Arc::new(Mutex::new(false));

        let watcher = Self::init_file_watcher(tx.clone(), Arc::clone(&is_compiling));

        let mut app = Self {
            menu_registry: MenuRegistry::new(),
            compiler_manager: CompilerStateManager::new(),
            rx,
            _watcher: watcher,
            current_lib: None,
        };

        app.compiler_manager.set_success();

        app.reload_and_load_library();

        EngineCompilerState::global().unlock_watcher();
        app
    }

    fn reload_and_load_library(&mut self) {
        mgprint!("HotReload", "Clearing resources and loading new library...");

        self.menu_registry.clear();
        self.current_lib = None;

        #[cfg(target_os = "linux")]
        let lib_path = "./target/debug/libmagic_editor.so";
        #[cfg(target_os = "windows")]
        let lib_path = "./target/debug/magic_editor.dll";
        #[cfg(target_os = "macos")]
        let lib_path = "./target/debug/libmagic_editor.dylib";

        if Path::new(lib_path).exists() {
            unsafe {
                if let Ok(lib) = libloading::Library::new(lib_path) {
                    //type RegisterFn = unsafe extern "Rust" fn(&mut SystemManager);

                    /*if let Ok(register_all_packages) = lib.get::<RegisterFn>(b"register_all_packages\0") {
                        register_all_packages(&mut self.system_manager);
                        mgsuccess!("HotReload", "Dynamic profile packages registered directly into launcher workspace.");
                    } else {
                        mgerror!("HotReload", "Failed to resolve 'register_all_packages' symbol from binary.");
                    }*/
                    self.current_lib = Some(lib);
                } else {
                    mgerror!(
                        "HotReload",
                        "Error occurred while loading magic_editor dynamic library."
                    );
                }
            }
        } else {
            mgwarn!("HotReload", "Library file not found at: {}", lib_path);
        }

        self.mock_packages();
    }

    fn mock_packages(&mut self) {
        self.menu_registry
            .register("File", "Save Scene", Box::new(|| println!("Scene Saved!")));
        self.menu_registry
            .register("File", "Exit", Box::new(|| std::process::exit(0)));
    }

    fn init_file_watcher(
        tx: Sender<CompilerMessage>,
        is_compiling: Arc<Mutex<bool>>,
    ) -> notify::RecommendedWatcher {
        let last_change = Arc::new(Mutex::new(Instant::now()));
        let tx_clone = tx.clone();
        let is_compiling_clone = Arc::clone(&is_compiling);

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if *EngineCompilerState::global().lock_mode.lock().unwrap()
                == CompilerLockMode::EditorLock
            {
                return;
            }

            if let Ok(event) = res {
                if event.kind.is_modify() || event.kind.is_create() {
                    let is_valid_change = event.paths.iter().any(|p| {
                        let path_str = p.to_string_lossy();
                        (path_str.ends_with(".rs") || path_str.ends_with("package.json"))
                            && !path_str.contains("/target/")
                            && !path_str.contains("/.git/")
                    });

                    if is_valid_change {
                        if *is_compiling_clone.lock().unwrap() {
                            return;
                        }

                        let mut last = last_change.lock().unwrap();
                        let now = Instant::now();

                        if now.duration_since(*last) < Duration::from_millis(500) {
                            *last = now;
                            return;
                        }
                        *last = now;

                        let tx_worker = tx_clone.clone();
                        let is_compiling_worker = Arc::clone(&is_compiling_clone);
                        let last_change_worker = Arc::clone(&last_change);

                        thread::spawn(move || {
                            thread::sleep(Duration::from_secs(1));
                            let last_time = *last_change_worker.lock().unwrap();
                            if Instant::now().duration_since(last_time)
                                >= Duration::from_millis(800)
                            {
                                Self::execute_compile(tx_worker, is_compiling_worker);
                            }
                        });
                    }
                }
            }
        })
        .unwrap();

        let packages_path = Path::new("./packages");
        if packages_path.exists() {
            let absolute_packages = packages_path.canonicalize().unwrap();
            watcher
                .watch(&absolute_packages, RecursiveMode::Recursive)
                .unwrap();
        }

        let profiles_path = Path::new("./profiles");
        if profiles_path.exists() {
            let absolute_profiles = profiles_path.canonicalize().unwrap();
            watcher
                .watch(&absolute_profiles, RecursiveMode::Recursive)
                .unwrap();
        }

        let editor_src_path = Path::new("./magic_editor/src");
        if editor_src_path.exists() {
            let absolute_editor = editor_src_path.canonicalize().unwrap();
            watcher
                .watch(&absolute_editor, RecursiveMode::Recursive)
                .unwrap();
        }

        watcher
    }

    fn execute_compile(tx: Sender<CompilerMessage>, is_compiling: Arc<Mutex<bool>>) {
        {
            let mut lock = is_compiling.lock().unwrap();
            if *lock {
                return;
            }
            *lock = true;
        }

        thread::spawn(move || {
            let check_output = Command::new("cargo")
                .args(["check", "-p", "magic_editor", "--message-format=short"])
                .output();

            if let Ok(output) = check_output {
                if output.status.success() {
                    let _ = tx.send(CompilerMessage::Indexing(
                        "Compiling dynamic module...".to_string(),
                        1,
                    ));

                    let build_status = Command::new("cargo")
                        .args(["build", "-p", "magic_editor", "--lib"])
                        .status();

                    if build_status.is_ok() && build_status.unwrap().success() {
                        let _ = tx.send(CompilerMessage::CompileSuccess);
                    } else {
                        let _ = tx.send(CompilerMessage::CompileError("Build failed.".to_string()));
                    }
                } else {
                    let _ = tx.send(CompilerMessage::CompileError(
                        "Cargo check failed.".to_string(),
                    ));
                }
            } else {
                let _ = tx.send(CompilerMessage::CompileError(
                    "Failed to invoke cargo check.".to_string(),
                ));
            }

            *is_compiling.lock().unwrap() = false;
        });
    }
}

impl eframe::App for MagicBevyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let compiler_state = EngineCompilerState::global();
        if let Ok(mut req) = compiler_state.request_reload.lock() {
            if *req {
                *req = false;
                let is_compiling = Arc::new(Mutex::new(false));
                let (tx, _) = channel();
                Self::execute_compile(tx, is_compiling);
            }
        }

        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                CompilerMessage::Indexing(file, count) => {
                    self.compiler_manager.set_compiling(&file, count);
                }
                CompilerMessage::CompileSuccess => {
                    self.compiler_manager.set_success();
                    self.reload_and_load_library();
                }
                CompilerMessage::CompileError(_err) => {
                    self.compiler_manager.set_error("Error");
                }
            }
        }

        draw_top_bar(ctx, &self.menu_registry, &self.compiler_manager.status);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                ui.heading("🌌 Magic Bevy Workspace");
            });
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    std::fs::create_dir_all("./packages").unwrap();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("MagicBevy")
            .with_inner_size([1280.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "magic-bevy",
        options,
        Box::new(|cc| Ok(Box::new(MagicBevyApp::new(cc)))),
    )
}
