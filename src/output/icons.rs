use ansi_term::Style;

use crate::fs::File;
use crate::info::filetype::FileExtensions;
use lazy_static::lazy_static;
use std::collections::HashMap;


pub trait FileIcon {
    fn icon_file(&self, file: &File<'_>) -> Option<char>;
}


#[derive(Copy, Clone)]
pub enum Icons {
    Audio,
    Image,
    Video,
}

impl Icons {
    pub fn value(self) -> char {
        match self {
            Self::Audio  => '\u{f001}',
            Self::Image  => '\u{f1c5}',
            Self::Video  => '\u{f03d}',
        }
    }
}


/// Converts the style used to paint a file name into the style that should be
/// used to paint an icon.
///
/// - The background colour should be preferred to the foreground colour, as
///   if one is set, it’s the more “obvious” colour choice.
/// - If neither is set, just use the default style.
/// - Attributes such as bold or underline should not be used to paint the
///   icon, as they can make it look weird.
pub fn iconify_style(style: Style) -> Style {
    style.background.or(style.foreground)
         .map(Style::from)
         .unwrap_or_default()
}



lazy_static! {
    static ref MAP_BY_NAME: HashMap<&'static str, char> = {
        let mut m = HashMap::new();
        m.insert(".Trash", '\u{f1f8}'); // 
        m.insert(".atom", '\u{e764}'); // 
        m.insert(".bashprofile", '\u{e615}'); // 
        m.insert(".bashrc", '\u{f489}'); // 
        m.insert(".clang-format", '\u{e615}'); // 
        m.insert(".git", '\u{e5fb}'); // 
        m.insert(".gitattributes", '\u{f1d3}'); // 
        m.insert(".gitconfig", '\u{f1d3}'); // 
        m.insert(".github", '\u{e5fd}'); // 
        m.insert(".gitignore", '\u{f1d3}'); // 
        m.insert(".gitmodules", '\u{f1d3}'); // 
        m.insert(".gitlab-ci.yml", '\u{f296}'); // 
        m.insert(".htaccess", '\u{e615}'); // 
        m.insert(".htpasswd", '\u{e615}'); // 
        m.insert(".node_repl_history", '\u{e718}'); // 
        m.insert(".python_history", '\u{e606}'); // 
        m.insert(".release.toml", '\u{e7a8}'); // 
        m.insert(".rustfmt.toml", '\u{e7a8}'); // 
        m.insert(".rvm", '\u{e21e}'); // 
        m.insert(".ssh", '\u{f023}'); // 
        m.insert(".vim", '\u{e62b}'); // 
        m.insert(".viminfo", '\u{e62b}'); // 
        m.insert(".vimrc", '\u{e62b}'); // 
        m.insert(".vscode", '\u{e70c}'); // 
        m.insert(".xinitrc", '\u{e615}'); // 
        m.insert(".Xauthority", '\u{e615}'); // 
        m.insert(".Xresources", '\u{e615}'); // 
        m.insert(".zshrc", '\u{f489}'); // 
        m.insert(".zsh_history", '\u{e615}'); // 
        m.insert("Cargo.lock", '\u{e7a8}'); // 
        m.insert("Cargo.toml", '\u{e7a8}'); // 
        m.insert("a.out", '\u{f489}');
        m.insert("authorized_keys", '\u{e60a}'); // 
        m.insert("autostart", '\u{f489}'); // 
        m.insert("bin", '\u{e5fc}'); // 
        m.insert("bspwmrc", '\u{e615}'); // 
        m.insert("composer.json", '\u{e608}'); // 
        m.insert("composer.lock", '\u{e608}'); // 
        m.insert("config", '\u{e5fc}'); // 
        m.insert("config.ac", '\u{e615}'); // 
        m.insert("config.el", '\u{e779}'); // 
        m.insert("config.m4", '\u{e615}'); // 
        m.insert("config.mk", '\u{e615}'); // 
        m.insert("configure", '\u{f489}'); // 
        m.insert("contributing", '\u{e60a}'); // 
        m.insert("cron.d", '\u{e5fc}'); // 
        m.insert("cron.daily", '\u{e5fc}'); // 
        m.insert("cron.hourly", '\u{e5fc}'); // 
        m.insert("cron.monthly", '\u{e5fc}'); // 
        m.insert("cron.weekly", '\u{e5fc}'); // 
        m.insert("crontab", '\u{e615}'); // 
        m.insert("crypttab", '\u{e615}'); // 
        m.insert("custom.el", '\u{e779}'); // 
        m.insert("Desktop", '\u{f108}'); // 
        m.insert("docker-compose.yml", '\u{f308}'); // 
        m.insert("Dockerfile", '\u{f308}'); // 
        m.insert("Downloads", '\u{f498}'); // 
        m.insert("ds_store", '\u{f179}'); // 
        m.insert("etc", '\u{e5fc}'); // 
        m.insert("favicon.ico", '\u{f005}'); // 
        m.insert("favicon.png", '\u{f005}'); // 
        m.insert("FUNDING.yml", '\u{f408}'); // 
        m.insert("fstab", '\u{f1c0}'); // 
        m.insert("gitignore_global", '\u{f1d3}'); // 
        m.insert("go.mod", '\u{e627}'); // 
        m.insert("go.sum", '\u{e627}'); // 
        m.insert("gradle", '\u{e256}'); // 
        m.insert("gruntfile.coffee", '\u{e611}'); // 
        m.insert("gruntfile.js", '\u{e611}'); // 
        m.insert("gruntfile.ls", '\u{e611}'); // 
        m.insert("group", '\u{e615}'); // 
        m.insert("gshadow", '\u{e615}'); // 
        m.insert("gulpfile.coffee", '\u{e610}'); // 
        m.insert("gulpfile.js", '\u{e610}'); // 
        m.insert("gulpfile.ls", '\u{e610}'); // 
        m.insert("hidden", '\u{f023}'); // 
        m.insert("hostname", '\u{e615}'); // 
        m.insert("hosts", '\u{f502}'); // 
        m.insert("htoprc", '\u{e615}'); // 
        m.insert("include", '\u{e5fc}'); // 
        m.insert("init", '\u{e615}'); // 
        m.insert("init.el", '\u{e779}'); // 
        m.insert("known_hosts", '\u{e60a}'); // 
        m.insert("lib", '\u{f121}'); // 
        m.insert("LICENSE", '\u{e60a}'); // 
        m.insert("LICENSE.md", '\u{e60a}');
        m.insert("LICENSE.txt", '\u{e60a}');
        m.insert("localized", '\u{f179}'); // 
        m.insert("Makefile", '\u{e615}'); // 
        m.insert("Makefile.ac", '\u{e615}'); // 
        m.insert("muttrc", '\u{e615}'); // 
        m.insert("node_modules", '\u{e5fa}'); // 
        m.insert("npmignore", '\u{e71e}'); // 
        m.insert("package.json", '\u{e718}'); // 
        m.insert("package-lock.json", '\u{e718}'); // 
        m.insert("packages.el", '\u{e779}'); // 
        m.insert("passwd", '\u{f023}'); // 
        m.insert("PKGBUILD", '\u{f303}'); // 
        m.insert("profile", '\u{e615}'); // 
        m.insert("rc.lua", '\u{e615}'); // 
        m.insert("README", '\u{f48a}'); // 
        m.insert("README.org", '\u{f48a}'); // 
        m.insert("robots.txt", '\u{fba7}'); // ﮧ
        m.insert("root", '\u{f023}'); // 
        m.insert("rubydoc", '\u{e73b}'); // 
        m.insert("sha256sums", '\u{e60a}'); // 
        m.insert("shadow", '\u{e615}'); // 
        m.insert("shells", '\u{e615}'); // 
        m.insert("sudoers", '\u{f023}'); // 
        m.insert("sxhkdrc", '\u{e615}'); // 
        m.insert("tigrc", '\u{e615}'); // 
        m.insert("Vagrantfile", '\u{e615}'); // 
        m.insert("videos", '\u{f03d}'); // 
        m.insert("webpack.config.js", '\u{fc29}'); // ﰩ
        m.insert("xmonad.hs", '\u{e615}'); // 
        m.insert("xbps.d", '\u{e5fc}'); // 
        m.insert("xorg.conf.d", '\u{e5fc}'); // 
        m.insert("yarn.lock", '\u{e718}'); // 

        m
    };
}

pub fn icon_for_file(file: &File<'_>) -> char {
    let extensions = Box::new(FileExtensions);

    if let Some(icon) = MAP_BY_NAME.get(file.name.as_str()) { *icon }
    else if file.points_to_directory() {
        match file.name.as_str() {
            "bin"           => '\u{e5fc}', // 
            ".cargo"        => '\u{e7a8}', // 
            ".config"       => '\u{e5fc}',
            ".doom.d"       => '\u{e779}', // 
            ".emacs.d"      => '\u{e779}', // 
            ".git"          => '\u{e5fb}', // 
            ".npm"          => '\u{e5fa}', // 
            "node_modules"  => '\u{e5fa}', // 
            ".ssh"          => '\u{f023}', // 
            ".idea"         => '\u{e7b5}', // 
            "cron.d"        => '\u{e5fc}', // 
            "cron.daily"    => '\u{e5fc}', // 
            "cron.hourly"   => '\u{e5fc}', // 
            "cron.monthly"  => '\u{e5fc}', // 
            "cron.weekly"   => '\u{e5fc}', // 
            "Desktop"       => '\u{f108}', // 
            "Downloads"     => '\u{f498}', // 
            "etc"           => '\u{e5fc}', // 
            "git"           => '\u{e5fb}', // 
            "Pictures"      => '\u{f03e}', // 
            "Mail"          => '\u{f6ef}', // 
            "Music"         => '\u{f025}', // i
            "org"           => '\u{e779}', // 
            "Videos"        => '\u{f03d}', // 
            "xbps.d"        => '\u{e5fc}', // 
            "xorg.conf.d"   => '\u{e5fc}', // 
            _               => '\u{f115}'  // 
        }
    }
    else if let Some(icon) = extensions.icon_file(file) { icon }
    else if let Some(ext) = file.ext.as_ref() {
        match ext.as_str() {
            "1"             => '\u{f02d}', // 
            "7z"            => '\u{f410}', // 
            "a"             => '\u{e624}', // 
            "ai"            => '\u{e7b4}', // 
            "android"       => '\u{e70e}', // 
            "ape"           => '\u{f001}', // 
            "apk"           => '\u{e70e}', // 
            "apple"         => '\u{f179}', // 
            "asc"           => '\u{f023}', // 
            "asm"           => '\u{e614}', // 
            "asp"           => '\u{f121}', // 
            "avi"           => '\u{f03d}', // 
            "avif"          => '\u{f1c5}', // 
            "avro"          => '\u{e60b}', // 
            "awk"           => '\u{f489}', // 
            "bash"          => '\u{f489}', // 
            "bash_history"  => '\u{f489}', // 
            "bash_profile"  => '\u{f489}', // 
            "bashrc"        => '\u{f489}', // 
            "bat"           => '\u{f17a}', // 
            "bats"          => '\u{f489}', // 
            "bin"           => '\u{f489}', // 
            "bio"           => '\u{f910}', // 蘿
            "bmp"           => '\u{f1c5}', // 
            "bz"            => '\u{f410}', // 
            "bz2"           => '\u{f410}', // 
            "c"             => '\u{e61e}', // 
            "c++"           => '\u{e61d}', // 
            "cab"           => '\u{e70f}', // 
            "cc"            => '\u{e61d}', // 
            "cfg"           => '\u{e615}', // 
            "class"         => '\u{e256}', // 
            "clj"           => '\u{e768}', // 
            "cljs"          => '\u{e76a}', // 
            "cls"           => '\u{f034}', // 
            "cmd"           => '\u{f17a}', // 
            "coffee"        => '\u{f0f4}', // 
            "com"           => '\u{f17a}', // 
            "conf"          => '\u{e615}', // 
            "cp"            => '\u{e61d}', // 
            "cpio"          => '\u{f410}', // 
            "cpp"           => '\u{e61d}', // 
            "cs"            => '\u{f81a}', // 
            "csh"           => '\u{f489}', // 
            "cshtml"        => '\u{f1fa}', // 
            "csproj"        => '\u{f81a}', // 
            "css"           => '\u{e749}', // 
            "csv"           => '\u{f1c3}', // 
            "csx"           => '\u{f81a}', // 
            "cue"           => '\u{f910}', // 蘿
            "cxx"           => '\u{e61d}', // 
            "d"             => '\u{e7af}', // 
            "dart"          => '\u{e798}', // 
            "db"            => '\u{f1c0}', // 
            "deb"           => '\u{f187}', // 
            "desktop"       => '\u{f108}', // 
            "diff"          => '\u{f440}', // 
            "djvu"          => '\u{f02d}', // 
            "dll"           => '\u{f17a}', // 
            "doc"           => '\u{f1c2}', // 
            "docx"          => '\u{f1c2}', // 
            "ds_store"      => '\u{f179}', // 
            "DS_store"      => '\u{f179}', // 
            "dump"          => '\u{f1c0}', // 
            "ebook"         => '\u{e28b}', // 
            "ebuild"        => '\u{f30d}', // 
            "editorconfig"  => '\u{e615}', // 
            "ejs"           => '\u{e618}', // 
            "el"            => '\u{f671}', // 
            "elc"           => '\u{f671}', // 
            "elf"           => '\u{f489}', // 
            "elm"           => '\u{e62c}', // 
            "env"           => '\u{f462}', // 
            "eot"           => '\u{f031}', // 
            "epub"          => '\u{e28a}', // 
            "erb"           => '\u{e73b}', // 
            "erl"           => '\u{e7b1}', // 
            "ex"            => '\u{e62d}', // 
            "exe"           => '\u{f17a}', // 
            "exs"           => '\u{e62d}', // 
            "fish"          => '\u{f489}', // 
            "flac"          => '\u{f001}', // 
            "flv"           => '\u{f03d}', // 
            "font"          => '\u{f031}', // 
            "fs"            => '\u{e7a7}', // 
            "fsi"           => '\u{e7a7}', // 
            "fsx"           => '\u{e7a7}', // 
            "gdoc"          => '\u{f1c2}', // 
            "gem"           => '\u{e21e}', // 
            "gemfile"       => '\u{e21e}', // 
            "gemspec"       => '\u{e21e}', // 
            "gform"         => '\u{f298}', // 
            "gif"           => '\u{f1c5}', // 
            "git"           => '\u{f1d3}', // 
            "gitattributes" => '\u{f1d3}', // 
            "gitconfig"     => '\u{f1d3}',
            "gitignore"     => '\u{f1d3}', // 
            "gitmodules"    => '\u{f1d3}', // 
            "go"            => '\u{e627}', // 
            "gradle"        => '\u{e256}', // 
            "groovy"        => '\u{e775}', // 
            "gsheet"        => '\u{f1c3}', // 
            "gslides"       => '\u{f1c4}', // 
            "guardfile"     => '\u{e21e}', // 
            "gz"            => '\u{f410}', // 
            "h"             => '\u{f0fd}', // 
            "hbs"           => '\u{e60f}', // 
            "heic"          => '\u{f1c5}', // 
            "heif"          => '\u{f1c5}', // 
            "heix"          => '\u{f1c5}', // 
            "hpp"           => '\u{f0fd}', // 
            "hs"            => '\u{e777}', // 
            "htm"           => '\u{f13b}', // 
            "html"          => '\u{f13b}', // 
            "hxx"           => '\u{f0fd}', // 
            "ico"           => '\u{f1c5}', // 
            "image"         => '\u{f1c5}', // 
            "img"           => '\u{e271}', // 
            "iml"           => '\u{e7b5}', // 
            "ini"           => '\u{e615}', // 
            "ipynb"         => '\u{e606}', // 
            "iso"           => '\u{e271}', // 
            "j2c"           => '\u{f1c5}', // 
            "j2k"           => '\u{f1c5}', // 
            "jad"           => '\u{e256}', // 
            "jar"           => '\u{e256}', // 
            "java"          => '\u{e256}', // 
            "jfi"           => '\u{f1c5}', // 
            "jfif"          => '\u{f1c5}', // 
            "jif"           => '\u{f1c5}', // 
            "jl"            => '\u{e624}', // 
            "jmd"           => '\u{f48a}', // 
            "jp2"           => '\u{f1c5}', // 
            "jpe"           => '\u{f1c5}', // 
            "jpeg"          => '\u{f1c5}', // 
            "jpg"           => '\u{f1c5}', // 
            "jpx"           => '\u{f1c5}', // 
            "js"            => '\u{e74e}', // 
            "json"          => '\u{e60b}', // 
            "jsx"           => '\u{e7ba}', // 
            "jxl"           => '\u{f1c5}', // 
            "key"           => '\u{e60a}', // 
            "ksh"           => '\u{f489}', // 
            "latex"         => '\u{f034}', // 
            "ld"            => '\u{e624}', // 
            "ldb"           => '\u{f1c0}', // 
            "less"          => '\u{e758}', // 
            "lhs"           => '\u{e777}', // 
            "license"       => '\u{e60a}', // 
            "lisp"          => '\u{f671}', // 
            "localized"     => '\u{f179}', // 
            "lock"          => '\u{f023}', // 
            "log"           => '\u{f18d}', // 
            "lua"           => '\u{e620}', // 
            "lz"            => '\u{f410}', // 
            "lz4"           => '\u{f410}', // 
            "lzh"           => '\u{f410}', // 
            "lzma"          => '\u{f410}', // 
            "lzo"           => '\u{f410}', // 
            "m"             => '\u{e61e}', // 
            "mm"            => '\u{e61d}', // 
            "m3u"           => '\u{f910}', // 蘿
            "m3u8"          => '\u{f910}', // 蘿
            "m4a"           => '\u{f001}', // 
            "m4v"           => '\u{f008}', // 
            "magnet"        => '\u{f076}', // 
            "markdown"      => '\u{f48a}', // 
            "md"            => '\u{f48a}', // 
            "md5"           => '\u{f023}', // 
            "mjs"           => '\u{e74e}', // 
            "mk"            => '\u{f489}', // 
            "mkd"           => '\u{f48a}', // 
            "mkv"           => '\u{f008}', // 
            "mobi"          => '\u{e28b}', // 
            "mov"           => '\u{f008}', // 
            "mp3"           => '\u{f001}', // 
            "mp4"           => '\u{f008}', // 
            "msi"           => '\u{e70f}', // 
            "mustache"      => '\u{e60f}', // 
            "nix"           => '\u{f313}', // 
            "node"          => '\u{f898}', // 
            "npmignore"     => '\u{e71e}', // 
            "o"             => '\u{e624}', // 
            "odp"           => '\u{f1c4}', // 
            "ods"           => '\u{f1c3}', // 
            "odt"           => '\u{f1c2}', // 
            "ogg"           => '\u{f001}', // 
            "ogv"           => '\u{f008}', // 
            "opus"          => '\u{f001}', // 
            "org"           => '\u{e779}', // 
            "otf"           => '\u{f031}', // 
            "part"          => '\u{f43a}', // 
            "patch"         => '\u{f440}', // 
            "pdf"           => '\u{f1c1}', // 
            "phar"          => '\u{e608}', // 
            "php"           => '\u{e608}', // 
            "pkg"           => '\u{f187}', // 
            "pl"            => '\u{e769}', // 
            "plist"         => '\u{f302}', // 
            "plx"           => '\u{e769}', // 
            "pm"            => '\u{e769}', // 
            "png"           => '\u{f1c5}', // 
            "pod"           => '\u{e769}', // 
            "ppt"           => '\u{f1c4}', // 
            "pptx"          => '\u{f1c4}', // 
            "procfile"      => '\u{e21e}', // 
            "properties"    => '\u{e60b}', // 
            "ps1"           => '\u{f489}', // 
            "psd"           => '\u{e7b8}', // 
            "pub"           => '\u{e60a}', // 
            "pxm"           => '\u{f1c5}', // 
            "py"            => '\u{e606}', // 
            "pyc"           => '\u{e606}', // 
            "r"             => '\u{fcd2}', // ﳒ
            "rakefile"      => '\u{e21e}', // 
            "rar"           => '\u{f410}', // 
            "razor"         => '\u{f1fa}', // 
            "rb"            => '\u{e21e}', // 
            "rdata"         => '\u{fcd2}', // ﳒ
            "rdb"           => '\u{e76d}', // 
            "rdoc"          => '\u{f48a}', // 
            "rds"           => '\u{fcd2}', // ﳒ
            "readme"        => '\u{f48a}', // 
            "rlib"          => '\u{e7a8}', // 
            "rmd"           => '\u{f48a}', // 
            "rpm"           => '\u{f187}', // 
            "rproj"         => '\u{fac5}', // 鉶
            "rs"            => '\u{e7a8}', // 
            "rspec"         => '\u{e21e}', // 
            "rspec_parallel"=> '\u{e21e}', // 
            "rspec_status"  => '\u{e21e}', // 
            "rss"           => '\u{f09e}', // 
            "rtf"           => '\u{f15c}', // 
            "ru"            => '\u{e21e}', // 
            "rubydoc"       => '\u{e73b}', // 
            "s"             => '\u{e614}', // 
            "S"             => '\u{e614}', // 
            "sass"          => '\u{e603}', // 
            "scala"         => '\u{e737}', // 
            "scss"          => '\u{e603}', // 
            "sh"            => '\u{f489}', // 
            "sha1"          => '\u{f023}', // 
            "sha256"        => '\u{f023}', // 
            "shell"         => '\u{f489}', // 
            "sig"           => '\u{e60a}', // 
            "slim"          => '\u{e73b}', // 
            "sln"           => '\u{e70c}', // 
            "so"            => '\u{e624}', // 
            "sql"           => '\u{f1c0}', // 
            "sqlite3"       => '\u{e7c4}', // 
            "srt"           => '\u{f02d}', // 
            "sty"           => '\u{f034}', // 
            "styl"          => '\u{e600}', // 
            "stylus"        => '\u{e600}', // 
            "sub"           => '\u{f02d}', // 
            "sublime-package" => '\u{e7aa}', // 
            "sublime-session" => '\u{e7aa}', // 
            "svg"           => '\u{f1c5}', // 
            "swift"         => '\u{e755}', // 
            "sym"           => '\u{e624}', // 
            "t"             => '\u{e769}', // 
            "tar"           => '\u{f410}', // 
            "taz"           => '\u{f410}', // 
            "tbz"           => '\u{f410}', // 
            "tbz2"          => '\u{f410}', // 
            "tex"           => '\u{f034}', // 
            "tgz"           => '\u{f410}', // 
            "tiff"          => '\u{f1c5}', // 
            "tlz"           => '\u{f410}', // 
            "toml"          => '\u{e615}', // 
            "torrent"       => '\u{f98c}', // 歷
            "ts"            => '\u{e628}', // 
            "tsv"           => '\u{f1c3}', // 
            "tsx"           => '\u{e7ba}', // 
            "ttc"           => '\u{f031}', // 
            "ttf"           => '\u{f031}', // 
            "twig"          => '\u{e61c}', // 
            "txt"           => '\u{f15c}', // 
            "txz"           => '\u{f410}', // 
            "tz"            => '\u{f410}', // 
            "tzo"           => '\u{f410}', // 
            "video"         => '\u{f008}', // 
            "vim"           => '\u{e62b}', // 
            "vlc"           => '\u{f910}', // 蘿
            "vue"           => '\u{fd42}', // ﵂
            "war"           => '\u{e256}', // 
            "wav"           => '\u{f001}', // 
            "webm"          => '\u{f008}', // 
            "webp"          => '\u{f1c5}', // 
            "windows"       => '\u{f17a}', // 
            "wma"           => '\u{f001}', // 
            "wmv"           => '\u{f008}', // 
            "woff"          => '\u{f031}', // 
            "woff2"         => '\u{f031}', // 
            "xbps"          => '\u{f187}', // 
            "xcf"           => '\u{f1c5}', // 
            "xhtml"         => '\u{f13b}', // 
            "xls"           => '\u{f1c3}', // 
            "xlsx"          => '\u{f1c3}', // 
            "xml"           => '\u{f121}', // 
            "xul"           => '\u{f121}', // 
            "xz"            => '\u{f410}', // 
            "yaml"          => '\u{f481}', // 
            "yml"           => '\u{f481}', // 
            "zip"           => '\u{f410}', // 
            "zsh"           => '\u{f489}', // 
            "zsh-theme"     => '\u{f489}', // 
            "zshrc"         => '\u{f489}', // 
            "zst"           => '\u{f410}', // 
            _               => '\u{f15b}'  // 
        }
    }
    else {
        '\u{f016}'
    }
}
