use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Rebuild if any file in the docs directory changes
    println!("cargo:rerun-if-changed=../../docs");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("docs_bundle.rs");

    let mut bundle_content = String::new();
    bundle_content.push_str("pub static DOCS: &[(&str, &str)] = &[\n");

    let docs_dir = Path::new("../../docs");
    if docs_dir.exists() {
        if let Err(e) = visit_dirs(docs_dir, &mut bundle_content) {
            eprintln!("Error walking docs directory: {}", e);
        }
    }

    bundle_content.push_str("];\n");
    fs::write(&dest_path, bundle_content).unwrap();
}

fn visit_dirs(dir: &Path, bundle: &mut String) -> std::io::Result<()> {
    if dir.is_dir() {
        // Collect and sort entries for deterministic builds
        let mut entries = fs::read_dir(dir)?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, _>>()?;
        entries.sort();

        for path in entries {
            if path.is_dir() {
                visit_dirs(&path, bundle)?;
            } else if path.is_file() {
                if path.file_name().and_then(|n| n.to_str()) == Some("master_knowledge.md") {
                    continue;
                }
                if let Some(ext) = path.extension() {
                    if ext == "md" {
                        let content = fs::read_to_string(&path)?;
                        let relative_path = path.strip_prefix("../../").unwrap_or(&path);
                        
                        bundle.push_str(&format!(
                            "    ({:?}, {:?}),\n",
                            relative_path.to_string_lossy(),
                            content
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}
