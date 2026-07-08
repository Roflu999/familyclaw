use chrono::Local;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn is_within(parent: &Path, child: &Path) -> bool {
    let abs_parent = std::fs::canonicalize(parent).unwrap_or_else(|_| parent.to_path_buf());
    let abs_child = std::fs::canonicalize(child).unwrap_or_else(|_| child.to_path_buf());
    abs_child.starts_with(&abs_parent)
}

fn sanitize_zip_path(name: &str) -> Option<PathBuf> {
    let path = Path::new(name);
    let mut clean = PathBuf::new();
    for comp in path.components() {
        match comp {
            std::path::Component::Normal(p) => clean.push(p),
            std::path::Component::CurDir => {}
            _ => return None, // Reject .., prefix, root
        }
    }
    Some(clean)
}

pub async fn create_backup() -> anyhow::Result<String> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("home dir not found"))?;
    let source = home.join(".openclaw");
    if !source.exists() {
        anyhow::bail!("OpenClaw directory not found");
    }

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let backup_name = format!("openclaw_backup_{}.zip", timestamp);
    let desktop = dirs::desktop_dir().unwrap_or_else(|| home.clone());
    let backup_path = desktop.join(&backup_name);

    let file = File::create(&backup_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    for entry in WalkDir::new(&source).follow_links(false) {
        let entry = entry?;
        let path = entry.path();

        // Skip symlinks to prevent traversal outside .openclaw
        if entry.file_type().is_symlink() {
            continue;
        }

        if path.is_file() {
            // Ensure file is actually inside .openclaw (defense in depth)
            if !is_within(&source, path) {
                continue;
            }

            let name = path.strip_prefix(&home).unwrap_or(path);
            let mut f = File::open(path)?;
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer)?;
            zip.start_file(name.to_string_lossy(), options)?;
            zip.write_all(&buffer)?;
        }
    }

    zip.finish()?;
    Ok(backup_path.to_string_lossy().to_string())
}

pub async fn restore_backup(path: &str) -> anyhow::Result<()> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("home dir not found"))?;
    let target = home.join(".openclaw");
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name();

        // Reject any path containing .. or absolute components
        let rel = match sanitize_zip_path(name) {
            Some(p) => p,
            None => continue,
        };

        let outpath = home.join(&rel);

        // Final safety check: must resolve inside home
        if !is_within(&home, &outpath) {
            continue;
        }

        if name.ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}
