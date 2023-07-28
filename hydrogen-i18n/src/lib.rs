#![cfg_attr(not(feature = "std"), no_std)]

/// Recommended default language commonly used by Discord (en_US).
pub const DEFAULT_LANGUAGE: &str = "en-US";

#[cfg(feature = "std")]
use std::{
    collections::HashMap as Map,
    fmt::{self, Display, Formatter},
    fs::{read_dir, DirEntry, File},
    io,
    path::Path,
    result,
    sync::Arc,
};

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{
    borrow::ToOwned,
    collections::BTreeMap as Map,
    fmt::{self, Display, Formatter},
    format,
    string::String,
    sync::Arc,
};

#[cfg(feature = "serenity")]
use serenity::builder::{CreateApplicationCommand, CreateApplicationCommandOption};

#[cfg(feature = "std")]
/// An enum containing the different types of errors, from different sources, that can occur.
#[derive(Debug)]
pub enum Error {
    /// Error related to the deserialization of a JSON document through `serde_json`.
    Serde(serde_json::Error),

    /// Error related to reading a file or folder from the operating system.
    Io(io::Error),
}

#[cfg(feature = "std")]
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serde(e) => e.fmt(f),
            Self::Io(e) => write!(f, "{}", e),
        }
    }
}

#[cfg(feature = "std")]
/// Just a `Result` with the error type set to `hydrogen-i18n::Error`.
pub type Result<T> = result::Result<T, Error>;

/// Just a key-value map, where the key is a language code and its value is a `Language`.
pub type Cache = Map<String, Language>;
/// Just a key-value map, where the key is the category name and its value is a `Category`.
pub type Language = Map<String, Category>;
/// Just a key-value map, where the key is a translation key and its value is the translation itself.
pub type Category = Map<String, String>;

/// Read the contents of the file, transforming it into a `Language` that can be inserted into a `Cache`.
#[cfg(feature = "std")]
pub fn load_file<P>(path: P) -> Result<Language>
where
    P: AsRef<Path>,
{
    let file = File::open(path).map_err(Error::Io)?;
    let data = serde_json::from_reader::<_, Language>(file).map_err(Error::Serde)?;
    Ok(data)
}

/// It loads a single entry within a directory, this method exists to group the errors generated during this process as they should not finish loading the directory as a whole.
#[cfg(feature = "std")]
fn load_dir_entry(entry: io::Result<DirEntry>) -> Option<(String, Language)> {
    let file = entry.ok()?;
    let file_name = file.file_name().into_string().ok()?;
    let language = file_name.strip_suffix(".json")?;
    Some((language.to_owned(), load_file(file.path()).ok()?))
}

/// Loads an entire directory of JSON documents, using the file name without extension (.json) as the language code.
#[cfg(feature = "std")]
pub fn load_dir<P>(path: P) -> Result<Cache>
where
    P: AsRef<Path>,
{
    let mut cache = Cache::new();
    let dir = read_dir(path).map_err(Error::Io)?;

    for entry in dir {
        if let Some((language, data)) = load_dir_entry(entry) {
            cache.insert(language, data);
        }
    }

    Ok(cache)
}

/// Just a struct used by `Translator` to allow the use of a shared reference but still allow cloning of the contents of that reference.
struct Internal {
    /// Cache containing all languages that have been loaded into memory for use by a `Translator`.
    cache: Cache,

    /// Default language to be used by `Translator` if a certain language is not found.
    default_language: String,
}

/// It manages all the translations stored internally, guaranteeing that there will always be a translation available for the category-key combination, in addition to containing utility methods for working with certain tools.
///
/// This struct already implements `Arc` internally and therefore cloning it will not create a copy of its contents, you can use `cloned()` if you want to copy the contents.
///
/// Remember that the `Translator` will not initialize or allow the internal `Cache` to be written, for that it will be necessary to create a new `Translator`.
#[derive(Clone)]
pub struct Translator {
    /// `Translator` internal memory, responsible for allowing access to internal properties without them being wrapped by an `Arc`.
    internal: Arc<Internal>,
}

impl Translator {
    /// Initializes a new `Translator` with the desired default language and a `Cache` already populated.
    pub fn new(default_language: &str, cache: Cache) -> Self {
        Self {
            internal: Arc::new(Internal {
                cache,
                default_language: default_language.to_owned(),
            }),
        }
    }

    /// Initializes a new `Translator` by providing a clone of the contents of the internal properties of that `Translator`.
    pub fn cloned(&self) -> Self {
        Self {
            internal: Arc::new(Internal {
                cache: self.cache(),
                default_language: self.default_language(),
            }),
        }
    }

    /// Creates a clone of the internal `Cache` which can be manipulated and used to initialize a new `Translator`.
    pub fn cache(&self) -> Cache {
        self.internal.cache.clone()
    }

    /// Gets a clone of the default language used by this `Translator`.
    pub fn default_language(&self) -> String {
        self.internal.default_language.clone()
    }

    /// Gets the translation of the category-key combination for the selected language, if there is a translation.
    pub fn get(&self, lang: &str, category: &str, name: &str) -> Option<String> {
        self.internal
            .cache
            .get(lang)?
            .get(category)?
            .get(name)
            .cloned()
    }

    /// Gets all translations for the category-key combination, returning a `HashMap` or a `BTreeMap` if the `std` feature is disabled, containing the language code as the key and the translation as the value.
    pub fn get_all(&self, category: &str, name: &str) -> Map<String, String> {
        let mut translations = Map::new();
        for lang in self.internal.cache.keys() {
            if let Some(value) = self.get(lang, category, name) {
                translations.insert(lang.clone(), value);
            }
        }
        translations
    }

    /// Translates using the default language of this `Translator`, returning `category.key` if no translation is found.
    fn translate_with_default(&self, category: &str, name: &str) -> String {
        self.get(&self.internal.default_language, category, name)
            .unwrap_or(format!("{}.{}", category, name))
    }

    /// Translates into the selected language, returning the translation in the default language or `category.key` if no translation is found.
    pub fn translate(&self, lang: &str, category: &str, name: &str) -> String {
        self.get(lang, category, name)
            .unwrap_or(self.translate_with_default(category, name))
    }

    /// Sets the localized name of the application command for each of the languages that have a translation for the category-key combination.
    #[cfg(feature = "serenity")]
    pub fn translate_command_name<'a>(
        &self,
        category: &str,
        name: &str,
        application_command: &'a mut CreateApplicationCommand,
    ) {
        for (locale, value) in self.get_all(category, name) {
            application_command.name_localized(locale, value);
        }
    }

    /// Sets the localized description of the application command for each of the languages that have a translation for the category-key combination.
    #[cfg(feature = "serenity")]
    pub fn translate_command_description<'a>(
        &self,
        category: &str,
        name: &str,
        application_command: &'a mut CreateApplicationCommand,
    ) {
        for (locale, value) in self.get_all(category, name) {
            application_command.description_localized(locale, value);
        }
    }

    // Sets the localized name of the application command option for each of the languages that have a translation for the category-key combination.
    #[cfg(feature = "serenity")]
    pub fn translate_option_name<'a>(
        &self,
        category: &str,
        name: &str,
        application_command_option: &'a mut CreateApplicationCommandOption,
    ) {
        for (locale, value) in self.get_all(category, name) {
            application_command_option.name_localized(locale, value);
        }
    }

    /// Sets the localized description of the application command option for each of the languages that have a translation for the category-key combination.
    #[cfg(feature = "serenity")]
    pub fn translate_option_description<'a>(
        &self,
        category: &str,
        name: &str,
        application_command_option: &'a mut CreateApplicationCommandOption,
    ) {
        for (locale, value) in self.get_all(category, name) {
            application_command_option.description_localized(locale, value);
        }
    }
}

impl From<Cache> for Translator {
    fn from(value: Cache) -> Self {
        Self::new(DEFAULT_LANGUAGE, value)
    }
}
