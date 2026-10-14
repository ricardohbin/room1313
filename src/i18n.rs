//! Languages. The player picks one on the first screen (`language.rs`).
//!
//! The game is written in US English, and the English text is the key: every
//! other language is a table from the English text to its translation
//! (`i18n/pt_br.rs`). `t` looks text up in the chosen language just before it's
//! shown, so the deck, the films and the engine's tests stay in English.
//! Text without a translation falls back to English; a test checks that none is missing.

mod pt_br;

use std::cell::{Cell, RefCell};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    EnUs,
    PtBr,
}

impl Lang {
    /// In the order they're offered on the language screen.
    pub const ALL: [Lang; 2] = [Lang::EnUs, Lang::PtBr];

    /// BCP 47 tag, for `<html lang>` and for remembering the choice.
    pub fn code(self) -> &'static str {
        match self {
            Lang::EnUs => "en-US",
            Lang::PtBr => "pt-BR",
        }
    }

    /// The language's name for itself.
    pub fn native_name(self) -> &'static str {
        match self {
            Lang::EnUs => "English",
            Lang::PtBr => "Português",
        }
    }

    /// The country, in that language.
    pub fn country(self) -> &'static str {
        match self {
            Lang::EnUs => "United States",
            Lang::PtBr => "Brasil",
        }
    }

    /// The language for a tag like "pt-BR", or failing that one of the same language ("pt-PT").
    pub fn from_code(code: &str) -> Option<Lang> {
        let code = code.to_ascii_lowercase();
        let primary = |l: Lang| l.code().split('-').next().map(str::to_ascii_lowercase);
        Self::ALL
            .into_iter()
            .find(|l| l.code().to_ascii_lowercase() == code)
            .or_else(|| Self::ALL.into_iter().find(|&l| primary(l).as_deref() == code.split('-').next()))
    }

    /// English text -> this language's text.
    fn table(self) -> &'static [(&'static str, &'static str)] {
        match self {
            Lang::EnUs => &[],
            Lang::PtBr => pt_br::TABLE,
        }
    }

    /// `en` in this language, whatever the current one is (the language screen
    /// speaks the language under the cursor).
    pub fn tr(self, en: &'static str) -> &'static str {
        self.table().iter().find(|(k, _)| *k == en).map_or(en, |&(_, v)| v)
    }
}

thread_local! {
    static CURRENT: Cell<Lang> = const { Cell::new(Lang::EnUs) };
    static INDEX: RefCell<HashMap<&'static str, &'static str>> = RefCell::new(HashMap::new());
}

/// Switches the game to `lang`.
pub fn set(lang: Lang) {
    CURRENT.set(lang);
    INDEX.with_borrow_mut(|index| *index = lang.table().iter().copied().collect());
}

pub fn current() -> Lang {
    CURRENT.get()
}

/// `en` in the current language (English if it has no translation).
pub fn t(en: &str) -> &str {
    INDEX.with_borrow(|index| index.get(en).copied()).unwrap_or(en)
}

/// Every piece of English text the game shows, for the translation tests.
#[cfg(test)]
pub(crate) fn english() -> Vec<&'static str> {
    use crate::cards::{deck, Arrival, Character, Item};
    use crate::engine::Stat;

    let mut all: Vec<&'static str> = Vec::new();
    // The deck's text is owned; leaking it is fine in a test.
    for scene in deck() {
        for s in [scene.text, scene.left.label, scene.right.label] {
            all.push(Box::leak(s.into_boxed_str()));
        }
    }
    all.extend(Character::ALL.iter().flat_map(|c| [c.name(), c.role()]));
    all.extend(Item::ALL.map(Item::name));
    all.extend([Arrival::Summoned, Arrival::Unannounced].map(Arrival::label));
    all.extend(Stat::ALL.map(Stat::name));
    all.extend(crate::splash::texts());
    all.extend(crate::language::texts());
    all.extend(crate::options::texts());
    all.extend(crate::cinematic::texts());
    all.extend(crate::intro::texts());
    all.extend(crate::endings::texts());
    all.extend(crate::art::texts());
    all
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn every_text_is_translated() {
        for lang in Lang::ALL.into_iter().filter(|&l| l != Lang::EnUs) {
            let keys: HashSet<_> = lang.table().iter().map(|(k, _)| *k).collect();
            let missing: Vec<_> = english().into_iter().filter(|s| !keys.contains(s)).collect();
            assert!(missing.is_empty(), "{lang:?} is missing {} translations: {missing:#?}", missing.len());
        }
    }

    #[test]
    fn no_stale_or_duplicate_translations() {
        let english: HashSet<_> = english().into_iter().collect();
        for lang in Lang::ALL {
            let mut seen = HashSet::new();
            for (k, _) in lang.table() {
                assert!(seen.insert(k), "{lang:?} translates {k:?} twice");
                assert!(english.contains(k), "{lang:?} translates {k:?}, which the game never shows");
            }
        }
    }

    #[test]
    fn lookup_follows_the_current_language() {
        set(Lang::PtBr);
        assert_eq!(t("New Game"), "Novo Jogo");
        assert_eq!(t("not in any table"), "not in any table");
        set(Lang::EnUs);
        assert_eq!(t("New Game"), "New Game");
        assert_eq!(Lang::PtBr.tr("New Game"), "Novo Jogo");
    }

    #[test]
    fn codes_match_the_closest_language() {
        assert_eq!(Lang::from_code("pt-BR"), Some(Lang::PtBr));
        assert_eq!(Lang::from_code("pt-PT"), Some(Lang::PtBr));
        assert_eq!(Lang::from_code("en-GB"), Some(Lang::EnUs));
        assert_eq!(Lang::from_code("EN-us"), Some(Lang::EnUs));
        assert_eq!(Lang::from_code("fr-FR"), None);
    }
}
