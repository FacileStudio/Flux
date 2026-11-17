//! Which of the two shell languages to speak.
//!
//! Every widget carries its strings in English and French. The language comes
//! from the POSIX message locale: the first non-empty of `LC_ALL`,
//! `LC_MESSAGES` and `LANG`. Anything that is not French reads English.

/// The languages the shell is written in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    En,
    Fr,
}

/// The language this process should speak.
pub fn current() -> Lang {
    let vars = ["LC_ALL", "LC_MESSAGES", "LANG"].map(|name| std::env::var(name).ok());
    from_vars(&vars.each_ref().map(|value| value.as_deref()))
}

/// The language the first set locale variable names.
pub fn from_vars(vars: &[Option<&str>]) -> Lang {
    let locale = vars
        .iter()
        .flatten()
        .find(|value| !value.is_empty())
        .copied()
        .unwrap_or_default();
    if locale.starts_with("fr") {
        Lang::Fr
    } else {
        Lang::En
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_set_variable_decides() {
        assert_eq!(from_vars(&[None, None, Some("fr_FR.UTF-8")]), Lang::Fr);
        assert_eq!(from_vars(&[Some("en_US.UTF-8"), None, Some("fr_FR.UTF-8")]), Lang::En);
        assert_eq!(from_vars(&[Some(""), Some("fr_CA.UTF-8"), None]), Lang::Fr);
        assert_eq!(from_vars(&[None, None, None]), Lang::En);
        assert_eq!(from_vars(&[Some("C.UTF-8"), None, None]), Lang::En);
    }
}
