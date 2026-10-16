//! The gallery's translations (GALLERY-50..52): English, German, French and Hebrew Fluent
//! resources for the window's chrome, the language switch and the "i18n" kitchen-sink card.
use tantu::prelude::*;
use tantu::view::{BuildCx, Keyed};

/// Noto Sans Hebrew (OFL 1.1), queued so Hebrew renders even where no system font has it.
const HEBREW_FONT: &[u8] = include_bytes!("../fonts/NotoSansHebrew-Regular.ttf");

/// The languages offered, as (locale, its own name).
pub(super) const LANGUAGES: [(&str, &str); 4] = [
    ("en", "English"),
    ("de", "Deutsch"),
    ("fr", "Français"),
    ("he", "עברית"),
];

const EN: &str = "\
title = Tantu gallery
language = Language
section = Section: { $name }
section-buttons = Buttons
section-text = Text
section-selection = Selection
section-text-field = Text field
section-lists = Lists
section-media = Media
section-animation = Animation
section-data-grid = Data grid
section-tree-view = Tree view
section-docking = Docking
section-split-view = Split view
section-tabs = Tabs
section-kitchen-sink = Kitchen sink
inbox = { $count ->
    [one] You have { $count } new message
   *[other] You have { $count } new messages
}
more = One more
";

const DE: &str = "\
title = Tantu-Galerie
language = Sprache
section = Abschnitt: { $name }
section-buttons = Schaltflächen
section-text = Text
section-selection = Auswahl
section-text-field = Textfeld
section-lists = Listen
section-media = Medien
section-animation = Animation
section-data-grid = Datenraster
section-tree-view = Baumansicht
section-docking = Andocken
section-split-view = Geteilte Ansicht
section-tabs = Registerkarten
section-kitchen-sink = Alles auf einen Blick
inbox = { $count ->
    [one] Du hast { $count } neue Nachricht
   *[other] Du hast { $count } neue Nachrichten
}
more = Noch eine
";

const FR: &str = "\
title = Galerie Tantu
language = Langue
section = Section : { $name }
section-buttons = Boutons
section-text = Texte
section-selection = Sélection
section-text-field = Champ de texte
section-lists = Listes
section-media = Médias
section-animation = Animation
section-data-grid = Grille de données
section-tree-view = Arborescence
section-docking = Ancrage
section-split-view = Vue partagée
section-tabs = Onglets
section-kitchen-sink = Tout en un
inbox = { $count ->
    [one] Vous avez { $count } nouveau message
   *[other] Vous avez { $count } nouveaux messages
}
more = Encore un
";

const HE: &str = "\
title = גלריית Tantu
language = שפה
section = מקטע: { $name }
section-buttons = כפתורים
section-text = טקסט
section-selection = בחירה
section-text-field = שדה טקסט
section-lists = רשימות
section-media = מדיה
section-animation = הנפשה
section-data-grid = טבלת נתונים
section-tree-view = תצוגת עץ
section-docking = עגינה
section-split-view = תצוגה מפוצלת
section-tabs = כרטיסיות
section-kitchen-sink = הכול במקום אחד
inbox = { $count ->
    [one] יש לך הודעה חדשה אחת
   *[other] יש לך { $count } הודעות חדשות
}
more = עוד אחת
";

fn locale(id: &str) -> Locale {
    Locale::parse(id).expect("the gallery's locales are valid")
}

/// The gallery's localization, starting in English; queues the Hebrew font.
pub(super) fn localization(cx: &mut BuildCx<'_>) -> Localization {
    cx.ensure_font(HEBREW_FONT);
    let mut translations = Translations::new(locale("en"));
    for (id, source) in [("en", EN), ("de", DE), ("fr", FR), ("he", HE)] {
        translations
            .add(locale(id), source)
            .expect("the gallery's resources are valid Fluent");
    }
    Localization::new(translations, locale("en"))
}

/// A section's translated name.
pub(super) fn section_name(loc: Localization, section: &str) -> String {
    loc.text(&format!(
        "section-{}",
        section.to_lowercase().replace(' ', "-")
    ))
}

/// The header's language picker (GALLERY-50).
pub(super) fn language_picker(loc: Localization) -> impl View {
    let chosen = signal(Some(0usize));
    ComboBox::new(chosen, LANGUAGES.map(|(_, name)| name))
        .width(160.0)
        .label("Language")
        .on_change(move |i| loc.set_locale(locale(LANGUAGES[i].0)))
}

/// The i18n card: a plural message with a counter, and numbers, dates and times in the
/// gallery's language (GALLERY-51, -52).
pub(super) fn sink(loc: Localization) -> impl View {
    let count = signal(1i64);
    let date = Date::new(2026, 10, 16).expect("exists");
    let time = Time::new(21, 5).expect("exists");
    let body = Keyed::new(
        "sink-localizations",
        Column::new()
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .spacing(8.0)
            .child(Keyed::new(
                "sink-i18n-more",
                Button::new(move || loc.text("more")).on_press(move || count.update(|c| *c += 1)),
            ))
            .child(Keyed::new(
                "sink-i18n-format",
                Text::new(move || {
                    format!(
                        "{} · {} · {}",
                        loc.format_decimal(1_234_567.891, 2),
                        loc.format_date(date, DateLength::Long),
                        loc.format_time(time)
                    )
                }),
            )),
    );
    super::card(
        "i18n",
        "Localization",
        body,
        super::echo(move || loc.text_with("inbox", &Args::new().set("count", count.get()))),
    )
}
