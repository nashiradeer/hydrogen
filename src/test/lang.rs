use std::env;

use crate::lang::{HydrogenLang, HydrogenLangError, HYDROGEN_DEFAULT_LANG};

#[test]
fn lang_translate() {
    let Ok(lang_path) = env::var("HYDROGEN_LANG_PATH") else {
        panic!("Lang: Invalid HYDROGEN_LANG_PATH variable in environmental")
    };

    let mut i18n = HydrogenLang::new(&String::from(HYDROGEN_DEFAULT_LANG));

    if let Err(e) = i18n.parse_dir(&lang_path) {
        match e {
            HydrogenLangError::IOError(err) => panic!("{:?}", err),
            HydrogenLangError::SerdeError(err) => panic!("{:?}", err),
        }
    }

    assert_eq!(
        i18n.get("pt-BR", "test", "translate", &[]),
        "Valor a ser traduzido"
    );
}

#[test]
fn lang_only_english() {
    let Ok(lang_path) = env::var("HYDROGEN_LANG_PATH") else {
        panic!("Lang: Invalid HYDROGEN_LANG_PATH variable in environmental")
    };

    let mut i18n = HydrogenLang::new(&String::from(HYDROGEN_DEFAULT_LANG));

    if let Err(e) = i18n.parse_dir(&lang_path) {
        match e {
            HydrogenLangError::IOError(err) => panic!("{:?}", err),
            HydrogenLangError::SerdeError(err) => panic!("{:?}", err),
        }
    }

    assert_eq!(
        i18n.get("pt-BR", "test", "only_english", &[]),
        "This value only exists in english"
    );
}

#[test]
fn lang_variable() {
    let Ok(lang_path) = env::var("HYDROGEN_LANG_PATH") else {
        panic!("Lang: Invalid HYDROGEN_LANG_PATH variable in environmental")
    };

    let mut i18n = HydrogenLang::new(&String::from(HYDROGEN_DEFAULT_LANG));

    if let Err(e) = i18n.parse_dir(&lang_path) {
        match e {
            HydrogenLangError::IOError(err) => panic!("{:?}", err),
            HydrogenLangError::SerdeError(err) => panic!("{:?}", err),
        }
    }

    assert_eq!(
        i18n.get(
            "pt-BR",
            "test",
            "variable",
            &[("HYDROGEN_VERSION", "0.0.1")]
        ),
        "Essa chave foi criada em 0.0.1"
    );
}

#[test]
fn lang_nexists() {
    let Ok(lang_path) = env::var("HYDROGEN_LANG_PATH") else {
        panic!("Lang: Invalid HYDROGEN_LANG_PATH variable in environmental")
    };

    let mut i18n = HydrogenLang::new(&String::from(HYDROGEN_DEFAULT_LANG));

    if let Err(e) = i18n.parse_dir(&lang_path) {
        match e {
            HydrogenLangError::IOError(err) => panic!("{:?}", err),
            HydrogenLangError::SerdeError(err) => panic!("{:?}", err),
        }
    }

    assert_eq!(i18n.get("pt-BR", "test", "nexists", &[]), "test.nexists");
}
