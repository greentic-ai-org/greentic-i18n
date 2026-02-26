use std::path::{Path, PathBuf};

pub fn default_i18n_dir(repo_root: &Path) -> PathBuf {
    repo_root.join("i18n")
}

pub fn en_json(repo_root: &Path) -> PathBuf {
    default_i18n_dir(repo_root).join("en.json")
}

pub fn lang_json(repo_root: &Path, lang: &str) -> PathBuf {
    default_i18n_dir(repo_root).join(format!("{lang}.json"))
}
