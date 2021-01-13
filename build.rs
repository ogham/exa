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

extern crate datetime;
use std::env;
use std::io;


fn git_hash() -> String {
    use std::process::Command;

    String::from_utf8_lossy(
        &Command::new("git")
            .args(&["rev-parse", "--short", "HEAD"])
            .output().unwrap()
            .stdout).trim().to_string()
}

fn main() {
    write_statics().unwrap();
}

fn is_development_version() -> bool {
    // Both weekly releases and actual releases are --release releases,
    // but actual releases will have a proper version number
    cargo_version().ends_with("-pre") || env::var("PROFILE").unwrap() == "debug"
}

fn cargo_version() -> String {
    env::var("CARGO_PKG_VERSION").unwrap()
}

fn build_date() -> String {
    use datetime::{LocalDateTime, ISO};

    let now = LocalDateTime::now();
    format!("{}", now.date().iso())
}

fn write_statics() -> io::Result<()> {
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    let ver = if is_development_version() {
        format!("exa v{} ({} built on {})", cargo_version(), git_hash(), build_date())
    }
    else {
        format!("exa v{}", cargo_version())
    };

    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let mut f = File::create(&out.join("version_string.txt"))?;
    write!(f, "{:?}", ver)
}
