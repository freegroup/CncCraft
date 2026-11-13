//! The languages of the interface. The texts are German in the source and are looked up by that
//! text in `locales/<code>.json` (rust-i18n); the language can be switched at any time because the
//! interface is drawn anew every frame.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    De,
    En,
}

impl Lang {
    pub const ALL: [Lang; 2] = [Lang::De, Lang::En];

    pub fn code(self) -> &'static str {
        match self {
            Lang::De => "de",
            Lang::En => "en",
        }
    }

    /// The language's name in that language.
    pub fn native_name(self) -> &'static str {
        match self {
            Lang::De => "Deutsch",
            Lang::En => "English",
        }
    }

    pub fn from_code(code: &str) -> Option<Lang> {
        let base = code.split(['-', '_']).next().unwrap_or("").to_ascii_lowercase();
        Lang::ALL.into_iter().find(|l| l.code() == base)
    }

    /// The language of the operating system when it is one of ours, else German.
    pub fn system() -> Lang {
        sys_locale::get_locale().and_then(|c| Lang::from_code(&c)).unwrap_or(Lang::De)
    }

    pub fn apply(self) {
        rust_i18n::set_locale(self.code());
    }
}
