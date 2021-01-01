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


fn main() -> shadow_rs::SdResult<()>{
    shadow_rs::new()
}