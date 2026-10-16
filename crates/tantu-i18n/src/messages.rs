//! [`Translations`], [`Args`] and [`ResourceError`]: Fluent resources per locale. Spec:
//! `docs/specs/i18n/localization.md` (I18N-MSG-01..05).

use std::collections::HashSet;
use std::rc::Rc;

use fluent_bundle::types::FluentNumber;
use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use fluent_syntax::ast;
use intl_memoizer::IntlLangMemoizer;

use crate::Locale;
use crate::formats::NumberFormatter;

/// A problem in a Fluent resource (syntax error or duplicate id), with its byte offset.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceError {
    /// What went wrong.
    pub message: String,
    /// Where, as a byte offset into the resource.
    pub offset: usize,
}

/// One locale's messages.
type Bundle = FluentBundle<Rc<FluentResource>>;

/// Fluent resources for a set of locales.
pub struct Translations {
    fallback: Locale,
    bundles: Vec<(Locale, Bundle)>,
}

impl Translations {
    /// No resources yet; lookups end at `fallback`.
    pub fn new(fallback: Locale) -> Self {
        Translations {
            fallback,
            bundles: Vec::new(),
        }
    }

    /// Adds a resource (Fluent syntax) for `locale`; a later resource's messages replace
    /// earlier ones with the same id. Errors are returned and the valid messages kept
    /// (I18N-MSG-01).
    pub fn add(&mut self, locale: Locale, source: &str) -> Result<(), Vec<ResourceError>> {
        let (resource, mut errors) = match FluentResource::try_new(source.to_owned()) {
            Ok(resource) => (resource, Vec::new()),
            Err((resource, parse)) => (
                resource,
                parse
                    .into_iter()
                    .map(|e| ResourceError {
                        message: e.kind.to_string(),
                        offset: e.pos.start,
                    })
                    .collect(),
            ),
        };
        // Duplicate ids within this resource (the bundle would silently keep the last).
        let mut seen = HashSet::new();
        for entry in resource.entries() {
            let id = match entry {
                ast::Entry::Message(m) => m.id.name,
                ast::Entry::Term(t) => t.id.name,
                _ => continue,
            };
            if !seen.insert(id) {
                errors.push(ResourceError {
                    message: format!("duplicate id `{id}`"),
                    offset: definitions(source, id).nth(1).unwrap_or(0),
                });
            }
        }
        self.bundle_mut(&locale)
            .add_resource_overriding(Rc::new(resource));
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// The locales with resources, the fallback included.
    pub fn locales(&self) -> Vec<Locale> {
        let mut locales: Vec<Locale> = self.bundles.iter().map(|(l, _)| l.clone()).collect();
        if !locales.contains(&self.fallback) {
            locales.push(self.fallback.clone());
        }
        locales
    }

    /// The best available locale for `requested` (in preference order), else the fallback
    /// (I18N-MSG-05).
    pub fn negotiate(&self, requested: &[Locale]) -> Locale {
        let available = self.locales();
        let requested: Vec<_> = requested.iter().map(Locale::langid).collect();
        let langids: Vec<_> = available.iter().map(Locale::langid).collect();
        let fallback = self.fallback.langid();
        let chosen = fluent_langneg::negotiate_languages(
            &requested,
            &langids,
            Some(&fallback),
            fluent_langneg::NegotiationStrategy::Filtering,
        );
        chosen
            .first()
            .and_then(|id| langids.iter().position(|l| l == *id))
            .map_or_else(|| self.fallback.clone(), |i| available[i].clone())
    }

    /// The message `id` in the first locale of `chain` (then the fallback) that has it.
    pub(crate) fn format(
        &self,
        chain: &[Locale],
        id: &str,
        args: Option<&FluentArgs<'_>>,
    ) -> Option<String> {
        let fallback = std::iter::once(&self.fallback);
        for locale in chain.iter().chain(fallback) {
            let Some((_, bundle)) = self.bundles.iter().find(|(l, _)| l == locale) else {
                continue;
            };
            let Some(pattern) = bundle.get_message(id).and_then(|m| m.value()) else {
                continue;
            };
            let mut errors = Vec::new();
            return Some(
                bundle
                    .format_pattern(pattern, args, &mut errors)
                    .into_owned(),
            );
        }
        None
    }

    fn bundle_mut(&mut self, locale: &Locale) -> &mut Bundle {
        let index = match self.bundles.iter().position(|(l, _)| l == locale) {
            Some(index) => index,
            None => {
                let mut bundle = FluentBundle::new(vec![locale.langid()]);
                // Paragraphs handle bidi themselves (I18N-MSG-03).
                bundle.set_use_isolating(false);
                bundle.set_formatter(Some(format_value));
                self.bundles.push((locale.clone(), bundle));
                self.bundles.len() - 1
            }
        };
        &mut self.bundles[index].1
    }
}

/// The byte offsets of the lines defining `id` (`id =` at a line start).
fn definitions<'a>(source: &'a str, id: &'a str) -> impl Iterator<Item = usize> + 'a {
    let mut offset = 0;
    source.split_inclusive('\n').filter_map(move |line| {
        let start = offset;
        offset += line.len();
        let rest = line.strip_prefix(id)?;
        rest.trim_start().starts_with('=').then_some(start)
    })
}

/// Numbers inside messages in the bundle's locale format (I18N-MSG-03).
fn format_value(value: &FluentValue<'_>, memoizer: &IntlLangMemoizer) -> Option<String> {
    let FluentValue::Number(FluentNumber { value, options }) = value else {
        return None;
    };
    let fraction = options.minimum_fraction_digits;
    memoizer
        .with_try_get::<NumberFormatter, _, _>((), |f| match fraction {
            Some(digits) => f.format_fixed(*value, digits.min(usize::from(u8::MAX)) as u8),
            None => f.format_shortest(*value),
        })
        .ok()
}

/// A string or number argument.
#[derive(Clone, Debug, PartialEq)]
pub enum ArgValue {
    /// Inserted as is.
    Text(String),
    /// Selects plural variants and is written in the locale's number format.
    Number(f64),
}

impl From<&str> for ArgValue {
    fn from(value: &str) -> Self {
        ArgValue::Text(value.to_owned())
    }
}

impl From<String> for ArgValue {
    fn from(value: String) -> Self {
        ArgValue::Text(value)
    }
}

macro_rules! number_args {
    ($($ty:ty),*) => {
        $(
            impl From<$ty> for ArgValue {
                fn from(value: $ty) -> Self {
                    ArgValue::Number(value as f64)
                }
            }
        )*
    };
}

number_args!(i32, i64, u32, u64, usize, f32, f64);

/// Message arguments.
#[derive(Clone, Debug, Default)]
pub struct Args {
    pub(crate) values: Vec<(String, ArgValue)>,
}

impl Args {
    /// No arguments.
    pub fn new() -> Self {
        Args::default()
    }

    /// Sets the argument `name` (replacing an earlier value).
    pub fn set(mut self, name: &str, value: impl Into<ArgValue>) -> Self {
        let value = value.into();
        self.values.retain(|(n, _)| n != name);
        self.values.push((name.to_owned(), value));
        self
    }

    /// The arguments as Fluent's.
    pub(crate) fn fluent(&self) -> FluentArgs<'static> {
        let mut args = FluentArgs::new();
        for (name, value) in &self.values {
            match value {
                ArgValue::Text(text) => args.set(name.clone(), FluentValue::from(text.clone())),
                ArgValue::Number(n) => args.set(name.clone(), FluentValue::from(*n)),
            }
        }
        args
    }
}
