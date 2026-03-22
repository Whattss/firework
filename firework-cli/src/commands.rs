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

pub fn run_dev(hot_reload: bool, impure: bool) {
    println!("{} Starting development server...", "🚀".bright_yellow());
    
    if hot_reload {
        println!("{} Hot reload enabled", "🔥".bright_yellow());
        run_with_file_watcher(impure);
    } else {
        run_cargo(&["run"], impure);
    }
}

pub fn run_release(impure: bool) {
    println!("{} Building release...", "📦".bright_yellow());
    run_cargo(&["build", "--release"], impure);
    
    println!("{} Running release binary...", "🚀".bright_yellow());
    let binary = get_binary_name();
    run_command("./target/release/", &binary, &[], impure);
}

pub fn run_build(impure: bool) {
    println!("{} Building project...", "🔨".bright_yellow());
    run_cargo(&["build"], impure);
}

pub fn run_script(name: &str, impure: bool) {
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
            run_shell_command(script, impure);
        } else {
            eprintln!("{} Script '{}' not found in Firework.toml", "✗".bright_red(), name);
            std::process::exit(1);
        }
    } else {
        eprintln!("{} No scripts defined in Firework.toml", "✗".bright_red());
        std::process::exit(1);
    }
}

fn run_with_file_watcher(impure: bool) {
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
            let mut build_cmd = Command::new("cargo");
            build_cmd.args(&["build"])
                .stdin(Stdio::null())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit());
            if impure {
                build_cmd.env("FIREWORK_IMPURE", "1");
            }
            let build_status = build_cmd.status();
                
            if let Ok(status) = build_status {
                if status.success() {
                    // Get binary name and run it
                    let binary_name = get_binary_name();
                    let binary_path = format!("./target/debug/{}", binary_name);
                    
                    let mut run_cmd = Command::new(&binary_path);
                    run_cmd
                        .stdin(Stdio::null())
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit());
                    if impure {
                        run_cmd.env("FIREWORK_IMPURE", "1");
                    }
                    child = run_cmd.spawn().ok();
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

fn run_cargo(args: &[&str], impure: bool) {
    let mut cmd = Command::new("cargo");
    cmd.args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    if impure {
        cmd.env("FIREWORK_IMPURE", "1");
    }
    let status = cmd.status().expect("Failed to run cargo");
    
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
}

fn run_command(dir: &str, cmd: &str, args: &[&str], impure: bool) {
    let mut command = Command::new(format!("{}{}", dir, cmd));
    command
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    if impure {
        command.env("FIREWORK_IMPURE", "1");
    }
    let status = command.status().expect("Failed to run command");
    
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
}

fn run_shell_command(cmd: &str, impure: bool) {
    let mut shell = Command::new("sh");
    shell
        .arg("-c")
        .arg(cmd)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    if impure {
        shell.env("FIREWORK_IMPURE", "1");
    }
    let status = shell.status().expect("Failed to run shell command");
    
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

pub fn list_routes(filter: Option<&str>, verbose: bool, export: Option<&str>, check: bool, stats: bool) {
    use std::collections::HashMap;
    
    println!("{} Scanning for routes...\n", "🔍".bright_yellow());
    
    // Parse src directory for route macros
    let routes = scan_routes_in_directory("src");
    
    if routes.is_empty() {
        println!("{} No routes found", "⚠".bright_yellow());
        println!("\n  Make sure you're using #[get], #[post], etc. macros from firework");
        return;
    }
    
    // Check for duplicates if requested
    if check {
        check_route_conflicts(&routes);
        return;
    }
    
    // Show stats if requested
    if stats {
        show_route_stats(&routes);
        return;
    }
    
    // Export to OpenAPI if requested
    if let Some(format) = export {
        export_routes(&routes, format);
        return;
    }
    
    // Group by method
    let mut grouped: HashMap<String, Vec<RouteInfo>> = HashMap::new();
    for route in routes {
        if let Some(filter_str) = filter {
            if !route.path.contains(filter_str) && !route.handler.contains(filter_str) {
                continue;
            }
        }
        
        grouped.entry(route.method.clone())
            .or_insert_with(Vec::new)
            .push(route);
    }
    
    if grouped.is_empty() {
        if let Some(f) = filter {
            println!("{} No routes match filter '{}'", "⚠".bright_yellow(), f);
        }
        return;
    }
    
    // Sort methods
    let mut methods: Vec<_> = grouped.keys().cloned().collect();
    methods.sort_by(|a, b| {
        let order = ["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS", "HEAD"];
        let a_pos = order.iter().position(|&m| m == a).unwrap_or(99);
        let b_pos = order.iter().position(|&m| m == b).unwrap_or(99);
        a_pos.cmp(&b_pos)
    });
    
    if !verbose {
        println!("  {}", "────────────────────────────────────────────────────────────────".dimmed());
    }
    
    // Print routes
    let mut total = 0;
    for method in methods {
        if let Some(route_list) = grouped.get(&method) {
            let mut routes = route_list.clone();
            // Sort by path
            routes.sort_by(|a, b| a.path.cmp(&b.path));
            
            total += routes.len();
            
            for route in routes {
                print_route(&route, verbose);
            }
        }
    }
    
    if !verbose {
        println!("  {}", "────────────────────────────────────────────────────────────────".dimmed());
    }
    
    println!("\n{} {} routes registered", "✓".bright_green(), total.to_string().bright_cyan());
    
    if filter.is_some() {
        println!("  {}", "(filtered)".dimmed());
    }
    
    println!("\n  Tip: Use {} for detailed information", "--verbose".bright_cyan());
    if filter.is_none() {
        println!("       Use {} to filter routes", "--filter <pattern>".bright_cyan());
    }
    println!("       Use {} to check for conflicts", "--check".bright_cyan());
    println!("       Use {} to show statistics", "--stats".bright_cyan());
    println!("       Use {} to export as OpenAPI", "--export openapi".bright_cyan());
}

#[derive(Debug, Clone)]
struct RouteInfo {
    method: String,
    path: String,
    handler: String,
    file: String,
    line: usize,
    middleware: Option<String>,
}

fn scan_routes_in_directory(dir: &str) -> Vec<RouteInfo> {
    use std::fs;
    use walkdir::WalkDir;
    
    let mut routes = Vec::new();
    
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            if let Ok(content) = fs::read_to_string(path) {
                if let Some(path_str) = path.to_str() {
                    routes.extend(parse_routes_from_file(&content, path_str));
                }
            }
        }
    }
    
    routes
}

fn parse_routes_from_file(content: &str, file: &str) -> Vec<RouteInfo> {
    use regex::Regex;
    
    let mut routes = Vec::new();
    
    // Regex to match route macros - these are compile-time constants so unwrap is safe
    let route_regex = Regex::new(r#"#\[(get|post|put|patch|delete|options|head)\("([^"]+)"\)\]"#)
        .expect("Invalid route regex pattern");
    let handler_regex = Regex::new(r"(?:async\s+)?fn\s+(\w+)")
        .expect("Invalid handler regex pattern");
    
    let lines: Vec<&str> = content.lines().collect();
    
    for (i, line) in lines.iter().enumerate() {
        if let Some(caps) = route_regex.captures(line) {
            // Regex captures are guaranteed by the pattern
            let method = caps.get(1)
                .map(|m| m.as_str().to_uppercase())
                .unwrap_or_else(|| "UNKNOWN".to_string());
            let path = caps.get(2)
                .map(|m| m.as_str().to_string())
                .unwrap_or_else(|| "/unknown".to_string());
            
            // Look for handler name in next few lines
            let handler = lines.get(i + 1)
                .and_then(|next_line| handler_regex.captures(next_line))
                .and_then(|caps| caps.get(1))
                .map(|m| m.as_str().to_string())
                .unwrap_or_else(|| "<unknown>".to_string());
            
            // Try to detect middleware (look for #[middleware] or similar)
            let middleware = detect_middleware(&lines, i);
            
            routes.push(RouteInfo {
                method,
                path,
                handler,
                file: file.to_string(),
                line: i + 1,
                middleware,
            });
        }
    }
    
    routes
}

// Detect middleware from route annotations
fn detect_middleware(lines: &[&str], route_line: usize) -> Option<String> {
    // Look backwards for middleware annotations
    for i in (route_line.saturating_sub(5)..route_line).rev() {
        if let Some(line) = lines.get(i) {
            if line.contains("#[middleware") {
                // Extract middleware name
                if let Some(start) = line.find('(') {
                    if let Some(end) = line.find(')') {
                        return Some(line[start + 1..end].trim_matches('"').to_string());
                    }
                }
            }
        }
    }
    None
}

fn print_route(route: &RouteInfo, verbose: bool) {
    use colored::Colorize;
    
    let method_colored = match route.method.as_str() {
        "GET" => route.method.bright_green(),
        "POST" => route.method.bright_blue(),
        "PUT" => route.method.bright_yellow(),
        "PATCH" => route.method.bright_cyan(),
        "DELETE" => route.method.bright_red(),
        _ => route.method.bright_white(),
    };
    
    let path_colored = route.path.bright_white();
    let handler_colored = route.handler.bright_magenta();
    
    if verbose {
        println!("  {} {}", 
            method_colored,
            path_colored
        );
        println!("    {} {}", "Handler:".dimmed(), handler_colored);
        println!("    {} {}:{}", "Location:".dimmed(), route.file.dimmed(), route.line.to_string().dimmed());
        if let Some(middleware) = &route.middleware {
            println!("    {} {}", "Middleware:".dimmed(), middleware.bright_yellow());
        }
        println!();
    } else {
        print!("  {:7} {:40} {}", 
            method_colored,
            path_colored,
            handler_colored.dimmed()
        );
        
        if let Some(middleware) = &route.middleware {
            print!("  [{}]", middleware.bright_yellow());
        }
        
        println!();
    }
}

// Check for route conflicts
fn check_route_conflicts(routes: &[RouteInfo]) {
    use std::collections::HashMap;
    use colored::Colorize;
    
    println!("{} Checking for route conflicts...\n", "🔍".bright_yellow());
    
    let mut conflicts = HashMap::new();
    let mut has_conflicts = false;
    
    for route in routes {
        let key = format!("{} {}", route.method, route.path);
        conflicts.entry(key.clone())
            .or_insert_with(Vec::new)
            .push(route);
    }
    
    for (key, routes_list) in conflicts {
        if routes_list.len() > 1 {
            has_conflicts = true;
            println!("{} Duplicate route found: {}", "⚠".bright_red(), key.bright_yellow());
            for route in routes_list {
                println!("    {} {}:{}", "→".dimmed(), route.file.dimmed(), route.line);
            }
            println!();
        }
    }
    
    // Check for parameter conflicts
    let mut param_groups: HashMap<String, Vec<&RouteInfo>> = HashMap::new();
    for route in routes {
        let pattern = route.path.split('/').map(|segment| {
            if segment.starts_with(':') {
                ":param"
            } else {
                segment
            }
        }).collect::<Vec<_>>().join("/");
        
        param_groups.entry(format!("{} {}", route.method, pattern))
            .or_insert_with(Vec::new)
            .push(route);
    }
    
    for (pattern, routes_list) in param_groups {
        if routes_list.len() > 1 {
            println!("{} Potential parameter conflict: {}", "⚠".bright_yellow(), pattern);
            for route in routes_list {
                println!("    {} {} ({}:{})", "→".dimmed(), route.path.bright_cyan(), route.file.dimmed(), route.line);
            }
            println!();
        }
    }
    
    if !has_conflicts {
        println!("{} No route conflicts found!", "✓".bright_green());
    }
}

// Show route statistics
fn show_route_stats(routes: &[RouteInfo]) {
    use std::collections::HashMap;
    use colored::Colorize;
    
    println!("{} Route Statistics\n", "📊".bright_yellow());
    
    let mut method_counts: HashMap<String, usize> = HashMap::new();
    let mut file_counts: HashMap<String, usize> = HashMap::new();
    let mut total_params = 0;
    let mut middleware_count = 0;
    
    for route in routes {
        *method_counts.entry(route.method.clone()).or_insert(0) += 1;
        *file_counts.entry(route.file.clone()).or_insert(0) += 1;
        
        if route.path.contains(':') {
            total_params += route.path.matches(':').count();
        }
        
        if route.middleware.is_some() {
            middleware_count += 1;
        }
    }
    
    println!("  {} Total Routes", "📍".bright_cyan());
    println!("    {}", routes.len().to_string().bright_green());
    println!();
    
    println!("  {} By HTTP Method", "🔤".bright_cyan());
    let mut methods: Vec<_> = method_counts.iter().collect();
    methods.sort_by(|a, b| b.1.cmp(a.1));
    for (method, count) in methods {
        let method_colored = match method.as_str() {
            "GET" => method.bright_green(),
            "POST" => method.bright_blue(),
            "PUT" => method.bright_yellow(),
            "PATCH" => method.bright_cyan(),
            "DELETE" => method.bright_red(),
            _ => method.bright_white(),
        };
        println!("    {:7} {}", method_colored, count.to_string().bright_white());
    }
    println!();
    
    println!("  {} Top Files", "📁".bright_cyan());
    let mut files: Vec<_> = file_counts.iter().collect();
    files.sort_by(|a, b| b.1.cmp(a.1));
    for (file, count) in files.iter().take(5) {
        let short_file = file.split('/').last().unwrap_or(file);
        println!("    {:20} {}", short_file.dimmed(), count.to_string().bright_white());
    }
    println!();
    
    println!("  {} Route Parameters", "🔖".bright_cyan());
    println!("    {} total params in {} routes", 
        total_params.to_string().bright_green(),
        routes.iter().filter(|r| r.path.contains(':')).count().to_string().bright_white()
    );
    println!();
    
    if middleware_count > 0 {
        println!("  {} Middleware Usage", "🔧".bright_cyan());
        println!("    {} routes with middleware", middleware_count.to_string().bright_green());
        println!();
    }
}

// Export routes to OpenAPI/Swagger format
fn export_routes(routes: &[RouteInfo], format: &str) {
    use colored::Colorize;
    use std::fs;
    
    match format {
        "openapi" | "swagger" => {
            println!("{} Exporting routes to OpenAPI 3.0 format...\n", "📤".bright_yellow());
            
            let openapi = generate_openapi_spec(routes);
            let filename = "openapi.json";
            
            fs::write(filename, openapi).expect("Failed to write OpenAPI spec");
            
            println!("{} OpenAPI spec exported to {}", "✓".bright_green(), filename.bright_cyan());
            println!("\n  You can now:");
            println!("    • Import into Postman");
            println!("    • Use with Swagger UI");
            println!("    • Generate client SDKs");
        }
        "markdown" | "md" => {
            println!("{} Exporting routes to Markdown...\n", "📤".bright_yellow());
            
            let markdown = generate_markdown_docs(routes);
            let filename = "ROUTES.md";
            
            fs::write(filename, markdown).expect("Failed to write markdown");
            
            println!("{} Route documentation exported to {}", "✓".bright_green(), filename.bright_cyan());
        }
        _ => {
            eprintln!("{} Unknown export format: {}", "✗".bright_red(), format);
            eprintln!("  Supported formats: openapi, swagger, markdown, md");
        }
    }
}

// Generate OpenAPI 3.0 specification
fn generate_openapi_spec(routes: &[RouteInfo]) -> String {
    use serde_json::json;
    
    let mut paths = serde_json::Map::new();
    
    for route in routes {
        let path_entry = paths.entry(route.path.clone()).or_insert_with(|| json!({}));
        if let Some(path_obj) = path_entry.as_object_mut() {
            let method = route.method.to_lowercase();
            path_obj.insert(method, json!({
                "summary": route.handler.clone(),
                "operationId": route.handler.clone(),
                "responses": {
                    "200": {
                        "description": "Successful response"
                    }
                },
                "tags": [extract_tag_from_path(&route.path)]
            }));
        }
    }
    
    let spec = json!({
        "openapi": "3.0.0",
        "info": {
            "title": "Firework API",
            "version": "1.0.0",
            "description": "Auto-generated API documentation"
        },
        "paths": paths
    });
    
    serde_json::to_string_pretty(&spec)
        .unwrap_or_else(|_| String::from("{}"))
}

// Generate Markdown documentation
fn generate_markdown_docs(routes: &[RouteInfo]) -> String {
    use std::collections::HashMap;
    
    let mut md = String::from("# API Routes\n\n");
    md.push_str("Auto-generated route documentation\n\n");
    md.push_str("---\n\n");
    
    // Group by tag (extracted from path)
    let mut grouped: HashMap<String, Vec<&RouteInfo>> = HashMap::new();
    for route in routes {
        let tag = extract_tag_from_path(&route.path);
        grouped.entry(tag).or_insert_with(Vec::new).push(route);
    }
    
    for (tag, routes_list) in grouped {
        md.push_str(&format!("## {}\n\n", tag));
        
        for route in routes_list {
            md.push_str(&format!("### {} `{}`\n\n", route.method, route.path));
            md.push_str(&format!("**Handler:** `{}`\n\n", route.handler));
            if let Some(middleware) = &route.middleware {
                md.push_str(&format!("**Middleware:** `{}`\n\n", middleware));
            }
            md.push_str("---\n\n");
        }
    }
    
    md
}

// Extract tag from path (e.g., /api/users/... -> Users)
fn extract_tag_from_path(path: &str) -> String {
    let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
    
    if parts.len() >= 2 {
        let tag = parts[1];
        // Capitalize first letter
        tag.chars().next()
            .map(|c| c.to_uppercase().collect::<String>() + &tag[1..])
            .unwrap_or_else(|| tag.to_string())
    } else {
        "General".to_string()
    }
}
