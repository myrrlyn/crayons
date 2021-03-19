use rustler::{Atom, Binary, Encoder, Env, Error as NifError, NifResult, Term};

use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
    io::Cursor,
    sync::RwLock,
};

use syntect::{
    easy::HighlightLines,
    highlighting::ThemeSet,
    html::highlighted_html_for_string,
    parsing::{SyntaxDefinition as SyntaxDefn, SyntaxSet},
    util::as_24_bit_terminal_escaped,
};

use tap::{Pipe, Tap};

mod atoms {
    rustler::rustler_atoms! {
        atom html;
        atom terminal;
    }
}

/// Error-kind atoms indicating which specific failure occurred.
#[derive(rustler::NifUnitEnum, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ErrorKind {
    UnknownTheme,
    UnknownFormat,
    InvalidLangDefn,
    InvalidThemeDefn,
}

/// The atoms `:ok` and `:error`
#[derive(rustler::NifUnitEnum, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NifStatus {
    Ok,
    Error,
}

lazy_static::lazy_static! {
    pub static ref SYNTAX_SET: RwLock<Option<SyntaxSet>> = RwLock::new(Some(SyntaxSet::load_defaults_nonewlines()));
    pub static ref THEME_SET: RwLock<ThemeSet> = RwLock::new(ThemeSet::load_defaults());
}

rustler::rustler_export_nifs! {
    "Elixir.Crayons.Native",
    [
        ("color", 4, color),
        ("add_lang", 3, add_lang),
        ("add_theme", 2, add_theme),
        ("list_langs", 0, list_langs),
        ("list_themes", 0, list_themes),
    ],
    None
}

/// NIF entry hook.
///
/// This receives a BEAM environment reference and a variadic arglist.
///
/// # BEAM Arguments
///
/// - `lang`: An atom or string naming the source language of the text to be
///   colored.
/// - `text`: Some text to be colored. This must be a BEAM binary, and will
///   cause the function to exit with `{:error, :invalid_text}` if it is not
///   UTF-8
/// - `format`: Either `:html` or `:terminal`
/// - `theme`: One of the theme names defined in [`syntect`][themes]. Currently,
///   this library does not permit loading additional theme definitions at
///   runtime.
///
/// # Blocking
///
/// This blocks the system thread when there are calls to [`add_lang`] or
/// [`add_theme`] ongoing.
///
/// [themes]: https://docs.rs/syntect/4.5.0/syntect/highlighting/struct.ThemeSet.html#method.load_defaults
pub fn color<'env>(env: Env<'env>, args: &[Term<'env>]) -> NifResult<Term<'env>> {
    let mut args = args.into_iter();
    let text: &'env str = args.next().ok_or(NifError::BadArg)?.decode()?;
    let lang_term = args.next().ok_or(NifError::BadArg)?;
    let lang: Cow<'env, str> = if lang_term.is_atom() {
        Cow::Owned(lang_term.atom_to_string()?)
    } else {
        Cow::Borrowed(lang_term.decode()?)
    };
    let fmt: Atom = args.next().ok_or(NifError::BadArg)?.decode()?;
    let theme: &'env str = args.next().ok_or(NifError::BadArg)?.decode()?;

    // TODO(myrrlyn): Replace blocking reads with yield loops
    let theme_set = THEME_SET.read().map_err(|_| poison())?;
    let syntax_set = SYNTAX_SET.read().map_err(|_| poison())?;

    let theme = match theme_set.themes.get(theme) {
        None => return fail(env, UnknownTheme::new(theme)),
        Some(t) => t,
    };
    let syntax_set = syntax_set
        .as_ref()
        .expect("a read lock cannot observe an empty syntax set");
    let syntax = match syntax_set.find_syntax_by_token(&lang) {
        None => return Ok((NifStatus::Ok, text).encode(env)),
        Some(s) => s,
    };

    let colored = match fmt {
        f if f == atoms::html() => highlighted_html_for_string(text, &syntax_set, syntax, theme),
        f if f == atoms::terminal() => {
            let mut h = HighlightLines::new(syntax, theme);
            text.lines()
                .map(|line| {
                    let ranges = h.highlight(line, &syntax_set);
                    as_24_bit_terminal_escaped(&ranges[..], true)
                })
                .collect::<Vec<_>>()
                .join("\n")
                .tap_mut(|out| out.push_str("\x1b[0m"))
        }
        _ => return fail(env, ErrorKind::UnknownFormat),
    };

    Ok((NifStatus::Ok, colored).encode(env))
}

/// Adds a syntax definition to the library.
///
/// # BEAM Arguments
///
/// - `contents`: The contents of a `.sublime-syntax` or `.tmLanguage` file,
///   which defines a text shape.
/// - `name`: A name (can be `nil`) of the language being defined.
/// - `incl_newline`: A bool indicating whether the grammar expects newlines in
///   text parsed by it or not.
///
/// # Blocking
///
/// This blocks the system thread when there are calls to [`color`] or other
/// calls to itself ongoing.
pub fn add_lang<'env>(env: Env<'env>, args: &[Term<'env>]) -> NifResult<Term<'env>> {
    let syntax_content: &'env str = args.get(0).ok_or(NifError::BadArg)?.decode()?;
    let name: Option<&'env str> = args.get(1).ok_or(NifError::BadArg)?.decode()?;
    let incl_newline: bool = args.get(2).ok_or(NifError::BadArg)?.decode()?;

    Ok(
        match SyntaxDefn::load_from_str(syntax_content, incl_newline, name) {
            Ok(syntax) => {
                let mut syntax_set = SYNTAX_SET.write().map_err(|_| poison())?;
                let mut builder = syntax_set
                    .take()
                    // Should never run
                    .unwrap_or_else(SyntaxSet::load_defaults_nonewlines)
                    .into_builder();
                let name = syntax.name.encode(env);
                builder.add(syntax);
                *syntax_set = Some(builder.build());
                (NifStatus::Ok, name).encode(env)
            }
            Err(e) => (
                NifStatus::Error,
                ErrorKind::InvalidLangDefn,
                format!("{}", e),
            )
                .encode(env),
        },
    )
}

/// Adds a theme definition to the library. If the named theme already existed,
/// the old theme definition is discarded.
///
/// # BEAM Arguments
///
/// - `contents`: A `Binary` containing the contents of a `.tmTheme` file.
/// - `name`: The name of the theme, which can be used in calls to [`color`].
///
/// # Blocking
///
/// This blocks the system thread when there are calls to [`color`] or other
/// calls to itself ongoing.
pub fn add_theme<'env>(env: Env<'env>, args: &[Term<'env>]) -> NifResult<Term<'env>> {
    let theme_content: Binary<'env> = args.get(0).ok_or(NifError::BadArg)?.decode()?;
    let name: &'env str = args.get(1).ok_or(NifError::BadArg)?.decode()?;

    let mut cursor = Cursor::new(theme_content.as_slice());
    Ok(match ThemeSet::load_from_reader(&mut cursor) {
        Ok(theme) => {
            let mut theme_set = THEME_SET.write().map_err(|_| poison())?;
            theme_set.themes.insert(name.to_owned(), theme);
            (NifStatus::Ok, name).encode(env)
        }
        Err(e) => (
            NifStatus::Error,
            ErrorKind::InvalidThemeDefn,
            format!("{}", e),
        )
            .encode(env),
    })
}

/// Lists all languages currently in the library.
pub fn list_langs<'env>(env: Env<'env>, _args: &[Term<'env>]) -> NifResult<Term<'env>> {
    SYNTAX_SET
        .read()
        .map_err(|_| poison())?
        .as_ref()
        .expect("a read lock can never observe an empty SyntaxSet")
        .syntaxes()
        .into_iter()
        .filter(|syntax| !syntax.hidden)
		.map(|syntax| syntax.name.to_lowercase())
        .collect::<Vec<_>>()
        .encode(env)
        .pipe(Ok)
}

/// Lists all themes currently in the library.
pub fn list_themes<'env>(env: Env<'env>, _args: &[Term<'env>]) -> NifResult<Term<'env>> {
    THEME_SET
        .read()
        .map_err(|_| poison())?
        .themes
        .keys()
        .map(|k| &**k)
        .collect::<Vec<_>>()
        .encode(env)
        .pipe(Ok)
}

fn fail<'env, T: Encoder>(env: Env<'env>, term: T) -> NifResult<Term<'env>> {
    Ok((NifStatus::Error, term).encode(env))
}

fn poison() -> NifError {
    NifError::Atom("library_poisoned")
}

#[derive(Clone, Debug)]
pub struct UnknownTheme<'env> {
    inner: &'env str,
}

impl<'env> UnknownTheme<'env> {
    fn new(theme: &'env str) -> Self {
        Self { inner: theme }
    }
}

impl Display for UnknownTheme<'_> {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "Unknown theme: {}", self.inner)
    }
}

impl Encoder for UnknownTheme<'_> {
    fn encode<'env>(&self, env: Env<'env>) -> Term<'env> {
        ErrorKind::UnknownTheme.encode(env)
    }
}
