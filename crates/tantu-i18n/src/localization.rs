//! [`Localization`]: the current locale, message lookup and formatting. Specs:
//! `docs/specs/i18n/localization.md` and `docs/specs/i18n/formatting.md`.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use tantu_view::BuildCx;
use tantu_view::reactive::{Signal, signal};

use crate::formats::Formats;
use crate::{Args, Date, Locale, Time, Translations, Weekday};

/// How much of a date to write.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DateLength {
    /// `10/16/26`.
    Short,
    /// `Oct 16, 2026`.
    #[default]
    Medium,
    /// `October 16, 2026`.
    Long,
    /// `Friday, October 16, 2026`.
    Full,
}

/// The current locale and the translations (`Copy`; provided by the `Localizations`
/// widget). Create it while a runtime is current (its locale is a signal).
#[derive(Clone, Copy)]
pub struct Localization {
    locale: Signal<Locale>,
    inner: Signal<Rc<Inner>>,
}

/// What every copy of a localization shares.
struct Inner {
    translations: Translations,
    /// Formatters per locale, created on first use.
    formats: RefCell<Vec<(Locale, Rc<Formats>)>>,
    /// Missing messages already logged, per locale.
    warned: RefCell<HashSet<(String, String)>>,
}

impl Localization {
    /// `translations` shown in `locale` (negotiated against the available ones).
    pub fn new(translations: Translations, locale: Locale) -> Self {
        let locale = translations.negotiate(&[locale]);
        Localization {
            locale: signal(locale),
            inner: signal(Rc::new(Inner {
                translations,
                formats: RefCell::default(),
                warned: RefCell::default(),
            })),
        }
    }

    fn inner(&self) -> Rc<Inner> {
        self.inner.get_untracked()
    }

    /// The current locale (a tracked read).
    pub fn locale(&self) -> Locale {
        self.locale.get()
    }

    /// Switches the locale (negotiated against the available ones, I18N-MSG-04).
    pub fn set_locale(&self, locale: Locale) {
        let locale = self.inner().translations.negotiate(&[locale]);
        if self.locale.get_untracked() != locale {
            self.locale.set(locale);
        }
    }

    /// The message `id` in the current locale, or `id` itself when missing (a tracked read,
    /// I18N-MSG-02).
    pub fn text(&self, id: &str) -> String {
        self.resolve(id, None)
    }

    /// The message with arguments (I18N-MSG-03).
    pub fn text_with(&self, id: &str, args: &Args) -> String {
        self.resolve(id, Some(args))
    }

    /// The message, or `None` when no locale in the chain has it (a tracked read,
    /// I18N-MSG-06).
    pub fn message(&self, id: &str, args: Option<&Args>) -> Option<String> {
        let locale = self.locale.get();
        let args = args.map(Args::fluent);
        self.inner()
            .translations
            .format(&locale.chain(), id, args.as_ref())
    }

    fn resolve(&self, id: &str, args: Option<&Args>) -> String {
        if let Some(text) = self.message(id, args) {
            return text;
        }
        let locale = self.locale.get_untracked().to_string();
        let inner = self.inner();
        if inner
            .warned
            .borrow_mut()
            .insert((locale.clone(), id.to_owned()))
        {
            tracing::warn!("no message `{id}` for {locale} or its fallbacks");
        }
        id.to_owned()
    }

    /// The nearest localization above the element being built.
    pub fn of(cx: &BuildCx<'_>) -> Option<Localization> {
        cx.inherit::<Localization>()
    }

    /// The current locale's formatters (a tracked read).
    fn formats(&self) -> Rc<Formats> {
        let locale = self.locale.get();
        let inner = self.inner();
        let mut formats = inner.formats.borrow_mut();
        if let Some((_, f)) = formats.iter().find(|(l, _)| *l == locale) {
            return Rc::clone(f);
        }
        let created = Rc::new(Formats::new(&locale));
        formats.push((locale, Rc::clone(&created)));
        created
    }

    /// `value` with the locale's grouping and digits (a tracked read, I18N-FMT-01).
    pub fn format_integer(&self, value: i64) -> String {
        self.formats().integer(value)
    }

    /// `value` rounded half-even to `fraction_digits` digits after the separator (shown even
    /// when zero); NaN and infinities are written as "NaN", "∞" and "-∞".
    pub fn format_decimal(&self, value: f64, fraction_digits: u8) -> String {
        self.formats().decimal(value, fraction_digits)
    }

    /// The date at `length` in the locale's pattern (I18N-FMT-02).
    pub fn format_date(&self, date: Date, length: DateLength) -> String {
        self.formats().date(date, length)
    }

    /// Hours and minutes, with the locale's 12- or 24-hour convention (I18N-FMT-03).
    pub fn format_time(&self, time: Time) -> String {
        self.formats().time(time)
    }

    /// A date and a time joined in the locale's way.
    pub fn format_date_time(&self, date: Date, time: Time, length: DateLength) -> String {
        self.formats().date_time(date, time, length)
    }

    /// "October", or "Oct" when `abbreviated`; empty for a month outside 1..=12
    /// (I18N-FMT-04).
    pub fn month_name(&self, month: u32, abbreviated: bool) -> String {
        self.formats().month_name(month, abbreviated)
    }

    /// "Friday", or the abbreviated form when `short`.
    pub fn weekday_name(&self, day: Weekday, short: bool) -> String {
        self.formats().weekday_name(day, short)
    }

    /// The month and year as a calendar header writes them ("October 2026").
    pub fn format_month_year(&self, year: i32, month: u32) -> String {
        self.formats().month_year(year, month)
    }

    /// The first day of the week.
    pub fn first_weekday(&self) -> Weekday {
        self.formats().first_weekday()
    }
}
