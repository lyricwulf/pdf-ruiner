use anyhow::Result;

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
        }
    }
    Ok(files)
}
