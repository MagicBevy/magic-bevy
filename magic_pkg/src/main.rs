use clap::{Parser, Subcommand};
use magic_core::{mgerror, mgprint, mgsuccess, mgwarn};
use magic_pkg::{PackageManager, PackageManifest, PackageTarget};
use std::path::PathBuf;

const TAG: &str = "PackageManager";

#[derive(Parser)]
#[command(name = "magic_pkg")]
#[command(about = "MagicBevy Package Manager CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true, default_value = ".")]
    path: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new package
    Init {
        /// Package identifier in vendor/name format (e.g. my_studio/physics_tool)
        package: String,
        /// Include editor target
        #[arg(long)]
        editor: bool,
        /// Include runtime target
        #[arg(long)]
        runtime: bool,
    },
    /// Remove an existing package
    Remove {
        /// Package identifier in vendor/name format
        package: String,
    },
    /// Enable a package
    Enable {
        /// Package identifier in vendor/name format
        package: String,
    },
    /// Disable a package
    Disable {
        /// Package identifier in vendor/name format
        package: String,
    },
    /// Sync package dependencies with Cargo manifests
    Sync,
    /// List all discovered packages
    List,
}

fn main() {
    let cli = Cli::parse();
    let manager = PackageManager::new(&cli.path);

    match cli.command {
        Commands::Init {
            package,
            editor,
            runtime,
        } => {
            let (vendor, name) = match parse_vendor_and_name(&package) {
                Ok(res) => res,
                Err(err) => {
                    mgerror!(TAG, "{}", err);
                    std::process::exit(1);
                }
            };

            let mut targets = Vec::new();
            if editor {
                targets.push(PackageTarget::Editor);
            }
            if runtime {
                targets.push(PackageTarget::Runtime);
            }
            if targets.is_empty() {
                targets.push(PackageTarget::Editor);
                targets.push(PackageTarget::Runtime);
            }

            match manager.init_package(vendor, name, targets) {
                Ok((manifest, report)) => {
                    mgsuccess!(
                        TAG,
                        "Created package {} at {}",
                        manifest.id(),
                        manifest.root.display()
                    );
                    mgprint!(
                        TAG,
                        "Synchronized {} enabled package(s) (Editor: {}, Runtime: {}, updated {} manifest files).",
                        report.enabled,
                        report.editor_packages,
                        report.runtime_packages,
                        report.updated_manifests
                    );
                }
                Err(err) => {
                    mgerror!(TAG, "Failed to initialize package: {}", err);
                    std::process::exit(1);
                }
            }
        }

        Commands::Remove { package } => {
            let (vendor, name) = match parse_vendor_and_name(&package) {
                Ok(res) => res,
                Err(err) => {
                    mgerror!(TAG, "{}", err);
                    std::process::exit(1);
                }
            };

            match manager.remove_package(vendor, name) {
                Ok((pkg_id, report)) => {
                    mgsuccess!(TAG, "Removed package {}.", pkg_id);
                    mgprint!(
                        TAG,
                        "Synchronized workspace (Remaining enabled: {}, updated {} manifest files).",
                        report.enabled,
                        report.updated_manifests
                    );
                }
                Err(err) => {
                    mgerror!(TAG, "Failed to remove package: {}", err);
                    std::process::exit(1);
                }
            }
        }

        Commands::Enable { package } => {
            set_status(&manager, &package, true);
        }

        Commands::Disable { package } => {
            set_status(&manager, &package, false);
        }

        Commands::Sync => match manager.sync() {
            Ok(report) => {
                mgsuccess!(
                    TAG,
                    "Sync complete: {} discovered, {} enabled (Editor: {}, Runtime: {}, updated {} manifest files).",
                    report.discovered,
                    report.enabled,
                    report.editor_packages,
                    report.runtime_packages,
                    report.updated_manifests
                );
            }
            Err(err) => {
                mgerror!(TAG, "Sync failed: {}", err);
                std::process::exit(1);
            }
        },

        Commands::List => match manager.discover() {
            Ok(packages) => {
                if packages.is_empty() {
                    mgwarn!(TAG, "No packages found in packages directory.");
                } else {
                    mgprint!(TAG, "Discovered {} package(s):", packages.len());
                    print_packages_table(&packages);
                }
            }
            Err(err) => {
                mgerror!(TAG, "Failed to list packages: {}", err);
                std::process::exit(1);
            }
        },
    }
}

fn set_status(manager: &PackageManager, package: &str, enabled: bool) {
    let (vendor, name) = match parse_vendor_and_name(package) {
        Ok(res) => res,
        Err(err) => {
            mgerror!(TAG, "{}", err);
            std::process::exit(1);
        }
    };

    match manager.set_package_status(vendor, name, enabled) {
        Ok((pkg_id, report)) => {
            let action = if enabled { "Enabled" } else { "Disabled" };
            mgsuccess!(TAG, "{} package {}.", action, pkg_id);
            mgprint!(
                TAG,
                "Synchronized workspace (Enabled: {}, updated {} manifest files).",
                report.enabled,
                report.updated_manifests
            );
        }
        Err(err) => {
            mgerror!(TAG, "Failed to update package status: {}", err);
            std::process::exit(1);
        }
    }
}

fn parse_vendor_and_name(input: &str) -> Result<(&str, &str), String> {
    let parts: Vec<&str> = input.split('/').collect();
    if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
        Ok((parts[0], parts[1]))
    } else {
        Err(format!(
            "Invalid package format '{input}'. Expected 'vendor/name' (e.g. 'my_studio/physics_tool')"
        ))
    }
}

fn print_packages_table(packages: &[PackageManifest]) {
    let mut id_w = 12;
    let mut ver_w = 7;
    let mut stat_w = 8;
    let mut targ_w = 10;

    for p in packages {
        id_w = id_w.max(p.id().len());
        ver_w = ver_w.max(p.version.len());
        let status_str = if p.enabled { "Enabled" } else { "Disabled" };
        stat_w = stat_w.max(status_str.len());

        let targets_str = p
            .targets
            .iter()
            .map(|t| match t {
                PackageTarget::Editor => "Editor",
                PackageTarget::Runtime => "Runtime",
            })
            .collect::<Vec<_>>()
            .join(", ");
        targ_w = targ_w.max(targets_str.len());
    }

    id_w += 2;
    ver_w += 2;
    stat_w += 2;
    targ_w += 2;

    let top = format!(
        "┌{}┬{}┬{}┬{}┐",
        "─".repeat(id_w),
        "─".repeat(ver_w),
        "─".repeat(stat_w),
        "─".repeat(targ_w)
    );
    let header = format!(
        "│ {:<id_len$} │ {:<ver_len$} │ {:<stat_len$} │ {:<targ_len$} │",
        "Package ID",
        "Version",
        "Status",
        "Targets",
        id_len = id_w - 2,
        ver_len = ver_w - 2,
        stat_len = stat_w - 2,
        targ_len = targ_w - 2
    );
    let sep = format!(
        "├{}┼{}┼{}┼{}┤",
        "─".repeat(id_w),
        "─".repeat(ver_w),
        "─".repeat(stat_w),
        "─".repeat(targ_w)
    );
    let bot = format!(
        "└{}┴{}┴{}┴{}┘",
        "─".repeat(id_w),
        "─".repeat(ver_w),
        "─".repeat(stat_w),
        "─".repeat(targ_w)
    );

    println!("    {top}");
    println!("    {header}");
    println!("    {sep}");
    for p in packages {
        let status_str = if p.enabled { "Enabled" } else { "Disabled" };
        let targets_str = p
            .targets
            .iter()
            .map(|t| match t {
                PackageTarget::Editor => "Editor",
                PackageTarget::Runtime => "Runtime",
            })
            .collect::<Vec<_>>()
            .join(", ");

        println!(
            "    │ {:<id_len$} │ {:<ver_len$} │ {:<stat_len$} │ {:<targ_len$} │",
            p.id(),
            p.version,
            status_str,
            targets_str,
            id_len = id_w - 2,
            ver_len = ver_w - 2,
            stat_len = stat_w - 2,
            targ_len = targ_w - 2
        );
    }
    println!("    {bot}");
}