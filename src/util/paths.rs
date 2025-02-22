use std::path::{Component, Path, PathBuf};

pub fn normalize_path(path: &Path) -> String {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                result.pop();
            }
            Component::Normal(name) => {
                result.push(name);
            }
            Component::RootDir => {
                result.clear();
                result.push(Component::RootDir.as_os_str());
            }
            Component::Prefix(prefix) => {
                result.push(prefix.as_os_str());
            }
        }
    }

    result
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/")
}

mod tests {
    #[test]
    fn test_normalize_path() {
        use std::path::PathBuf;
        let path = PathBuf::from("a\\b/c");
        assert_eq!(super::normalize_path(&path), "a/b/c");
    }
}
