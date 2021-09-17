/// The version string isn’t the simplest: we want to show the version,
/// current Git hash, and compilation date when building *debug* versions, but
/// just the version for *release* versions so the builds are reproducible.
///
/// This script generates the string from the environment variables that Cargo
/// adds (http://doc.crates.io/environment-variables.html) and runs `git` to
/// get the SHA1 hash. It then writes the string into a file, which exa then
/// includes at build-time.
///
/// - https://stackoverflow.com/q/43753491/3484614
/// - https://crates.io/crates/vergen

use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

use datetime::{LocalDateTime, ISO};


/// The build script entry point.
fn main() -> io::Result<()> {
    #![allow(clippy::write_with_newline)]

    let tagline = "exa - list files on the command-line";
    let url     = "https://the.exa.website/";

    let ver =
        if is_debug_build() {
            format!("{}\nv{} \\1;31m(pre-release debug build!)\\0m\n\\1;4;34m{}\\0m", tagline, version_string(), url)
        }
        else if is_development_version() {
            format!("{}\nv{} [{}] built on {} \\1;31m(pre-release!)\\0m\n\\1;4;34m{}\\0m", tagline, version_string(), git_hash(), build_date(), url)
        }
        else {
            format!("{}\nv{}\n\\1;4;34m{}\\0m", tagline, version_string(), url)
        };

    // We need to create these files in the Cargo output directory.
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let path = &out.join("version_string.txt");

    // Bland version text
    let mut f = File::create(path).expect(&path.to_string_lossy());
    writeln!(f, "{}", strip_codes(&ver))?;

    Ok(())
}

/// Removes escape codes from a string.
fn strip_codes(input: &str) -> String {
    input.replace("\\0m", "")
         .replace("\\1;31m", "")
         .replace("\\1;4;34m", "")
}

/// Retrieve the project’s current Git hash, as a string.
fn git_hash() -> String {
    use std::process::Command;

    String::from_utf8_lossy(
        &Command::new("git")
            .args(&["rev-parse", "--short", "HEAD"])
            .output().unwrap()
            .stdout).trim().to_string()
}

/// Whether we should show pre-release info in the version string.
///
/// Both weekly releases and actual releases are --release releases,
/// but actual releases will have a proper version number.
fn is_development_version() -> bool {
    cargo_version().ends_with("-pre") || env::var("PROFILE").unwrap() == "debug"
}

/// Whether we are building in debug mode.
fn is_debug_build() -> bool {
    env::var("PROFILE").unwrap() == "debug"
}

/// Retrieves the [package] version in Cargo.toml as a string.
fn cargo_version() -> String {
    env::var("CARGO_PKG_VERSION").unwrap()
}

/// Returns the version and build parameters string.
fn version_string() -> String {
    let mut ver = cargo_version();

    let feats = nonstandard_features_string();
    if ! feats.is_empty() {
        ver.push_str(&format!(" [{}]", &feats));
    }

    ver
}

/// Finds whether a feature is enabled by examining the Cargo variable.
fn feature_enabled(name: &str) -> bool {
    env::var(&format!("CARGO_FEATURE_{}", name))
        .map(|e| ! e.is_empty())
        .unwrap_or(false)
}

/// A comma-separated list of non-standard feature choices.
fn nonstandard_features_string() -> String {
    let mut s = Vec::new();

    if feature_enabled("GIT") {
        s.push("+git");
    }
    else {
        s.push("-git");
    }

    s.join(", ")
}

/// Formats the current date as an ISO 8601 string.
fn build_date() -> String {
    let now = LocalDateTime::now();
    format!("{}", now.date().iso())
}
