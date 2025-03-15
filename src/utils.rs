use std::path::PathBuf;

pub fn strip_quotes(text: &str) -> String {
    text.trim()
        .strip_prefix("'")
        .unwrap()
        .strip_suffix("'")
        .unwrap()
        .to_owned()
}

pub fn compile_path(dir: &str, file_name: &str) -> PathBuf {
    let dir = strip_quotes(dir);
    let file_name = strip_quotes(file_name);

    let mut path = PathBuf::new();
    path.push(dir);
    path.push(file_name);
    path
}
