#![doc = include_str!("../README.md")]

pub mod error;
pub mod error_messages;
mod extensions;
mod format_dependant;
mod utils;

use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[cfg(not(any(feature = "json", feature = "json5", feature = "toml", feature = "yaml")))]
compile_error!("You must install at least one format feature: `json`, `json5`, `toml`, or `yaml`");
// ^ --- HEY, user! --- ^
// To do this, you can replace `fast_config = ".."` with
// `fast_config = { version = "..", features = ["json"] }` in your cargo.toml file.
// You can simply replace that "json" with any of the stated above if you want other formats.

// Bug testing
#[cfg(test)]
mod tests;

// Separated things
#[allow(unused)]
pub use error_messages::*;

/// Enum used to configure the [`Config`]s file format.
///
/// You can use it in a [`ConfigSetupOptions`], inside [`Config::from_options`]
///
/// ## ⚠️ Make sure to enable the feature flag for a format before using it!
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ConfigFormat {
    JSON,
    JSON5,
    TOML,
    YAML,
}

impl ConfigFormat {
    /// Mainly used to convert file extensions into [`ConfigFormat`]s <br/>
    /// Also chooses the correct extension for both JSON types based on the enabled feature. _(ex: if JSON5 is enabled, it chooses it for the "json" file extension)_ <br/>
    /// Returns [`None`] if the string/extension doesn't match any known format.
    ///
    /// # Example:
    /// ```
    /// # use std::ffi::OsStr;
    /// # use fast_config::ConfigFormat;
    /// if cfg!(feature = "json") {
    ///     assert_eq!(
    ///         ConfigFormat::from_extension(OsStr::new("json")).unwrap(),
    ///         ConfigFormat::JSON
    ///     );
    /// } else if cfg!(feature = "json5") {
    ///     assert_eq!(
    ///         ConfigFormat::from_extension(OsStr::new("json5")).unwrap(),
    ///         ConfigFormat::JSON5
    ///     );
    /// }
    /// ```
    pub fn from_extension(ext: &OsStr) -> Option<Self> {
        let ext = ext
            .to_ascii_lowercase()
            .to_string_lossy()
            .replace('\u{FFFD}', "");

        // Special case for JSON5 since it shares a format with JSON
        if cfg!(feature = "json5") && !cfg!(feature = "json") && (ext == "json" || ext == "json5") {
            return Some(ConfigFormat::JSON5);
        }
        
        // Matching
        match ext.as_str() {
            "json" => Some(ConfigFormat::JSON),
            "toml" => Some(ConfigFormat::TOML),
            "yaml" | "yml" => Some(ConfigFormat::YAML),
            _ => None,
        }
    }
}

impl Display for ConfigFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            ConfigFormat::JSON => "json",
            ConfigFormat::JSON5 => "json5",
            ConfigFormat::TOML => "toml",
            ConfigFormat::YAML => "yaml",
        };
        write!(f, "{output}")
    }
}

impl Default for ConfigFormat {
    fn default() -> Self {
        format_dependant::get_first_enabled_feature()
    }
}

/// Used to configure the [`Config`] object
///
/// [`UnknownFormatError`]: error::UnknownFormatError
///
/// # Attributes
/// - `pretty` - Makes the contents of the config file more humanly-readable.
///   When `false`, it will try to compact down the config file data so it takes up less storage space.
///   I recommend you keep it on unless you know what you're doing as most modern systems have enough
///   space to handle spaces and newline characters even at scale.
///
/// - `format` - An [`Option`] containing an enum of type [`ConfigFormat`].
///   Used to specify the format language to use *(ex: JSON, TOML, etc.)* <br/>
///   If you don't select a format *(Option::None)* it will try to guess the format
///   based on the file extension and enabled features. <br/>
///   If this step fails, an [`UnknownFormatError`] will be returned.
///
/// # More options are to be added later!
/// Pass `.. `[`Default::default()`] at the end of your construction
/// to prevent yourself from getting errors in the future!
///
/// # Examples:
/// ```
/// use fast_config::{ConfigSetupOptions, ConfigFormat, Config};
/// use serde::{Serialize, Deserialize};
///
/// // Creating a config struct to store our data
/// #[derive(Serialize, Deserialize)]
/// pub struct MyData {
///     pub some_data: i32
/// }
///
/// // Creating the options
/// let options = ConfigSetupOptions {
///     pretty: false,
///     format: Some(ConfigFormat::JSON),
///     .. Default::default()
/// };
///
/// // Creating the data and setting it's default values
/// let data = MyData {
///     some_data: 12345
/// };
///
/// // Creating the config itself
/// let mut config = Config::from_options("./config/myconfig", options, data).unwrap();
/// // [.. do stuff here]
/// # // Cleanup
/// # match std::fs::remove_dir_all("./config/") {
/// #     Err(e) => {
/// #        log::error!("{e}");
/// #     },
/// #     Ok(_) => {}
/// # }
/// ```
#[derive(Clone, Copy)]
pub struct ConfigSetupOptions {
    pub pretty: bool,
    pub format: Option<ConfigFormat>,

    #[allow(deprecated)]
    #[deprecated(note = "This option can result in I/O during program exit and can potentially corrupt config files!\nUse [`Config::save`] while your program is exiting instead!")]
    pub save_on_drop: bool,
}

impl Default for ConfigSetupOptions {
    fn default() -> Self {
        #[allow(deprecated)]
        Self {
            pretty: true,
            format: None,
            save_on_drop: false,
        }
    }
}

/// The internally-stored settings type for [`Config`] <br/>
/// Works and looks like [`ConfigSetupOptions`], with a few internally-required key differences.
pub struct InternalOptions {
    pub pretty: bool,
    pub format: ConfigFormat,
    pub save_on_drop: bool,
}
impl TryFrom<ConfigSetupOptions> for InternalOptions {
    /// This function converts a [`ConfigSetupOptions`] into an internally-used [`InternalOptions`].
    ///
    /// This function is not recommended to be used outside the `fast_config` source code
    /// unless you know what you're doing and accept the risks. <br/>
    /// The signature or behaviour of the function may be modified in the future.
    type Error = String;

    fn try_from(options: ConfigSetupOptions) -> Result<Self, Self::Error> {
        // Getting the formatting language.
        let format = match options.format {
            Some(format) => format,
            None => Err("The file format could not be guessed! It appears to be None!")?,
        };

        // Constructing a converted type
        Ok(Self {
            pretty: options.pretty,
            format,
            #[allow(deprecated)] save_on_drop: options.save_on_drop,
        })
    }
}

/// The main class you use to create/access your configuration files!
///
/// # Construction
/// See [`Config::new`] and [`Config::from_options`] if you wish to construct a new `Config`!
///
/// # Data
/// This class stores data within a data struct you define yourself.
/// This allows for the most amount of performance and safety,
/// while also allowing you to add additional features by adding `impl` blocks on your struct.
///
/// Your data struct needs to implement [`Serialize`] and [`Deserialize`].
/// In most cases you can just use `#[derive(Serialize, Deserialize)]` to derive them.
///
/// # Examples
/// Here is a code example on how you could define the data to pass into the constructors on this class:
/// ```
/// use serde::{Serialize, Deserialize};
///
/// // Creating a config struct to store our data
/// #[derive(Serialize, Deserialize)]
/// struct MyData {
///     pub student_debt: i32,
/// }
///
/// // Making our data and setting its default values
/// let data = MyData { 
///     student_debt: 20
/// };
/// // ..
/// ```
/// Implementing [`Serialize`] and [`Deserialize`] yourself is quite complicated but will provide the most flexibility.
///
/// *If you wish to implement them yourself I'd recommend reading the Serde docs on it*
///
pub struct Config<D>
where
    for<'a> D: Deserialize<'a> + Serialize,
{
    pub data: D,
    pub path: PathBuf,
    pub options: InternalOptions,
}

impl<D> Config<D>
where
    for<'a> D: Deserialize<'a> + Serialize,
{
    /// Constructs and returns a new config object using the default options.
    ///
    /// If there is a file at `path`, the file will be opened. <br/>
    ///
    /// - `path`: Takes in a path to where the config file is or should be located.
    ///   If the file has no extension, the crate will attempt to guess the extension from one available format `feature`.
    ///
    /// - `data`: Takes in a struct that inherits [`Serialize`] and [`Deserialize`]
    ///   You have to make this struct yourself, construct it, and pass it in.
    ///   More info about it is provided at [`Config`].
    ///
    /// If you'd like to configure this object, you should take a look at using [`Config::from_options`] instead.
    pub fn new(path: impl AsRef<Path>, data: D) -> Result<Config<D>, error::ConfigError> {
        Self::construct(path, ConfigSetupOptions::default(), data)
    }

    /// Constructs and returns a new config object from a set of custom options.
    ///
    /// - `path`: Takes in a path to where the config file is or should be located. <br/>
    ///   If the file has no extension, and there is no `format` selected in your `options`,
    ///   the crate will attempt to guess the extension from one available format `feature`s.
    //
    /// - `options`: Takes in a [`ConfigSetupOptions`],
    ///   used to configure the format language, styling of the data, and other things. <br/>
    ///   Remember to add `..` [`Default::default()`] at the end of your `options` as more options are
    ///   going to be added to the crate later on.
    ///
    /// - `data`: Takes in a struct that inherits [`Serialize`] and [`Deserialize`]
    ///   You have to make this struct yourself, construct it, and pass it in.
    ///   More info is provided at [`Config`].
    pub fn from_options(
        path: impl AsRef<Path>,
        options: ConfigSetupOptions,
        data: D,
    ) -> Result<Config<D>, error::ConfigError> {
        Self::construct(path, options, data)
    }

    // Main, private constructor
    fn construct(
        path: impl AsRef<Path>,
        mut options: ConfigSetupOptions,
        mut data: D,
    ) -> Result<Config<D>, error::ConfigError> {
        let mut path = PathBuf::from(path.as_ref());

        // Setting up variables
        let enabled_features = format_dependant::get_enabled_features();
        let first_enabled_feature = format_dependant::get_first_enabled_feature();
        let guess_from_feature = || {
            if enabled_features.len() > 1 {
                Err(error::ConfigError::UnknownFormat(
                    error::UnknownFormatError::new(None, enabled_features.clone()),
                ))
            } else {
                Ok(Some(first_enabled_feature))
            }
        };

        // Manual format option  >  file extension  >  guessed feature
        if options.format.is_none() {
            options.format = match path.extension() {
                Some(extension) => {
                    // - Based on the extension
                    match ConfigFormat::from_extension(extension) {
                        Some(value) => Some(value),
                        None => guess_from_feature()?,
                    }
                }
                _ => {
                    // - Guessing based on the enabled features
                    guess_from_feature()?
                }
            };
        }

        // Converting the user options into a more convenient internally-used type
        let options: InternalOptions = match InternalOptions::try_from(options) {
            Ok(value) => value,
            Err(message) => {
                return Err(error::ConfigError::UnknownFormat(
                    error::UnknownFormatError::new(Some(message), enabled_features),
                ));
            }
        };

        // Setting the file format
        if path.extension().is_none() {
            path.set_extension(options.format.to_string());
        }

        // Reading from the file if a file was found
        if let Ok(mut file) = fs::File::open(&path) {
            let mut content = String::new();
            if let Err(err) = file.read_to_string(&mut content) {
                return Err(error::ConfigError::InvalidFileEncoding(err, path));
            };

            // Deserialization
            // (Getting data from a string)
            if let Ok(value) = format_dependant::from_string(&content, &options.format) {
                data = value;
            } else {
                return Err(error::ConfigError::DataParseError(
                    error::DataParseError::Deserialize(options.format, content),
                ));
            };
        }

        // Returning the Config object

        Ok(Self {
            data,
            path,
            options,
        })
    }

    /// Saves the config file to the disk.
    ///
    /// It uses the [`Config`]'s object own internal `path` property to get the path required to save the file
    /// so there is no need to pass in the path to save it at.
    ///
    /// If you wish to specify the path to save it at
    /// you can change the path yourself by setting the Config's `path` property.
    /// <br/> <br/>
    /// ## save_at method
    /// There used to be a built-in function called `save_at` while i was developing the crate,
    /// but I ended up removing it due to the fact i didn't see anyone actually using it,
    /// and it might've ended up in some users getting confused, as well as a tiny bit of performance overhead.
    ///
    /// If you'd like this feature to be back feel free to open an issue and I'll add it back right away!
    pub fn save(&self) -> Result<(), error::ConfigSaveError> {
        let to_string = format_dependant::to_string(&self.data, &self.options);
        match to_string {
            // If the conversion was successful
            Ok(data) => {
                if let Some(parent_dir) = self.path.parent() {
                    fs::create_dir_all(parent_dir)?;
                };

                let mut file = fs::File::create(&self.path)?;

                write!(file, "{data}")?;
            }
            // If the conversion failed
            Err(e) => {
                // This error triggering sometimes seems to mean a data type you're using in your
                // custom data struct isn't supported, but I haven't fully tested it.
                return Err(error::ConfigSaveError::SerializationError(e));
            }
        };
        Ok(())
    }

    /// Gets the name of the config file
    pub fn filename(&self) -> String {
        self.path.file_name().unwrap().to_string_lossy().to_string()
    }
}

impl<D> Drop for Config<D>
where
    for<'a> D: Deserialize<'a> + Serialize,
{
    fn drop(&mut self) {
        if self.options.save_on_drop {
            let _ = self.save();
        }
    }
}
