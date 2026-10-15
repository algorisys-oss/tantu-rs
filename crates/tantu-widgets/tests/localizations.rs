//! Tests for `docs/specs/i18n/localization.md` (WIDGETS-I18N-01..02) and
//! `docs/specs/i18n/formatting.md` (WIDGETS-I18N-03).

use std::cell::RefCell;
use std::rc::Rc;

use tantu_i18n::{Locale, Localization, Translations};
use tantu_test::{Finder, WidgetTester, accesskit};
use tantu_view::layout::{CrossAxisAlignment, TextDirection};
use tantu_view::reactive::signal;
use tantu_view::{Anchor, Builder, Keyed, LogicalKey, Modifiers, OverlayPortal};
use tantu_widgets::{
    CalendarDatePicker, Column, ComboBox, Date, DatePickerField, Directionality, Localizations,
    SizedBox, Weekday,
};

fn locale(text: &str) -> Locale {
    Locale::parse(text).expect("a locale")
}

/// English (the fallback) and German with two widget strings overridden, and Hebrew.
fn translations() -> Translations {
    let mut t = Translations::new(locale("en"));
    t.add(locale("en"), "app = App\n").expect("valid");
    t.add(
        locale("de"),
        "app = Anwendung\ntantu-combo-no-matches = Keine Treffer\ntantu-date-choose = Datum wählen\n",
    )
    .expect("valid");
    t.add(locale("he"), "app = יישום\n").expect("valid");
    t
}

type Seen = Rc<RefCell<Vec<(&'static str, TextDirection, Option<String>)>>>;

/// Records the direction and the localized "app" a builder sees.
fn probe(seen: Seen, name: &'static str) -> impl tantu_view::View {
    Builder::new(move |cx| {
        let text = Localization::of(cx).map(|l| l.text("app"));
        seen.borrow_mut().push((name, Directionality::of(cx), text));
        SizedBox::new(1.0, 1.0)
    })
}

#[test]
fn widgets_i18n_01_localizations() {
    let seen: Seen = Rc::default();
    let mut t = {
        let seen = seen.clone();
        WidgetTester::with_size(300.0, 200.0, move || {
            let loc = Localization::new(translations(), locale("he"));
            Column::new()
                .child(probe(seen.clone(), "outside"))
                .child(Localizations::new(
                    loc,
                    Column::new()
                        .child(probe(seen.clone(), "inside"))
                        .child(Directionality::new(
                            TextDirection::Ltr,
                            probe(seen.clone(), "override"),
                        ))
                        .child(
                            OverlayPortal::new(SizedBox::new(10.0, 10.0))
                                .visible(true)
                                .anchor(Anchor::below())
                                .overlay({
                                    let seen = seen.clone();
                                    move || probe(seen.clone(), "overlay")
                                }),
                        ),
                ))
        })
    };
    t.pump();
    t.pump();
    let seen = seen.borrow();
    let of = |name| {
        seen.iter()
            .find(|(n, _, _)| *n == name)
            .map(|(_, d, text)| (*d, text.clone()))
            .unwrap_or_else(|| panic!("{name} not built"))
    };
    assert_eq!(of("outside"), (TextDirection::Ltr, None));
    assert_eq!(of("inside"), (TextDirection::Rtl, Some("יישום".to_owned())));
    assert_eq!(of("override").0, TextDirection::Ltr);
    assert_eq!(
        of("overlay"),
        (TextDirection::Rtl, Some("יישום".to_owned()))
    );
}

#[test]
fn widgets_i18n_02_widget_strings() {
    let out: Rc<RefCell<Option<Localization>>> = Rc::default();
    let mut t = {
        let out = out.clone();
        WidgetTester::with_size(400.0, 400.0, move || {
            let loc = Localization::new(translations(), locale("de"));
            *out.borrow_mut() = Some(loc);
            Localizations::new(
                loc,
                Column::new()
                    .cross_axis_alignment(CrossAxisAlignment::Start)
                    .child(Keyed::new(
                        "combo",
                        ComboBox::new(signal(None), ["One"]).placeholder("Pick"),
                    ))
                    .child(DatePickerField::new(signal(None))),
            )
        })
    };
    t.pump();
    // The overridden button name, and the English default where German has none.
    t.find(&Finder::semantics_label("Datum wählen"));
    // A press opens the list and focuses the field; typing filters to nothing.
    t.tap(&Finder::key("combo"));
    for ch in ["z", "z"] {
        t.press_key(LogicalKey::Character(ch.into()), Modifiers::default());
    }
    t.pump();
    t.pump();
    t.find(&Finder::text("Keine Treffer"));
    // English after switching.
    let loc = out.borrow().expect("built");
    t.enter(|| loc.set_locale(locale("en")));
    t.pump();
    t.pump();
    t.find(&Finder::text("No matches"));
    t.find(&Finder::semantics_label("Choose date"));
}

#[test]
fn widgets_i18n_03_calendar() {
    let date = Date::new(2026, 10, 16).expect("exists");
    let calendar = |locale_id: &'static str, first: Option<Weekday>| {
        let mut t = WidgetTester::with_size(400.0, 400.0, move || {
            let loc = Localization::new(translations(), locale(locale_id));
            let mut picker = CalendarDatePicker::new(signal(Some(date)));
            if let Some(first) = first {
                picker = picker.first_day_of_week(first);
            }
            Localizations::new(loc, picker)
        });
        t.pump();
        t.pump();
        t
    };
    let names = |t: &mut WidgetTester, role| -> Vec<String> {
        t.a11y_tree()
            .nodes
            .into_iter()
            .filter(|(_, n)| n.role() == role)
            .map(|(_, n)| n.label().unwrap_or_default().to_owned())
            .collect()
    };
    let mut de = calendar("de", None);
    assert_eq!(names(&mut de, accesskit::Role::Grid), ["Oktober 2026"]);
    let headers = names(&mut de, accesskit::Role::ColumnHeader);
    assert_eq!(headers[0], "Montag");
    assert!(
        names(&mut de, accesskit::Role::Cell).contains(&"Freitag, 16. Oktober 2026".to_owned())
    );
    de.find(&Finder::text("Fr"));
    // English starts on Sunday unless told otherwise.
    let mut en = calendar("en", None);
    assert_eq!(names(&mut en, accesskit::Role::ColumnHeader)[0], "Sunday");
    en.find(&Finder::text("Fri"));
    let mut en = calendar("en", Some(Weekday::Monday));
    assert_eq!(names(&mut en, accesskit::Role::ColumnHeader)[0], "Monday");
}
