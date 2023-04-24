use std::{collections::HashMap, env};

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

    let pt_br = String::from("pt-BR");
    let category = String::from("test");
    let translate_key = String::from("translate");

    assert_eq!(
        i18n.get(&pt_br, &category, &translate_key, None),
        String::from("Valor a ser traduzido")
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

    let pt_br = String::from("pt-BR");
    let category = String::from("test");
    let only_english_key = String::from("only_english");

    assert_eq!(
        i18n.get(&pt_br, &category, &only_english_key, None),
        String::from("This value only exists in english")
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

    let pt_br = String::from("pt-BR");
    let category = String::from("test");
    let variable_key = String::from("variable");

    assert_eq!(
        i18n.get(
            &pt_br,
            &category,
            &variable_key,
            Some(HashMap::from([(
                String::from("HYDROGEN_VERSION"),
                String::from("0.0.1")
            )]))
        ),
        String::from("Essa chave foi criada em 0.0.1")
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

    let pt_br = String::from("pt-BR");
    let category = String::from("test");
    let nexists_key = String::from("nexists");

    assert_eq!(
        i18n.get(&pt_br, &category, &nexists_key, None),
        String::from("test.nexists")
    );
}
