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
        ("BUILD_URL_JS", "templates/script.js", "/script-{hash}.js"),
        (
            "BUILD_URL_CSS",
            "templates/styles.css",
            "/styles-{hash}.css"
        ),
        (
            "BUILD_URL_ICON",
            "templates/favicon.png",
            "/favicon-{hash}.png"
        ),
        ("BUILD_URL_BG", "templates/bg.webp", "/bg-{hash}.webp"),
        ("BUILD_VERSION", "", env!("CARGO_PKG_VERSION")),
    ];

    //**** HASHING FOR URL CACHE-BUSTING ****//
    let names: Vec<(String, String, String)> = names
        .iter()
        .map(|(env_var, file, url)| {
            if file.is_empty() {
                return (env_var.to_string(), file.to_string(), url.to_string());
            }

            if let Some(hash) = {
                let file_path = Path::new(file);
                if !file_path.exists() {
                    println!("cargo:warning=[ERROR] File not found: {file}");
                    std::process::exit(1);
                } else {
                    compute_file_hash(file_path).ok()
                }
            } {
                let url = url.replace("{hash}", &hash);
                (env_var.to_string(), file.to_string(), url)
            } else {
                (env_var.to_string(), file.to_string(), url.to_string())
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
        // pass the file path and URL to the build script
        println!("cargo:rustc-env={env_var}={url}");
        println!("cargo:rerun-if-changed={file}");
    }

    //**** REPLACE CONSTANTS IN HTML/CSS FILES ****//
    let replace_files = ["styles.css", "index.html"];

    for file in &replace_files {
        let file_path_in = format!("{}/templates/{}", env!("CARGO_MANIFEST_DIR"), file);
        let file_path_out = format!("{}/target/user_dir/{}", env!("CARGO_MANIFEST_DIR"), file);

        // Replace constants in the file
        if let Err(e) = replace_constants_in_file(file_path_in.clone(), file_path_out, &names) {
            println!("cargo:warning=[ERROR] Failed to replace constants in {file_path_in}: {e}");
            std::process::exit(1);
        }
    }
}

fn replace_constants_in_file<P: AsRef<Path>>(
    file_path_in: P,
    file_path_out: P,
    names: &[(String, String, String)],
) -> Result<(), std::io::Error> {
    let contents = fs::read_to_string(file_path_in)?;
    let mut new_contents = contents;

    for (env_var, _, url) in names {
        new_contents = new_contents.replace(&format!("{{{{{env_var}}}}}"), url);
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
