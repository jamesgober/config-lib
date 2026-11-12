//! # Options for loading configuration
//!
//! [`LoadOptions`] holds the limits and the include policy that every load
//! applies. The defaults are safe for files the application owns:
//!
//! - files larger than 16 MiB, and anything that is not a regular file
//!   (a directory, a FIFO, a device), are refused;
//! - NOML `include` may only read files inside the directory of the file
//!   being loaded (or the current directory, for text);
//! - NOML `${...}` references may expand the document to at most 64 MiB,
//!   with at most 10,000 references and chains at most 32 references deep.
//!
//! [`LoadOptions::strict`] turns on strict NOML for files and hot reload as
//! well: `include` and `env()` are refused (see
//! [`Config::from_string_strict`](crate::Config::from_string_strict)).
//!
//! Since v1.4.0.

use crate::error::{Error, Result};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

/// Default largest configuration file read: 16 MiB.
pub const DEFAULT_MAX_FILE_SIZE: u64 = 16 * 1024 * 1024;

/// Default largest size a NOML document may expand to through `${...}`:
/// 64 MiB.
pub const DEFAULT_MAX_EXPANDED_SIZE: u64 = 64 * 1024 * 1024;

/// Default largest number of `${...}` references in one NOML document.
pub const DEFAULT_MAX_EXPANSIONS: usize = 10_000;

/// Default longest chain of `${...}` references (a reference to a value
/// that is itself a reference, and so on).
pub const DEFAULT_MAX_EXPANSION_DEPTH: usize = 32;

/// Default largest number of files a NOML document may include, counted
/// over every level.
pub const DEFAULT_MAX_INCLUDES: usize = 64;

/// Deepest nesting of NOML includes (an included file that includes another
/// file, and so on). The same limit the noml crate applies.
#[cfg(feature = "noml")]
pub(crate) const MAX_INCLUDE_DEPTH: usize = 10;

/// Limits and include policy for loading configuration.
///
/// Build with [`LoadOptions::new`] (the safe defaults) and the builder
/// methods, then pass to [`Config::from_file_with`](crate::Config::from_file_with),
/// [`Config::from_string_with`](crate::Config::from_string_with),
/// [`ConfigBuilder::load_options`](crate::ConfigBuilder::load_options) or
/// [`HotReloadConfig::from_file_with`](crate::hot_reload::HotReloadConfig::from_file_with).
///
/// # Examples
///
/// ```rust
/// use config_lib::{Config, LoadOptions};
///
/// // Strict NOML for a file: no include, no env()
/// let options = LoadOptions::new().strict(true);
/// let config = Config::from_string_with("port = 8080", Some("conf"), &options)?;
/// assert_eq!(config.get_or("port", 0u16), 8080);
/// # Ok::<(), config_lib::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct LoadOptions {
    pub(crate) strict: bool,
    pub(crate) include_dirs: Vec<PathBuf>,
    pub(crate) includes_anywhere: bool,
    pub(crate) max_file_size: u64,
    pub(crate) max_includes: usize,
    pub(crate) max_expanded_size: u64,
    pub(crate) max_expansions: usize,
    pub(crate) max_expansion_depth: usize,
}

impl Default for LoadOptions {
    fn default() -> Self {
        Self {
            strict: false,
            include_dirs: Vec::new(),
            includes_anywhere: false,
            max_file_size: DEFAULT_MAX_FILE_SIZE,
            max_includes: DEFAULT_MAX_INCLUDES,
            max_expanded_size: DEFAULT_MAX_EXPANDED_SIZE,
            max_expansions: DEFAULT_MAX_EXPANSIONS,
            max_expansion_depth: DEFAULT_MAX_EXPANSION_DEPTH,
        }
    }
}

impl LoadOptions {
    /// The default options (see the [module documentation](self)).
    pub fn new() -> Self {
        Self::default()
    }

    /// Strict options: the defaults with [`LoadOptions::strict`] set.
    pub fn strict_defaults() -> Self {
        Self::default().strict(true)
    }

    /// Read NOML strictly: `include` and every function call (`env()`
    /// included) are refused with [`Error::Parse`]. Other formats read only
    /// their own text either way.
    pub fn strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }

    /// Whether NOML is read strictly.
    pub fn is_strict(&self) -> bool {
        self.strict
    }

    /// Also allow NOML includes of files inside `dir` (and its
    /// subdirectories). The directory of the file being loaded is always
    /// allowed. The check is made on the canonical path, so `..` and
    /// symbolic links cannot leave the allowed directories.
    pub fn allow_include_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.include_dirs.push(dir.into());
        self
    }

    /// Allow NOML includes of any regular file the process can read. Not
    /// recommended for files other users can write.
    pub fn allow_includes_anywhere(mut self) -> Self {
        self.includes_anywhere = true;
        self
    }

    /// Largest file read, in bytes, for the configuration file and each
    /// included file. Default [`DEFAULT_MAX_FILE_SIZE`].
    pub fn max_file_size(mut self, bytes: u64) -> Self {
        self.max_file_size = bytes;
        self
    }

    /// Largest number of included files, over all levels. Default
    /// [`DEFAULT_MAX_INCLUDES`].
    pub fn max_includes(mut self, count: usize) -> Self {
        self.max_includes = count;
        self
    }

    /// Largest size, in bytes, a NOML document may expand to through
    /// `${...}` references. Default [`DEFAULT_MAX_EXPANDED_SIZE`]. The size
    /// is an estimate that counts a small overhead for every value, so it
    /// is somewhat larger than the text of the values.
    pub fn max_expanded_size(mut self, bytes: u64) -> Self {
        self.max_expanded_size = bytes;
        self
    }

    /// Largest number of `${...}` references in a NOML document, included
    /// files counted. Default [`DEFAULT_MAX_EXPANSIONS`].
    pub fn max_expansions(mut self, count: usize) -> Self {
        self.max_expansions = count;
        self
    }

    /// Longest chain of `${...}` references. Default
    /// [`DEFAULT_MAX_EXPANSION_DEPTH`].
    pub fn max_expansion_depth(mut self, depth: usize) -> Self {
        self.max_expansion_depth = depth;
        self
    }
}

fn refuse(path: &Path, kind: io::ErrorKind, why: &str) -> Error {
    Error::io(
        path.display().to_string(),
        io::Error::new(kind, why.to_string()),
    )
}

/// Read a configuration file as text.
///
/// Refuses anything that is not a regular file and anything larger than
/// `max` bytes. The size is checked before reading and again while reading,
/// so a file that grows in between is still refused.
pub(crate) fn read_file(path: &Path, max: u64) -> Result<String> {
    let io_err = |e| Error::io(path.display().to_string(), e);
    let meta = std::fs::metadata(path).map_err(io_err)?;
    check_meta(path, &meta, max)?;
    let file = std::fs::File::open(path).map_err(io_err)?;
    let meta = file.metadata().map_err(io_err)?;
    check_meta(path, &meta, max)?;
    let mut bytes = Vec::with_capacity(usize::try_from(meta.len().min(max)).unwrap_or(0));
    file.take(max.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(io_err)?;
    finish(path, bytes, max)
}

/// Async version of [`read_file`]
#[cfg(feature = "async")]
pub(crate) async fn read_file_async(path: &Path, max: u64) -> Result<String> {
    use tokio::io::AsyncReadExt;
    let io_err = |e| Error::io(path.display().to_string(), e);
    let meta = tokio::fs::metadata(path).await.map_err(io_err)?;
    check_meta(path, &meta, max)?;
    let file = tokio::fs::File::open(path).await.map_err(io_err)?;
    let meta = file.metadata().await.map_err(io_err)?;
    check_meta(path, &meta, max)?;
    let mut bytes = Vec::with_capacity(usize::try_from(meta.len().min(max)).unwrap_or(0));
    file.take(max.saturating_add(1))
        .read_to_end(&mut bytes)
        .await
        .map_err(io_err)?;
    finish(path, bytes, max)
}

fn check_meta(path: &Path, meta: &std::fs::Metadata, max: u64) -> Result<()> {
    if !meta.is_file() {
        return Err(refuse(
            path,
            io::ErrorKind::InvalidInput,
            "not a regular file",
        ));
    }
    if meta.len() > max {
        return Err(refuse(
            path,
            io::ErrorKind::InvalidData,
            "file is larger than the size limit",
        ));
    }
    Ok(())
}

fn finish(path: &Path, bytes: Vec<u8>, max: u64) -> Result<String> {
    if bytes.len() as u64 > max {
        return Err(refuse(
            path,
            io::ErrorKind::InvalidData,
            "file is larger than the size limit",
        ));
    }
    String::from_utf8(bytes)
        .map_err(|_| refuse(path, io::ErrorKind::InvalidData, "file is not valid UTF-8"))
}

/// Where NOML includes are read from, for one load.
#[cfg(feature = "noml")]
#[derive(Debug, Clone)]
pub(crate) struct IncludePolicy {
    /// Directory relative includes in the top-level document resolve
    /// against
    pub(crate) base_dir: PathBuf,
    /// Canonical directories an included file must be inside
    roots: Vec<PathBuf>,
    anywhere: bool,
    pub(crate) max_file_size: u64,
    pub(crate) max_includes: usize,
}

#[cfg(feature = "noml")]
impl IncludePolicy {
    /// The policy for a document read from `file`, or from text when `file`
    /// is `None` (relative includes then resolve against the current
    /// directory, as they always have).
    pub(crate) fn new(file: Option<&Path>, options: &LoadOptions) -> Self {
        let base_dir = match file.and_then(Path::parent) {
            Some(dir) if !dir.as_os_str().is_empty() => dir.to_path_buf(),
            _ => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        };
        let mut roots = Vec::with_capacity(options.include_dirs.len() + 1);
        roots.extend(std::fs::canonicalize(&base_dir).ok());
        roots.extend(
            options
                .include_dirs
                .iter()
                .filter_map(|dir| std::fs::canonicalize(dir).ok()),
        );
        Self {
            base_dir,
            roots,
            anywhere: options.includes_anywhere,
            max_file_size: options.max_file_size,
            max_includes: options.max_includes,
        }
    }

    /// Resolve and read the file an `include` names.
    ///
    /// `from_dir` is the directory of the file holding the include. Returns
    /// the canonical path and the text, or a fixed reason that never quotes
    /// the path or the file's contents.
    pub(crate) fn read(
        &self,
        from_dir: &Path,
        include: &str,
    ) -> std::result::Result<(PathBuf, String), &'static str> {
        if include.starts_with("http://") || include.starts_with("https://") {
            return Err("remote includes are not supported");
        }
        let joined = from_dir.join(include);
        let canonical =
            std::fs::canonicalize(&joined).map_err(|_| "the included file does not exist")?;
        if !self.anywhere && !self.roots.iter().any(|root| canonical.starts_with(root)) {
            return Err("the included file is outside the allowed directories");
        }
        let text = read_file(&canonical, self.max_file_size).map_err(|e| match e {
            Error::Io { source, .. } if source.kind() == io::ErrorKind::InvalidInput => {
                "the included path is not a regular file"
            }
            Error::Io { source, .. } if source.kind() == io::ErrorKind::InvalidData => {
                "the included file is too large or not UTF-8"
            }
            _ => "the included file could not be read",
        })?;
        Ok((canonical, text))
    }
}
