use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, Read},
    path::Path,
};

pub const HYDROGEN_DEFAULT_LANG: &str = "en-US";

pub enum HydrogenLangError {
    SerdeError(serde_json::Error),
    IOError(io::Error),
}

pub struct HydrogenLang {
    pub langs: HashMap<String, HashMap<String, HashMap<String, String>>>,
    pub default_lang: String,
}

impl HydrogenLang {
    pub fn new(default_lang: &str) -> HydrogenLang {
        HydrogenLang {
            langs: HashMap::new(),
            default_lang: default_lang.to_owned(),
        }
    }

    pub fn parse_string(&mut self, data: &str, lang: &str) -> Result<(), HydrogenLangError> {
        let lang_data: HashMap<String, HashMap<String, String>> = match serde_json::from_str(data) {
            Ok(ok) => ok,
            Err(e) => return Err(HydrogenLangError::SerdeError(e)),
        };
        self.langs.insert(lang.to_owned(), lang_data);

        Ok(())
    }

    pub fn parse_reader<R>(&mut self, data: R, lang: &str) -> Result<(), HydrogenLangError>
    where
        R: Read,
    {
        let lang_data: HashMap<String, HashMap<String, String>> =
            match serde_json::from_reader(data) {
                Ok(ok) => ok,
                Err(e) => return Err(HydrogenLangError::SerdeError(e)),
            };
        self.langs.insert(lang.to_owned(), lang_data);

        Ok(())
    }

    pub fn parse_dir(&mut self, path: &str) -> Result<(), HydrogenLangError> {
        let files = match fs::read_dir(path) {
            Ok(ok) => ok,
            Err(e) => return Err(HydrogenLangError::IOError(e)),
        };

        for file in files {
            let Ok(file) = file else {
                continue;
            };
            let file_name = file.file_name();
            let Some(file_prefix) = Path::new(&file_name).file_stem() else {
                continue;
            };
            let file_path = Path::new(path).join(&file_name);
            let Ok(file_stream) = File::open(file_path) else {
                continue;
            };
            let Some(file_prefix_str) = file_prefix.to_str() else {
                continue;
            };
            _ = self.parse_reader(file_stream, file_prefix_str);
        }

        Ok(())
    }

    pub fn get(&self, lang: &str, category: &str, key: &str, vars: &[(&str, &str)]) -> String {
        if let Some(language_map) = self.langs.get(lang) {
            if let Some(category_map) = language_map.get(category) {
                if let Some(value) = category_map.get(key) {
                    let mut value = value.to_string();
                    for var in vars {
                        value = value.replace(&format!("${}", var.0), var.1);
                    }
                    return value;
                }
            }
        }

        if lang != self.default_lang {
            return self.get(&self.default_lang, category, key, vars);
        }

        format!("{}.{}", category, key)
    }
}
