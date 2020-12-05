use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

/// Wrapper for 'dirs' that treats macOS more like Linux, by following the XDG specification.
/// This means that the `XDG_CONFIG_HOME` environment variable is checked first.
/// The fallback directory is `~/.config/exa`.
fn config_dir() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    let config_dir_op = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| dirs::home_dir().map(|d| d.join(".config")));

    #[cfg(not(target_os = "macos"))]
    let config_dir_op = dirs::config_dir();

    config_dir_op.map(|d| d.join("exa"))
}

fn config_file() -> PathBuf {
    env::var("EXA_CONFIG_PATH")
        .ok()
        .map(PathBuf::from)
        .filter(|config_path| config_path.is_file())
        .unwrap_or_else(|| {
            config_dir()
                .expect("Could not get home directory")
                .join("config")
        })
}

pub fn get_args_from_config_file() -> Result<Vec<OsString>, shell_words::ParseError> {
    Ok(fs::read_to_string(config_file())
        .ok()
        .map(|content| get_args_from_str(&content))
        .transpose()?
        .unwrap_or_else(Vec::new))
}

pub fn get_args_from_env_var() -> Option<Result<Vec<OsString>, shell_words::ParseError>> {
    env::var("EXA_OPTS").ok().map(|s| get_args_from_str(&s))
}

fn get_args_from_str(content: &str) -> Result<Vec<OsString>, shell_words::ParseError> {
    let args_per_line = content
        .split('\n')
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .filter(|line| !line.starts_with('#'))
        .map(|line| shell_words::split(line))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(args_per_line
        .iter()
        .flatten()
        .map(|line| line.into())
        .collect())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn empty() {
        let args = get_args_from_str("").unwrap();
        assert!(args.is_empty());
    }

    #[test]
    fn single() {
        assert_eq!(vec!["--binary"], get_args_from_str("--binary").unwrap());
    }

    #[test]
    fn multiple() {
        assert_eq!(
            vec!["--binary", "--time-style=iso"],
            get_args_from_str("--binary --time-style=iso").unwrap()
        );
    }

    #[test]
    fn quotes() {
        assert_eq!(
            vec!["--time-style", "iso"],
            get_args_from_str("--time-style \"iso\"").unwrap()
        );
    }

    #[test]
    fn multi_line() {
        let config = "
        -l
        --sort newest
        --time-style=iso
        ";
        assert_eq!(
            vec!["-l", "--sort", "newest", "--time-style=iso"],
            get_args_from_str(config).unwrap()
        );
    }

    #[test]
    fn comments() {
        let config = "
        # Display file metadata as a table
        -l
        # Sort by newest
        --sort newest
        # Use ISO timestamp format
        --time-style=iso
        ";
        assert_eq!(
            vec!["-l", "--sort", "newest", "--time-style=iso"],
            get_args_from_str(config).unwrap()
        );
    }
}
