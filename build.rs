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
    generate_directory_icon_map(file.get_mut())?;
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
                 .entry("heix",      "FileType::Image")
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
                 .entry("xcf",       "FileType::Image")
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
                 .entry("mp2",       "FileType::Music")
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
                 .entry("kbx",       "FileType::Crypto")
                 .entry("md5",       "FileType::Crypto")
                 .entry("p12",       "FileType::Crypto")
                 .entry("pfx",       "FileType::Crypto")
                 .entry("pgp",       "FileType::Crypto")
                 .entry("sha1",      "FileType::Crypto")
                 .entry("sha256",    "FileType::Crypto")
                 .entry("sig",       "FileType::Crypto")
                 .entry("signature", "FileType::Crypto")
                 /* Document files */
                 .entry("djvu",      "FileType::Document")
                 .entry("doc",       "FileType::Document")
                 .entry("docx",      "FileType::Document")
                 .entry("eml",       "FileType::Document")
                 .entry("fotd",      "FileType::Document")
                 .entry("gdoc",      "FileType::Document")
                 .entry("key",       "FileType::Document")
                 .entry("keynote",   "FileType::Document")
                 .entry("numbers",   "FileType::Document")
                 .entry("odp",       "FileType::Document")
                 .entry("ods",       "FileType::Document")
                 .entry("odt",       "FileType::Document")
                 .entry("pages",     "FileType::Document")
                 .entry("pdf",       "FileType::Document")
                 .entry("ppt",       "FileType::Document")
                 .entry("pptx",      "FileType::Document")
                 .entry("rtf",       "FileType::Document")
                 .entry("xls",       "FileType::Document")
                 .entry("xlsm",      "FileType::Document")
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
                 .entry("lzo",       "FileType::Compressed")
                 .entry("lzma",      "FileType::Compressed")
                 .entry("phar",      "FileType::Compressed")
                 .entry("qcow",      "FileType::Compressed")
                 .entry("qcow2",     "FileType::Compressed")
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
                 .entry("Earthfile",         "FileType::Immediate")
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
                 .entry("webpack.config.cjs","FileType::Immediate")
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
                 .entry("aac",             "Icons::AUDIO")            // 󰝚
                 .entry("alac",            "Icons::AUDIO")            // 󰝚
                 .entry("ape",             "Icons::AUDIO")            // 󰝚
                 .entry("flac",            "Icons::AUDIO")            // 󰝚
                 .entry("m4a",             "Icons::AUDIO")            // 󰝚
                 .entry("mka",             "Icons::AUDIO")            // 󰝚
                 .entry("mp2",             "Icons::AUDIO")            // 󰝚
                 .entry("mp3",             "Icons::AUDIO")            // 󰝚
                 .entry("ogg",             "Icons::AUDIO")            // 󰝚
                 .entry("opus",            "Icons::AUDIO")            // 󰝚
                 .entry("wav",             "Icons::AUDIO")            // 󰝚
                 .entry("wma",             "Icons::AUDIO")            // 󰝚
                 .entry("ical",            "Icons::CALENDAR")         // 
                 .entry("icalendar",       "Icons::CALENDAR")         // 
                 .entry("ics",             "Icons::CALENDAR")         // 
                 .entry("ifb",             "Icons::CALENDAR")         // 
                 .entry("7z",              "Icons::COMPRESSED")       // 
                 .entry("ar",              "Icons::COMPRESSED")       // 
                 .entry("bz",              "Icons::COMPRESSED")       // 
                 .entry("bz2",             "Icons::COMPRESSED")       // 
                 .entry("cpio",            "Icons::COMPRESSED")       // 
                 .entry("gz",              "Icons::COMPRESSED")       // 
                 .entry("lz",              "Icons::COMPRESSED")       // 
                 .entry("lz4",             "Icons::COMPRESSED")       // 
                 .entry("lzh",             "Icons::COMPRESSED")       // 
                 .entry("lzma",            "Icons::COMPRESSED")       // 
                 .entry("lzo",             "Icons::COMPRESSED")       // 
                 .entry("rar",             "Icons::COMPRESSED")       // 
                 .entry("tar",             "Icons::COMPRESSED")       // 
                 .entry("taz",             "Icons::COMPRESSED")       // 
                 .entry("tbz",             "Icons::COMPRESSED")       // 
                 .entry("tbz2",            "Icons::COMPRESSED")       // 
                 .entry("tgz",             "Icons::COMPRESSED")       // 
                 .entry("tlz",             "Icons::COMPRESSED")       // 
                 .entry("txz",             "Icons::COMPRESSED")       // 
                 .entry("tz",              "Icons::COMPRESSED")       // 
                 .entry("tzo",             "Icons::COMPRESSED")       // 
                 .entry("xz",              "Icons::COMPRESSED")       // 
                 .entry("z",               "Icons::COMPRESSED")       // 
                 .entry("zip",             "Icons::COMPRESSED")       // 
                 .entry("zst",             "Icons::COMPRESSED")       // 
                 .entry("cfg",             "Icons::CONFIG")           // 
                 .entry("conf",            "Icons::CONFIG")           // 
                 .entry("editorconfig",    "Icons::CONFIG")           // 
                 .entry("ini",             "Icons::CONFIG")           // 
                 .entry("toml",            "Icons::CONFIG")           // 
                 .entry("dump",            "Icons::DATABASE")         // 
                 .entry("ldb",             "Icons::DATABASE")         // 
                 .entry("mdb",             "Icons::DATABASE")         // 
                 .entry("sql",             "Icons::DATABASE")         // 
                 .entry("db",              "Icons::DATABASE")         // 
                 .entry("dmg",             "Icons::DISK_IMAGE")       // 
                 .entry("image",           "Icons::DISK_IMAGE")       // 
                 .entry("img",             "Icons::DISK_IMAGE")       // 
                 .entry("iso",             "Icons::DISK_IMAGE")       // 
                 .entry("qcow",            "Icons::DISK_IMAGE")       // 
                 .entry("qcow2",           "Icons::DISK_IMAGE")       // 
                 .entry("tc",              "Icons::DISK_IMAGE")       // 
                 .entry("vdi",             "Icons::DISK_IMAGE")       // 
                 .entry("vhd",             "Icons::DISK_IMAGE")       // 
                 .entry("vmdk",            "Icons::DISK_IMAGE")       // 
                 .entry("doc",             "Icons::DOCUMENT")         // 
                 .entry("docx",            "Icons::DOCUMENT")         // 
                 .entry("gdoc",            "Icons::DOCUMENT")         // 
                 .entry("odt",             "Icons::DOCUMENT")         // 
                 .entry("el",              "Icons::EMACS")            // 
                 .entry("elc",             "Icons::EMACS")            // 
                 .entry("eot",             "Icons::FONT")             // 
                 .entry("font",            "Icons::FONT")             // 
                 .entry("otf",             "Icons::FONT")             // 
                 .entry("ttc",             "Icons::FONT")             // 
                 .entry("ttf",             "Icons::FONT")             // 
                 .entry("woff",            "Icons::FONT")             // 
                 .entry("woff2",           "Icons::FONT")             // 
                 .entry("git",             "Icons::GIT")              // 
                 .entry("gradle",          "Icons::GRADLE")           // 
                 .entry("htm",             "Icons::HTML5")            // 
                 .entry("html",            "Icons::HTML5")            // 
                 .entry("xhtml",           "Icons::HTML5")            // 
                 .entry("arw",             "Icons::IMAGE")            // 
                 .entry("avif",            "Icons::IMAGE")            // 
                 .entry("bmp",             "Icons::IMAGE")            // 
                 .entry("cbr",             "Icons::IMAGE")            // 
                 .entry("cbz",             "Icons::IMAGE")            // 
                 .entry("cr2",             "Icons::IMAGE")            // 
                 .entry("dvi",             "Icons::IMAGE")            // 
                 .entry("gif",             "Icons::IMAGE")            // 
                 .entry("heif",            "Icons::IMAGE")            // 
                 .entry("heix",            "Icons::IMAGE")            // 
                 .entry("ico",             "Icons::IMAGE")            // 
                 .entry("j2c",             "Icons::IMAGE")            // 
                 .entry("j2k",             "Icons::IMAGE")            // 
                 .entry("jfi",             "Icons::IMAGE")            // 
                 .entry("jfif",            "Icons::IMAGE")            // 
                 .entry("jif",             "Icons::IMAGE")            // 
                 .entry("jp2",             "Icons::IMAGE")            // 
                 .entry("jpe",             "Icons::IMAGE")            // 
                 .entry("jpeg",            "Icons::IMAGE")            // 
                 .entry("jpf",             "Icons::IMAGE")            // 
                 .entry("jpg",             "Icons::IMAGE")            // 
                 .entry("jpx",             "Icons::IMAGE")            // 
                 .entry("jxl",             "Icons::IMAGE")            // 
                 .entry("nef",             "Icons::IMAGE")            // 
                 .entry("orf",             "Icons::IMAGE")            // 
                 .entry("pbm",             "Icons::IMAGE")            // 
                 .entry("pgm",             "Icons::IMAGE")            // 
                 .entry("png",             "Icons::IMAGE")            // 
                 .entry("pnm",             "Icons::IMAGE")            // 
                 .entry("ppm",             "Icons::IMAGE")            // 
                 .entry("ps",              "Icons::IMAGE")            // 
                 .entry("pxm",             "Icons::IMAGE")            // 
                 .entry("raw",             "Icons::IMAGE")            // 
                 .entry("stl",             "Icons::IMAGE")            // 
                 .entry("tif",             "Icons::IMAGE")            // 
                 .entry("tiff",            "Icons::IMAGE")            // 
                 .entry("webp",            "Icons::IMAGE")            // 
                 .entry("xcf",             "Icons::IMAGE")            // 
                 .entry("xpm",             "Icons::IMAGE")            // 
                 .entry("avro",            "Icons::JSON")             // 
                 .entry("json",            "Icons::JSON")             // 
                 .entry("properties",      "Icons::JSON")             // 
                 .entry("c",               "Icons::LANG_C")           // 
                 .entry("h",               "Icons::LANG_C")           // 
                 .entry("m",               "Icons::LANG_C")           // 
                 .entry("c++",             "Icons::LANG_CPP")         // 
                 .entry("cc",              "Icons::LANG_CPP")         // 
                 .entry("cp",              "Icons::LANG_CPP")         // 
                 .entry("cpp",             "Icons::LANG_CPP")         // 
                 .entry("cxx",             "Icons::LANG_CPP")         // 
                 .entry("hpp",             "Icons::LANG_CPP")         // 
                 .entry("hxx",             "Icons::LANG_CPP")         // 
                 .entry("mm",              "Icons::LANG_CPP")         // 
                 .entry("cs",              "Icons::LANG_CSHARP")      // 󰌛
                 .entry("csproj",          "Icons::LANG_CSHARP")      // 󰌛
                 .entry("csx",             "Icons::LANG_CSHARP")      // 󰌛
                 .entry("fs",              "Icons::LANG_FSHARP")      // 
                 .entry("fsi",             "Icons::LANG_FSHARP")      // 
                 .entry("fsx",             "Icons::LANG_FSHARP")      // 
                 .entry("go",              "Icons::LANG_GO")          // 
                 .entry("class",           "Icons::LANG_JAVA")        // 
                 .entry("jad",             "Icons::LANG_JAVA")        // 
                 .entry("jar",             "Icons::LANG_JAVA")        // 
                 .entry("java",            "Icons::LANG_JAVA")        // 
                 .entry("war",             "Icons::LANG_JAVA")        // 
                 .entry("cjs",             "Icons::LANG_JAVASCRIPT")  // 
                 .entry("js",              "Icons::LANG_JAVASCRIPT")  // 
                 .entry("mjs",             "Icons::LANG_JAVASCRIPT")  // 
                 .entry("ml",              "Icons::LANG_OCAML")       // 
                 .entry("mli",             "Icons::LANG_OCAML")       // 
                 .entry("mll",             "Icons::LANG_OCAML")       // 
                 .entry("mly",             "Icons::LANG_OCAML")       // 
                 .entry("pl",              "Icons::LANG_PERL")        // 
                 .entry("plx",             "Icons::LANG_PERL")        // 
                 .entry("pm",              "Icons::LANG_PERL")        // 
                 .entry("pod",             "Icons::LANG_PERL")        // 
                 .entry("t",               "Icons::LANG_PERL")        // 
                 .entry("phar",            "Icons::LANG_PHP")         // 󰌟
                 .entry("php",             "Icons::LANG_PHP")         // 󰌟
                 .entry("py",              "Icons::LANG_PYTHON")      // 
                 .entry("pyc",             "Icons::LANG_PYTHON")      // 
                 .entry("pyi",             "Icons::LANG_PYTHON")      // 
                 .entry("pyo",             "Icons::LANG_PYTHON")      // 
                 .entry("whl",             "Icons::LANG_PYTHON")      // 
                 .entry("r",               "Icons::LANG_R")           // 
                 .entry("rdata",           "Icons::LANG_R")           // 
                 .entry("rds",             "Icons::LANG_R")           // 
                 .entry("gem",             "Icons::LANG_RUBY")        // 
                 .entry("gemfile",         "Icons::LANG_RUBY")        // 
                 .entry("gemspec",         "Icons::LANG_RUBY")        // 
                 .entry("guardfile",       "Icons::LANG_RUBY")        // 
                 .entry("procfile",        "Icons::LANG_RUBY")        // 
                 .entry("rb",              "Icons::LANG_RUBY")        // 
                 .entry("rspec",           "Icons::LANG_RUBY")        // 
                 .entry("rspec_parallel",  "Icons::LANG_RUBY")        // 
                 .entry("rspec_status",    "Icons::LANG_RUBY")        // 
                 .entry("ru",              "Icons::LANG_RUBY")        // 
                 .entry("rvmrc",           "Icons::LANG_RUBY")        // 
                 .entry("erb",             "Icons::LANG_RUBYRAILS")   // 
                 .entry("rubydoc",         "Icons::LANG_RUBYRAILS")   // 
                 .entry("slim",            "Icons::LANG_RUBYRAILS")   // 
                 .entry("rlib",            "Icons::LANG_RUST")        // 
                 .entry("rmeta",           "Icons::LANG_RUST")        // 
                 .entry("rs",              "Icons::LANG_RUST")        // 
                 .entry("bib",             "Icons::LANG_TEX")         // 
                 .entry("bst",             "Icons::LANG_TEX")         // 
                 .entry("cls",             "Icons::LANG_TEX")         // 
                 .entry("latex",           "Icons::LANG_TEX")         // 
                 .entry("sty",             "Icons::LANG_TEX")         // 
                 .entry("tex",             "Icons::LANG_TEX")         // 
                 .entry("cts",             "Icons::LANG_TYPESCRIPT")  // 
                 .entry("mts",             "Icons::LANG_TYPESCRIPT")  // 
                 .entry("ts",              "Icons::LANG_TYPESCRIPT")  // 
                 .entry("asc",             "Icons::LOCK")             // 
                 .entry("kbx",             "Icons::LOCK")             // 
                 .entry("lock",            "Icons::LOCK")             // 
                 .entry("md5",             "Icons::LOCK")             // 
                 .entry("sha1",            "Icons::LOCK")             // 
                 .entry("sha256",          "Icons::LOCK")             // 
                 .entry("jmd",             "Icons::MARKDOWN")         // 
                 .entry("markdown",        "Icons::MARKDOWN")         // 
                 .entry("md",              "Icons::MARKDOWN")         // 
                 .entry("mkd",             "Icons::MARKDOWN")         // 
                 .entry("rdoc",            "Icons::MARKDOWN")         // 
                 .entry("readme",          "Icons::MARKDOWN")         // 
                 .entry("rmd",             "Icons::MARKDOWN")         // 
                 .entry("_ds_store",       "Icons::OS_APPLE")         // 
                 .entry("apple",           "Icons::OS_APPLE")         // 
                 .entry("ds_store",        "Icons::OS_APPLE")         // 
                 .entry("dyld",            "Icons::OS_APPLE")         // 
                 .entry("localized",       "Icons::OS_APPLE")         // 
                 .entry("plist",           "Icons::OS_APPLE")         // 
                 .entry("a",               "Icons::OS_LINUX")         // 
                 .entry("ko",              "Icons::OS_LINUX")         // 
                 .entry("so",              "Icons::OS_LINUX")         // 
                 .entry("cab",             "Icons::OS_WINDOWS")       // 
                 .entry("cmd",             "Icons::OS_WINDOWS")       // 
                 .entry("dll",             "Icons::OS_WINDOWS")       // 
                 .entry("msi",             "Icons::OS_WINDOWS")       // 
                 .entry("windows",         "Icons::OS_WINDOWS")       // 
                 .entry("bat",             "Icons::OS_WINDOWS_CMD")   // 
                 .entry("com",             "Icons::OS_WINDOWS_CMD")   // 
                 .entry("exe",             "Icons::OS_WINDOWS_CMD")   // 
                 .entry("cue",             "Icons::PLAYLIST")         // 󰲹
                 .entry("m3u",             "Icons::PLAYLIST")         // 󰲹
                 .entry("m3u8",            "Icons::PLAYLIST")         // 󰲹
                 .entry("ps1",             "Icons::POWERSHELL")       // 
                 .entry("psd1",            "Icons::POWERSHELL")       // 
                 .entry("psm1",            "Icons::POWERSHELL")       // 
                 .entry("pub",             "Icons::PUBLIC_KEY")       // 󰷖
                 .entry("csv",             "Icons::SHEET")            // 
                 .entry("gsheet",          "Icons::SHEET")            // 
                 .entry("ods",             "Icons::SHEET")            // 
                 .entry("tsv",             "Icons::SHEET")            // 
                 .entry("xls",             "Icons::SHEET")            // 
                 .entry("xlsm",            "Icons::SHEET")            // 
                 .entry("xlsx",            "Icons::SHEET")            // 
                 .entry("awk",             "Icons::SHELL")            // 
                 .entry("bash",            "Icons::SHELL")            // 
                 .entry("bats",            "Icons::SHELL")            // 
                 .entry("csh",             "Icons::SHELL")            // 
                 .entry("fish",            "Icons::SHELL")            // 
                 .entry("ksh",             "Icons::SHELL")            // 
                 .entry("mk",              "Icons::SHELL")            // 
                 .entry("sh",              "Icons::SHELL")            // 
                 .entry("shell",           "Icons::SHELL")            // 
                 .entry("zsh",             "Icons::SHELL")            // 
                 .entry("zsh-theme",       "Icons::SHELL")            // 
                 .entry("gslides",         "Icons::SLIDE")            // 
                 .entry("odp",             "Icons::SLIDE")            // 
                 .entry("ppt",             "Icons::SLIDE")            // 
                 .entry("pptx",            "Icons::SLIDE")            // 
                 .entry("tf",              "Icons::TERRAFORM")        // 󱁢
                 .entry("tfstate",         "Icons::TERRAFORM")        // 󱁢
                 .entry("tfvars",          "Icons::TERRAFORM")        // 󱁢
                 .entry("avi",             "Icons::VIDEO")            // 
                 .entry("flv",             "Icons::VIDEO")            // 
                 .entry("heic",            "Icons::VIDEO")            // 
                 .entry("m2ts",            "Icons::VIDEO")            // 
                 .entry("m2v",             "Icons::VIDEO")            // 
                 .entry("m4v",             "Icons::VIDEO")            // 
                 .entry("mkv",             "Icons::VIDEO")            // 
                 .entry("mov",             "Icons::VIDEO")            // 
                 .entry("mp4",             "Icons::VIDEO")            // 
                 .entry("mpeg",            "Icons::VIDEO")            // 
                 .entry("mpg",             "Icons::VIDEO")            // 
                 .entry("ogm",             "Icons::VIDEO")            // 
                 .entry("ogv",             "Icons::VIDEO")            // 
                 .entry("video",           "Icons::VIDEO")            // 
                 .entry("vob",             "Icons::VIDEO")            // 
                 .entry("webm",            "Icons::VIDEO")            // 
                 .entry("wmv",             "Icons::VIDEO")            // 
                 .entry("vim",             "Icons::VIM")              // 
                 .entry("ai",              "'\u{e7b4}'")              // 
                 .entry("acf",             "'\u{f1b6}'")              // 
                 .entry("android",         "'\u{e70e}'")              // 
                 .entry("apk",             "'\u{e70e}'")              // 
                 .entry("asm",             "'\u{e637}'")              // 
                 .entry("asp",             "'\u{f121}'")              // 
                 .entry("bin",             "'\u{eae8}'")              // 
                 .entry("clj",             "'\u{e768}'")              // 
                 .entry("cljs",            "'\u{e76a}'")              // 
                 .entry("coffee",          "'\u{f0f4}'")              // 
                 .entry("cert",            "'\u{eafa}'")              // 
                 .entry("crt",             "'\u{eafa}'")              // 
                 .entry("cshtml",          "'\u{f1fa}'")              // 
                 .entry("css",             "'\u{e749}'")              // 
                 .entry("cu",              "'\u{e64b}'")              // 
                 .entry("d",               "'\u{e7af}'")              // 
                 .entry("dart",            "'\u{e798}'")              // 
                 .entry("deb",             "'\u{e77d}'")              // 
                 .entry("desktop",         "'\u{ebd1}'")              // 
                 .entry("diff",            "'\u{f440}'")              // 
                 .entry("djvu",            "'\u{f02d}'")              // 
                 .entry("download",        "'\u{f01da}'")             // 󰇚
                 .entry("dot",             "'\u{f1049}'")             // 󱁉
                 .entry("drawio",          "'\u{ebba}'")              // 
                 .entry("ebook",           "'\u{e28b}'")              // 
                 .entry("ebuild",          "'\u{f30d}'")              // 
                 .entry("ejs",             "'\u{e618}'")              // 
                 .entry("elm",             "'\u{e62c}'")              // 
                 .entry("eml",             "'\u{f003}'")              // 
                 .entry("env",             "'\u{f462}'")              // 
                 .entry("eps",             "'\u{f0559}'")             // 󰕙
                 .entry("epub",            "'\u{e28a}'")              // 
                 .entry("erl",             "'\u{e7b1}'")              // 
                 .entry("ex",              "'\u{e62d}'")              // 
                 .entry("exs",             "'\u{e62d}'")              // 
                 .entry("gform",           "'\u{f298}'")              // 
                 .entry("gpg",             "'\u{e60a}'")              // 
                 .entry("groovy",          "'\u{e775}'")              // 
                 .entry("hbs",             "'\u{e60f}'")              // 
                 .entry("hs",              "'\u{e777}'")              // 
                 .entry("iml",             "'\u{e7b5}'")              // 
                 .entry("ipynb",           "'\u{e678}'")              // 
                 .entry("jl",              "'\u{e624}'")              // 
                 .entry("jsx",             "'\u{e7ba}'")              // 
                 .entry("kdb",             "'\u{f23e}'")              // 
                 .entry("kdbx",            "'\u{f23e}'")              // 
                 .entry("key",             "'\u{eb11}'")              // 
                 .entry("kt",              "'\u{e634}'")              // 
                 .entry("kts",             "'\u{e634}'")              // 
                 .entry("less",            "'\u{e758}'")              // 
                 .entry("lhs",             "'\u{e777}'")              // 
                 .entry("license",         "'\u{f02d}'")              // 
                 .entry("lisp",            "'\u{f0172}'")             // 󰅲
                 .entry("log",             "'\u{f18d}'")              // 
                 .entry("lua",             "'\u{e620}'")              // 
                 .entry("magnet",          "'\u{f076}'")              // 
                 .entry("mobi",            "'\u{e28b}'")              // 
                 .entry("mustache",        "'\u{e60f}'")              // 
                 .entry("nix",             "'\u{f313}'")              // 
                 .entry("node",            "'\u{f0399}'")             // 󰎙
                 .entry("o",               "'\u{eae8}'")              // 
                 .entry("out",             "'\u{eb2c}'")              // 
                 .entry("org",             "'\u{e633}'")              // 
                 .entry("part",            "'\u{f43a}'")              // 
                 .entry("patch",           "'\u{f440}'")              // 
                 .entry("pem",             "'\u{eb11}'")              // 
                 .entry("pdf",             "'\u{f1c1}'")              // 
                 .entry("pkg",             "'\u{eb29}'")              // 
                 .entry("psd",             "'\u{e7b8}'")              // 
                 .entry("razor",           "'\u{f1fa}'")              // 
                 .entry("rdb",             "'\u{e76d}'")              // 
                 .entry("rpm",             "'\u{e7bb}'")              // 
                 .entry("rss",             "'\u{f09e}'")              // 
                 .entry("rst",             "'\u{f15c}'")              // 
                 .entry("rtf",             "'\u{f0219}'")             // 󰈙
                 .entry("s",               "'\u{e637}'")              // 
                 .entry("sass",            "'\u{e603}'")              // 
                 .entry("scala",           "'\u{e737}'")              // 
                 .entry("scss",            "'\u{e603}'")              // 
                 .entry("service",         "'\u{eba2}'")              // 
                 .entry("sig",             "'\u{e60a}'")              // 
                 .entry("sln",             "'\u{e70c}'")              // 
                 .entry("sqlite3",         "'\u{e7c4}'")              // 
                 .entry("srt",             "'\u{f0a16}'")             // 󰨖
                 .entry("styl",            "'\u{e600}'")              // 
                 .entry("stylus",          "'\u{e600}'")              // 
                 .entry("sub",             "'\u{f0a16}'")             // 󰨖
                 .entry("sublime-package", "'\u{e7aa}'")              // 
                 .entry("sublime-session", "'\u{e7aa}'")              // 
                 .entry("svelte",          "'\u{e697}'")              // 
                 .entry("svg",             "'\u{f0559}'")             // 󰕙
                 .entry("swift",           "'\u{e755}'")              // 
                 .entry("torrent",         "'\u{e275}'")              // 
                 .entry("tsx",             "'\u{e7ba}'")              // 
                 .entry("twig",            "'\u{e61c}'")              // 
                 .entry("txt",             "'\u{f15c}'")              // 
                 .entry("unity",           "'\u{e721}'")              // 
                 .entry("unity3d",         "'\u{e721}'")              // 
                 .entry("vue",             "'\u{f0844}'")             // 󰡄
                 .entry("xbps",            "'\u{f187}'")              // 
                 .entry("xml",             "'\u{f05c0}'")             // 󰗀
                 .entry("xul",             "'\u{f05c0}'")             // 󰗀
                 .entry("yaml",            "'\u{f481}'")              // 
                 .entry("yml",             "'\u{f481}'")              // 
                 .build()
    )
}

/// Generate mapping from full filenames to file type. This mapping should also contain all the
/// "dot" directories that have a custom icon.  See output/render/icons.rs for a partial list of
/// icon constants.
fn generate_filename_icon_map(file: &mut File) -> io::Result<()> {
    writeln!(file, "static FILENAME_ICONS: phf::Map<&'static str, char> = {};\n",
             phf_codegen::Map::new()
                 .entry(".Xauthority",        "Icons::CONFIG")        // 
                 .entry(".Xresources",        "Icons::CONFIG")        // 
                 .entry(".clang-format",      "Icons::CONFIG")        // 
                 .entry(".htaccess",          "Icons::CONFIG")        // 
                 .entry(".htpasswd",          "Icons::CONFIG")        // 
                 .entry(".xinitrc",           "Icons::CONFIG")        // 
                 .entry("Vagrantfile",        "Icons::CONFIG")        // 
                 .entry("bspwmrc",            "Icons::CONFIG")        // 
                 .entry("crontab",            "Icons::CONFIG")        // 
                 .entry("crypttab",           "Icons::CONFIG")        // 
                 .entry("environment",        "Icons::CONFIG")        // 
                 .entry("group",              "Icons::CONFIG")        // 
                 .entry("gshadow",            "Icons::CONFIG")        // 
                 .entry("hostname",           "Icons::CONFIG")        // 
                 .entry("inputrc",            "Icons::CONFIG")        // 
                 .entry("shadow",             "Icons::CONFIG")        // 
                 .entry("shells",             "Icons::CONFIG")        // 
                 .entry(".emacs",             "Icons::EMACS")         // 
                 .entry(".gitattributes",     "Icons::GIT")           // 
                 .entry(".gitconfig",         "Icons::GIT")           // 
                 .entry(".gitignore",         "Icons::GIT")           // 
                 .entry(".gitignore_global",  "Icons::GIT")           // 
                 .entry(".gitmodules",        "Icons::GIT")           // 
                 .entry("build.gradle.kts",   "Icons::GRADLE")        // 
                 .entry("gradle.properties",  "Icons::GRADLE")        // 
                 .entry("gradlew",            "Icons::GRADLE")        // 
                 .entry("gradlew.bat",        "Icons::GRADLE")        // 
                 .entry("settings.gradle.kts","Icons::GRADLE")        // 
                 .entry("gruntfile.coffee",   "Icons::GRUNT")         // 
                 .entry("gruntfile.js",       "Icons::GRUNT")         // 
                 .entry("gruntfile.ls",       "Icons::GRUNT")         // 
                 .entry("gulpfile.coffee",    "Icons::GULP")          // 
                 .entry("gulpfile.js",        "Icons::GULP")          // 
                 .entry("gulpfile.ls",        "Icons::GULP")          // 
                 .entry("go.mod",             "Icons::LANG_GO")       // 
                 .entry("go.sum",             "Icons::LANG_GO")       // 
                 .entry("composer.json",      "Icons::LANG_PHP")      // 󰌟
                 .entry("composer.lock",      "Icons::LANG_PHP")      // 󰌟
                 .entry(".python_history",    "Icons::LANG_PYTHON")   // 
                 .entry("MANIFEST",           "Icons::LANG_PYTHON")   // 
                 .entry("MANIFEST.in",        "Icons::LANG_PYTHON")   // 
                 .entry("pyproject.toml",     "Icons::LANG_PYTHON")   // 
                 .entry("requirements.txt",   "Icons::LANG_PYTHON")   // 
                 .entry("Rakefile",           "Icons::LANG_RUBY")     // 
                 .entry("rubydoc",            "Icons::LANG_RUBYRAILS")// 
                 .entry(".release.toml",      "Icons::LANG_RUST")     // 
                 .entry(".rustfmt.toml",      "Icons::LANG_RUST")     // 
                 .entry("Cargo.lock",         "Icons::LANG_RUST")     // 
                 .entry("Cargo.toml",         "Icons::LANG_RUST")     // 
                 .entry("LICENCE",            "Icons::LICENSE")       // 
                 .entry("LICENCE.md",         "Icons::LICENSE")       // 
                 .entry("LICENCE.txt",        "Icons::LICENSE")       // 
                 .entry("LICENSE",            "Icons::LICENSE")       // 
                 .entry("LICENSE.md",         "Icons::LICENSE")       // 
                 .entry("LICENSE.txt",        "Icons::LICENSE")       // 
                 .entry("authorized_keys",    "Icons::LICENSE")       // 
                 .entry("hidden",             "Icons::LOCK")          // 
                 .entry("passwd",             "Icons::LOCK")          // 
                 .entry("sudoers",            "Icons::LOCK")          // 
                 .entry("GNUmakefile",        "Icons::MAKE")          // 
                 .entry("Makefile",           "Icons::MAKE")          // 
                 .entry("Makefile.ac",        "Icons::MAKE")          // 
                 .entry("Makefile.am",        "Icons::MAKE")          // 
                 .entry("Makefile.in",        "Icons::MAKE")          // 
                 .entry("configure.ac",       "Icons::MAKE")          // 
                 .entry("configure.in",       "Icons::MAKE")          // 
                 .entry("makefile",           "Icons::MAKE")          // 
                 .entry(".npmignore",         "Icons::NPM")           // 
                 .entry(".npmrc",             "Icons::NPM")           // 
                 .entry("npm-shrinkwrap.json","Icons::NPM")           // 
                 .entry("npmrc",              "Icons::NPM")           // 
                 .entry("package-lock.json",  "Icons::NPM")           // 
                 .entry("package.json",       "Icons::NPM")           // 
                 .entry(".localized",         "Icons::OS_APPLE")      // 
                 .entry("id_dsa",             "Icons::PRIVATE_KEY")   // 󰌆
                 .entry("id_ecdsa",           "Icons::PRIVATE_KEY")   // 󰌆
                 .entry("id_ecdsa_sk",        "Icons::PRIVATE_KEY")   // 󰌆
                 .entry("id_ed25519",         "Icons::PRIVATE_KEY")   // 󰌆
                 .entry("id_ed25519_sk",      "Icons::PRIVATE_KEY")   // 󰌆
                 .entry("id_rsa",             "Icons::PRIVATE_KEY")   // 󰌆
                 .entry(".bash_history",      "Icons::SHELL")         // 
                 .entry(".bash_logout",       "Icons::SHELL")         // 
                 .entry(".bash_profile",      "Icons::SHELL")         // 
                 .entry(".bashrc",            "Icons::SHELL")         // 
                 .entry(".cshrc",             "Icons::SHELL")         // 
                 .entry(".kshrc",             "Icons::SHELL")         // 
                 .entry(".login",             "Icons::SHELL")         // 
                 .entry(".logout",            "Icons::SHELL")         // 
                 .entry(".profile",           "Icons::SHELL")         // 
                 .entry(".tcshrc",            "Icons::SHELL")         // 
                 .entry(".zlogin",            "Icons::SHELL")         // 
                 .entry(".zlogout",           "Icons::SHELL")         // 
                 .entry(".zprofile",          "Icons::SHELL")         // 
                 .entry(".zsh_history",       "Icons::SHELL")         // 
                 .entry(".zshenv",            "Icons::SHELL")         // 
                 .entry(".zshrc",             "Icons::SHELL")         // 
                 .entry("csh.cshrc",          "Icons::SHELL")         // 
                 .entry("csh.login",          "Icons::SHELL")         // 
                 .entry("csh.logout",         "Icons::SHELL")         // 
                 .entry("profile",            "Icons::SHELL")         // 
                 .entry("zlogin",             "Icons::SHELL")         // 
                 .entry("zlogout",            "Icons::SHELL")         // 
                 .entry("zprofile",           "Icons::SHELL")         // 
                 .entry("zshenv",             "Icons::SHELL")         // 
                 .entry("zshrc",              "Icons::SHELL")         // 
                 .entry(".ideavimrc",         "Icons::VIM")           // 
                 .entry(".viminfo",           "Icons::VIM")           // 
                 .entry(".vimrc",             "Icons::VIM")           // 
                 .entry(".atom",              "'\u{e764}'")           // 
                 .entry(".gitlab-ci.yml",     "'\u{f296}'")           // 
                 .entry(".node_repl_history", "'\u{e718}'")           // 
                 .entry("Dockerfile",         "'\u{f308}'")           // 
                 .entry("Earthfile",          "'\u{f0ac}'")           // 
                 .entry("PKGBUILD",           "'\u{f303}'")           // 
                 .entry("a.out",              "'\u{f489}'")           // 
                 .entry("autostart",          "'\u{f489}'")           // 
                 .entry("config",             "'\u{e615}'")           // 
                 .entry("configure",          "'\u{f489}'")           // 
                 .entry("docker-compose.yml", "'\u{f308}'")           // 
                 .entry("known_hosts",        "'\u{f08c0}'")          // 󰣀
                 .entry("localtime",          "'\u{f43a}'")           // 
                 .entry("robots.txt",         "'\u{f06a9}'")          // 󰚩
                 .entry("timezone",           "'\u{f43a}'")           // 
                 .entry("webpack.config.js",  "'\u{f072b}'")          // 󰜫
                 .entry("yarn.lock",          "'\u{e6a7}'")           // 
                 .build()
    )
}

fn generate_directory_icon_map(file: &mut File) -> io::Result<()> {
    writeln!(file, "static DIRECTORY_ICONS: phf::Map<&'static str, char> = {};\n",
             phf_codegen::Map::new()
                 .entry(".config",            "Icons::CONFIG_FOLDER") // 
                 .entry("bin",                "Icons::CONFIG_FOLDER") // 
                 .entry("config",             "Icons::CONFIG_FOLDER") // 
                 .entry("cron.d",             "Icons::CONFIG_FOLDER") // 
                 .entry("cron.daily",         "Icons::CONFIG_FOLDER") // 
                 .entry("cron.hourly",        "Icons::CONFIG_FOLDER") // 
                 .entry("cron.monthly",       "Icons::CONFIG_FOLDER") // 
                 .entry("cron.weekly",        "Icons::CONFIG_FOLDER") // 
                 .entry("etc",                "Icons::CONFIG_FOLDER") // 
                 .entry("include",            "Icons::CONFIG_FOLDER") // 
                 .entry("xbps.d",             "Icons::CONFIG_FOLDER") // 
                 .entry("xorg.conf.d",        "Icons::CONFIG_FOLDER") // 
                 .entry(".doom.d",            "Icons::EMACS")         // 
                 .entry(".emacs.d",           "Icons::EMACS")         // 
                 .entry(".gradle",            "Icons::GRADLE")        // 
                 .entry("gradle",             "Icons::GRADLE")        // 
                 .entry(".rvm",               "Icons::LANG_RUBY")     // 
                 .entry(".cargo",             "Icons::LANG_RUST")     // 
                 .entry(".rustup",            "Icons::LANG_RUST")     // 
                 .entry("hidden",             "Icons::LOCK")          // 
                 .entry(".npm",               "Icons::NPM_FOLDER")    // 
                 .entry("node_modules",       "Icons::NPM_FOLDER")    // 
                 .entry(".localized",         "Icons::OS_APPLE")      // 
                 .entry(".zsh_sessions",      "Icons::SHELL")         // 
                 .entry(".vim",               "Icons::VIM")           // 
                 .entry(".Trash",             "'\u{f1f8}'")           // 
                 .entry(".git",               "'\u{e5fb}'")           // 
                 .entry(".github",            "'\u{e5fd}'")           // 
                 .entry(".idea",              "'\u{e7b5}'")           // 
                 .entry(".ssh",               "'\u{f10ec}'")          // 󱃬
                 .entry(".vscode",            "'\u{e70c}'")           // 
                 .entry(".vscode-cli",        "'\u{e70c}'")           // 
                 .entry("Desktop",            "'\u{f108}'")           // 
                 .entry("Downloads",          "'\u{f024d}'")          // 󰉍
                 .entry("Mail",               "'\u{f01f0}'")          // 󰇰
                 .entry("Movies",             "'\u{f03d}'")           // 
                 .entry("Music",              "'\u{f1359}'")          // 󱍙
                 .entry("Pictures",           "'\u{f024f}'")          // 󰉏
                 .entry("Videos",             "'\u{f03d}'")           // 
                 .entry("lib",                "'\u{f121}'")           // 
                 .build()
    )
}
