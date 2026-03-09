use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Serialize)]
pub struct GitStatus {
    /// Map of relative path → status code ("M", "A", "D", "U", "?")
    pub files: HashMap<String, String>,
    /// The git repo root (so frontend can compute relative paths)
    pub root: String,
}

/// Walk up from `start` to find the nearest `.git` directory.
fn find_git_root(start: &Path) -> Option<&Path> {
    let mut current = start;
    loop {
        if current.join(".git").exists() {
            return Some(current);
        }
        current = current.parent()?;
    }
}

/// Parse `git status --porcelain` output into a path → status map.
/// Porcelain format: XY PATH (where X=index status, Y=worktree status)
fn parse_porcelain(output: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in output.lines() {
        if line.len() < 4 {
            continue;
        }
        let index_status = line.as_bytes()[0];
        let work_status = line.as_bytes()[1];
        // Skip the space at position 2; path starts at position 3
        let path = &line[3..];
        // For renamed files, porcelain shows "old -> new" — take the new path
        let path = if let Some(arrow) = path.find(" -> ") {
            &path[arrow + 4..]
        } else {
            path
        };

        let status = match (index_status, work_status) {
            (b'?', b'?') => "U",         // Untracked
            (b'A', _) => "A",             // Added (staged)
            (b'D', _) | (_, b'D') => "D", // Deleted
            (_, b'M') | (b'M', _) => "M", // Modified
            (b'R', _) => "M",             // Renamed (show as modified)
            _ => continue,
        };

        map.insert(path.to_string(), status.to_string());
    }
    map
}

#[tauri::command]
pub async fn git_status(path: String) -> Result<GitStatus, String> {
    tokio::task::spawn_blocking(move || git_status_inner(&path))
        .await
        .map_err(|e| format!("Git status task failed: {}", e))?
}

fn git_status_inner(path: &str) -> Result<GitStatus, String> {
    let dir = Path::new(path);
    let git_root = find_git_root(dir).ok_or_else(|| "Not a git repository".to_string())?;

    let output = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .current_dir(git_root)
        .output()
        .map_err(|e| format!("Failed to run git: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git status failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let files = parse_porcelain(&stdout);

    Ok(GitStatus {
        files,
        root: git_root.to_string_lossy().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_porcelain_empty() {
        let result = parse_porcelain("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_porcelain_modified() {
        let result = parse_porcelain(" M src/main.rs\n");
        assert_eq!(result.get("src/main.rs").map(|s| s.as_str()), Some("M"));
    }

    #[test]
    fn test_parse_porcelain_staged_modified() {
        let result = parse_porcelain("M  src/lib.rs\n");
        assert_eq!(result.get("src/lib.rs").map(|s| s.as_str()), Some("M"));
    }

    #[test]
    fn test_parse_porcelain_added() {
        let result = parse_porcelain("A  new_file.txt\n");
        assert_eq!(result.get("new_file.txt").map(|s| s.as_str()), Some("A"));
    }

    #[test]
    fn test_parse_porcelain_deleted() {
        let result = parse_porcelain(" D removed.txt\n");
        assert_eq!(result.get("removed.txt").map(|s| s.as_str()), Some("D"));
    }

    #[test]
    fn test_parse_porcelain_untracked() {
        let result = parse_porcelain("?? untracked.txt\n");
        assert_eq!(result.get("untracked.txt").map(|s| s.as_str()), Some("U"));
    }

    #[test]
    fn test_parse_porcelain_renamed() {
        let result = parse_porcelain("R  old.txt -> new.txt\n");
        assert_eq!(result.get("new.txt").map(|s| s.as_str()), Some("M"));
        assert!(result.get("old.txt").is_none());
    }

    #[test]
    fn test_parse_porcelain_multiple() {
        let output = " M file1.rs\nA  file2.rs\n?? file3.rs\n D file4.rs\n";
        let result = parse_porcelain(output);
        assert_eq!(result.len(), 4);
        assert_eq!(result["file1.rs"], "M");
        assert_eq!(result["file2.rs"], "A");
        assert_eq!(result["file3.rs"], "U");
        assert_eq!(result["file4.rs"], "D");
    }

    #[test]
    fn test_parse_porcelain_short_line_ignored() {
        let result = parse_porcelain("ab\n");
        assert!(result.is_empty());
    }
}
