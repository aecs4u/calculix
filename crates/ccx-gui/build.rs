use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct Unit {
    legacy_rel_path: String,
    module_name: String,
    language: &'static str,
    line_count: usize,
}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let legacy_root = manifest_dir.join("../../calculix_migration_tooling/cgx_2.23/src");
    if !legacy_root.is_dir() {
        panic!(
            "legacy gui tree not found at {}",
            legacy_root.to_string_lossy()
        );
    }

    let mut units = Vec::<Unit>::new();
    visit_dir(&legacy_root, &legacy_root, &mut units).expect("scan legacy tree");
    units.sort_by(|a, b| a.legacy_rel_path.cmp(&b.legacy_rel_path));

    let mut generated = String::new();
    generated.push_str("pub const LEGACY_GUI_SOURCE_UNITS: &[LegacyGuiSourceUnit] = &[\n");
    for unit in units {
        generated.push_str("    LegacyGuiSourceUnit {\n");
        generated.push_str(&format!(
            "        legacy_rel_path: {:?},\n",
            unit.legacy_rel_path
        ));
        generated.push_str(&format!("        module_name: {:?},\n", unit.module_name));
        generated.push_str(&format!(
            "        language: LegacyGuiLanguage::{},\n",
            unit.language
        ));
        generated.push_str(&format!("        line_count: {},\n", unit.line_count));
        generated.push_str("    },\n");
    }
    generated.push_str("];\n");

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let out_file = out_dir.join("legacy_gui_source_units.rs");
    fs::write(&out_file, generated).expect("write generated catalog");
}

fn visit_dir(root: &Path, dir: &Path, units: &mut Vec<Unit>) -> io::Result<()> {
    println!("cargo:rerun-if-changed={}", dir.display());

    let mut entries = fs::read_dir(dir)?.collect::<Result<Vec<_>, io::Error>>()?;
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            visit_dir(root, &path, units)?;
            continue;
        }
        if !path.is_file() {
            continue;
        }

        println!("cargo:rerun-if-changed={}", path.display());
        let rel = path
            .strip_prefix(root)
            .expect("path must be within root")
            .to_string_lossy()
            .replace('\\', "/");
        let bytes = fs::read(&path)?;
        let line_count = if bytes.is_empty() {
            0
        } else {
            bytes.iter().filter(|&&byte| byte == b'\n').count() + 1
        };

        units.push(Unit {
            module_name: to_rust_ident(&rel),
            language: detect_language(&path),
            legacy_rel_path: rel,
            line_count,
        });
    }

    Ok(())
}

fn detect_language(path: &Path) -> &'static str {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match ext.as_str() {
        "c" => "C",
        "cpp" | "cxx" | "cc" => "Cpp",
        "h" | "hpp" => "Header",
        _ => "Other",
    }
}

fn to_rust_ident(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        return "_".to_string();
    }
    if out
        .as_bytes()
        .first()
        .expect("identifier should not be empty")
        .is_ascii_digit()
    {
        out.insert(0, '_');
    }
    out
}
