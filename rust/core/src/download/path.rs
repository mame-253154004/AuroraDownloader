use std::path::{Path, PathBuf};

use crate::models::DownloadError;

pub fn sanitize_file_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len());

    for ch in name.chars() {
        let blocked =
            matches!(ch, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') || ch.is_control();
        if blocked {
            out.push('_');
        } else {
            out.push(ch);
        }
    }

    let out = out.trim_matches('.').trim().to_string();
    if out.is_empty() {
        "download.bin".to_string()
    } else {
        out
    }
}

pub fn safe_target_path(
    destination_dir: &Path,
    requested_name: &str,
) -> Result<PathBuf, DownloadError> {
    let file_name = sanitize_file_name(requested_name);
    let candidate = destination_dir.join(file_name);

    if candidate
        .components()
        .any(|comp| matches!(comp, std::path::Component::ParentDir))
    {
        return Err(DownloadError::PolicyViolation {
            reason: "path traversal detected".to_string(),
        });
    }

    Ok(candidate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_filename() {
        assert_eq!(sanitize_file_name("../a:b?.txt"), "_a_b_.txt");
    }

    #[test]
    fn blocks_empty_names() {
        assert_eq!(sanitize_file_name("..."), "download.bin");
    }

    #[test]
    fn safe_target_stays_inside_destination() {
        let target = safe_target_path(Path::new("/tmp/downloads"), "../../evil.sh").expect("path");
        assert!(target.starts_with("/tmp/downloads"));
        assert!(target.to_string_lossy().contains("evil.sh"));
    }
}
