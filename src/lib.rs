mod config;
mod data;
pub mod hunspell;

use std::io;
use std::path::PathBuf;

pub use config::load_config;
pub use data::LangProvider;
use directories::ProjectDirs;
use unicode_segmentation::UnicodeSegmentation;

pub(crate) fn dirs() -> ProjectDirs {
    ProjectDirs::from("", "", "rspell").expect("get project directories")
}

#[derive(Debug)]
#[non_exhaustive]
pub enum SpellError {
    DicNotFound(PathBuf),
    NoDicSource,
    FileCachingError(io::Error),
    RemoveDicError(io::Error),
    ConversionError(io::Error),
    InitConfigError(io::Error),
    ReadConfigError(io::Error),
    LoadConfigError(toml::de::Error),
}

/// Result of a check. Wraps possible corrections when this is an incorrect result.
#[derive(Debug)]
pub enum SpellResult {
    /// The checked word was correct.
    Correct,
    /// The checked word was incorrect, suggestions for correction are available.
    Incorrect { suggestions: Vec<String> },
}

impl SpellResult {
    /// Returns `true` when it is a correct result, `false` otherwise.
    pub fn correct(&self) -> bool {
        match self {
            Self::Correct => true,
            _ => false,
        }
    }
}

/// Represents a misspelt word in a text.
#[derive(Debug)]
pub struct BadWord<'a> {
    /// Offset from the text beginning, 0-based.
    pub offset: usize,
    /// The misspelt word.
    pub word: &'a str,
    /// Possible corrections for the word.
    pub suggestions: Vec<String>,
}

/// This is the main structure. It allows to gather dictionaries for a given
/// language and spellcheck individual word or whole text using it.
///
/// # Examples
///
/// ```
/// let spell = rspell::Spell::new("en_US").unwrap();
///
/// assert!(!spell.check_word("colour").correct());
/// assert!(spell.check_word("color").correct());
///
/// for bad in spell.check("Wht color is this flg?") {
///     println!(
///         "{} (offset: {}): possible corrections: {:?}",
///         bad.word, bad.offset, bad.suggestions
///     );
/// }
/// ```
pub struct Spell {
    hs: hunspell::Hunspell,
}

impl Spell {
    /// Creates a new spellchecker for the given language code.
    ///
    /// This function will also ensure that data files for the language are
    /// available. If not it will try to get it using a
    /// [LangProvider](struct.LangProvider.html).
    pub fn new(lang: &str) -> Result<Spell, SpellError> {
        if cfg!(feature = "local_files") {
            let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("files");
            Ok(Spell {
                hs: hunspell::Hunspell::new(
                    dir.join(&format!("{}.aff", lang)),
                    dir.join(&format!("{}.dic", lang)),
                ),
            })
        } else {
            let config = config::load_config()?;
            let provider = LangProvider::new(lang, &config);
            provider.ensure_data()?;
            Ok(Spell {
                hs: provider.into(),
            })
        }
    }

    /// Checks spelling for the given word.
    pub fn check_word(&self, word: &str) -> SpellResult {
        if self.hs.spell(word) {
            SpellResult::Correct
        } else {
            SpellResult::Incorrect {
                suggestions: self.hs.suggest(word),
            }
        }
    }

    /// Checks spelling for the given text.
    ///
    /// Word boundaries are determined using unicode segmentation rules.
    pub fn check<'a>(&self, text: &'a str) -> Vec<BadWord<'a>> {
        text.split_word_bound_indices()
            .filter(|(_, word)| word.chars().any(char::is_alphanumeric))
            .filter_map(|(i, word)| match self.check_word(word) {
                SpellResult::Correct => None,
                SpellResult::Incorrect { suggestions } => Some(BadWord {
                    offset: i,
                    word,
                    suggestions,
                }),
            })
            .collect()
    }
}
