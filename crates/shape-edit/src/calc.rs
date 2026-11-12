//! A small calculator for typed values, so a size can be entered as a formula ("120/3",
//! "2*(15+7.5)", "-4", "12,5"), as the original allows in its dimension fields. It also resolves
//! **variable names** (`breite`, `rand`, `Platte_B`) through a caller-supplied lookup, so the
//! same field can read a value from the document's variables (parametric sizes). The original's
//! own formula field is arithmetic-only (`utFormulaParser`, 0x10033f80c); names are an extension
//! on top of its `utDocumentVariable` concept (see app/docs/PARAMETRIK.md).

/// What a name resolves to while evaluating, or why it can't.
pub trait Lookup {
    /// The value of a variable, or `Err` with a reason (unknown, or a cycle was hit).
    fn get(&self, name: &str) -> Result<f64, CalcError>;
}

/// No variables: every name is unknown. Used by [`evaluate`].
pub struct NoVars;

impl Lookup for NoVars {
    fn get(&self, name: &str) -> Result<f64, CalcError> {
        Err(CalcError::Unknown(name.to_string()))
    }
}

/// A closure as a [`Lookup`].
impl<F: Fn(&str) -> Result<f64, CalcError>> Lookup for F {
    fn get(&self, name: &str) -> Result<f64, CalcError> {
        self(name)
    }
}

/// Why a formula couldn't be turned into a number.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CalcError {
    /// The text isn't a valid expression.
    Syntax,
    /// A name isn't a known variable.
    Unknown(String),
    /// `{name` without the closing brace.
    MissingBrace,
    /// A variable depends on itself (directly or through others).
    Cycle(String),
    /// The result isn't a finite number (e.g. divide by zero).
    NotFinite,
}

impl CalcError {
    /// A short plain-language reason (German, as the app speaks).
    pub fn message(&self) -> String {
        match self {
            CalcError::Syntax => "ungültige Formel".into(),
            // the original's wording ("Unknown variable: %s")
            CalcError::Unknown(n) => format!("Unbekannte Variable: {n}"),
            CalcError::MissingBrace => "Fehlende '}' nach dem Namen der Variablen".into(),
            CalcError::Cycle(n) => format!("„{n}“ hängt von sich selbst ab"),
            CalcError::NotFinite => "kein endliches Ergebnis".into(),
        }
    }
}

/// The value of `text` with no variables (a plain number or arithmetic). `None` when it isn't a
/// valid expression or the result isn't finite. Kept for the many call sites that have no
/// variables; [`eval`] is the same with a name lookup.
pub fn evaluate(text: &str) -> Option<f64> {
    eval(text, &NoVars).ok()
}

/// The value of `text`, resolving names through `vars`. A trailing "=" (how the original
/// finishes a formula) and spaces are ignored; a comma counts as a decimal point.
pub fn eval(text: &str, vars: &dyn Lookup) -> Result<f64, CalcError> {
    let cleaned: String = text.trim().trim_end_matches('=').chars().filter(|c| !c.is_whitespace()).map(|c| if c == ',' { '.' } else { c }).collect();
    let mut p = Parser { s: cleaned.as_bytes(), i: 0, vars };
    let v = p.sum()?;
    if p.i != p.s.len() {
        return Err(CalcError::Syntax);
    }
    v.is_finite().then_some(v).ok_or(CalcError::NotFinite)
}

/// Whether `text` is more than a plain number, i.e. something to evaluate. As in the original
/// (0x1003447ec, called before its evaluator 0x100344678): a plain number is only digits, comma,
/// point, spaces and tabs, with a minus allowed as the very first character; an operator, a
/// bracket, a name or anything else makes it a formula. A trailing "=" (the "finished" mark) is
/// not counted. Empty text is not a formula.
pub fn is_formula(text: &str) -> bool {
    let t = text.trim_end_matches('=');
    t.chars().enumerate().any(|(i, c)| !(c.is_ascii_digit() || matches!(c, ' ' | '\t' | ',' | '.') || (i == 0 && c == '-')))
}

/// The variable names mentioned in `text` (for dependency analysis), in order, without repeats.
pub fn names_in(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if is_name_start(chars[i]) {
            let start = i;
            while i < chars.len() && is_name_part(chars[i]) {
                i += 1;
            }
            let name: String = chars[start..i].iter().collect();
            if !out.contains(&name) {
                out.push(name);
            }
        } else {
            i += 1;
        }
    }
    out
}

/// `text` with every use of the variable `from` (bare or in braces) replaced by `to`; other
/// names, numbers and operators stay as they are. A longer name that merely starts like `from`
/// (`hoehe2` when renaming `hoehe`) is not touched.
pub fn rename_in(text: &str, from: &str, to: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < chars.len() {
        if is_name_start(chars[i]) {
            let start = i;
            while i < chars.len() && is_name_part(chars[i]) {
                i += 1;
            }
            let name: String = chars[start..i].iter().collect();
            out.push_str(if name == from { to } else { &name });
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

/// A valid variable name: a letter or `_` to start, then letters, digits or `_`.
pub fn is_valid_name(name: &str) -> bool {
    let mut cs = name.chars();
    cs.next().is_some_and(is_name_start) && name.chars().all(is_name_part)
}

/// A variable name starts with a letter and may not start with `_` — the original's rule
/// ("Ein Variablenname darf nicht mit _ beginnen", "…muss mit einem Buchstaben beginnen und dann
/// Buchstaben, Zahlen und Underscores besitzen").
pub fn is_name_start(c: char) -> bool {
    c.is_alphabetic()
}

pub fn is_name_part(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
    vars: &'a dyn Lookup,
}

impl Parser<'_> {
    fn peek(&self) -> Option<u8> {
        self.s.get(self.i).copied()
    }

    fn eat(&mut self, c: u8) -> bool {
        let hit = self.peek() == Some(c);
        if hit {
            self.i += 1;
        }
        hit
    }

    fn sum(&mut self) -> Result<f64, CalcError> {
        let mut v = self.product()?;
        loop {
            if self.eat(b'+') {
                v += self.product()?;
            } else if self.eat(b'-') {
                v -= self.product()?;
            } else {
                return Ok(v);
            }
        }
    }

    fn product(&mut self) -> Result<f64, CalcError> {
        let mut v = self.power()?;
        loop {
            // '*' and the middle dot/multiplication sign multiply; 'x' no longer does, so it can
            // be a variable name (app/docs/PARAMETRIK.md)
            if self.eat(b'*') || self.eat_char('×') || self.eat_char('·') {
                v *= self.power()?;
            } else if self.eat(b'/') || self.eat_char('÷') {
                v /= self.power()?;
            } else {
                return Ok(v);
            }
        }
    }

    fn power(&mut self) -> Result<f64, CalcError> {
        let base = self.unary()?;
        if self.eat(b'^') {
            return Ok(base.powf(self.power()?));
        }
        Ok(base)
    }

    fn unary(&mut self) -> Result<f64, CalcError> {
        if self.eat(b'-') {
            return Ok(-self.unary()?);
        }
        if self.eat(b'+') {
            return self.unary();
        }
        if self.eat(b'(') {
            let v = self.sum()?;
            return self.eat(b')').then_some(v).ok_or(CalcError::Syntax);
        }
        // {name}: how the original refers to a document variable ("Use {%s} to reference the
        // document variable"); a missing '}' is its own error
        if self.eat(b'{') {
            let start = self.i;
            while self.peek().is_some_and(|c| c != b'}') {
                self.i += 1;
            }
            let name = std::str::from_utf8(&self.s[start..self.i]).map_err(|_| CalcError::Syntax)?.to_string();
            if !self.eat(b'}') {
                return Err(CalcError::MissingBrace);
            }
            return self.vars.get(&name);
        }
        // a bare name
        if self.peek().is_some_and(|c| is_name_start(c as char)) {
            let start = self.i;
            while self.peek().is_some_and(|c| is_name_part(c as char)) {
                self.i += 1;
            }
            let name = std::str::from_utf8(&self.s[start..self.i]).map_err(|_| CalcError::Syntax)?;
            return self.vars.get(name);
        }
        // a number
        let start = self.i;
        while matches!(self.peek(), Some(b'0'..=b'9' | b'.')) {
            self.i += 1;
        }
        if self.i == start {
            return Err(CalcError::Syntax);
        }
        std::str::from_utf8(&self.s[start..self.i]).ok().and_then(|t| t.parse().ok()).ok_or(CalcError::Syntax)
    }

    /// Eat a multi-byte character like `×` if it's next.
    fn eat_char(&mut self, c: char) -> bool {
        let mut buf = [0u8; 4];
        let bytes = c.encode_utf8(&mut buf).as_bytes();
        if self.s[self.i..].starts_with(bytes) {
            self.i += bytes.len();
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars<'a>(pairs: &'a [(&'a str, f64)]) -> impl Lookup + 'a {
        move |n: &str| pairs.iter().find(|(k, _)| *k == n).map(|(_, v)| *v).ok_or_else(|| CalcError::Unknown(n.to_string()))
    }

    #[test]
    fn numbers_and_formulas() {
        assert_eq!(evaluate("12.5"), Some(12.5));
        assert_eq!(evaluate(" 12,5 "), Some(12.5));
        assert_eq!(evaluate("120/3"), Some(40.0));
        assert_eq!(evaluate("2*(15+7.5)="), Some(45.0));
        assert_eq!(evaluate("10-2-3"), Some(5.0), "left to right");
        assert_eq!(evaluate("2+3*4"), Some(14.0), "× before +");
        assert_eq!(evaluate("-4+10"), Some(6.0));
        assert_eq!(evaluate("2^3^2"), Some(512.0), "powers from the right");
        assert_eq!(evaluate("6×7"), Some(42.0));
    }

    #[test]
    fn rejects_nonsense() {
        assert_eq!(evaluate(""), None);
        assert_eq!(evaluate("(1+2"), None);
        assert_eq!(evaluate("1/0"), None, "not finite");
        assert_eq!(evaluate("1+"), None);
        // a bare name with no variables is unknown, not a number
        assert_eq!(evaluate("abc"), None);
    }

    #[test]
    fn resolves_names() {
        let v = vars(&[("breite", 100.0), ("rand", 5.0)]);
        assert_eq!(eval("breite", &v), Ok(100.0));
        assert_eq!(eval("breite - 2*rand", &v), Ok(90.0));
        assert_eq!(eval("breite/2 + 10", &v), Ok(60.0));
        assert_eq!(eval("x", &v), Err(CalcError::Unknown("x".into())), "x is a name now, not times");
    }

    #[test]
    fn names_are_listed_in_order_without_repeats() {
        assert_eq!(names_in("breite - 2*rand + breite"), vec!["breite", "rand"]);
        assert_eq!(names_in("30"), Vec::<String>::new());
        assert_eq!(names_in("a_1 + B2"), vec!["a_1", "B2"]);
    }

    #[test]
    fn renaming_changes_whole_names_only() {
        assert_eq!(rename_in("hoehe/2 + hoehe2 - {hoehe}", "hoehe", "h"), "h/2 + hoehe2 - {h}");
        assert_eq!(rename_in("30", "hoehe", "h"), "30");
        assert_eq!(rename_in("a*b", "b", "c"), "a*c");
    }

    #[test]
    fn valid_names() {
        assert!(is_valid_name("breite"));
        assert!(!is_valid_name("_x"), "no leading underscore, as in the original");
        assert!(is_valid_name("Platte_B"));
        assert!(!is_valid_name("2b"));
        assert!(!is_valid_name(""));
        assert!(!is_valid_name("a b"));
    }

    #[test]
    fn braces_refer_to_variables_like_in_the_original() {
        let v = vars(&[("breite", 100.0)]);
        assert_eq!(eval("{breite}/2", &v), Ok(50.0));
        assert_eq!(eval("{nope}", &v), Err(CalcError::Unknown("nope".into())));
        assert_eq!(eval("{breite", &v), Err(CalcError::MissingBrace));
        assert_eq!(CalcError::Unknown("nope".into()).message(), "Unbekannte Variable: nope");
    }

    #[test]
    fn tells_formulas_from_numbers() {
        assert!(!is_formula("12.5"));
        assert!(!is_formula("-3"));
        assert!(!is_formula("1 000,5"));
        assert!(!is_formula(""));
        assert!(is_formula("+3"), "a plus makes it a formula, as in the original");
        assert!(is_formula("2-3"), "a minus is only plain at the very start");
        assert!(is_formula("(1+2)"));
        assert!(is_formula("120/3"));
        assert!(is_formula("2*(1+1)="));
        assert!(is_formula("breite"));
    }
}
