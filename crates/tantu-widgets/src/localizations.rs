//! [`Localizations`]: provides a localization and its locale's direction. Spec:
//! `docs/specs/i18n/localization.md` (WIDGETS-I18N-01..02).

use tantu_i18n::Localization;
use tantu_view::{AnyView, BuildCx, ElementId, View};

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
    fn build(self, _cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (self.localization, self.child);
        todo!("WIDGETS-I18N-01")
    }
}
