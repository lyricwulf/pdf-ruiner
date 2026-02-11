use anyhow::Result;

// List files recursively in a directory with a specific extension
pub fn list_files(dir: &str, extension: &str) -> Result<Vec<String>> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file()
            && let Some(ext) = path.extension()
            && ext == extension
        {
            files.push(path.to_str().unwrap().to_string());
        } else if path.is_dir() {
            files.extend(list_files(path.to_str().unwrap(), extension)?);
        }
    }
    Ok(files)
}
