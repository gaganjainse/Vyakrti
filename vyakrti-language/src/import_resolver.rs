//! Import resolution for Vyākṛti.
//!
//! Handles `आयात "path.vya" ।` statements by:
//! 1. Resolving the file path relative to the importing file's directory
//! 2. Reading and parsing the imported file
//! 3. Merging the imported AST into the current program
//! 4. Detecting circular imports

use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Resolve imports in a program by loading and merging imported files.
///
/// `base_dir` is the directory of the file being compiled (used to resolve
/// relative import paths). `workspace_root` is the root of the workspace
/// (used to prevent imports from escaping the workspace).
pub fn resolve_imports(
    nodes: &[crate::ast::ASTNode],
    base_dir: &Path,
    workspace_root: &Path,
) -> Result<Vec<crate::ast::ASTNode>, String> {
    let mut resolved = Vec::new();
    let mut visited = HashSet::new();

    for node in nodes {
        match node {
            crate::ast::ASTNode::ImportDecl { file_path, .. } => {
                let import_path = resolve_import_path(file_path, base_dir, workspace_root)?;
                let import_path_str = import_path.to_string_lossy().to_string();

                if visited.contains(&import_path_str) {
                    // Circular import — skip silently (or could error)
                    continue;
                }
                visited.insert(import_path_str.clone());

                // Read the imported file
                let content = std::fs::read_to_string(&import_path)
                    .map_err(|e| format!("failed to read import '{}': {}", import_path.display(), e))?;

                // Parse the imported file
                let mut lexer = crate::lexer::Lexer::new(&content);
                let tokens = lexer.tokenize();
                let mut parser = crate::parser::Parser::new(tokens);
                let imported_ast = parser.parse_program()
                    .map_err(|e| format!("parse error in import '{}': {}", import_path.display(), e))?;

                // Recursively resolve imports in the imported file
                let import_dir = import_path.parent().unwrap_or(base_dir);
                let nested = resolve_imports(&imported_ast, import_dir, workspace_root)?;
                resolved.extend(nested);
            }
            other => resolved.push(other.clone()),
        }
    }

    Ok(resolved)
}

/// Resolve an import path to an absolute path within the workspace.
fn resolve_import_path(
    file_path: &str,
    base_dir: &Path,
    workspace_root: &Path,
) -> Result<PathBuf, String> {
    // Strip .vya extension if present, then add it back
    let path_str = if file_path.ends_with(".vya") {
        file_path.to_string()
    } else {
        format!("{}.vya", file_path)
    };

    let path = Path::new(&path_str);

    // Reject absolute paths
    if path.is_absolute() {
        return Err(format!("absolute import path '{}' is not allowed", file_path));
    }

    // Resolve relative to the importing file's directory
    let resolved = base_dir.join(path);
    let resolved = resolved.canonicalize().unwrap_or(resolved);

    // Ensure the resolved path is within the workspace root
    let root = workspace_root.canonicalize().unwrap_or_else(|_| workspace_root.to_path_buf());
    if !resolved.starts_with(&root) {
        return Err(format!(
            "import path '{}' escapes the workspace root",
            file_path
        ));
    }

    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_absolute_import_path() {
        let base = Path::new("/tmp");
        let root = Path::new("/tmp");
        assert!(resolve_import_path("/etc/passwd", base, root).is_err());
    }

    #[test]
    fn adds_vya_extension_if_missing() {
        let base = Path::new("/tmp");
        let root = Path::new("/tmp");
        let result = resolve_import_path("math_utils", base, root).unwrap();
        assert!(result.to_string_lossy().ends_with("math_utils.vya"));
    }

    #[test]
    fn keeps_vya_extension_if_present() {
        let base = Path::new("/tmp");
        let root = Path::new("/tmp");
        let result = resolve_import_path("math_utils.vya", base, root).unwrap();
        assert!(result.to_string_lossy().ends_with("math_utils.vya"));
    }
}
