use colored::Colorize;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn new_project(name: &str, template: Option<&str>) {
    println!("{} Creating new Firework project: {}", "🔥".bright_yellow(), name.bright_cyan());
    
    let template_type = template.unwrap_or("basic");
    
    if Path::new(name).exists() {
        eprintln!("{} Directory '{}' already exists", "✗".bright_red(), name);
        std::process::exit(1);
    }
    
    fs::create_dir_all(format!("{}/src", name)).expect("Failed to create project directory");
    
    match template_type {
        "basic" => create_basic_template(name),
        "api" => create_api_template(name),
        "fullstack" => create_fullstack_template(name),
        _ => {
            eprintln!("{} Unknown template: {}", "✗".bright_red(), template_type);
            std::process::exit(1);
        }
    }
    
    println!("{} Project created successfully!", "✓".bright_green());
    println!("\n  cd {}", name);
    println!("  fwk run dev --hot-reload\n");
}

pub fn create_config() {
    if Path::new("Firework.toml").exists() {
        eprintln!("{} Firework.toml already exists", "✗".bright_red());
        std::process::exit(1);
    }
    
    let config = crate::templates::get_config_template();
    fs::write("Firework.toml", config).expect("Failed to write config");
    
    println!("{} Created Firework.toml", "✓".bright_green());
}

pub fn run_dev(hot_reload: bool) {
    println!("{} Starting development server...", "🚀".bright_yellow());
    
    if hot_reload {
        println!("{} Hot reload enabled", "🔥".bright_yellow());
        run_with_file_watcher();
    } else {
        run_cargo(&["run"]);
    }
}

pub fn run_release() {
    println!("{} Building release...", "📦".bright_yellow());
    run_cargo(&["build", "--release"]);
    
    println!("{} Running release binary...", "🚀".bright_yellow());
    let binary = get_binary_name();
    run_command("./target/release/", &binary, &[]);
}

pub fn run_build() {
    println!("{} Building project...", "🔨".bright_yellow());
    run_cargo(&["build"]);
}

pub fn run_script(name: &str) {
    let config_path = Path::new("Firework.toml");
    if !config_path.exists() {
        eprintln!("{} Firework.toml not found", "✗".bright_red());
        std::process::exit(1);
    }
    
    let config_str = fs::read_to_string(config_path).expect("Failed to read Firework.toml");
    let config: toml::Value = toml::from_str(&config_str).expect("Failed to parse Firework.toml");
    
    if let Some(scripts) = config.get("scripts").and_then(|s| s.as_table()) {
        if let Some(script) = scripts.get(name).and_then(|s| s.as_str()) {
            println!("{} Running script: {}", "⚡".bright_yellow(), name.bright_cyan());
            run_shell_command(script);
        } else {
            eprintln!("{} Script '{}' not found in Firework.toml", "✗".bright_red(), name);
            std::process::exit(1);
        }
    } else {
        eprintln!("{} No scripts defined in Firework.toml", "✗".bright_red());
        std::process::exit(1);
    }
}

fn run_with_file_watcher() {
    use notify::{Watcher, RecursiveMode, Event};
    use std::sync::mpsc::channel;
    use std::time::Duration;
    use std::process::{Child, Stdio};
    
    let (tx, rx) = channel::<notify::Result<Event>>();
    
    let mut watcher = notify::recommended_watcher(tx).expect("Failed to create watcher");
    
    // Watch src directory
    if Path::new("src").exists() {
        watcher.watch(Path::new("src"), RecursiveMode::Recursive)
            .expect("Failed to watch src");
    }
    
    // Watch Cargo.toml
    if Path::new("Cargo.toml").exists() {
        watcher.watch(Path::new("Cargo.toml"), RecursiveMode::NonRecursive)
            .expect("Failed to watch Cargo.toml");
    }
    
    println!("{} Watching for changes...", "👀".bright_yellow());
    
    let mut child: Option<Child> = None;
    let mut needs_rebuild = true;
    
    loop {
        if needs_rebuild {
            // Kill existing process
            if let Some(mut c) = child.take() {
                let _ = c.kill();
                let _ = c.wait();
            }
            
            println!("\n{} Rebuilding and restarting...", "🔄".bright_yellow());
            
            // Build the project
            let build_status = Command::new("cargo")
                .args(&["build"])
                .stdin(Stdio::null())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status();
                
            if let Ok(status) = build_status {
                if status.success() {
                    // Get binary name and run it
                    let binary_name = get_binary_name();
                    let binary_path = format!("./target/debug/{}", binary_name);
                    
                    child = Command::new(&binary_path)
                        .stdin(Stdio::null())
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit())
                        .spawn()
                        .ok();
                } else {
                    println!("{} Build failed, waiting for changes...", "✗".bright_red());
                }
            }
                
            needs_rebuild = false;
        }
        
        // Wait for file changes
        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(Ok(event)) => {
                // Ignore certain files
                let should_ignore = event.paths.iter().any(|p| {
                    let path_str = p.to_string_lossy();
                    path_str.contains("4913") || 
                    path_str.ends_with(".swp") ||
                    path_str.ends_with("~") ||
                    path_str.contains(".git") ||
                    path_str.contains("/target/")
                });
                
                if !should_ignore {
                    needs_rebuild = true;
                }
            }
            Ok(Err(e)) => eprintln!("{} Watch error: {:?}", "⚠".bright_yellow(), e),
            Err(_) => {} // Timeout, continue
        }
    }
}

fn run_cargo(args: &[&str]) {
    let status = Command::new("cargo")
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("Failed to run cargo");
    
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
}

fn run_command(dir: &str, cmd: &str, args: &[&str]) {
    let status = Command::new(format!("{}{}", dir, cmd))
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("Failed to run command");
    
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
}

fn run_shell_command(cmd: &str) {
    let status = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("Failed to run shell command");
    
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
}

fn get_binary_name() -> String {
    let cargo_toml = fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");
    let cargo: toml::Value = toml::from_str(&cargo_toml).expect("Failed to parse Cargo.toml");
    
    cargo
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("app")
        .to_string()
}

fn create_basic_template(name: &str) {
    let cargo_toml = crate::templates::get_cargo_template(name, "basic");
    let main_rs = crate::templates::get_main_template("basic");
    let config = crate::templates::get_config_template();
    
    fs::write(format!("{}/Cargo.toml", name), cargo_toml).expect("Failed to write Cargo.toml");
    fs::write(format!("{}/src/main.rs", name), main_rs).expect("Failed to write main.rs");
    fs::write(format!("{}/Firework.toml", name), config).expect("Failed to write Firework.toml");
}

fn create_api_template(name: &str) {
    let cargo_toml = crate::templates::get_cargo_template(name, "api");
    let main_rs = crate::templates::get_main_template("api");
    let config = crate::templates::get_config_template();
    
    fs::write(format!("{}/Cargo.toml", name), cargo_toml).expect("Failed to write Cargo.toml");
    fs::write(format!("{}/src/main.rs", name), main_rs).expect("Failed to write main.rs");
    fs::write(format!("{}/Firework.toml", name), config).expect("Failed to write Firework.toml");
}

fn create_fullstack_template(name: &str) {
    let cargo_toml = crate::templates::get_cargo_template(name, "fullstack");
    let main_rs = crate::templates::get_main_template("fullstack");
    let config = crate::templates::get_config_template();
    
    fs::create_dir_all(format!("{}/static", name)).expect("Failed to create static directory");
    fs::create_dir_all(format!("{}/templates", name)).expect("Failed to create templates directory");
    
    fs::write(format!("{}/Cargo.toml", name), cargo_toml).expect("Failed to write Cargo.toml");
    fs::write(format!("{}/src/main.rs", name), main_rs).expect("Failed to write main.rs");
    fs::write(format!("{}/Firework.toml", name), config).expect("Failed to write Firework.toml");
    
    let index_html = r#"<!DOCTYPE html>
<html>
<head>
    <title>Firework App</title>
</head>
<body>
    <h1>Welcome to Firework!</h1>
</body>
</html>"#;
    
    fs::write(format!("{}/static/index.html", name), index_html).expect("Failed to write index.html");
}
