use std::path::{Path, PathBuf};

pub fn find_project_root(start: &Path) -> PathBuf {
    let start_dir = if start.is_file() {
        start.parent().unwrap_or(start)
    } else {
        start
    };
    if let Some(project_root) = find_murali_project_root(start_dir) {
        return project_root;
    }

    // A project can omit murali.toml and use engine defaults. Keep a useful
    // project root for relative resources in that case, without making either
    // the Rust or Python package manifest part of Murali's config contract.
    let mut current = Some(start_dir);
    while let Some(dir) = current {
        if dir.join("pyproject.toml").is_file() || dir.join("Cargo.toml").is_file() {
            return dir.to_path_buf();
        }
        current = dir.parent();
    }

    start_dir.to_path_buf()
}

pub fn find_murali_project_root(start: &Path) -> Option<PathBuf> {
    let start_dir = if start.is_file() {
        start.parent().unwrap_or(start)
    } else {
        start
    };
    start_dir
        .ancestors()
        .find(|dir| dir.join("murali.toml").is_file())
        .map(Path::to_path_buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_project(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after the Unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("murali-{name}-{}-{nonce}", std::process::id()))
    }

    #[test]
    fn nearest_murali_toml_defines_the_project_root() {
        let root = temp_project("config-root");
        let scene_dir = root.join("scenes").join("nested");
        fs::create_dir_all(&scene_dir).unwrap();
        fs::write(root.join("murali.toml"), "[preview]\nfps = 60\n").unwrap();
        fs::write(root.join("pyproject.toml"), "[project]\nname = \"demo\"\n").unwrap();

        assert_eq!(find_project_root(&scene_dir), root);

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn accepts_a_python_script_as_the_search_start() {
        let root = temp_project("script-start");
        let scene_dir = root.join("scenes");
        let script = scene_dir.join("intro.py");
        fs::create_dir_all(&scene_dir).unwrap();
        fs::write(root.join("murali.toml"), "").unwrap();
        fs::write(&script, "# scene\n").unwrap();

        assert_eq!(find_project_root(&script), root);

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn package_manifest_is_only_a_fallback_when_config_is_absent() {
        let root = temp_project("manifest-fallback");
        let scene_dir = root.join("scenes");
        fs::create_dir_all(&scene_dir).unwrap();
        fs::write(root.join("pyproject.toml"), "[project]\nname = \"demo\"\n").unwrap();

        assert_eq!(find_project_root(&scene_dir), root);

        fs::remove_dir_all(&root).unwrap();
    }
}
