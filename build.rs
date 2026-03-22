use std::env;
use std::fs;
use std::path::PathBuf;

const PHF_FALLBACK: &str = "pub static STATIC_ROUTE_METHOD_PATH: ::core::option::Option<::phf::Map<&'static str, &'static str>> = ::core::option::Option::None;";

fn main() {
    println!("cargo:rerun-if-env-changed=FIREWORK_ROUTES_MANIFEST");

    let manifest_path = env::var("FIREWORK_ROUTES_MANIFEST")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("target/firework/routes.manifest"));

    println!("cargo:rerun-if-changed={}", manifest_path.display());

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR must be set"));
    let generated = out_dir.join("firework_phf_routes.rs");

    if !manifest_path.exists() {
        write_fallback(&generated, "manifest not found");
        return;
    }

    let content = fs::read_to_string(&manifest_path).unwrap_or_default();
    let parsed = parse_manifest(&content);
    if parsed.source_hash.is_empty() || parsed.source_root.is_empty() {
        write_fallback(&generated, "manifest missing source metadata");
        return;
    }

    let source_root = PathBuf::from(&parsed.source_root)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(&parsed.source_root));
    if !source_root.exists() {
        write_fallback(&generated, "manifest source_root does not exist");
        return;
    }

    let current_hash = compute_source_hash(&source_root);
    if current_hash != parsed.source_hash {
        println!(
            "cargo:warning=firework PHF manifest is stale (source hash mismatch). Falling back until manifest is refreshed."
        );
        write_fallback(&generated, "stale manifest source hash");
        return;
    }

    let mut map = phf_codegen::Map::new();
    let mut inserted = 0usize;
    let mut keys: Vec<String> = Vec::new();
    let mut values: Vec<String> = Vec::new();

    for entry in parsed.routes {
        let method = entry.method.trim();
        let path = entry.path.trim();
        let is_static = entry.is_static.trim();
        if method.is_empty() || path.is_empty() || is_static != "1" {
            continue;
        }
        keys.push(format!("{method}|{path}"));
        values.push(format!("{path:?}"));
        inserted += 1;
    }

    for (k, v) in keys.iter().zip(values.iter()) {
        map.entry(k, v);
    }

    if inserted == 0 {
        write_fallback(&generated, "no static routes in manifest");
        return;
    }

    let src = format!(
        "pub static STATIC_ROUTE_METHOD_PATH: ::core::option::Option<::phf::Map<&'static str, &'static str>> = ::core::option::Option::Some({});",
        map.build()
    );
    fs::write(&generated, src).expect("failed to write generated PHF file");
}

fn write_fallback(path: &PathBuf, reason: &str) {
    println!("cargo:warning=firework PHF fallback: {reason}");
    fs::write(path, PHF_FALLBACK).expect("failed to write fallback PHF file");
}

#[derive(Debug)]
struct RouteEntry {
    method: String,
    path: String,
    is_static: String,
}

#[derive(Debug, Default)]
struct ParsedManifest {
    source_root: String,
    source_hash: String,
    routes: Vec<RouteEntry>,
}

fn parse_manifest(content: &str) -> ParsedManifest {
    let mut parsed = ParsedManifest::default();
    for line in content.lines() {
        let mut parts = line.split('\t');
        match parts.next().unwrap_or("") {
            "meta" => {
                let key = parts.next().unwrap_or("").trim();
                let value = parts.next().unwrap_or("").trim();
                match key {
                    "source_root" => parsed.source_root = value.to_string(),
                    "source_hash" => parsed.source_hash = value.to_string(),
                    _ => {}
                }
            }
            "route" => {
                let method = parts.next().unwrap_or("").trim().to_string();
                let path = parts.next().unwrap_or("").trim().to_string();
                let is_static = parts.next().unwrap_or("").trim().to_string();
                parsed.routes.push(RouteEntry {
                    method,
                    path,
                    is_static,
                });
            }
            _ => {}
        }
    }
    parsed
}

fn should_skip_source_dir(path: &std::path::Path) -> bool {
    matches!(
        path.file_name().and_then(|s| s.to_str()),
        Some("target") | Some(".git") | Some("node_modules")
    )
}

fn collect_rs_files(dir: &std::path::Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if should_skip_source_dir(dir) {
        return Ok(());
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(dir)? {
        entries.push(entry?.path());
    }
    entries.sort();

    for path in entries {
        if path.is_dir() {
            collect_rs_files(&path, out)?;
            continue;
        }
        if path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext == "rs")
        {
            out.push(path);
        }
    }
    Ok(())
}

fn fnv1a_update(mut hash: u64, bytes: &[u8]) -> u64 {
    const FNV_PRIME: u64 = 0x100000001b3;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn compute_source_hash(source_root: &std::path::Path) -> String {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    let mut files = Vec::new();
    if collect_rs_files(source_root, &mut files).is_err() {
        return String::new();
    }
    files.sort();

    let mut hash = FNV_OFFSET;
    for file in files {
        let rel = file.strip_prefix(source_root).unwrap_or(&file);
        hash = fnv1a_update(hash, rel.to_string_lossy().as_bytes());
        hash = fnv1a_update(hash, b"\0");
        match fs::read(&file) {
            Ok(content) => {
                hash = fnv1a_update(hash, &content);
                hash = fnv1a_update(hash, b"\n");
            }
            Err(_) => return String::new(),
        }
    }

    format!("{hash:016x}")
}
