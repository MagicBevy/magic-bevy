fn main() {
    let workspace = std::env::current_dir().expect("Unable to determine the current directory");
    let manager = magic_pkg::PackageManager::new(workspace);
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    let result = match arguments.first().map(String::as_str) {
        None | Some("sync") => sync(&manager),
        Some("init") => init(&manager, &arguments[1..]),
        Some("remove") | Some("rm") => remove(&manager, &arguments[1..]),
        Some("enable") => set_status(&manager, &arguments[1..], true), 
        Some("disable") => set_status(&manager, &arguments[1..], false),
        Some("list") => list(&manager),
        Some("help") | Some("--help") | Some("-h") => {
            print_help();
            Ok(())
        }
        Some(command) => Err(format!("Unknown command '{command}'. Run 'magic_pkg help'.")),
    };
    if let Err(error) = result {
        eprintln!("magic_pkg: {error}");
        std::process::exit(1);
    }
}

fn init(manager: &magic_pkg::PackageManager, arguments: &[String]) -> Result<(), String> {
    let mut name = None;
    let mut vendor = None;
    let mut targets = Vec::new();
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--name" => {
                index += 1;
                name = arguments.get(index).cloned();
            }
            "--vendor" => {
                index += 1;
                vendor = arguments.get(index).cloned();
            }
            "--editor" => targets.push(magic_pkg::PackageTarget::Editor),
            "--runtime" => targets.push(magic_pkg::PackageTarget::Runtime),
            "--path" => {
                index += 1;
                if let Some(path_str) = arguments.get(index) {
                    let path = std::path::Path::new(path_str);
                    let components: Vec<_> = path
                        .components()
                        .map(|c| c.as_os_str().to_string_lossy().into_owned())
                        .filter(|s| s != "." && s != "..")
                        .collect();
                    if let Some(n) = components.last() {
                        if name.is_none() {
                            name = Some(n.clone());
                        }
                        if components.len() >= 2 {
                            let v = &components[components.len() - 2];
                            if v != "packages" && vendor.is_none() {
                                vendor = Some(v.clone());
                            }
                        }
                    }
                }
            }
            value if !value.starts_with('-') => {
                if value.contains('/') || value.contains('\\') {
                    let parts: Vec<&str> = if value.contains('/') {
                        value.split('/').collect()
                    } else {
                        value.split('\\').collect()
                    };
                    if parts.len() >= 2 {
                        let parsed_name = parts.last().unwrap().to_string();
                        let parsed_vendor = parts[parts.len() - 2].to_string();
                        if name.is_none() {
                            name = Some(parsed_name);
                        }
                        if vendor.is_none() && parsed_vendor != "packages" {
                            vendor = Some(parsed_vendor);
                        }
                    } else if let Some(n) = parts.last() {
                        if name.is_none() {
                            name = Some(n.to_string());
                        }
                    }
                } else if name.is_none() {
                    name = Some(value.to_owned());
                }
            }
            option => return Err(format!("Unknown init option '{option}'.")),
        }
        index += 1;
    }
    let name = name.ok_or("Package name is required. Example: magic_pkg init my_tool OR magic_pkg init my_studio/my_tool")?;
    let vendor = vendor.unwrap_or_else(|| "local".to_owned());
    if targets.is_empty() {
        targets = vec![magic_pkg::PackageTarget::Editor, magic_pkg::PackageTarget::Runtime];
    }
    let package = manager.init_package(&vendor, &name, targets).map_err(|error| error.to_string())?;
    println!("Created package {} at {}.", package.id(), package.root.display());
    sync(manager)
}

fn remove(manager: &magic_pkg::PackageManager, arguments: &[String]) -> Result<(), String> {
    let target = arguments.first().ok_or(
        "Package target is required. Example: magic_pkg remove local/my_tool"
    )?;

    let (vendor, name) = if target.contains('/') {
        let parts: Vec<&str> = target.split('/').collect();
        (parts[0], parts[1])
    } else {
        ("local", target.as_str())
    };

    let removed_id = manager
        .remove_package(vendor, name)
        .map_err(|error| error.to_string())?;

    println!("Removed package {removed_id} and cleared its profiles.");
    Ok(())
}

fn set_status(manager: &magic_pkg::PackageManager, arguments: &[String], enabled: bool) -> Result<(), String> {
    let action_name = if enabled { "enable" } else { "disable" };
    let target = arguments.first().ok_or_else(|| {
        format!("Package target is required. Example: magic_pkg {action_name} local/my_tool")
    })?;

    let (vendor, name) = if target.contains('/') {
        let parts: Vec<&str> = target.split('/').collect();
        (parts[0], parts[1])
    } else {
        ("local", target.as_str())
    };

    let pkg_id = manager
        .set_package_status(vendor, name, enabled)
        .map_err(|error| error.to_string())?;

    let state_str = if enabled { "enabled" } else { "disabled" };
    println!("Package {pkg_id} is now {state_str}.");
    Ok(())
}

fn list(manager: &magic_pkg::PackageManager) -> Result<(), String> {
    let packages = manager.discover().map_err(|error| error.to_string())?;
    if packages.is_empty() {
        println!("No packages found.");
        return Ok(());
    }

    println!("{:<30} {:<10} {:<10}", "PACKAGE", "VERSION", "STATUS");
    println!("{}", "-".repeat(52));
    for package in packages {
        let status = if package.enabled { "[enabled]" } else { "[disabled]" };
        println!("{:<30} {:<10} {:<10}", package.id(), package.version, status);
    }
    Ok(())
}

fn sync(manager: &magic_pkg::PackageManager) -> Result<(), String> {
    let report = manager.sync_profiles().map_err(|error| error.to_string())?;
    println!(
        "Synchronized {} enabled packages ({} profiles written, {} removed).",
        report.enabled, report.written_profiles, report.removed_profiles
    );
    Ok(())
}

fn print_help() {
    println!(
        "MagicBevy package manager\n\nCommands:\n  magic_pkg init [vendor/]<name> [--editor] [--runtime]\n  magic_pkg enable [vendor/]<name>\n  magic_pkg disable [vendor/]<name>\n  magic_pkg remove [vendor/]<name>\n  magic_pkg sync\n  magic_pkg list"
    );
}
