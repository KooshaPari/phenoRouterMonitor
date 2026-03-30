//! String inflection utilities for pluralization and singularization.

use std::collections::HashMap;
use std::sync::OnceLock;

/// Returns the plural form of a noun.
pub fn pluralize(word: &str) -> String {
    if word.is_empty() {
        return word.to_string();
    }

    if let Some(plural) = get_irregular_plurals().get(word) {
        return plural.to_string();
    }

    match word {
        w if ends_with_any(w, &["ss", "x", "z", "ch", "sh", "us"]) => {
            format!("{}es", word)
        }
        w if w.len() >= 2
            && w.ends_with('y')
            && is_consonant(w.chars().nth(w.len() - 2).unwrap()) =>
        {
            format!("{}ies", &word[..word.len() - 1])
        }
        w if w.ends_with("f") => format!("{}ves", &word[..word.len() - 1]),
        w if w.ends_with("fe") => format!("{}ves", &word[..word.len() - 2]),
        w => format!("{}s", w),
    }
}

/// Returns the singular form of a noun.
pub fn singularize(word: &str) -> String {
    if word.is_empty() {
        return word.to_string();
    }

    if let Some(singular) = get_irregular_singulars().get(word) {
        return singular.to_string();
    }

    match word {
        w if w.ends_with("ies") && w.len() > 3 => {
            format!("{}y", &word[..word.len() - 3])
        }
        w if w.ends_with("ves") && w.len() > 3 => {
            let stem = &word[..word.len() - 3];
            match stem.chars().last() {
                Some('a' | 'e' | 'i' | 'o' | 'u') => {
                    format!("{}fe", stem)
                }
                _ => format!("{}f", stem),
            }
        }
        w if w.ends_with("es") && w.len() > 2 => {
            let stem = &word[..word.len() - 2];
            if ends_with_any(stem, &["s", "x", "z", "ch", "sh"]) {
                stem.to_string()
            } else {
                word.to_string()
            }
        }
        w if w.ends_with('s') && !w.ends_with("ss") && w.len() > 1 => {
            word[..word.len() - 1].to_string()
        }
        w => w.to_string(),
    }
}

/// Inflection provides builder-style interface for inflection operations.
pub struct Inflection {
    word: String,
}

impl Inflection {
    pub fn new(word: &str) -> Self {
        Inflection {
            word: word.to_string(),
        }
    }

    pub fn pluralize(self) -> String {
        pluralize(&self.word)
    }

    pub fn singularize(self) -> String {
        singularize(&self.word)
    }

    pub fn is_plural(&self) -> bool {
        let singular = singularize(&self.word);
        singular != self.word
    }

    pub fn is_singular(&self) -> bool {
        let plural = pluralize(&self.word);
        plural != self.word && singularize(&plural) == self.word
    }
}

fn get_irregular_plurals() -> &'static HashMap<&'static str, &'static str> {
    static PLURALS: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    PLURALS.get_or_init(|| {
        let mut map = HashMap::new();
        map.insert("child", "children");
        map.insert("person", "people");
        map.insert("man", "men");
        map.insert("woman", "women");
        map.insert("tooth", "teeth");
        map.insert("foot", "feet");
        map.insert("goose", "geese");
        map.insert("mouse", "mice");
        map.insert("ox", "oxen");
        map.insert("deer", "deer");
        map.insert("fish", "fish");
        map.insert("sheep", "sheep");
        map.insert("moose", "moose");
        map.insert("crisis", "crises");
        map.insert("analysis", "analyses");
        map.insert("thesis", "theses");
        map.insert("phenomenon", "phenomena");
        map.insert("criterion", "criteria");
        map.insert("datum", "data");
        map
    })
}

fn get_irregular_singulars() -> &'static HashMap<&'static str, &'static str> {
    static SINGULARS: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    SINGULARS.get_or_init(|| {
        let mut map = HashMap::new();
        map.insert("children", "child");
        map.insert("people", "person");
        map.insert("men", "man");
        map.insert("women", "woman");
        map.insert("teeth", "tooth");
        map.insert("feet", "foot");
        map.insert("geese", "goose");
        map.insert("mice", "mouse");
        map.insert("oxen", "ox");
        map.insert("deer", "deer");
        map.insert("fish", "fish");
        map.insert("sheep", "sheep");
        map.insert("moose", "moose");
        map.insert("crises", "crisis");
        map.insert("analyses", "analysis");
        map.insert("theses", "thesis");
        map.insert("phenomena", "phenomenon");
        map.insert("criteria", "criterion");
        map.insert("data", "datum");
        map
    })
}

fn is_consonant(c: char) -> bool {
    matches!(
        c,
        'b' | 'c'
            | 'd'
            | 'f'
            | 'g'
            | 'h'
            | 'j'
            | 'k'
            | 'l'
            | 'm'
            | 'n'
            | 'p'
            | 'q'
            | 'r'
            | 's'
            | 't'
            | 'v'
            | 'w'
            | 'x'
            | 'y'
            | 'z'
    )
}

fn ends_with_any(word: &str, suffixes: &[&str]) -> bool {
    suffixes.iter().any(|suffix| word.ends_with(suffix))
}

#[cfg(test)]
mod tests {
    use super::*;

    // FR-PHENO-STR-003: Inflection - pluralize regular nouns
    #[test]
    fn test_pluralize_regular() {
        assert_eq!(pluralize("cat"), "cats");
        assert_eq!(pluralize("dog"), "dogs");
        assert_eq!(pluralize("book"), "books");
    }

    // FR-PHENO-STR-003: Inflection - pluralize sibilants
    #[test]
    fn test_pluralize_sibilants() {
        assert_eq!(pluralize("box"), "boxes");
        assert_eq!(pluralize("bus"), "buses");
        assert_eq!(pluralize("church"), "churches");
        assert_eq!(pluralize("brush"), "brushes");
    }

    // FR-PHENO-STR-003: Inflection - pluralize consonant-y words
    #[test]
    fn test_pluralize_consonant_y() {
        assert_eq!(pluralize("baby"), "babies");
        assert_eq!(pluralize("city"), "cities");
    }

    // FR-PHENO-STR-003: Inflection - pluralize f/fe words
    #[test]
    fn test_pluralize_f_fe() {
        assert_eq!(pluralize("leaf"), "leaves");
        assert_eq!(pluralize("knife"), "knives");
    }

    // FR-PHENO-STR-003: Inflection - pluralize irregular nouns
    #[test]
    fn test_pluralize_irregular() {
        assert_eq!(pluralize("child"), "children");
        assert_eq!(pluralize("person"), "people");
        assert_eq!(pluralize("man"), "men");
        assert_eq!(pluralize("woman"), "women");
        assert_eq!(pluralize("tooth"), "teeth");
        assert_eq!(pluralize("foot"), "feet");
        assert_eq!(pluralize("mouse"), "mice");
    }

    // FR-PHENO-STR-003: Inflection - singularize regular nouns
    #[test]
    fn test_singularize_regular() {
        assert_eq!(singularize("cats"), "cat");
        assert_eq!(singularize("dogs"), "dog");
        assert_eq!(singularize("books"), "book");
    }

    // FR-PHENO-STR-003: Inflection - singularize sibilants
    #[test]
    fn test_singularize_sibilants() {
        assert_eq!(singularize("boxes"), "box");
        assert_eq!(singularize("buses"), "bus");
        assert_eq!(singularize("churches"), "church");
    }

    // FR-PHENO-STR-003: Inflection - singularize ies words
    #[test]
    fn test_singularize_ies() {
        assert_eq!(singularize("babies"), "baby");
        assert_eq!(singularize("cities"), "city");
    }

    // FR-PHENO-STR-003: Inflection - singularize ves words
    #[test]
    fn test_singularize_ves() {
        assert_eq!(singularize("knives"), "knife");
        assert_eq!(singularize("wives"), "wife");
    }

    // FR-PHENO-STR-003: Inflection - singularize irregular nouns
    #[test]
    fn test_singularize_irregular() {
        assert_eq!(singularize("children"), "child");
        assert_eq!(singularize("people"), "person");
        assert_eq!(singularize("men"), "man");
        assert_eq!(singularize("women"), "woman");
        assert_eq!(singularize("teeth"), "tooth");
        assert_eq!(singularize("feet"), "foot");
        assert_eq!(singularize("mice"), "mouse");
    }

    // FR-PHENO-STR-003: Inflection - builder pattern pluralize
    #[test]
    fn test_inflection_pluralize() {
        assert_eq!(Inflection::new("cat").pluralize(), "cats");
        assert_eq!(Inflection::new("box").pluralize(), "boxes");
    }

    // FR-PHENO-STR-003: Inflection - builder pattern singularize
    #[test]
    fn test_inflection_singularize() {
        assert_eq!(Inflection::new("cats").singularize(), "cat");
        assert_eq!(Inflection::new("boxes").singularize(), "box");
    }

    // FR-PHENO-STR-003: Inflection - is_plural predicate
    #[test]
    fn test_inflection_is_plural() {
        assert!(Inflection::new("cats").is_plural());
        assert!(!Inflection::new("cat").is_plural());
    }

    // FR-PHENO-STR-003: Inflection - is_singular predicate
    #[test]
    fn test_inflection_is_singular() {
        assert!(Inflection::new("cat").is_singular());
        assert!(!Inflection::new("cats").is_singular());
    }
}
