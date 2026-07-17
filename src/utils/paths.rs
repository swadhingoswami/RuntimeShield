use std::path::PathBuf;

pub fn executable_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
}

pub fn manifest_path(name: &str) -> Option<PathBuf> {
    executable_dir().map(|dir| dir.join(format!("{}.manifest.json", name)))
}

pub fn default_policy_path() -> Option<PathBuf> {
    executable_dir().map(|dir| dir.join("runtime_policy.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executable_dir() {
        let dir = executable_dir();
        assert!(dir.is_some());
        assert!(dir.unwrap().exists());
    }

    #[test]
    fn test_manifest_path() {
        let path = manifest_path("test");
        assert!(path.is_some());
        assert!(path
            .unwrap()
            .to_string_lossy()
            .ends_with("test.manifest.json"));
    }
}
