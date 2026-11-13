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

/// A text that is translated when it is shown, not when it is made: it lives in tables and
/// structs as the German original and always reads in the language in use at that moment.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Tr(&'static str);

impl Tr {
    pub const fn new(key: &'static str) -> Tr {
        Tr(key)
    }

    /// The German original, which is also the key.
    pub const fn key(self) -> &'static str {
        self.0
    }

    pub fn get(self) -> String {
        rust_i18n::t!(self.0).into_owned()
    }
}

impl std::fmt::Display for Tr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.get())
    }
}

impl From<Tr> for String {
    fn from(t: Tr) -> String {
        t.get()
    }
}

impl From<Tr> for egui::WidgetText {
    fn from(t: Tr) -> egui::WidgetText {
        t.get().into()
    }
}

/// The reason a formula could not be worked out, in the language in use (the calculator itself
/// only knows what went wrong).
pub fn calc_message(e: &shape_edit::calc::CalcError) -> String {
    use shape_edit::calc::CalcError;
    match e {
        CalcError::Syntax => t!("ungültige Formel"),
        CalcError::Unknown(n) => t!("Unbekannte Variable: %{name}", name = n),
        CalcError::MissingBrace => t!("Fehlende '}' nach dem Namen der Variablen"),
        CalcError::Cycle(n) => t!("„%{name}“ hängt von sich selbst ab", name = n),
        CalcError::NotFinite => t!("kein endliches Ergebnis"),
    }
}
