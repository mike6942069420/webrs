use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

macro_rules! names_vec {
    ( $( ($a:expr, $b:expr, $c:expr) ),* $(,)? ) => {
        vec![
            $( ($a.into(), $b.into(), $c.into()) ),*
        ]
    };
}

const HASH_LENGTH: usize = 8;

fn main() {
    // Associate env_name with path name and URL
    let names: Vec<(String, String, String)> = names_vec![
        ("BUILD_URL_JS", "templates/script.js", "/{hash}.js"),
        ("BUILD_URL_CSS", "templates/styles.css", "/{hash}.css"),
        ("BUILD_URL_ICON", "templates/favicon.png", "/{hash}.png"),
        ("BUILD_URL_BG", "templates/bg.webp", "/{hash}.webp"),
        ("BUILD_VERSION", "", env!("CARGO_PKG_VERSION")),
    ];
    let replace_files = ["index.html", "styles.css"];

    //**** HASHING FOR URL CACHE-BUSTING ****//
    let names: Vec<(String, String, String)> = names
        .iter()
        .map(|(env_var, file, url)| {
            if file.is_empty() {
                return (env_var.to_string(), file.to_string(), url.to_string());
            }

            let file_path = Path::new(file);
            if !file_path.exists() {
                println!("cargo:warning=[ERROR] File not found: {file}");
                std::process::exit(1);
            }

            match compute_file_hash(file_path) {
                Ok(hash) => {
                    let new_url = url.replace("{hash}", &hash);
                    (env_var.to_string(), file.to_string(), new_url)
                }
                Err(e) => {
                    println!("cargo:warning=[ERROR] Failed to hash {file}: {e}");
                    std::process::exit(1);
                }
            }
        })
        .collect();

    // Pass env variables to the build script
    for (env_var, file, url) in &names {
        if file.is_empty() {
            println!("cargo:warning=[INFO] {env_var} -> {url}");
            continue;
        }

        println!("cargo:warning=[INFO] {env_var}: ({file}) -> {url}");
        println!("cargo:rustc-env={env_var}={url}");
        println!("cargo:rerun-if-changed={file}");
    }

    //**** REPLACE CONSTANTS IN HTML/CSS FILES ****//
    let input_base = Path::new("templates");
    let output_base = Path::new("target").join("user_dir");

    for file in &replace_files {
        let file_path_in = input_base.join(file);
        let file_path_out = output_base.join(file);

        println!("cargo:rerun-if-changed={}", file_path_in.display());

        match replace_constants_in_file(&file_path_in, &file_path_out, &names) {
            Ok(_) => {
                println!(
                    "cargo:warning=[INFO] Successfully replaced constants in {}",
                    file_path_in.display()
                );
            }
            Err(e) => {
                println!(
                    "cargo:warning=[ERROR] Failed to replace constants in {}: {}",
                    file_path_in.display(),
                    e
                );
                std::process::exit(1);
            }
        }
    }
}

fn replace_constants_in_file(
    file_path_in: &Path,
    file_path_out: &Path,
    names: &[(String, String, String)],
) -> Result<(), std::io::Error> {
    let contents = fs::read_to_string(file_path_in)?;
    let mut new_contents = contents;

    for (env_var, _, url) in names {
        new_contents = new_contents.replace(&format!("{{{{{env_var}}}}}"), url);
    }

    if let Some(parent) = file_path_out.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(file_path_out, new_contents)?;
    Ok(())
}

/// Computes the SHA-256 hash of a file and returns the first 8 characters of the hex digest.
fn compute_file_hash<P: AsRef<Path>>(path: P) -> Result<String, std::io::Error> {
    let bytes = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let result = hasher.finalize();
    Ok(format!("{result:x}")[..HASH_LENGTH].to_string())
}
