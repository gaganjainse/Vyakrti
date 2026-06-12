use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Debug, Deserialize)]
pub struct FilePathRequest {
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct WriteFileRequest {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceFile {
    pub path: String,
    pub size: u64,
}

#[derive(Debug, Serialize)]
pub struct ReadFileResponse {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct WriteFileResponse {
    pub path: String,
    pub saved: bool,
}

#[derive(Debug, Serialize)]
pub struct SearchMatch {
    pub path: String,
    pub line: usize,
    pub column: usize,
    pub preview: String,
}

fn workspace_root() -> PathBuf {
    std::env::var("VYAKRTI_WORKSPACE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

fn normalize_relative(path: &str) -> Result<PathBuf, String> {
    let raw = Path::new(path);
    if raw.is_absolute() {
        return Err("absolute paths are not allowed".into());
    }
    let mut clean = PathBuf::new();
    for part in raw.components() {
        match part {
            std::path::Component::Normal(p) => clean.push(p),
            std::path::Component::CurDir => {}
            _ => return Err("path must stay inside the workspace".into()),
        }
    }
    if clean.as_os_str().is_empty() {
        return Err("path is empty".into());
    }
    Ok(clean)
}

fn resolve_workspace_path(path: &str) -> Result<PathBuf, String> {
    let root = workspace_root();
    Ok(root.join(normalize_relative(path)?))
}

pub async fn list_files() -> Result<Vec<WorkspaceFile>, String> {
    let root = workspace_root();
    let mut stack = vec![root.clone()];
    let mut files = Vec::new();

    while let Some(dir) = stack.pop() {
        let mut entries = fs::read_dir(&dir).await.map_err(|e| e.to_string())?;
        while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
            let path = entry.path();
            let metadata = entry.metadata().await.map_err(|e| e.to_string())?;
            if metadata.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name != "target" && name != "node_modules" && name != ".git" && name != "dist" {
                    stack.push(path);
                }
            } else if metadata.is_file() {
                let rel = path.strip_prefix(&root).unwrap_or(&path).to_string_lossy().replace('\\', "/");
                if rel.ends_with(".vya") {
                    files.push(WorkspaceFile { path: rel, size: metadata.len() });
                }
            }
        }
    }

    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(files)
}

pub async fn read_file(path: &str) -> Result<ReadFileResponse, String> {
    let full_path = resolve_workspace_path(path)?;
    let content = fs::read_to_string(&full_path).await.map_err(|e| e.to_string())?;
    Ok(ReadFileResponse { path: path.to_string(), content })
}

pub async fn write_file(path: &str, content: &str) -> Result<WriteFileResponse, String> {
    let full_path = resolve_workspace_path(path)?;
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent).await.map_err(|e| e.to_string())?;
    }
    fs::write(&full_path, content).await.map_err(|e| e.to_string())?;
    Ok(WriteFileResponse { path: path.to_string(), saved: true })
}

pub async fn search(query: &str) -> Result<Vec<SearchMatch>, String> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }
    let mut matches = Vec::new();
    for file in list_files().await? {
        let content = match read_file(&file.path).await {
            Ok(f) => f.content,
            Err(_) => continue,
        };
        for (line_idx, line) in content.lines().enumerate() {
            if let Some(col) = line.find(query) {
                matches.push(SearchMatch {
                    path: file.path.clone(),
                    line: line_idx + 1,
                    column: col + 1,
                    preview: line.trim().to_string(),
                });
            }
        }
    }
    Ok(matches)
}

#[cfg(test)]
mod tests {
    use super::normalize_relative;

    #[test]
    fn rejects_paths_outside_workspace() {
        assert!(normalize_relative("../secret.vya").is_err());
        assert!(normalize_relative("..\\secret.vya").is_err());
        assert!(normalize_relative("C:\\secret.vya").is_err());
    }

    #[test]
    fn accepts_nested_relative_vya_path() {
        let path = normalize_relative("src/main.vya").expect("relative path should be accepted");
        assert_eq!(path.to_string_lossy().replace('\\', "/"), "src/main.vya");
    }
}
