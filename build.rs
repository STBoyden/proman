use std::{env, fs, fs::File, io::Write, path::Path};

fn main() {
    let file_paths = fs::read_dir("default-plugins")
        .unwrap()
        .filter_map(|e| e.ok())
        .collect::<Vec<_>>();
    let out_dir = env::var("OUT_DIR").unwrap();
    let mut plugins_file = File::create(Path::new(&out_dir).join("plugins.include")).unwrap();

    _ = plugins_file.write_fmt(format_args!(
        r#"/// Generated at compile time, contains the raw bytes of the files in the repository in
/// `default-plugins/`.
pub const DEFAULT_PLUGINS_BYTES: [&[u8]; {}] = [
"#,
        file_paths.len()
    ));

    for path in file_paths {
        let path = path.path();

        let contents = fs::read(&path)
            .map_err(|error| Err::<(), String>(format!("could read file at \"{path:?}\": {error}")))
            .unwrap();

        plugins_file
            .write_fmt(format_args!("&{contents:?},\n"))
            .unwrap()
    }

    _ = plugins_file.write("];".as_bytes()).unwrap();
}
