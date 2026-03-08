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
pub fn git_status(path: String) -> Result<GitStatus, String> {
    let dir = Path::new(&path);
    let git_root = find_git_root(dir).ok_or_else(|| "Not a git repository".to_string())?;

    let output = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .arg("-uall")
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
