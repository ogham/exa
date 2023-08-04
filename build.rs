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
    create_version_string_file()?;
    create_file_typing_hash_file()?;
    create_file_icon_hash_file()
}
    

/// Create the version_string.txt file
fn create_version_string_file() -> io::Result<()> {
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
    let mut f = File::create(path).unwrap_or_else(|_| { panic!("{}", path.to_string_lossy().to_string()) });
    writeln!(f, "{}", strip_codes(&ver))
}

/// Create the perfect hashing for file typing
fn create_file_typing_hash_file() -> io::Result<()> {
    let path = &PathBuf::from(env::var("OUT_DIR").unwrap()).join("filetype_maps.rs");
    let mut file = io::BufWriter::new(File::create(path).unwrap_or_else(|_| { panic!("{}", path.to_string_lossy().to_string()) }));
    generate_extension_type_map(file.get_mut())?;
    generate_filename_type_map(file.get_mut())?;
    file.flush()
}

/// Create the perfect hashing for file icons
fn create_file_icon_hash_file() -> io::Result<()> {
    let path = &PathBuf::from(env::var("OUT_DIR").unwrap()).join("icon_maps.rs");
    let mut file = io::BufWriter::new(File::create(path).unwrap_or_else(|_| { panic!("{}", path.to_string_lossy().to_string()) }));
    generate_extension_icon_map(file.get_mut())?;
    generate_filename_icon_map(file.get_mut())?;
    file.flush()
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
        #[allow(clippy::format_push_string)]
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

/// Generate mapping from lowercase file extension to file type.  If an image, video, music, or
/// lossless extension is added also update the extension icon map. For file types see
/// info/filetype.rs
fn generate_extension_type_map(file: &mut File) -> io::Result<()> {
    // Extension are converted to lower case for comparison
    writeln!(file, "static EXTENSION_TYPES: phf::Map<&'static str, FileType> = {};\n",
             phf_codegen::Map::new()
                 /* Image files */
                 .entry("arw",       "FileType::Image")
                 .entry("avif",      "FileType::Image")
                 .entry("bmp",       "FileType::Image")
                 .entry("cbr",       "FileType::Image")
                 .entry("cbz",       "FileType::Image")
                 .entry("cr2",       "FileType::Image")
                 .entry("dvi",       "FileType::Image")
                 .entry("eps",       "FileType::Image")
                 .entry("gif",       "FileType::Image")
                 .entry("heif",      "FileType::Image")
                 .entry("ico",       "FileType::Image")
                 .entry("j2c",       "FileType::Image")
                 .entry("j2k",       "FileType::Image")
                 .entry("jfi",       "FileType::Image")
                 .entry("jfif",      "FileType::Image")
                 .entry("jif",       "FileType::Image")
                 .entry("jp2",       "FileType::Image")
                 .entry("jpe",       "FileType::Image")
                 .entry("jpeg",      "FileType::Image")
                 .entry("jpf",       "FileType::Image")
                 .entry("jpg",       "FileType::Image")
                 .entry("jpx",       "FileType::Image")
                 .entry("jxl",       "FileType::Image")
                 .entry("nef",       "FileType::Image")
                 .entry("orf",       "FileType::Image")
                 .entry("pbm",       "FileType::Image")
                 .entry("pgm",       "FileType::Image")
                 .entry("png",       "FileType::Image")
                 .entry("pnm",       "FileType::Image")
                 .entry("ppm",       "FileType::Image")
                 .entry("ps",        "FileType::Image")
                 .entry("psd",       "FileType::Image")
                 .entry("pxm",       "FileType::Image")
                 .entry("raw",       "FileType::Image")
                 .entry("stl",       "FileType::Image")
                 .entry("svg",       "FileType::Image")
                 .entry("tif",       "FileType::Image")
                 .entry("tiff",      "FileType::Image")
                 .entry("webp",      "FileType::Image")
                 .entry("xpm",       "FileType::Image")
                 /* Video files */
                 .entry("avi",       "FileType::Video")
                 .entry("flv",       "FileType::Video")
                 .entry("heic",      "FileType::Video")
                 .entry("m2ts",      "FileType::Video")
                 .entry("m2v",       "FileType::Video")
                 .entry("m4v",       "FileType::Video")
                 .entry("mkv",       "FileType::Video")
                 .entry("mov",       "FileType::Video")
                 .entry("mp4",       "FileType::Video")
                 .entry("mpeg",      "FileType::Video")
                 .entry("mpg",       "FileType::Video")
                 .entry("ogm",       "FileType::Video")
                 .entry("ogv",       "FileType::Video")
                 .entry("video",     "FileType::Video")
                 .entry("vob",       "FileType::Video")
                 .entry("webm",      "FileType::Video")
                 .entry("wmv",       "FileType::Video")
                 /* Music files */
                 .entry("aac",       "FileType::Music")
                 .entry("m4a",       "FileType::Music")
                 .entry("mka",       "FileType::Music")
                 .entry("mp3",       "FileType::Music")
                 .entry("ogg",       "FileType::Music")
                 .entry("opus",      "FileType::Music")
                 .entry("wma",       "FileType::Music")
                 /* Lossless music, rather than any other kind of data... */
                 .entry("alac",      "FileType::Lossless")
                 .entry("ape",       "FileType::Lossless")
                 .entry("flac",      "FileType::Lossless")
                 .entry("wav",       "FileType::Lossless")
                 /* Cryptology files */
                 .entry("asc",       "FileType::Crypto")
                 .entry("enc",       "FileType::Crypto")
                 .entry("gpg",       "FileType::Crypto")
                 .entry("p12",       "FileType::Crypto")
                 .entry("pfx",       "FileType::Crypto")
                 .entry("pgp",       "FileType::Crypto")
                 .entry("sig",       "FileType::Crypto")
                 .entry("signature", "FileType::Crypto")
                 /* Document files */
                 .entry("djvu",      "FileType::Document")
                 .entry("doc",       "FileType::Document")
                 .entry("docx",      "FileType::Document")
                 // .entry("dvi", "FileType::Document")  # Duplicate extension from FileType::Image
                 .entry("eml",       "FileType::Document")
                 // .entry("eps", "FileType::Document")  # Duplicate extension from FileType::Image
                 .entry("fotd",      "FileType::Document")
                 .entry("key",       "FileType::Document")
                 .entry("keynote",   "FileType::Document")
                 .entry("numbers",   "FileType::Document")
                 .entry("odp",       "FileType::Document")
                 .entry("odt",       "FileType::Document")
                 .entry("pages",     "FileType::Document")
                 .entry("pdf",       "FileType::Document")
                 .entry("ppt",       "FileType::Document")
                 .entry("pptx",      "FileType::Document")
                 .entry("rtf",       "FileType::Document")
                 .entry("xls",       "FileType::Document")
                 .entry("xlsx",      "FileType::Document")
                 /* Compressed/archive files */
                 .entry("7z",        "FileType::Compressed")
                 .entry("a",         "FileType::Compressed")
                 .entry("ar",        "FileType::Compressed")
                 .entry("bz",        "FileType::Compressed")
                 .entry("bz2",       "FileType::Compressed")
                 .entry("cpio",      "FileType::Compressed")
                 .entry("deb",       "FileType::Compressed")
                 .entry("dmg",       "FileType::Compressed")
                 .entry("gz",        "FileType::Compressed")
                 .entry("iso",       "FileType::Compressed")
                 .entry("lz",        "FileType::Compressed")
                 .entry("lz4",       "FileType::Compressed")
                 .entry("lzh",       "FileType::Compressed")
                 .entry("lzma",      "FileType::Compressed")
                 .entry("rar",       "FileType::Compressed")
                 .entry("rpm",       "FileType::Compressed")
                 .entry("tar",       "FileType::Compressed")
                 .entry("taz",       "FileType::Compressed")
                 .entry("tbz",       "FileType::Compressed")
                 .entry("tbz2",      "FileType::Compressed")
                 .entry("tc",        "FileType::Compressed")
                 .entry("tgz",       "FileType::Compressed")
                 .entry("tlz",       "FileType::Compressed")
                 .entry("txz",       "FileType::Compressed")
                 .entry("tz",        "FileType::Compressed")
                 .entry("xz",        "FileType::Compressed")
                 .entry("z",         "FileType::Compressed")
                 .entry("zip",       "FileType::Compressed")
                 .entry("zst",       "FileType::Compressed")
                 /* Temporary files */
                 .entry("bak",       "FileType::Temp")
                 .entry("bk",        "FileType::Temp")
                 .entry("bkp",       "FileType::Temp")
                 .entry("swn",       "FileType::Temp")
                 .entry("swo",       "FileType::Temp")
                 .entry("swp",       "FileType::Temp")
                 .entry("tmp",       "FileType::Temp")
                 /* Compiler output files */
                 .entry("class",     "FileType::Compiled")
                 .entry("elc",       "FileType::Compiled")
                 .entry("hi",        "FileType::Compiled")
                 .entry("ko",        "FileType::Compiled")
                 .entry("o",         "FileType::Compiled")
                 .entry("pyc",       "FileType::Compiled")
                 .entry("pyo",       "FileType::Compiled")
                 .entry("so",        "FileType::Compiled")
                 .entry("zwc",       "FileType::Compiled")
                 /* Immediate file - kick off the build of a project */
                 .entry("ninja",     "FileType::Immediate")
                 .build()
    )
}

/// Generate mapping from full filenames to file type. For file types see info/filetype.rs
fn generate_filename_type_map(file: &mut File) -> io::Result<()> {
    writeln!(file, "static FILENAME_TYPES: phf::Map<&'static str, FileType> = {};\n",
             phf_codegen::Map::new()
                 /* Immediate file - kick off the build of a project */
                 .entry("Brewfile",          "FileType::Immediate")
                 .entry("bsconfig.json",     "FileType::Immediate")
                 .entry("BUILD",             "FileType::Immediate")
                 .entry("BUILD.bazel",       "FileType::Immediate")
                 .entry("build.gradle",      "FileType::Immediate")
                 .entry("build.sbt",         "FileType::Immediate")
                 .entry("build.xml",         "FileType::Immediate")
                 .entry("Cargo.toml",        "FileType::Immediate")
                 .entry("CMakeLists.txt",    "FileType::Immediate")
                 .entry("composer.json",     "FileType::Immediate")
                 .entry("Containerfile",     "FileType::Immediate")
                 .entry("Dockerfile",        "FileType::Immediate")
                 .entry("Gemfile",           "FileType::Immediate")
                 .entry("GNUmakefile",       "FileType::Immediate")
                 .entry("Gruntfile.coffee",  "FileType::Immediate")
                 .entry("Gruntfile.js",      "FileType::Immediate")
                 .entry("Justfile",          "FileType::Immediate")
                 .entry("Makefile",          "FileType::Immediate")
                 .entry("makefile",          "FileType::Immediate")
                 .entry("meson.build",       "FileType::Immediate")
                 .entry("mix.exs",           "FileType::Immediate")
                 .entry("package.json",      "FileType::Immediate")
                 .entry("Pipfile",           "FileType::Immediate")
                 .entry("PKGBUILD",          "FileType::Immediate")
                 .entry("Podfile",           "FileType::Immediate")
                 .entry("pom.xml",           "FileType::Immediate")
                 .entry("Procfile",          "FileType::Immediate")
                 .entry("pyproject.toml",    "FileType::Immediate")
                 .entry("Rakefile",          "FileType::Immediate")
                 .entry("RoboFile.php",      "FileType::Immediate")
                 .entry("SConstruct",        "FileType::Immediate")
                 .entry("tsconfig.json",     "FileType::Immediate")
                 .entry("Vagrantfile",       "FileType::Immediate")
                 .entry("webpack.config.js", "FileType::Immediate")
                 .entry("WORKSPACE",         "FileType::Immediate")
                 .build()
    )
}

/// Generate mapping from lowercase file extension to icons.  If an image, video, or audio
/// extension is add also update the extension filetype map.  See output/render/icons.rs for
/// a partial list of icon constants.
fn generate_extension_icon_map(file: &mut File) -> io::Result<()> {
    writeln!(file, "static EXTENSION_ICONS: phf::Map<&'static str, char> = {};\n",
             phf_codegen::Map::new()
                 .entry("aac",             "Icons::AUDIO") // 
                 .entry("alac",            "Icons::AUDIO") // 
                 .entry("ape",             "Icons::AUDIO") // 
                 .entry("flac",            "Icons::AUDIO") // 
                 .entry("m4a",             "Icons::AUDIO") // 
                 .entry("mka",             "Icons::AUDIO") // 
                 .entry("mp3",             "Icons::AUDIO") // 
                 .entry("ogg",             "Icons::AUDIO") // 
                 .entry("opus",            "Icons::AUDIO") // 
                 .entry("wav",             "Icons::AUDIO") // 
                 .entry("wma",             "Icons::AUDIO") // 
                 .entry("avi",             "Icons::VIDEO") // 
                 .entry("flv",             "Icons::VIDEO") // 
                 .entry("heic",            "Icons::VIDEO") // 
                 .entry("m2ts",            "Icons::VIDEO") // 
                 .entry("m2v",             "Icons::VIDEO") // 
                 .entry("m4v",             "Icons::VIDEO") // 
                 .entry("mkv",             "Icons::VIDEO") // 
                 .entry("mov",             "Icons::VIDEO") // 
                 .entry("mp4",             "Icons::VIDEO") // 
                 .entry("mpeg",            "Icons::VIDEO") // 
                 .entry("mpg",             "Icons::VIDEO") // 
                 .entry("ogm",             "Icons::VIDEO") // 
                 .entry("ogv",             "Icons::VIDEO") // 
                 .entry("video",           "Icons::VIDEO") // 
                 .entry("vob",             "Icons::VIDEO") // 
                 .entry("webm",            "Icons::VIDEO") // 
                 .entry("wmv",             "Icons::VIDEO") // 
                 .entry("arw",             "Icons::IMAGE") // 
                 .entry("avif",            "Icons::IMAGE") // 
                 .entry("bmp",             "Icons::IMAGE") // 
                 .entry("cbr",             "Icons::IMAGE") // 
                 .entry("cbz",             "Icons::IMAGE") // 
                 .entry("cr2",             "Icons::IMAGE") // 
                 .entry("dvi",             "Icons::IMAGE") // 
                 .entry("gif",             "Icons::IMAGE") // 
                 .entry("heif",            "Icons::IMAGE") // 
                 .entry("ico",             "Icons::IMAGE") // 
                 .entry("j2c",             "Icons::IMAGE") // 
                 .entry("j2k",             "Icons::IMAGE") // 
                 .entry("jfi",             "Icons::IMAGE") // 
                 .entry("jfif",            "Icons::IMAGE") // 
                 .entry("jif",             "Icons::IMAGE") // 
                 .entry("jp2",             "Icons::IMAGE") // 
                 .entry("jpe",             "Icons::IMAGE") // 
                 .entry("jpeg",            "Icons::IMAGE") // 
                 .entry("jpf",             "Icons::IMAGE") // 
                 .entry("jpg",             "Icons::IMAGE") // 
                 .entry("jpx",             "Icons::IMAGE") // 
                 .entry("jxl",             "Icons::IMAGE") // 
                 .entry("nef",             "Icons::IMAGE") // 
                 .entry("orf",             "Icons::IMAGE") // 
                 .entry("pbm",             "Icons::IMAGE") // 
                 .entry("pgm",             "Icons::IMAGE") // 
                 .entry("png",             "Icons::IMAGE") // 
                 .entry("pnm",             "Icons::IMAGE") // 
                 .entry("ppm",             "Icons::IMAGE") // 
                 .entry("ps",              "Icons::IMAGE") // 
                 .entry("pxm",             "Icons::IMAGE") // 
                 .entry("raw",             "Icons::IMAGE") // 
                 .entry("stl",             "Icons::IMAGE") // 
                 .entry("tif",             "Icons::IMAGE") // 
                 .entry("tiff",            "Icons::IMAGE") // 
                 .entry("webp",            "Icons::IMAGE") // 
                 .entry("xpm",             "Icons::IMAGE") // 
                 .entry("7z",              "Icons::COMPRESSED") // 
                 .entry("a",               "Icons::COMPRESSED") // 
                 .entry("ar",              "Icons::COMPRESSED") // 
                 .entry("bz",              "Icons::COMPRESSED") // 
                 .entry("bz2",             "Icons::COMPRESSED") // 
                 .entry("cpio",            "Icons::COMPRESSED") // 
                 .entry("gz",              "Icons::COMPRESSED") // 
                 .entry("lz",              "Icons::COMPRESSED") // 
                 .entry("lz4",             "Icons::COMPRESSED") // 
                 .entry("lzh",             "Icons::COMPRESSED") // 
                 .entry("lzma",            "Icons::COMPRESSED") // 
                 .entry("lzo",             "Icons::COMPRESSED") // 
                 .entry("rar",             "Icons::COMPRESSED") // 
                 .entry("tar",             "Icons::COMPRESSED") // 
                 .entry("taz",             "Icons::COMPRESSED") // 
                 .entry("tbz",             "Icons::COMPRESSED") // 
                 .entry("tbz2",            "Icons::COMPRESSED") // 
                 .entry("tgz",             "Icons::COMPRESSED") // 
                 .entry("tlz",             "Icons::COMPRESSED") // 
                 .entry("txz",             "Icons::COMPRESSED") // 
                 .entry("tz",              "Icons::COMPRESSED") // 
                 .entry("tzo",             "Icons::COMPRESSED") // 
                 .entry("xz",              "Icons::COMPRESSED") // 
                 .entry("z",               "Icons::COMPRESSED") // 
                 .entry("zip",             "Icons::COMPRESSED") // 
                 .entry("zst",             "Icons::COMPRESSED") // 
                 .entry("awk",             "Icons::SHELL") // 
                 .entry("bash",            "Icons::SHELL") // 
                 .entry("bash_history",    "Icons::SHELL") // 
                 .entry("bash_login",      "Icons::SHELL") // 
                 .entry("bash_logout",     "Icons::SHELL") // 
                 .entry("bash_profile",    "Icons::SHELL") // 
                 .entry("bashrc",          "Icons::SHELL") // 
                 .entry("bats",            "Icons::SHELL") // 
                 .entry("csh",             "Icons::SHELL") // 
                 .entry("cshrc",           "Icons::SHELL") // 
                 .entry("fish",            "Icons::SHELL") // 
                 .entry("ksh",             "Icons::SHELL") // 
                 .entry("kshrc",           "Icons::SHELL") // 
                 .entry("login",           "Icons::SHELL") // 
                 .entry("logout",          "Icons::SHELL") // 
                 .entry("mk",              "Icons::SHELL") // 
                 .entry("profile",         "Icons::SHELL") // 
                 .entry("ps1",             "Icons::SHELL") // 
                 .entry("sh",              "Icons::SHELL") // 
                 .entry("shell",           "Icons::SHELL") // 
                 .entry("tshrc",           "Icons::SHELL") // 
                 .entry("zlogin",          "Icons::SHELL") // 
                 .entry("zlogout",         "Icons::SHELL") // 
                 .entry("zprofile",        "Icons::SHELL") // 
                 .entry("zsh",             "Icons::SHELL") // 
                 .entry("zsh-theme",       "Icons::SHELL") // 
                 .entry("zshenv",          "Icons::SHELL") // 
                 .entry("zshrc",           "Icons::SHELL") // 
                 .entry("zsh_sessions",    "Icons::SHELL") // 
                 .entry("git",             "Icons::GIT") // 
                 .entry("gitattributes",   "Icons::GIT") // 
                 .entry("gitconfig",       "Icons::GIT") // 
                 .entry("gitignore",       "Icons::GIT") // 
                 .entry("gitignore_global","Icons::GIT") // 
                 .entry("gitmodules",      "Icons::GIT") // 
                 .entry("jmd",             "Icons::MARKDOWN") // 
                 .entry("markdown",        "Icons::MARKDOWN") // 
                 .entry("md",              "Icons::MARKDOWN") // 
                 .entry("mkd",             "Icons::MARKDOWN") // 
                 .entry("rdoc",            "Icons::MARKDOWN") // 
                 .entry("readme",          "Icons::MARKDOWN") // 
                 .entry("rmd",             "Icons::MARKDOWN") // 
                 .entry("dmg",             "Icons::DISK_IMAGE") // 
                 .entry("image",           "Icons::DISK_IMAGE") // 
                 .entry("img",             "Icons::DISK_IMAGE") // 
                 .entry("iso",             "Icons::DISK_IMAGE") // 
                 .entry("tc",              "Icons::DISK_IMAGE") // 
                 .entry("cfg",             "Icons::CONFIG") // 
                 .entry("conf",            "Icons::CONFIG") // 
                 .entry("editorconfig",    "Icons::CONFIG") // 
                 .entry("ini",             "Icons::CONFIG") // 
                 .entry("toml",            "Icons::CONFIG") // 
                 .entry("avro",            "Icons::JSON") // 
                 .entry("json",            "Icons::JSON") // 
                 .entry("properties",      "Icons::JSON") // 
                 .entry("c",               "Icons::C_LANG") // 
                 .entry("h",               "Icons::C_LANG") // 
                 .entry("m",               "Icons::C_LANG") // 
                 .entry("c++",             "Icons::CPP_LANG") // 
                 .entry("cc",              "Icons::CPP_LANG") // 
                 .entry("cp",              "Icons::CPP_LANG") // 
                 .entry("cpp",             "Icons::CPP_LANG") // 
                 .entry("cxx",             "Icons::CPP_LANG") // 
                 .entry("hpp",             "Icons::CPP_LANG") // 
                 .entry("hxx",             "Icons::CPP_LANG") // 
                 .entry("mm",              "Icons::CPP_LANG") // 
                 .entry("cs",              "Icons::CSHARP_LANG") // 󰌛
                 .entry("csproj",          "Icons::CSHARP_LANG") // 󰌛
                 .entry("csx",             "Icons::CSHARP_LANG") // 󰌛
                 .entry("fs",              "Icons::FSHARP_LANG") // 
                 .entry("fsi",             "Icons::FSHARP_LANG") // 
                 .entry("fsx",             "Icons::FSHARP_LANG") // 
                 .entry("go",              "Icons::GO_LANG") // 
                 .entry("class",           "Icons::JAVA_LANG") // 
                 .entry("gradle",          "Icons::JAVA_LANG") // 
                 .entry("jad",             "Icons::JAVA_LANG") // 
                 .entry("jar",             "Icons::JAVA_LANG") // 
                 .entry("java",            "Icons::JAVA_LANG") // 
                 .entry("war",             "Icons::JAVA_LANG") // 
                 .entry("pl",              "Icons::PERL_LANG") // 
                 .entry("plx",             "Icons::PERL_LANG") // 
                 .entry("pm",              "Icons::PERL_LANG") // 
                 .entry("pod",             "Icons::PERL_LANG") // 
                 .entry("t",               "Icons::PERL_LANG") // 
                 .entry("py",              "Icons::PYTHON_LANG") // 
                 .entry("pyc",             "Icons::PYTHON_LANG") // 
                 .entry("pyi",             "Icons::PYTHON_LANG") // 
                 .entry("pyo",             "Icons::PYTHON_LANG") // 
                 .entry("whl",             "Icons::PYTHON_LANG") // 
                 .entry("r",               "Icons::R_LANG") // 
                 .entry("rdata",           "Icons::R_LANG") // 
                 .entry("rds",             "Icons::R_LANG") // 
                 .entry("gem",             "Icons::RUBY_LANG") // 
                 .entry("gemfile",         "Icons::RUBY_LANG") // 
                 .entry("gemspec",         "Icons::RUBY_LANG") // 
                 .entry("guardfile",       "Icons::RUBY_LANG") // 
                 .entry("procfile",        "Icons::RUBY_LANG") // 
                 .entry("rakefile",        "Icons::RUBY_LANG") // 
                 .entry("rb",              "Icons::RUBY_LANG") // 
                 .entry("rspec",           "Icons::RUBY_LANG") // 
                 .entry("rspec_parallel",  "Icons::RUBY_LANG") // 
                 .entry("rspec_status",    "Icons::RUBY_LANG") // 
                 .entry("ru",              "Icons::RUBY_LANG") // 
                 .entry("erb",             "Icons::RUBYRAILS_LANG") // 
                 .entry("rubydoc",         "Icons::RUBYRAILS_LANG") // 
                 .entry("slim",            "Icons::RUBYRAILS_LANG") // 
                 .entry("rlib",            "Icons::RUST_LANG") // 
                 .entry("rs",              "Icons::RUST_LANG") // 
                 .entry("cls",             "Icons::TEX_LANG") // 
                 .entry("latex",           "Icons::TEX_LANG") // 
                 .entry("sty",             "Icons::TEX_LANG") // 
                 .entry("tex",             "Icons::TEX_LANG") // 
                 .entry("apple",           "Icons::APPLE") // 
                 .entry("_ds_store",       "Icons::APPLE") // 
                 .entry("ds_store",        "Icons::APPLE") // 
                 .entry("localized",       "Icons::APPLE") // 
                 .entry("bat",             "Icons::WINDOWS") // 
                 .entry("cab",             "Icons::WINDOWS") // 
                 .entry("cmd",             "Icons::WINDOWS") // 
                 .entry("dll",             "Icons::WINDOWS") // 
                 .entry("exe",             "Icons::WINDOWS") // 
                 .entry("msi",             "Icons::WINDOWS") // 
                 .entry("windows",         "Icons::WINDOWS") // 
                 .entry("htm",             "Icons::HTML5") // 
                 .entry("html",            "Icons::HTML5") // 
                 .entry("xhtml",           "Icons::HTML5") // 
                 .entry("eot",             "Icons::FONT") // 
                 .entry("font",            "Icons::FONT") // 
                 .entry("otf",             "Icons::FONT") // 
                 .entry("ttf",             "Icons::FONT") // 
                 .entry("woff",            "Icons::FONT") // 
                 .entry("woff2",           "Icons::FONT") // 
                 .entry("db",              "Icons::DATABASE") // 
                 .entry("dump",            "Icons::DATABASE") // 
                 .entry("sql",             "Icons::DATABASE") // 
                 .entry("doc",             "Icons::DOCUMENT") // 
                 .entry("docx",            "Icons::DOCUMENT") // 
                 .entry("gdoc",            "Icons::DOCUMENT") // 
                 .entry("odt",             "Icons::DOCUMENT") // 
                 .entry("csv",             "Icons::SHEET") // 
                 .entry("gsheet",          "Icons::SHEET") // 
                 .entry("ods",             "Icons::SHEET") // 
                 .entry("tsv",             "Icons::SHEET") // 
                 .entry("xls",             "Icons::SHEET") // 
                 .entry("xlsx",            "Icons::SHEET") // 
                 .entry("gslides",         "Icons::SLIDE") // 
                 .entry("odp",             "Icons::SLIDE") // 
                 .entry("ppt",             "Icons::SLIDE") // 
                 .entry("pptx",            "Icons::SLIDE") // 
                 .entry("ai",              "'\u{e7b4}'")  // 
                 .entry("android",         "'\u{e70e}'")  // 
                 .entry("apk",             "'\u{e70e}'")  // 
                 .entry("clj",             "'\u{e768}'")  // 
                 .entry("cljs",            "'\u{e76a}'")  // 
                 .entry("coffee",          "'\u{f0f4}'")  // 
                 .entry("cshtml",          "'\u{f1fa}'")  // 
                 .entry("css",             "'\u{e749}'")  // 
                 .entry("d",               "'\u{e7af}'")  // 
                 .entry("dart",            "'\u{e798}'")  // 
                 .entry("deb",             "'\u{e77d}'")  // 
                 .entry("diff",            "'\u{f440}'")  // 
                 .entry("djvu",            "'\u{f02d}'")  // 
                 .entry("ebook",           "'\u{e28b}'")  // 
                 .entry("ebuild",          "'\u{f30d}'")  // 
                 .entry("ejs",             "'\u{e618}'")  // 
                 .entry("elm",             "'\u{e62c}'")  // 
                 .entry("env",             "'\u{f462}'")  // 
                 .entry("eps",             "'\u{f0559}'") // 󰕙
                 .entry("epub",            "'\u{e28a}'")  // 
                 .entry("erl",             "'\u{e7b1}'")  // 
                 .entry("ex",              "'\u{e62d}'")  // 
                 .entry("exs",             "'\u{e62d}'")  // 
                 .entry("gform",           "'\u{f298}'")  // 
                 .entry("groovy",          "'\u{e775}'")  // 
                 .entry("hbs",             "'\u{e60f}'")  // 
                 .entry("hs",              "'\u{e777}'")  // 
                 .entry("ideavimrc",       "'\u{e62b}'")  // 
                 .entry("iml",             "'\u{e7b5}'")  // 
                 .entry("ipynb",           "'\u{e678}'")  // 
                 .entry("jl",              "'\u{e624}'")  // 
                 .entry("js",              "'\u{e74e}'")  // 
                 .entry("jsx",             "'\u{e7ba}'")  // 
                 .entry("less",            "'\u{e758}'")  // 
                 .entry("lhs",             "'\u{e777}'")  // 
                 .entry("license",         "'\u{f0219}'") // 󰈙
                 .entry("lock",            "'\u{f023}'")  // 
                 .entry("log",             "'\u{f18d}'")  // 
                 .entry("lua",             "'\u{e620}'")  // 
                 .entry("mjs",             "'\u{e74e}'")  // 
                 .entry("mobi",            "'\u{e28b}'")  // 
                 .entry("mustache",        "'\u{e60f}'")  // 
                 .entry("nix",             "'\u{f313}'")  // 
                 .entry("node",            "'\u{f0399}'") // 󰎙
                 .entry("npmignore",       "'\u{e71e}'")  // 
                 .entry("npmrc",           "'\u{e71e}'")  // 
                 .entry("part",            "'\u{f43a}'")  // 
                 .entry("patch",           "'\u{f440}'")  // 
                 .entry("pdf",             "'\u{f1c1}'")  // 
                 .entry("php",             "'\u{e73d}'")  // 
                 .entry("psd",             "'\u{e7b8}'")  // 
                 .entry("razor",           "'\u{f1fa}'")  // 
                 .entry("rdb",             "'\u{e76d}'")  // 
                 .entry("rpm",             "'\u{e7bb}'")  // 
                 .entry("rss",             "'\u{f09e}'")  // 
                 .entry("rtf",             "'\u{f0219}'") // 󰈙
                 .entry("sass",            "'\u{e603}'")  // 
                 .entry("scala",           "'\u{e737}'")  // 
                 .entry("scss",            "'\u{e749}'")  // 
                 .entry("sln",             "'\u{e70c}'")  // 
                 .entry("so",              "'\u{f17c}'")  // 
                 .entry("sqlite3",         "'\u{e7c4}'")  // 
                 .entry("styl",            "'\u{e600}'")  // 
                 .entry("stylus",          "'\u{e600}'")  // 
                 .entry("svg",             "'\u{f0559}'") // 󰕙
                 .entry("swift",           "'\u{e755}'")  // 
                 .entry("torrent",         "'\u{e275}'")  // 
                 .entry("ts",              "'\u{e628}'")  // 
                 .entry("tsx",             "'\u{e7ba}'")  // 
                 .entry("twig",            "'\u{e61c}'")  // 
                 .entry("txt",             "'\u{f15c}'")  // 
                 .entry("vimrc",           "'\u{e62b}'")  // 
                 .entry("vue",             "'\u{f0844}'") // 󰡄
                 .entry("xml",             "'\u{f05c0}'") // 󰗀
                 .entry("xul",             "'\u{f05c0}'") // 󰗀
                 .entry("yaml",            "'\u{f481}'")  // 
                 .entry("yml",             "'\u{f481}'")  // 
                 .build()
    )
}

/// Generate mapping from full filenames to file type. This mapping should also contain all the
/// "dot" directories that have a custom icon.  See output/render/icons.rs for a partial list of
/// icon constants.
fn generate_filename_icon_map(file: &mut File) -> io::Result<()> {
    writeln!(file, "static FILENAME_ICONS: phf::Map<&'static str, char> = {};\n",
             phf_codegen::Map::new()
                 .entry("csh.cshrc",          "Icons::SHELL") // 
                 .entry("csh.login",          "Icons::SHELL") // 
                 .entry("csh.logout",         "Icons::SHELL") // 
                 .entry("profile",            "Icons::SHELL") // 
                 .entry("zlogin",             "Icons::SHELL") // 
                 .entry("zlogout",            "Icons::SHELL") // 
                 .entry("zprofile",           "Icons::SHELL") // 
                 .entry("zshenv",             "Icons::SHELL") // 
                 .entry("zshrc",              "Icons::SHELL") // 
                 .entry(".git",               "Icons::GIT") // 
                 .entry("go.mod",             "Icons::GO_LANG") // 
                 .entry("go.sum",             "Icons::GO_LANG") // 
                 .entry("gradle",             "Icons::JAVA_LANG") // 
                 .entry("MANIFEST",           "Icons::PYTHON_LANG") // 
                 .entry("MANIFEST.in",        "Icons::PYTHON_LANG") // 
                 .entry("pyproject.toml",     "Icons::PYTHON_LANG") // 
                 .entry(".rvm",               "Icons::RUBY_LANG") // 
                 .entry("rubydoc",            "Icons::RUBYRAILS_LANG") // 
                 .entry(".cargo",             "Icons::RUST_LANG") // 
                 .entry(".rustup",            "Icons::RUST_LANG") // 
                 .entry("Cargo.toml",         "Icons::RUST_LANG") // 
                 .entry("Cargo.lock",         "Icons::RUST_LANG") // 
                 .entry("localized",          "Icons::APPLE") // 
                 .entry(".atom",              "'\u{e764}'") // 
                 .entry(".github",            "'\u{f408}'") // 
                 .entry(".idea",              "'\u{e7b5}'") // 
                 .entry(".Trash",             "'\u{f1f8}'") // 
                 .entry(".vim",               "'\u{e62b}'") // 
                 .entry(".vscode",            "'\u{e70c}'") // 
                 .entry(".vscode-cli",        "'\u{e70c}'") // 
                 .entry("bin",                "'\u{e5fc}'") // 
                 .entry("config",             "'\u{e5fc}'") // 
                 .entry("docker-compose.yml", "'\u{f308}'") // 
                 .entry("Dockerfile",         "'\u{f308}'") // 
                 .entry("GNUmakefile",        "'\u{e673}'") // 
                 .entry("gruntfile.coffee",   "'\u{e611}'") // 
                 .entry("gruntfile.js",       "'\u{e611}'") // 
                 .entry("gruntfile.ls",       "'\u{e611}'") // 
                 .entry("gulpfile.coffee",    "'\u{e610}'") // 
                 .entry("gulpfile.js",        "'\u{e610}'") // 
                 .entry("gulpfile.ls",        "'\u{e610}'") // 
                 .entry("hidden",             "'\u{f023}'") // 
                 .entry("include",            "'\u{e5fc}'") // 
                 .entry("lib",                "'\u{f121}'") // 
                 .entry("Makefile",           "'\u{e673}'") // 
                 .entry("makefile",           "'\u{e673}'") // 
                 .entry("node_modules",       "'\u{e718}'") // 
                 .entry("npmignore",          "'\u{e71e}'") // 
                 .entry("PKGBUILD",           "'\u{f303}'") // 
                 .entry("yarn.lock",          "'\u{e718}'") // 
                 .build()
    )
}