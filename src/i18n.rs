use std::{collections::HashMap, result, fmt::Display, fs::{read_dir, File}, path::Path, io, sync::Arc};

use serenity::builder::{CreateApplicationCommand, CreateApplicationCommandOption};

#[derive(Debug)]
pub enum HydrogenI18nError {
    DefaultLanguageNotFound,
    Io(io::Error)
}

impl Display for HydrogenI18nError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HydrogenI18nError::DefaultLanguageNotFound => write!(f, "default language file doesn't exists"),
            HydrogenI18nError::Io(e) => write!(f, "{}", e)
        }
    }
}

pub type Result<T> = result::Result<T, HydrogenI18nError>;

type LanguageCache = HashMap<String, Language>;
type Language = HashMap<String, Category>;
type Category = HashMap<String, String>;

#[derive(Clone)]
pub struct HydrogenI18n {
    cache: Arc<LanguageCache>,
    default_language: String
}

impl HydrogenI18n {
    pub const DEFAULT_LANGUAGE: &str = "en-US";

    pub fn new<P>(path: P, default_language: &str) -> Result<Self>
        where
            P: AsRef<Path>
    {
        let mut cache = HashMap::new();

        let dirs = read_dir(path).map_err(|e| HydrogenI18nError::Io(e))?;

        for file in dirs {
            let Ok(file) = file else {
                continue;
            };

            let Ok(file_name) = file.file_name().into_string() else {
                continue;
            };

            let Some(language) = file_name.strip_suffix(".json") else {
                continue;
            };

            let Ok(file_stream) = File::open(file.path()) else {
                continue;
            };

            let Ok(data) = serde_json::from_reader::<_, Language>(file_stream) else {
                continue;
            };

            cache.insert(language.to_owned(), data);
        }

        if !cache.contains_key(default_language) {
            return Err(HydrogenI18nError::DefaultLanguageNotFound);
        }

        Ok(Self {
            default_language: default_language.to_owned(),
            cache: Arc::new(cache)
        })
    }

    fn get(&self, lang: &str, category: &str, name: &str) -> Option<String> {
        self.cache.get(lang)?.get(category)?.get(name).cloned()
    }

    fn translate_with_default(&self, category: &str, name: &str) -> String {
        self.get(&self.default_language, category, name).unwrap_or(format!("{}.{}", category, name))
    }

    pub fn translate(&self, lang: &str, category: &str, name: &str) -> String {
        self.get(lang, category, name).unwrap_or(self.translate_with_default(category, name))
    }

    pub fn translate_application_command_name<'a>(&self, category: &str, name: &str, application_command: &'a mut CreateApplicationCommand) {
        for lang in self.cache.keys() {
            if let Some(value) = self.get(lang, category, name) {
                application_command.name_localized(lang, value);
            }
        }
    }

    pub fn translate_application_command_description<'a>(&self, category: &str, name: &str, application_command: &'a mut CreateApplicationCommand) {
        for lang in self.cache.keys() {
            if let Some(value) = self.get(lang, category, name) {
                application_command.description_localized(lang, value);
            }
        }
    }

    pub fn translate_application_command_option_name<'a>(&self, category: &str, name: &str, application_command_option: &'a mut CreateApplicationCommandOption) {
        for lang in self.cache.keys() {
            if let Some(value) = self.get(lang, category, name) {
                application_command_option.name_localized(lang, value);
            }
        }
    }

    pub fn translate_application_command_option_description<'a>(&self, category: &str, name: &str, application_command_option: &'a mut CreateApplicationCommandOption) {
        for lang in self.cache.keys() {
            if let Some(value) = self.get(lang, category, name) {
                application_command_option.description_localized(lang, value);
            }
        }
    }
}