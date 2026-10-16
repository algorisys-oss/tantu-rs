//! The dashboard's English and German Fluent resources (DASH-05).

use tantu::prelude::*;

const EN: &str = "\
menu-file = File
menu-view = View
menu-help = Help
reset-layout = Reset layout
about = About
about-text = Tantu order dashboard, the Phase 4 milestone
reset-edits = Reset edits
order-number = Order { $id }
status-orders = { $count ->
    [one] { $count } order
   *[other] { $count } orders
}
status-selected = Order { $id } selected
panel-orders = Orders
panel-activity = Activity
panel-details = Details
panel-revenue = Revenue
column-no = No
column-customer = Customer
column-region = Region
column-amount = Amount
column-date = Date
column-status = Status
region-north = North
region-south = South
region-east = East
region-west = West
status-open = Open
status-shipped = Shipped
status-paid = Paid
select-order = Select an order
field-customer = Customer
field-amount = Amount
field-date = Date
field-region = Region
error-customer = Enter a customer
error-amount = Enter an amount above 0
save = Save
activity-updated = Order { $id } updated
kpi-orders = Orders
kpi-revenue = Total revenue
kpi-average = Average order
";

const DE: &str = "\
menu-file = Datei
menu-view = Ansicht
menu-help = Hilfe
reset-layout = Layout zurücksetzen
about = Info
about-text = Tantu-Auftragsübersicht, der Meilenstein von Phase 4
reset-edits = Änderungen verwerfen
order-number = Auftrag { $id }
status-orders = { $count ->
    [one] { $count } Auftrag
   *[other] { $count } Aufträge
}
status-selected = Auftrag { $id } ausgewählt
panel-orders = Aufträge
panel-activity = Aktivität
panel-details = Details
panel-revenue = Umsatz
column-no = Nr.
column-customer = Kunde
column-region = Region
column-amount = Betrag
column-date = Datum
column-status = Status
region-north = Nord
region-south = Süd
region-east = Ost
region-west = West
status-open = Offen
status-shipped = Versandt
status-paid = Bezahlt
select-order = Wählen Sie einen Auftrag
field-customer = Kunde
field-amount = Betrag
field-date = Datum
field-region = Region
error-customer = Geben Sie einen Kunden ein
error-amount = Geben Sie einen Betrag über 0 ein
save = Speichern
activity-updated = Auftrag { $id } geändert
kpi-orders = Aufträge
kpi-revenue = Gesamtumsatz
kpi-average = Durchschnittlicher Auftrag
";

fn locale(id: &str) -> Locale {
    Locale::parse(id).expect("a valid locale id")
}

/// English (the fallback) and German.
pub(crate) fn localization() -> Localization {
    let mut translations = Translations::new(locale("en"));
    for (id, source) in [("en", EN), ("de", DE)] {
        translations
            .add(locale(id), source)
            .expect("the dashboard's resources are valid Fluent");
    }
    Localization::new(translations, locale("en"))
}
