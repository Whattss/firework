use std::env;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

fn main() {
    println!("🔥 Firework Hot Reload Dev Server");
    println!("==================================\n");

    let args: Vec<String> = env::args().skip(1).collect();
    
    // Check for flags
    let with_assets = args.contains(&"--assets".to_string());
    let with_state = args.contains(&"--state".to_string());
    let args: Vec<String> = args.into_iter()
        .filter(|a| !a.starts_with("--"))
        .collect();
    
    // Detect if we're in a workspace or package directory
    let current_dir = env::current_dir().expect("Failed to get current directory");
    let firework_dir = if current_dir.join("firework/src").exists() {
        current_dir.join("firework")
    } else if current_dir.join("src").exists() {
        current_dir.clone()
    } else {
        eprintln!("❌ Error: Cannot find src/ directory");
        eprintln!("   Run from project root or firework directory");
        std::process::exit(1);
    };
    
    let (command, watch_paths) = if args.is_empty() {
        let src_path = firework_dir.join("src");
        let cmd = if firework_dir == current_dir {
            "cargo run".to_string()
        } else {
            format!("cargo run -p firework")
        };
        (cmd, vec![src_path])
    } else if args.len() == 1 {
        let examples_path = firework_dir.join("examples");
        let src_path = firework_dir.join("src");
        let cmd = if firework_dir == current_dir {
            format!("cargo run --example {}", args[0])
        } else {
            format!("cargo run -p firework --example {}", args[0])
        };
        (cmd, vec![examples_path, src_path])
    } else {
        (args.join(" "), vec![firework_dir.join("src")])
    };

    for path in &watch_paths {
        if !path.exists() {
            eprintln!("❌ Error: Watch path does not exist: {}", path.display());
            std::process::exit(1);
        }
    }

    println!("Command: {}", command);
    println!("Watching:");
    for path in &watch_paths {
        println!("  - {}", path.display());
    }
    if with_assets {
        println!("\n📦 Asset recompilation: enabled");
    }
    if with_state {
        println!("💾 State preservation: enabled");
    }
    println!();

    #[cfg(feature = "hot-reload")]
    {
        use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
        use std::sync::mpsc::channel;
        use std::time::Duration;

        let (tx, rx) = channel();
        
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            },
            Config::default().with_poll_interval(Duration::from_millis(500)),
        )
        .expect("Failed to create watcher");

        for path in &watch_paths {
            watcher
                .watch(path, RecursiveMode::Recursive)
                .expect("Failed to watch path");
        }

        // Also watch for asset files if enabled
        let mut asset_paths = Vec::new();
        if with_assets {
            // Watch common frontend directories
            let frontend_dirs = vec!["public", "static", "assets", "frontend", "web"];
            for dir_name in frontend_dirs {
                let asset_path = current_dir.join(dir_name);
                if asset_path.exists() {
                    asset_paths.push(asset_path.clone());
                    watcher
                        .watch(&asset_path, RecursiveMode::Recursive)
                        .ok(); // Don't fail if we can't watch
                    println!("📦 Watching assets: {}", asset_path.display());
                }
            }
        }

        println!("🚀 Starting initial build...\n");
        
        let running_process: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
        let running_clone = running_process.clone();
        
        // Handle Ctrl+C gracefully
        ctrlc::set_handler(move || {
            println!("\n\n🛑 Shutting down...");
            if let Ok(mut proc) = running_clone.lock() {
                if let Some(child) = proc.take() {
                    let _ = kill_process_tree(child);
                }
            }
            std::process::exit(0);
        }).expect("Error setting Ctrl-C handler");
        
        run_command(&command, &running_process);

        let mut last_rebuild = std::time::Instant::now();
        let debounce = Duration::from_millis(500);

        loop {
            match rx.recv() {
                Ok(event) => {
                    let is_asset_change = asset_paths.iter().any(|p| {
                        event.paths.iter().any(|ep| ep.starts_with(p))
                    });

                    if should_ignore(&event, is_asset_change) {
                        continue;
                    }

                    let now = std::time::Instant::now();
                    if now.duration_since(last_rebuild) < debounce {
                        continue;
                    }

                    last_rebuild = now;
                    
                    if is_asset_change {
                        println!("\n📦 Asset changed: {:?}", event.paths);
                        // Run asset build command if exists
                        run_asset_build();
                        println!("✅ Assets recompiled\n");
                    } else {
                        println!("\n🔄 File changed: {:?}", event.paths);
                        println!("🔨 Rebuilding...\n");
                        run_command(&command, &running_process);
                    }
                }
                Err(_) => break,
            }
        }
    }

    #[cfg(not(feature = "hot-reload"))]
    {
        eprintln!("❌ Hot reload feature not enabled!");
        eprintln!("   Compile with: cargo build --features hot-reload");
        std::process::exit(1);
    }
}

#[cfg(feature = "hot-reload")]
fn should_ignore(event: &notify::Event, is_asset: bool) -> bool {
    use notify::EventKind;
    
    // Ignore non-modify events
    match event.kind {
        EventKind::Modify(_) | EventKind::Create(_) => {},
        _ => return true,
    }
    
    for path in &event.paths {
        let path_str = path.to_string_lossy();
        
        // Ignore build artifacts, git, and temp files
        if path_str.contains("target") 
            || path_str.contains(".git") 
            || path_str.contains("node_modules")
            || path_str.contains("/.") 
            || path_str.ends_with(".log")
            || path_str.ends_with("~")
            || path_str.ends_with(".swp")
            || path_str.ends_with(".swo")
            || path_str.contains("~") {
            return true;
        }
        
        // Ignore Vim/Neovim temporary files (like "4913")
        if let Some(filename) = path.file_name() {
            let fname = filename.to_string_lossy();
            // Ignore numeric-only filenames (Vim temp files)
            if fname.chars().all(|c| c.is_numeric()) {
                return true;
            }
            // Ignore files starting with .
            if fname.starts_with('.') {
                return true;
            }
        }

        // For assets, allow more file types
        if is_asset {
            // Allow common asset extensions
            if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy();
                if ["css", "js", "html", "svg", "png", "jpg", "jpeg", "gif", "woff", "woff2", "ttf"].contains(&ext.as_ref()) {
                    return false;
                }
            }
        } else {
            // Only watch .rs and .toml files for code
            if let Some(ext) = path.extension() {
                if ext != "rs" && ext != "toml" {
                    return true;
                }
            } else {
                return true;
            }
        }
    }
    false
}

fn run_asset_build() {
    // Check for common asset build tools
    if std::path::Path::new("package.json").exists() {
        // Try npm run build or similar
        let _ = Command::new("npm")
            .args(&["run", "build:assets"])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status();
    }
    
    // Check for Tailwind
    if std::path::Path::new("tailwind.config.js").exists() {
        let _ = Command::new("npx")
            .args(&["tailwindcss", "-i", "./src/input.css", "-o", "./public/output.css"])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status();
    }
}

fn run_command(command: &str, running_process: &Arc<Mutex<Option<Child>>>) {
    // Kill the previous running process
    if let Ok(mut proc) = running_process.lock() {
        if let Some(child) = proc.take() {
            let _ = kill_process_tree(child);
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
    
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    match Command::new(parts[0])
        .args(&parts[1..])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
    {
        Ok(child) => {
            if let Ok(mut proc) = running_process.lock() {
                *proc = Some(child);
            }
            println!("\n✅ Server started!\n");
        }
        Err(e) => {
            eprintln!("\n❌ Failed to execute: {}\n", e);
        }
    }
}

#[cfg(unix)]
fn kill_process_tree(mut child: Child) {
    use std::process::Command as SysCommand;
    
    let pid = child.id();
    
    // Try to kill the process group first
    let _ = SysCommand::new("pkill")
        .args(&["-P", &pid.to_string()])
        .status();
    
    // Then kill the main process
    let _ = child.kill();
    let _ = child.wait();
}

#[cfg(not(unix))]
fn kill_process_tree(mut child: Child) {
    let _ = child.kill();
    let _ = child.wait();
}
