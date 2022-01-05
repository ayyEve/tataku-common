
pub fn exists<P: AsRef<std::path::Path>>(path: P) -> bool {
    path.as_ref().exists()
}