//! [`Localizations`]: provides a localization and its locale's direction; the built-in
//! widget strings. Spec: `docs/specs/i18n/localization.md` (WIDGETS-I18N-01..02).

use tantu_i18n::{Args, Localization};
use tantu_view::{AnyView, BuildCx, ElementId, Provide, View};

use crate::Directionality;

/// Provides a localization and its locale's text direction to `child` (Flutter's
/// `Localizations`, WIDGETS-I18N-01).
pub struct Localizations {
    localization: Localization,
    child: AnyView,
}

impl Localizations {
    /// `child` in `localization`'s locale and direction.
    pub fn new(localization: Localization, child: impl View) -> Self {
        Localizations {
            localization,
            child: AnyView::new(child),
        }
    }
}

impl View for Localizations {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let localization = self.localization;
        Provide::new(
            localization,
            Directionality::new(move || localization.locale().direction(), self.child),
        )
        .build(cx)
    }
}

/// A built-in string: the localization's message `id` (with `args`), else the English
/// default (a tracked read of the locale, WIDGETS-I18N-02).
pub(crate) fn string(
    localization: Option<Localization>,
    id: &str,
    args: Option<&Args>,
    english: impl FnOnce() -> String,
) -> String {
    localization
        .and_then(|l| l.message(id, args))
        .unwrap_or_else(english)
}
