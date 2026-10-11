//! The Material Icons glyph table: [`Icons`]. Generated from
//! `fonts/material-icons/MaterialIcons-Regular.codepoints` (one constant per line, the first of
//! two lines with the same name wins; names in upper snake case, digit-leading names prefixed
//! with `N`). Spec: `docs/specs/widgets/icon.md`, WIDGETS-ICON-01.

use crate::{IconData, MATERIAL_ICONS_FAMILY, MATERIAL_ICONS_FONT};

/// A Material Icons glyph.
const fn material(code_point: char) -> IconData {
    IconData::new(code_point, MATERIAL_ICONS_FAMILY).with_font(MATERIAL_ICONS_FONT)
}

/// Every Material Icons glyph, one constant each (`Icons::CHECK`, `Icons::ARROW_BACK`).
pub struct Icons;

#[allow(missing_docs)]
impl Icons {
    /// `10k` (U+E951).
    pub const N10K: IconData = material('\u{e951}');
    /// `10mp` (U+E952).
    pub const N10MP: IconData = material('\u{e952}');
    /// `11mp` (U+E953).
    pub const N11MP: IconData = material('\u{e953}');
    /// `123` (U+EB8D).
    pub const N123: IconData = material('\u{eb8d}');
    /// `12mp` (U+E954).
    pub const N12MP: IconData = material('\u{e954}');
    /// `13mp` (U+E955).
    pub const N13MP: IconData = material('\u{e955}');
    /// `14mp` (U+E956).
    pub const N14MP: IconData = material('\u{e956}');
    /// `15mp` (U+E957).
    pub const N15MP: IconData = material('\u{e957}');
    /// `16mp` (U+E958).
    pub const N16MP: IconData = material('\u{e958}');
    /// `17mp` (U+E959).
    pub const N17MP: IconData = material('\u{e959}');
    /// `18_up_rating` (U+F8FD).
    pub const N18_UP_RATING: IconData = material('\u{f8fd}');
    /// `18mp` (U+E95A).
    pub const N18MP: IconData = material('\u{e95a}');
    /// `19mp` (U+E95B).
    pub const N19MP: IconData = material('\u{e95b}');
    /// `1k` (U+E95C).
    pub const N1K: IconData = material('\u{e95c}');
    /// `1k_plus` (U+E95D).
    pub const N1K_PLUS: IconData = material('\u{e95d}');
    /// `1x_mobiledata` (U+EFCD).
    pub const N1X_MOBILEDATA: IconData = material('\u{efcd}');
    /// `20mp` (U+E95E).
    pub const N20MP: IconData = material('\u{e95e}');
    /// `21mp` (U+E95F).
    pub const N21MP: IconData = material('\u{e95f}');
    /// `22mp` (U+E960).
    pub const N22MP: IconData = material('\u{e960}');
    /// `23mp` (U+E961).
    pub const N23MP: IconData = material('\u{e961}');
    /// `24mp` (U+E962).
    pub const N24MP: IconData = material('\u{e962}');
    /// `2k` (U+E963).
    pub const N2K: IconData = material('\u{e963}');
    /// `2k_plus` (U+E964).
    pub const N2K_PLUS: IconData = material('\u{e964}');
    /// `2mp` (U+E965).
    pub const N2MP: IconData = material('\u{e965}');
    /// `30fps` (U+EFCE).
    pub const N30FPS: IconData = material('\u{efce}');
    /// `30fps_select` (U+EFCF).
    pub const N30FPS_SELECT: IconData = material('\u{efcf}');
    /// `360` (U+E577).
    pub const N360: IconData = material('\u{e577}');
    /// `3d_rotation` (U+E84D).
    pub const N3D_ROTATION: IconData = material('\u{e84d}');
    /// `3g_mobiledata` (U+EFD0).
    pub const N3G_MOBILEDATA: IconData = material('\u{efd0}');
    /// `3k` (U+E966).
    pub const N3K: IconData = material('\u{e966}');
    /// `3k_plus` (U+E967).
    pub const N3K_PLUS: IconData = material('\u{e967}');
    /// `3mp` (U+E968).
    pub const N3MP: IconData = material('\u{e968}');
    /// `3p` (U+EFD1).
    pub const N3P: IconData = material('\u{efd1}');
    /// `4g_mobiledata` (U+EFD2).
    pub const N4G_MOBILEDATA: IconData = material('\u{efd2}');
    /// `4g_plus_mobiledata` (U+EFD3).
    pub const N4G_PLUS_MOBILEDATA: IconData = material('\u{efd3}');
    /// `4k` (U+E072).
    pub const N4K: IconData = material('\u{e072}');
    /// `4k_plus` (U+E969).
    pub const N4K_PLUS: IconData = material('\u{e969}');
    /// `4mp` (U+E96A).
    pub const N4MP: IconData = material('\u{e96a}');
    /// `5g` (U+EF38).
    pub const N5G: IconData = material('\u{ef38}');
    /// `5k` (U+E96B).
    pub const N5K: IconData = material('\u{e96b}');
    /// `5k_plus` (U+E96C).
    pub const N5K_PLUS: IconData = material('\u{e96c}');
    /// `5mp` (U+E96D).
    pub const N5MP: IconData = material('\u{e96d}');
    /// `60fps` (U+EFD4).
    pub const N60FPS: IconData = material('\u{efd4}');
    /// `60fps_select` (U+EFD5).
    pub const N60FPS_SELECT: IconData = material('\u{efd5}');
    /// `6_ft_apart` (U+F21E).
    pub const N6_FT_APART: IconData = material('\u{f21e}');
    /// `6k` (U+E96E).
    pub const N6K: IconData = material('\u{e96e}');
    /// `6k_plus` (U+E96F).
    pub const N6K_PLUS: IconData = material('\u{e96f}');
    /// `6mp` (U+E970).
    pub const N6MP: IconData = material('\u{e970}');
    /// `7k` (U+E971).
    pub const N7K: IconData = material('\u{e971}');
    /// `7k_plus` (U+E972).
    pub const N7K_PLUS: IconData = material('\u{e972}');
    /// `7mp` (U+E973).
    pub const N7MP: IconData = material('\u{e973}');
    /// `8k` (U+E974).
    pub const N8K: IconData = material('\u{e974}');
    /// `8k_plus` (U+E975).
    pub const N8K_PLUS: IconData = material('\u{e975}');
    /// `8mp` (U+E976).
    pub const N8MP: IconData = material('\u{e976}');
    /// `9k` (U+E977).
    pub const N9K: IconData = material('\u{e977}');
    /// `9k_plus` (U+E978).
    pub const N9K_PLUS: IconData = material('\u{e978}');
    /// `9mp` (U+E979).
    pub const N9MP: IconData = material('\u{e979}');
    /// `abc` (U+EB94).
    pub const ABC: IconData = material('\u{eb94}');
    /// `ac_unit` (U+EB3B).
    pub const AC_UNIT: IconData = material('\u{eb3b}');
    /// `access_alarm` (U+E190).
    pub const ACCESS_ALARM: IconData = material('\u{e190}');
    /// `access_alarms` (U+E191).
    pub const ACCESS_ALARMS: IconData = material('\u{e191}');
    /// `access_time` (U+E192).
    pub const ACCESS_TIME: IconData = material('\u{e192}');
    /// `access_time_filled` (U+EFD6).
    pub const ACCESS_TIME_FILLED: IconData = material('\u{efd6}');
    /// `accessibility` (U+E84E).
    pub const ACCESSIBILITY: IconData = material('\u{e84e}');
    /// `accessibility_new` (U+E92C).
    pub const ACCESSIBILITY_NEW: IconData = material('\u{e92c}');
    /// `accessible` (U+E914).
    pub const ACCESSIBLE: IconData = material('\u{e914}');
    /// `accessible_forward` (U+E934).
    pub const ACCESSIBLE_FORWARD: IconData = material('\u{e934}');
    /// `account_balance` (U+E84F).
    pub const ACCOUNT_BALANCE: IconData = material('\u{e84f}');
    /// `account_balance_wallet` (U+E850).
    pub const ACCOUNT_BALANCE_WALLET: IconData = material('\u{e850}');
    /// `account_box` (U+E851).
    pub const ACCOUNT_BOX: IconData = material('\u{e851}');
    /// `account_circle` (U+E853).
    pub const ACCOUNT_CIRCLE: IconData = material('\u{e853}');
    /// `account_tree` (U+E97A).
    pub const ACCOUNT_TREE: IconData = material('\u{e97a}');
    /// `ad_units` (U+EF39).
    pub const AD_UNITS: IconData = material('\u{ef39}');
    /// `adb` (U+E60E).
    pub const ADB: IconData = material('\u{e60e}');
    /// `add` (U+E145).
    pub const ADD: IconData = material('\u{e145}');
    /// `add_a_photo` (U+E439).
    pub const ADD_A_PHOTO: IconData = material('\u{e439}');
    /// `add_alarm` (U+E193).
    pub const ADD_ALARM: IconData = material('\u{e193}');
    /// `add_alert` (U+E003).
    pub const ADD_ALERT: IconData = material('\u{e003}');
    /// `add_box` (U+E146).
    pub const ADD_BOX: IconData = material('\u{e146}');
    /// `add_business` (U+E729).
    pub const ADD_BUSINESS: IconData = material('\u{e729}');
    /// `add_call` (U+E0E8).
    pub const ADD_CALL: IconData = material('\u{e0e8}');
    /// `add_card` (U+EB86).
    pub const ADD_CARD: IconData = material('\u{eb86}');
    /// `add_chart` (U+E97B).
    pub const ADD_CHART: IconData = material('\u{e97b}');
    /// `add_circle` (U+E147).
    pub const ADD_CIRCLE: IconData = material('\u{e147}');
    /// `add_circle_outline` (U+E148).
    pub const ADD_CIRCLE_OUTLINE: IconData = material('\u{e148}');
    /// `add_comment` (U+E266).
    pub const ADD_COMMENT: IconData = material('\u{e266}');
    /// `add_home` (U+F8EB).
    pub const ADD_HOME: IconData = material('\u{f8eb}');
    /// `add_home_work` (U+F8ED).
    pub const ADD_HOME_WORK: IconData = material('\u{f8ed}');
    /// `add_ic_call` (U+E97C).
    pub const ADD_IC_CALL: IconData = material('\u{e97c}');
    /// `add_link` (U+E178).
    pub const ADD_LINK: IconData = material('\u{e178}');
    /// `add_location` (U+E567).
    pub const ADD_LOCATION: IconData = material('\u{e567}');
    /// `add_location_alt` (U+EF3A).
    pub const ADD_LOCATION_ALT: IconData = material('\u{ef3a}');
    /// `add_moderator` (U+E97D).
    pub const ADD_MODERATOR: IconData = material('\u{e97d}');
    /// `add_photo_alternate` (U+E43E).
    pub const ADD_PHOTO_ALTERNATE: IconData = material('\u{e43e}');
    /// `add_reaction` (U+E1D3).
    pub const ADD_REACTION: IconData = material('\u{e1d3}');
    /// `add_road` (U+EF3B).
    pub const ADD_ROAD: IconData = material('\u{ef3b}');
    /// `add_shopping_cart` (U+E854).
    pub const ADD_SHOPPING_CART: IconData = material('\u{e854}');
    /// `add_task` (U+F23A).
    pub const ADD_TASK: IconData = material('\u{f23a}');
    /// `add_to_drive` (U+E65C).
    pub const ADD_TO_DRIVE: IconData = material('\u{e65c}');
    /// `add_to_home_screen` (U+E1FE).
    pub const ADD_TO_HOME_SCREEN: IconData = material('\u{e1fe}');
    /// `add_to_photos` (U+E39D).
    pub const ADD_TO_PHOTOS: IconData = material('\u{e39d}');
    /// `add_to_queue` (U+E05C).
    pub const ADD_TO_QUEUE: IconData = material('\u{e05c}');
    /// `addchart` (U+EF3C).
    pub const ADDCHART: IconData = material('\u{ef3c}');
    /// `adf_scanner` (U+EADA).
    pub const ADF_SCANNER: IconData = material('\u{eada}');
    /// `adjust` (U+E39E).
    pub const ADJUST: IconData = material('\u{e39e}');
    /// `admin_panel_settings` (U+EF3D).
    pub const ADMIN_PANEL_SETTINGS: IconData = material('\u{ef3d}');
    /// `adobe` (U+EA96).
    pub const ADOBE: IconData = material('\u{ea96}');
    /// `ads_click` (U+E762).
    pub const ADS_CLICK: IconData = material('\u{e762}');
    /// `agriculture` (U+EA79).
    pub const AGRICULTURE: IconData = material('\u{ea79}');
    /// `air` (U+EFD8).
    pub const AIR: IconData = material('\u{efd8}');
    /// `airline_seat_flat` (U+E630).
    pub const AIRLINE_SEAT_FLAT: IconData = material('\u{e630}');
    /// `airline_seat_flat_angled` (U+E631).
    pub const AIRLINE_SEAT_FLAT_ANGLED: IconData = material('\u{e631}');
    /// `airline_seat_individual_suite` (U+E632).
    pub const AIRLINE_SEAT_INDIVIDUAL_SUITE: IconData = material('\u{e632}');
    /// `airline_seat_legroom_extra` (U+E633).
    pub const AIRLINE_SEAT_LEGROOM_EXTRA: IconData = material('\u{e633}');
    /// `airline_seat_legroom_normal` (U+E634).
    pub const AIRLINE_SEAT_LEGROOM_NORMAL: IconData = material('\u{e634}');
    /// `airline_seat_legroom_reduced` (U+E635).
    pub const AIRLINE_SEAT_LEGROOM_REDUCED: IconData = material('\u{e635}');
    /// `airline_seat_recline_extra` (U+E636).
    pub const AIRLINE_SEAT_RECLINE_EXTRA: IconData = material('\u{e636}');
    /// `airline_seat_recline_normal` (U+E637).
    pub const AIRLINE_SEAT_RECLINE_NORMAL: IconData = material('\u{e637}');
    /// `airline_stops` (U+E7D0).
    pub const AIRLINE_STOPS: IconData = material('\u{e7d0}');
    /// `airlines` (U+E7CA).
    pub const AIRLINES: IconData = material('\u{e7ca}');
    /// `airplane_ticket` (U+EFD9).
    pub const AIRPLANE_TICKET: IconData = material('\u{efd9}');
    /// `airplanemode_active` (U+E195).
    pub const AIRPLANEMODE_ACTIVE: IconData = material('\u{e195}');
    /// `airplanemode_inactive` (U+E194).
    pub const AIRPLANEMODE_INACTIVE: IconData = material('\u{e194}');
    /// `airplanemode_off` (U+E194).
    pub const AIRPLANEMODE_OFF: IconData = material('\u{e194}');
    /// `airplanemode_on` (U+E195).
    pub const AIRPLANEMODE_ON: IconData = material('\u{e195}');
    /// `airplay` (U+E055).
    pub const AIRPLAY: IconData = material('\u{e055}');
    /// `airport_shuttle` (U+EB3C).
    pub const AIRPORT_SHUTTLE: IconData = material('\u{eb3c}');
    /// `alarm` (U+E855).
    pub const ALARM: IconData = material('\u{e855}');
    /// `alarm_add` (U+E856).
    pub const ALARM_ADD: IconData = material('\u{e856}');
    /// `alarm_off` (U+E857).
    pub const ALARM_OFF: IconData = material('\u{e857}');
    /// `alarm_on` (U+E858).
    pub const ALARM_ON: IconData = material('\u{e858}');
    /// `album` (U+E019).
    pub const ALBUM: IconData = material('\u{e019}');
    /// `align_horizontal_center` (U+E00F).
    pub const ALIGN_HORIZONTAL_CENTER: IconData = material('\u{e00f}');
    /// `align_horizontal_left` (U+E00D).
    pub const ALIGN_HORIZONTAL_LEFT: IconData = material('\u{e00d}');
    /// `align_horizontal_right` (U+E010).
    pub const ALIGN_HORIZONTAL_RIGHT: IconData = material('\u{e010}');
    /// `align_vertical_bottom` (U+E015).
    pub const ALIGN_VERTICAL_BOTTOM: IconData = material('\u{e015}');
    /// `align_vertical_center` (U+E011).
    pub const ALIGN_VERTICAL_CENTER: IconData = material('\u{e011}');
    /// `align_vertical_top` (U+E00C).
    pub const ALIGN_VERTICAL_TOP: IconData = material('\u{e00c}');
    /// `all_inbox` (U+E97F).
    pub const ALL_INBOX: IconData = material('\u{e97f}');
    /// `all_inclusive` (U+EB3D).
    pub const ALL_INCLUSIVE: IconData = material('\u{eb3d}');
    /// `all_out` (U+E90B).
    pub const ALL_OUT: IconData = material('\u{e90b}');
    /// `alt_route` (U+F184).
    pub const ALT_ROUTE: IconData = material('\u{f184}');
    /// `alternate_email` (U+E0E6).
    pub const ALTERNATE_EMAIL: IconData = material('\u{e0e6}');
    /// `amp_stories` (U+EA13).
    pub const AMP_STORIES: IconData = material('\u{ea13}');
    /// `analytics` (U+EF3E).
    pub const ANALYTICS: IconData = material('\u{ef3e}');
    /// `anchor` (U+F1CD).
    pub const ANCHOR: IconData = material('\u{f1cd}');
    /// `android` (U+E859).
    pub const ANDROID: IconData = material('\u{e859}');
    /// `animation` (U+E71C).
    pub const ANIMATION: IconData = material('\u{e71c}');
    /// `announcement` (U+E85A).
    pub const ANNOUNCEMENT: IconData = material('\u{e85a}');
    /// `aod` (U+EFDA).
    pub const AOD: IconData = material('\u{efda}');
    /// `apartment` (U+EA40).
    pub const APARTMENT: IconData = material('\u{ea40}');
    /// `api` (U+F1B7).
    pub const API: IconData = material('\u{f1b7}');
    /// `app_blocking` (U+EF3F).
    pub const APP_BLOCKING: IconData = material('\u{ef3f}');
    /// `app_registration` (U+EF40).
    pub const APP_REGISTRATION: IconData = material('\u{ef40}');
    /// `app_settings_alt` (U+EF41).
    pub const APP_SETTINGS_ALT: IconData = material('\u{ef41}');
    /// `app_shortcut` (U+EAE4).
    pub const APP_SHORTCUT: IconData = material('\u{eae4}');
    /// `apple` (U+EA80).
    pub const APPLE: IconData = material('\u{ea80}');
    /// `approval` (U+E982).
    pub const APPROVAL: IconData = material('\u{e982}');
    /// `apps` (U+E5C3).
    pub const APPS: IconData = material('\u{e5c3}');
    /// `apps_outage` (U+E7CC).
    pub const APPS_OUTAGE: IconData = material('\u{e7cc}');
    /// `architecture` (U+EA3B).
    pub const ARCHITECTURE: IconData = material('\u{ea3b}');
    /// `archive` (U+E149).
    pub const ARCHIVE: IconData = material('\u{e149}');
    /// `area_chart` (U+E770).
    pub const AREA_CHART: IconData = material('\u{e770}');
    /// `arrow_back` (U+E5C4).
    pub const ARROW_BACK: IconData = material('\u{e5c4}');
    /// `arrow_back_ios` (U+E5E0).
    pub const ARROW_BACK_IOS: IconData = material('\u{e5e0}');
    /// `arrow_back_ios_new` (U+E2EA).
    pub const ARROW_BACK_IOS_NEW: IconData = material('\u{e2ea}');
    /// `arrow_circle_down` (U+F181).
    pub const ARROW_CIRCLE_DOWN: IconData = material('\u{f181}');
    /// `arrow_circle_left` (U+EAA7).
    pub const ARROW_CIRCLE_LEFT: IconData = material('\u{eaa7}');
    /// `arrow_circle_right` (U+EAAA).
    pub const ARROW_CIRCLE_RIGHT: IconData = material('\u{eaaa}');
    /// `arrow_circle_up` (U+F182).
    pub const ARROW_CIRCLE_UP: IconData = material('\u{f182}');
    /// `arrow_downward` (U+E5DB).
    pub const ARROW_DOWNWARD: IconData = material('\u{e5db}');
    /// `arrow_drop_down` (U+E5C5).
    pub const ARROW_DROP_DOWN: IconData = material('\u{e5c5}');
    /// `arrow_drop_down_circle` (U+E5C6).
    pub const ARROW_DROP_DOWN_CIRCLE: IconData = material('\u{e5c6}');
    /// `arrow_drop_up` (U+E5C7).
    pub const ARROW_DROP_UP: IconData = material('\u{e5c7}');
    /// `arrow_forward` (U+E5C8).
    pub const ARROW_FORWARD: IconData = material('\u{e5c8}');
    /// `arrow_forward_ios` (U+E5E1).
    pub const ARROW_FORWARD_IOS: IconData = material('\u{e5e1}');
    /// `arrow_left` (U+E5DE).
    pub const ARROW_LEFT: IconData = material('\u{e5de}');
    /// `arrow_outward` (U+F8CE).
    pub const ARROW_OUTWARD: IconData = material('\u{f8ce}');
    /// `arrow_right` (U+E5DF).
    pub const ARROW_RIGHT: IconData = material('\u{e5df}');
    /// `arrow_right_alt` (U+E941).
    pub const ARROW_RIGHT_ALT: IconData = material('\u{e941}');
    /// `arrow_upward` (U+E5D8).
    pub const ARROW_UPWARD: IconData = material('\u{e5d8}');
    /// `art_track` (U+E060).
    pub const ART_TRACK: IconData = material('\u{e060}');
    /// `article` (U+EF42).
    pub const ARTICLE: IconData = material('\u{ef42}');
    /// `aspect_ratio` (U+E85B).
    pub const ASPECT_RATIO: IconData = material('\u{e85b}');
    /// `assessment` (U+E85C).
    pub const ASSESSMENT: IconData = material('\u{e85c}');
    /// `assignment` (U+E85D).
    pub const ASSIGNMENT: IconData = material('\u{e85d}');
    /// `assignment_add` (U+F848).
    pub const ASSIGNMENT_ADD: IconData = material('\u{f848}');
    /// `assignment_ind` (U+E85E).
    pub const ASSIGNMENT_IND: IconData = material('\u{e85e}');
    /// `assignment_late` (U+E85F).
    pub const ASSIGNMENT_LATE: IconData = material('\u{e85f}');
    /// `assignment_return` (U+E860).
    pub const ASSIGNMENT_RETURN: IconData = material('\u{e860}');
    /// `assignment_returned` (U+E861).
    pub const ASSIGNMENT_RETURNED: IconData = material('\u{e861}');
    /// `assignment_turned_in` (U+E862).
    pub const ASSIGNMENT_TURNED_IN: IconData = material('\u{e862}');
    /// `assist_walker` (U+F8D5).
    pub const ASSIST_WALKER: IconData = material('\u{f8d5}');
    /// `assistant` (U+E39F).
    pub const ASSISTANT: IconData = material('\u{e39f}');
    /// `assistant_direction` (U+E988).
    pub const ASSISTANT_DIRECTION: IconData = material('\u{e988}');
    /// `assistant_navigation` (U+E989).
    pub const ASSISTANT_NAVIGATION: IconData = material('\u{e989}');
    /// `assistant_photo` (U+E3A0).
    pub const ASSISTANT_PHOTO: IconData = material('\u{e3a0}');
    /// `assured_workload` (U+EB6F).
    pub const ASSURED_WORKLOAD: IconData = material('\u{eb6f}');
    /// `atm` (U+E573).
    pub const ATM: IconData = material('\u{e573}');
    /// `attach_email` (U+EA5E).
    pub const ATTACH_EMAIL: IconData = material('\u{ea5e}');
    /// `attach_file` (U+E226).
    pub const ATTACH_FILE: IconData = material('\u{e226}');
    /// `attach_money` (U+E227).
    pub const ATTACH_MONEY: IconData = material('\u{e227}');
    /// `attachment` (U+E2BC).
    pub const ATTACHMENT: IconData = material('\u{e2bc}');
    /// `attractions` (U+EA52).
    pub const ATTRACTIONS: IconData = material('\u{ea52}');
    /// `attribution` (U+EFDB).
    pub const ATTRIBUTION: IconData = material('\u{efdb}');
    /// `audio_file` (U+EB82).
    pub const AUDIO_FILE: IconData = material('\u{eb82}');
    /// `audiotrack` (U+E3A1).
    pub const AUDIOTRACK: IconData = material('\u{e3a1}');
    /// `auto_awesome` (U+E65F).
    pub const AUTO_AWESOME: IconData = material('\u{e65f}');
    /// `auto_awesome_mosaic` (U+E660).
    pub const AUTO_AWESOME_MOSAIC: IconData = material('\u{e660}');
    /// `auto_awesome_motion` (U+E661).
    pub const AUTO_AWESOME_MOTION: IconData = material('\u{e661}');
    /// `auto_delete` (U+EA4C).
    pub const AUTO_DELETE: IconData = material('\u{ea4c}');
    /// `auto_fix_high` (U+E663).
    pub const AUTO_FIX_HIGH: IconData = material('\u{e663}');
    /// `auto_fix_normal` (U+E664).
    pub const AUTO_FIX_NORMAL: IconData = material('\u{e664}');
    /// `auto_fix_off` (U+E665).
    pub const AUTO_FIX_OFF: IconData = material('\u{e665}');
    /// `auto_graph` (U+E4FB).
    pub const AUTO_GRAPH: IconData = material('\u{e4fb}');
    /// `auto_mode` (U+EC20).
    pub const AUTO_MODE: IconData = material('\u{ec20}');
    /// `auto_stories` (U+E666).
    pub const AUTO_STORIES: IconData = material('\u{e666}');
    /// `autofps_select` (U+EFDC).
    pub const AUTOFPS_SELECT: IconData = material('\u{efdc}');
    /// `autorenew` (U+E863).
    pub const AUTORENEW: IconData = material('\u{e863}');
    /// `av_timer` (U+E01B).
    pub const AV_TIMER: IconData = material('\u{e01b}');
    /// `baby_changing_station` (U+F19B).
    pub const BABY_CHANGING_STATION: IconData = material('\u{f19b}');
    /// `back_hand` (U+E764).
    pub const BACK_HAND: IconData = material('\u{e764}');
    /// `backpack` (U+F19C).
    pub const BACKPACK: IconData = material('\u{f19c}');
    /// `backspace` (U+E14A).
    pub const BACKSPACE: IconData = material('\u{e14a}');
    /// `backup` (U+E864).
    pub const BACKUP: IconData = material('\u{e864}');
    /// `backup_table` (U+EF43).
    pub const BACKUP_TABLE: IconData = material('\u{ef43}');
    /// `badge` (U+EA67).
    pub const BADGE: IconData = material('\u{ea67}');
    /// `bakery_dining` (U+EA53).
    pub const BAKERY_DINING: IconData = material('\u{ea53}');
    /// `balance` (U+EAF6).
    pub const BALANCE: IconData = material('\u{eaf6}');
    /// `balcony` (U+E58F).
    pub const BALCONY: IconData = material('\u{e58f}');
    /// `ballot` (U+E172).
    pub const BALLOT: IconData = material('\u{e172}');
    /// `bar_chart` (U+E26B).
    pub const BAR_CHART: IconData = material('\u{e26b}');
    /// `barcode_reader` (U+F85C).
    pub const BARCODE_READER: IconData = material('\u{f85c}');
    /// `batch_prediction` (U+F0F5).
    pub const BATCH_PREDICTION: IconData = material('\u{f0f5}');
    /// `bathroom` (U+EFDD).
    pub const BATHROOM: IconData = material('\u{efdd}');
    /// `bathtub` (U+EA41).
    pub const BATHTUB: IconData = material('\u{ea41}');
    /// `battery_0_bar` (U+EBDC).
    pub const BATTERY_0_BAR: IconData = material('\u{ebdc}');
    /// `battery_1_bar` (U+EBD9).
    pub const BATTERY_1_BAR: IconData = material('\u{ebd9}');
    /// `battery_2_bar` (U+EBE0).
    pub const BATTERY_2_BAR: IconData = material('\u{ebe0}');
    /// `battery_3_bar` (U+EBDD).
    pub const BATTERY_3_BAR: IconData = material('\u{ebdd}');
    /// `battery_4_bar` (U+EBE2).
    pub const BATTERY_4_BAR: IconData = material('\u{ebe2}');
    /// `battery_5_bar` (U+EBD4).
    pub const BATTERY_5_BAR: IconData = material('\u{ebd4}');
    /// `battery_6_bar` (U+EBD2).
    pub const BATTERY_6_BAR: IconData = material('\u{ebd2}');
    /// `battery_alert` (U+E19C).
    pub const BATTERY_ALERT: IconData = material('\u{e19c}');
    /// `battery_charging_full` (U+E1A3).
    pub const BATTERY_CHARGING_FULL: IconData = material('\u{e1a3}');
    /// `battery_full` (U+E1A4).
    pub const BATTERY_FULL: IconData = material('\u{e1a4}');
    /// `battery_saver` (U+EFDE).
    pub const BATTERY_SAVER: IconData = material('\u{efde}');
    /// `battery_std` (U+E1A5).
    pub const BATTERY_STD: IconData = material('\u{e1a5}');
    /// `battery_unknown` (U+E1A6).
    pub const BATTERY_UNKNOWN: IconData = material('\u{e1a6}');
    /// `beach_access` (U+EB3E).
    pub const BEACH_ACCESS: IconData = material('\u{eb3e}');
    /// `bed` (U+EFDF).
    pub const BED: IconData = material('\u{efdf}');
    /// `bedroom_baby` (U+EFE0).
    pub const BEDROOM_BABY: IconData = material('\u{efe0}');
    /// `bedroom_child` (U+EFE1).
    pub const BEDROOM_CHILD: IconData = material('\u{efe1}');
    /// `bedroom_parent` (U+EFE2).
    pub const BEDROOM_PARENT: IconData = material('\u{efe2}');
    /// `bedtime` (U+EF44).
    pub const BEDTIME: IconData = material('\u{ef44}');
    /// `bedtime_off` (U+EB76).
    pub const BEDTIME_OFF: IconData = material('\u{eb76}');
    /// `beenhere` (U+E52D).
    pub const BEENHERE: IconData = material('\u{e52d}');
    /// `bento` (U+F1F4).
    pub const BENTO: IconData = material('\u{f1f4}');
    /// `bike_scooter` (U+EF45).
    pub const BIKE_SCOOTER: IconData = material('\u{ef45}');
    /// `biotech` (U+EA3A).
    pub const BIOTECH: IconData = material('\u{ea3a}');
    /// `blender` (U+EFE3).
    pub const BLENDER: IconData = material('\u{efe3}');
    /// `blind` (U+F8D6).
    pub const BLIND: IconData = material('\u{f8d6}');
    /// `blinds` (U+E286).
    pub const BLINDS: IconData = material('\u{e286}');
    /// `blinds_closed` (U+EC1F).
    pub const BLINDS_CLOSED: IconData = material('\u{ec1f}');
    /// `block` (U+E14B).
    pub const BLOCK: IconData = material('\u{e14b}');
    /// `block_flipped` (U+EF46).
    pub const BLOCK_FLIPPED: IconData = material('\u{ef46}');
    /// `bloodtype` (U+EFE4).
    pub const BLOODTYPE: IconData = material('\u{efe4}');
    /// `bluetooth` (U+E1A7).
    pub const BLUETOOTH: IconData = material('\u{e1a7}');
    /// `bluetooth_audio` (U+E60F).
    pub const BLUETOOTH_AUDIO: IconData = material('\u{e60f}');
    /// `bluetooth_connected` (U+E1A8).
    pub const BLUETOOTH_CONNECTED: IconData = material('\u{e1a8}');
    /// `bluetooth_disabled` (U+E1A9).
    pub const BLUETOOTH_DISABLED: IconData = material('\u{e1a9}');
    /// `bluetooth_drive` (U+EFE5).
    pub const BLUETOOTH_DRIVE: IconData = material('\u{efe5}');
    /// `bluetooth_searching` (U+E1AA).
    pub const BLUETOOTH_SEARCHING: IconData = material('\u{e1aa}');
    /// `blur_circular` (U+E3A2).
    pub const BLUR_CIRCULAR: IconData = material('\u{e3a2}');
    /// `blur_linear` (U+E3A3).
    pub const BLUR_LINEAR: IconData = material('\u{e3a3}');
    /// `blur_off` (U+E3A4).
    pub const BLUR_OFF: IconData = material('\u{e3a4}');
    /// `blur_on` (U+E3A5).
    pub const BLUR_ON: IconData = material('\u{e3a5}');
    /// `bolt` (U+EA0B).
    pub const BOLT: IconData = material('\u{ea0b}');
    /// `book` (U+E865).
    pub const BOOK: IconData = material('\u{e865}');
    /// `book_online` (U+F217).
    pub const BOOK_ONLINE: IconData = material('\u{f217}');
    /// `bookmark` (U+E866).
    pub const BOOKMARK: IconData = material('\u{e866}');
    /// `bookmark_add` (U+E598).
    pub const BOOKMARK_ADD: IconData = material('\u{e598}');
    /// `bookmark_added` (U+E599).
    pub const BOOKMARK_ADDED: IconData = material('\u{e599}');
    /// `bookmark_border` (U+E867).
    pub const BOOKMARK_BORDER: IconData = material('\u{e867}');
    /// `bookmark_outline` (U+E867).
    pub const BOOKMARK_OUTLINE: IconData = material('\u{e867}');
    /// `bookmark_remove` (U+E59A).
    pub const BOOKMARK_REMOVE: IconData = material('\u{e59a}');
    /// `bookmarks` (U+E98B).
    pub const BOOKMARKS: IconData = material('\u{e98b}');
    /// `border_all` (U+E228).
    pub const BORDER_ALL: IconData = material('\u{e228}');
    /// `border_bottom` (U+E229).
    pub const BORDER_BOTTOM: IconData = material('\u{e229}');
    /// `border_clear` (U+E22A).
    pub const BORDER_CLEAR: IconData = material('\u{e22a}');
    /// `border_color` (U+E22B).
    pub const BORDER_COLOR: IconData = material('\u{e22b}');
    /// `border_horizontal` (U+E22C).
    pub const BORDER_HORIZONTAL: IconData = material('\u{e22c}');
    /// `border_inner` (U+E22D).
    pub const BORDER_INNER: IconData = material('\u{e22d}');
    /// `border_left` (U+E22E).
    pub const BORDER_LEFT: IconData = material('\u{e22e}');
    /// `border_outer` (U+E22F).
    pub const BORDER_OUTER: IconData = material('\u{e22f}');
    /// `border_right` (U+E230).
    pub const BORDER_RIGHT: IconData = material('\u{e230}');
    /// `border_style` (U+E231).
    pub const BORDER_STYLE: IconData = material('\u{e231}');
    /// `border_top` (U+E232).
    pub const BORDER_TOP: IconData = material('\u{e232}');
    /// `border_vertical` (U+E233).
    pub const BORDER_VERTICAL: IconData = material('\u{e233}');
    /// `boy` (U+EB67).
    pub const BOY: IconData = material('\u{eb67}');
    /// `branding_watermark` (U+E06B).
    pub const BRANDING_WATERMARK: IconData = material('\u{e06b}');
    /// `breakfast_dining` (U+EA54).
    pub const BREAKFAST_DINING: IconData = material('\u{ea54}');
    /// `brightness_1` (U+E3A6).
    pub const BRIGHTNESS_1: IconData = material('\u{e3a6}');
    /// `brightness_2` (U+E3A7).
    pub const BRIGHTNESS_2: IconData = material('\u{e3a7}');
    /// `brightness_3` (U+E3A8).
    pub const BRIGHTNESS_3: IconData = material('\u{e3a8}');
    /// `brightness_4` (U+E3A9).
    pub const BRIGHTNESS_4: IconData = material('\u{e3a9}');
    /// `brightness_5` (U+E3AA).
    pub const BRIGHTNESS_5: IconData = material('\u{e3aa}');
    /// `brightness_6` (U+E3AB).
    pub const BRIGHTNESS_6: IconData = material('\u{e3ab}');
    /// `brightness_7` (U+E3AC).
    pub const BRIGHTNESS_7: IconData = material('\u{e3ac}');
    /// `brightness_auto` (U+E1AB).
    pub const BRIGHTNESS_AUTO: IconData = material('\u{e1ab}');
    /// `brightness_high` (U+E1AC).
    pub const BRIGHTNESS_HIGH: IconData = material('\u{e1ac}');
    /// `brightness_low` (U+E1AD).
    pub const BRIGHTNESS_LOW: IconData = material('\u{e1ad}');
    /// `brightness_medium` (U+E1AE).
    pub const BRIGHTNESS_MEDIUM: IconData = material('\u{e1ae}');
    /// `broadcast_on_home` (U+F8F8).
    pub const BROADCAST_ON_HOME: IconData = material('\u{f8f8}');
    /// `broadcast_on_personal` (U+F8F9).
    pub const BROADCAST_ON_PERSONAL: IconData = material('\u{f8f9}');
    /// `broken_image` (U+E3AD).
    pub const BROKEN_IMAGE: IconData = material('\u{e3ad}');
    /// `browse_gallery` (U+EBD1).
    pub const BROWSE_GALLERY: IconData = material('\u{ebd1}');
    /// `browser_not_supported` (U+EF47).
    pub const BROWSER_NOT_SUPPORTED: IconData = material('\u{ef47}');
    /// `browser_updated` (U+E7CF).
    pub const BROWSER_UPDATED: IconData = material('\u{e7cf}');
    /// `brunch_dining` (U+EA73).
    pub const BRUNCH_DINING: IconData = material('\u{ea73}');
    /// `brush` (U+E3AE).
    pub const BRUSH: IconData = material('\u{e3ae}');
    /// `bubble_chart` (U+E6DD).
    pub const BUBBLE_CHART: IconData = material('\u{e6dd}');
    /// `bug_report` (U+E868).
    pub const BUG_REPORT: IconData = material('\u{e868}');
    /// `build` (U+E869).
    pub const BUILD: IconData = material('\u{e869}');
    /// `build_circle` (U+EF48).
    pub const BUILD_CIRCLE: IconData = material('\u{ef48}');
    /// `bungalow` (U+E591).
    pub const BUNGALOW: IconData = material('\u{e591}');
    /// `burst_mode` (U+E43C).
    pub const BURST_MODE: IconData = material('\u{e43c}');
    /// `bus_alert` (U+E98F).
    pub const BUS_ALERT: IconData = material('\u{e98f}');
    /// `business` (U+E0AF).
    pub const BUSINESS: IconData = material('\u{e0af}');
    /// `business_center` (U+EB3F).
    pub const BUSINESS_CENTER: IconData = material('\u{eb3f}');
    /// `cabin` (U+E589).
    pub const CABIN: IconData = material('\u{e589}');
    /// `cable` (U+EFE6).
    pub const CABLE: IconData = material('\u{efe6}');
    /// `cached` (U+E86A).
    pub const CACHED: IconData = material('\u{e86a}');
    /// `cake` (U+E7E9).
    pub const CAKE: IconData = material('\u{e7e9}');
    /// `calculate` (U+EA5F).
    pub const CALCULATE: IconData = material('\u{ea5f}');
    /// `calendar_month` (U+EBCC).
    pub const CALENDAR_MONTH: IconData = material('\u{ebcc}');
    /// `calendar_today` (U+E935).
    pub const CALENDAR_TODAY: IconData = material('\u{e935}');
    /// `calendar_view_day` (U+E936).
    pub const CALENDAR_VIEW_DAY: IconData = material('\u{e936}');
    /// `calendar_view_month` (U+EFE7).
    pub const CALENDAR_VIEW_MONTH: IconData = material('\u{efe7}');
    /// `calendar_view_week` (U+EFE8).
    pub const CALENDAR_VIEW_WEEK: IconData = material('\u{efe8}');
    /// `call` (U+E0B0).
    pub const CALL: IconData = material('\u{e0b0}');
    /// `call_end` (U+E0B1).
    pub const CALL_END: IconData = material('\u{e0b1}');
    /// `call_made` (U+E0B2).
    pub const CALL_MADE: IconData = material('\u{e0b2}');
    /// `call_merge` (U+E0B3).
    pub const CALL_MERGE: IconData = material('\u{e0b3}');
    /// `call_missed` (U+E0B4).
    pub const CALL_MISSED: IconData = material('\u{e0b4}');
    /// `call_missed_outgoing` (U+E0E4).
    pub const CALL_MISSED_OUTGOING: IconData = material('\u{e0e4}');
    /// `call_received` (U+E0B5).
    pub const CALL_RECEIVED: IconData = material('\u{e0b5}');
    /// `call_split` (U+E0B6).
    pub const CALL_SPLIT: IconData = material('\u{e0b6}');
    /// `call_to_action` (U+E06C).
    pub const CALL_TO_ACTION: IconData = material('\u{e06c}');
    /// `camera` (U+E3AF).
    pub const CAMERA: IconData = material('\u{e3af}');
    /// `camera_alt` (U+E3B0).
    pub const CAMERA_ALT: IconData = material('\u{e3b0}');
    /// `camera_enhance` (U+E8FC).
    pub const CAMERA_ENHANCE: IconData = material('\u{e8fc}');
    /// `camera_front` (U+E3B1).
    pub const CAMERA_FRONT: IconData = material('\u{e3b1}');
    /// `camera_indoor` (U+EFE9).
    pub const CAMERA_INDOOR: IconData = material('\u{efe9}');
    /// `camera_outdoor` (U+EFEA).
    pub const CAMERA_OUTDOOR: IconData = material('\u{efea}');
    /// `camera_rear` (U+E3B2).
    pub const CAMERA_REAR: IconData = material('\u{e3b2}');
    /// `camera_roll` (U+E3B3).
    pub const CAMERA_ROLL: IconData = material('\u{e3b3}');
    /// `cameraswitch` (U+EFEB).
    pub const CAMERASWITCH: IconData = material('\u{efeb}');
    /// `campaign` (U+EF49).
    pub const CAMPAIGN: IconData = material('\u{ef49}');
    /// `cancel` (U+E5C9).
    pub const CANCEL: IconData = material('\u{e5c9}');
    /// `cancel_presentation` (U+E0E9).
    pub const CANCEL_PRESENTATION: IconData = material('\u{e0e9}');
    /// `cancel_schedule_send` (U+EA39).
    pub const CANCEL_SCHEDULE_SEND: IconData = material('\u{ea39}');
    /// `candlestick_chart` (U+EAD4).
    pub const CANDLESTICK_CHART: IconData = material('\u{ead4}');
    /// `car_crash` (U+EBF2).
    pub const CAR_CRASH: IconData = material('\u{ebf2}');
    /// `car_rental` (U+EA55).
    pub const CAR_RENTAL: IconData = material('\u{ea55}');
    /// `car_repair` (U+EA56).
    pub const CAR_REPAIR: IconData = material('\u{ea56}');
    /// `card_giftcard` (U+E8F6).
    pub const CARD_GIFTCARD: IconData = material('\u{e8f6}');
    /// `card_membership` (U+E8F7).
    pub const CARD_MEMBERSHIP: IconData = material('\u{e8f7}');
    /// `card_travel` (U+E8F8).
    pub const CARD_TRAVEL: IconData = material('\u{e8f8}');
    /// `carpenter` (U+F1F8).
    pub const CARPENTER: IconData = material('\u{f1f8}');
    /// `cases` (U+E992).
    pub const CASES: IconData = material('\u{e992}');
    /// `casino` (U+EB40).
    pub const CASINO: IconData = material('\u{eb40}');
    /// `cast` (U+E307).
    pub const CAST: IconData = material('\u{e307}');
    /// `cast_connected` (U+E308).
    pub const CAST_CONNECTED: IconData = material('\u{e308}');
    /// `cast_for_education` (U+EFEC).
    pub const CAST_FOR_EDUCATION: IconData = material('\u{efec}');
    /// `castle` (U+EAB1).
    pub const CASTLE: IconData = material('\u{eab1}');
    /// `catching_pokemon` (U+E508).
    pub const CATCHING_POKEMON: IconData = material('\u{e508}');
    /// `category` (U+E574).
    pub const CATEGORY: IconData = material('\u{e574}');
    /// `celebration` (U+EA65).
    pub const CELEBRATION: IconData = material('\u{ea65}');
    /// `cell_tower` (U+EBBA).
    pub const CELL_TOWER: IconData = material('\u{ebba}');
    /// `cell_wifi` (U+E0EC).
    pub const CELL_WIFI: IconData = material('\u{e0ec}');
    /// `center_focus_strong` (U+E3B4).
    pub const CENTER_FOCUS_STRONG: IconData = material('\u{e3b4}');
    /// `center_focus_weak` (U+E3B5).
    pub const CENTER_FOCUS_WEAK: IconData = material('\u{e3b5}');
    /// `chair` (U+EFED).
    pub const CHAIR: IconData = material('\u{efed}');
    /// `chair_alt` (U+EFEE).
    pub const CHAIR_ALT: IconData = material('\u{efee}');
    /// `chalet` (U+E585).
    pub const CHALET: IconData = material('\u{e585}');
    /// `change_circle` (U+E2E7).
    pub const CHANGE_CIRCLE: IconData = material('\u{e2e7}');
    /// `change_history` (U+E86B).
    pub const CHANGE_HISTORY: IconData = material('\u{e86b}');
    /// `charging_station` (U+F19D).
    pub const CHARGING_STATION: IconData = material('\u{f19d}');
    /// `chat` (U+E0B7).
    pub const CHAT: IconData = material('\u{e0b7}');
    /// `chat_bubble` (U+E0CA).
    pub const CHAT_BUBBLE: IconData = material('\u{e0ca}');
    /// `chat_bubble_outline` (U+E0CB).
    pub const CHAT_BUBBLE_OUTLINE: IconData = material('\u{e0cb}');
    /// `check` (U+E5CA).
    pub const CHECK: IconData = material('\u{e5ca}');
    /// `check_box` (U+E834).
    pub const CHECK_BOX: IconData = material('\u{e834}');
    /// `check_box_outline_blank` (U+E835).
    pub const CHECK_BOX_OUTLINE_BLANK: IconData = material('\u{e835}');
    /// `check_circle` (U+E86C).
    pub const CHECK_CIRCLE: IconData = material('\u{e86c}');
    /// `check_circle_outline` (U+E92D).
    pub const CHECK_CIRCLE_OUTLINE: IconData = material('\u{e92d}');
    /// `checklist` (U+E6B1).
    pub const CHECKLIST: IconData = material('\u{e6b1}');
    /// `checklist_rtl` (U+E6B3).
    pub const CHECKLIST_RTL: IconData = material('\u{e6b3}');
    /// `checkroom` (U+F19E).
    pub const CHECKROOM: IconData = material('\u{f19e}');
    /// `chevron_left` (U+E5CB).
    pub const CHEVRON_LEFT: IconData = material('\u{e5cb}');
    /// `chevron_right` (U+E5CC).
    pub const CHEVRON_RIGHT: IconData = material('\u{e5cc}');
    /// `child_care` (U+EB41).
    pub const CHILD_CARE: IconData = material('\u{eb41}');
    /// `child_friendly` (U+EB42).
    pub const CHILD_FRIENDLY: IconData = material('\u{eb42}');
    /// `chrome_reader_mode` (U+E86D).
    pub const CHROME_READER_MODE: IconData = material('\u{e86d}');
    /// `church` (U+EAAE).
    pub const CHURCH: IconData = material('\u{eaae}');
    /// `circle` (U+EF4A).
    pub const CIRCLE: IconData = material('\u{ef4a}');
    /// `circle_notifications` (U+E994).
    pub const CIRCLE_NOTIFICATIONS: IconData = material('\u{e994}');
    /// `class` (U+E86E).
    pub const CLASS: IconData = material('\u{e86e}');
    /// `clean_hands` (U+F21F).
    pub const CLEAN_HANDS: IconData = material('\u{f21f}');
    /// `cleaning_services` (U+F0FF).
    pub const CLEANING_SERVICES: IconData = material('\u{f0ff}');
    /// `clear` (U+E14C).
    pub const CLEAR: IconData = material('\u{e14c}');
    /// `clear_all` (U+E0B8).
    pub const CLEAR_ALL: IconData = material('\u{e0b8}');
    /// `close` (U+E5CD).
    pub const CLOSE: IconData = material('\u{e5cd}');
    /// `close_fullscreen` (U+F1CF).
    pub const CLOSE_FULLSCREEN: IconData = material('\u{f1cf}');
    /// `closed_caption` (U+E01C).
    pub const CLOSED_CAPTION: IconData = material('\u{e01c}');
    /// `closed_caption_disabled` (U+F1DC).
    pub const CLOSED_CAPTION_DISABLED: IconData = material('\u{f1dc}');
    /// `closed_caption_off` (U+E996).
    pub const CLOSED_CAPTION_OFF: IconData = material('\u{e996}');
    /// `cloud` (U+E2BD).
    pub const CLOUD: IconData = material('\u{e2bd}');
    /// `cloud_circle` (U+E2BE).
    pub const CLOUD_CIRCLE: IconData = material('\u{e2be}');
    /// `cloud_done` (U+E2BF).
    pub const CLOUD_DONE: IconData = material('\u{e2bf}');
    /// `cloud_download` (U+E2C0).
    pub const CLOUD_DOWNLOAD: IconData = material('\u{e2c0}');
    /// `cloud_off` (U+E2C1).
    pub const CLOUD_OFF: IconData = material('\u{e2c1}');
    /// `cloud_queue` (U+E2C2).
    pub const CLOUD_QUEUE: IconData = material('\u{e2c2}');
    /// `cloud_sync` (U+EB5A).
    pub const CLOUD_SYNC: IconData = material('\u{eb5a}');
    /// `cloud_upload` (U+E2C3).
    pub const CLOUD_UPLOAD: IconData = material('\u{e2c3}');
    /// `cloudy_snowing` (U+E810).
    pub const CLOUDY_SNOWING: IconData = material('\u{e810}');
    /// `co2` (U+E7B0).
    pub const CO2: IconData = material('\u{e7b0}');
    /// `co_present` (U+EAF0).
    pub const CO_PRESENT: IconData = material('\u{eaf0}');
    /// `code` (U+E86F).
    pub const CODE: IconData = material('\u{e86f}');
    /// `code_off` (U+E4F3).
    pub const CODE_OFF: IconData = material('\u{e4f3}');
    /// `coffee` (U+EFEF).
    pub const COFFEE: IconData = material('\u{efef}');
    /// `coffee_maker` (U+EFF0).
    pub const COFFEE_MAKER: IconData = material('\u{eff0}');
    /// `collections` (U+E3B6).
    pub const COLLECTIONS: IconData = material('\u{e3b6}');
    /// `collections_bookmark` (U+E431).
    pub const COLLECTIONS_BOOKMARK: IconData = material('\u{e431}');
    /// `color_lens` (U+E3B7).
    pub const COLOR_LENS: IconData = material('\u{e3b7}');
    /// `colorize` (U+E3B8).
    pub const COLORIZE: IconData = material('\u{e3b8}');
    /// `comment` (U+E0B9).
    pub const COMMENT: IconData = material('\u{e0b9}');
    /// `comment_bank` (U+EA4E).
    pub const COMMENT_BANK: IconData = material('\u{ea4e}');
    /// `comments_disabled` (U+E7A2).
    pub const COMMENTS_DISABLED: IconData = material('\u{e7a2}');
    /// `commit` (U+EAF5).
    pub const COMMIT: IconData = material('\u{eaf5}');
    /// `commute` (U+E940).
    pub const COMMUTE: IconData = material('\u{e940}');
    /// `compare` (U+E3B9).
    pub const COMPARE: IconData = material('\u{e3b9}');
    /// `compare_arrows` (U+E915).
    pub const COMPARE_ARROWS: IconData = material('\u{e915}');
    /// `compass_calibration` (U+E57C).
    pub const COMPASS_CALIBRATION: IconData = material('\u{e57c}');
    /// `compost` (U+E761).
    pub const COMPOST: IconData = material('\u{e761}');
    /// `compress` (U+E94D).
    pub const COMPRESS: IconData = material('\u{e94d}');
    /// `computer` (U+E30A).
    pub const COMPUTER: IconData = material('\u{e30a}');
    /// `confirmation_num` (U+E638).
    pub const CONFIRMATION_NUM: IconData = material('\u{e638}');
    /// `confirmation_number` (U+E638).
    pub const CONFIRMATION_NUMBER: IconData = material('\u{e638}');
    /// `connect_without_contact` (U+F223).
    pub const CONNECT_WITHOUT_CONTACT: IconData = material('\u{f223}');
    /// `connected_tv` (U+E998).
    pub const CONNECTED_TV: IconData = material('\u{e998}');
    /// `connecting_airports` (U+E7C9).
    pub const CONNECTING_AIRPORTS: IconData = material('\u{e7c9}');
    /// `construction` (U+EA3C).
    pub const CONSTRUCTION: IconData = material('\u{ea3c}');
    /// `contact_emergency` (U+F8D1).
    pub const CONTACT_EMERGENCY: IconData = material('\u{f8d1}');
    /// `contact_mail` (U+E0D0).
    pub const CONTACT_MAIL: IconData = material('\u{e0d0}');
    /// `contact_page` (U+F22E).
    pub const CONTACT_PAGE: IconData = material('\u{f22e}');
    /// `contact_phone` (U+E0CF).
    pub const CONTACT_PHONE: IconData = material('\u{e0cf}');
    /// `contact_support` (U+E94C).
    pub const CONTACT_SUPPORT: IconData = material('\u{e94c}');
    /// `contactless` (U+EA71).
    pub const CONTACTLESS: IconData = material('\u{ea71}');
    /// `contacts` (U+E0BA).
    pub const CONTACTS: IconData = material('\u{e0ba}');
    /// `content_copy` (U+E14D).
    pub const CONTENT_COPY: IconData = material('\u{e14d}');
    /// `content_cut` (U+E14E).
    pub const CONTENT_CUT: IconData = material('\u{e14e}');
    /// `content_paste` (U+E14F).
    pub const CONTENT_PASTE: IconData = material('\u{e14f}');
    /// `content_paste_go` (U+EA8E).
    pub const CONTENT_PASTE_GO: IconData = material('\u{ea8e}');
    /// `content_paste_off` (U+E4F8).
    pub const CONTENT_PASTE_OFF: IconData = material('\u{e4f8}');
    /// `content_paste_search` (U+EA9B).
    pub const CONTENT_PASTE_SEARCH: IconData = material('\u{ea9b}');
    /// `contrast` (U+EB37).
    pub const CONTRAST: IconData = material('\u{eb37}');
    /// `control_camera` (U+E074).
    pub const CONTROL_CAMERA: IconData = material('\u{e074}');
    /// `control_point` (U+E3BA).
    pub const CONTROL_POINT: IconData = material('\u{e3ba}');
    /// `control_point_duplicate` (U+E3BB).
    pub const CONTROL_POINT_DUPLICATE: IconData = material('\u{e3bb}');
    /// `conveyor_belt` (U+F867).
    pub const CONVEYOR_BELT: IconData = material('\u{f867}');
    /// `cookie` (U+EAAC).
    pub const COOKIE: IconData = material('\u{eaac}');
    /// `copy_all` (U+E2EC).
    pub const COPY_ALL: IconData = material('\u{e2ec}');
    /// `copyright` (U+E90C).
    pub const COPYRIGHT: IconData = material('\u{e90c}');
    /// `coronavirus` (U+F221).
    pub const CORONAVIRUS: IconData = material('\u{f221}');
    /// `corporate_fare` (U+F1D0).
    pub const CORPORATE_FARE: IconData = material('\u{f1d0}');
    /// `cottage` (U+E587).
    pub const COTTAGE: IconData = material('\u{e587}');
    /// `countertops` (U+F1F7).
    pub const COUNTERTOPS: IconData = material('\u{f1f7}');
    /// `create` (U+E150).
    pub const CREATE: IconData = material('\u{e150}');
    /// `create_new_folder` (U+E2CC).
    pub const CREATE_NEW_FOLDER: IconData = material('\u{e2cc}');
    /// `credit_card` (U+E870).
    pub const CREDIT_CARD: IconData = material('\u{e870}');
    /// `credit_card_off` (U+E4F4).
    pub const CREDIT_CARD_OFF: IconData = material('\u{e4f4}');
    /// `credit_score` (U+EFF1).
    pub const CREDIT_SCORE: IconData = material('\u{eff1}');
    /// `crib` (U+E588).
    pub const CRIB: IconData = material('\u{e588}');
    /// `crisis_alert` (U+EBE9).
    pub const CRISIS_ALERT: IconData = material('\u{ebe9}');
    /// `crop` (U+E3BE).
    pub const CROP: IconData = material('\u{e3be}');
    /// `crop_16_9` (U+E3BC).
    pub const CROP_16_9: IconData = material('\u{e3bc}');
    /// `crop_3_2` (U+E3BD).
    pub const CROP_3_2: IconData = material('\u{e3bd}');
    /// `crop_5_4` (U+E3BF).
    pub const CROP_5_4: IconData = material('\u{e3bf}');
    /// `crop_7_5` (U+E3C0).
    pub const CROP_7_5: IconData = material('\u{e3c0}');
    /// `crop_din` (U+E3C1).
    pub const CROP_DIN: IconData = material('\u{e3c1}');
    /// `crop_free` (U+E3C2).
    pub const CROP_FREE: IconData = material('\u{e3c2}');
    /// `crop_landscape` (U+E3C3).
    pub const CROP_LANDSCAPE: IconData = material('\u{e3c3}');
    /// `crop_original` (U+E3C4).
    pub const CROP_ORIGINAL: IconData = material('\u{e3c4}');
    /// `crop_portrait` (U+E3C5).
    pub const CROP_PORTRAIT: IconData = material('\u{e3c5}');
    /// `crop_rotate` (U+E437).
    pub const CROP_ROTATE: IconData = material('\u{e437}');
    /// `crop_square` (U+E3C6).
    pub const CROP_SQUARE: IconData = material('\u{e3c6}');
    /// `cruelty_free` (U+E799).
    pub const CRUELTY_FREE: IconData = material('\u{e799}');
    /// `css` (U+EB93).
    pub const CSS: IconData = material('\u{eb93}');
    /// `currency_bitcoin` (U+EBC5).
    pub const CURRENCY_BITCOIN: IconData = material('\u{ebc5}');
    /// `currency_exchange` (U+EB70).
    pub const CURRENCY_EXCHANGE: IconData = material('\u{eb70}');
    /// `currency_franc` (U+EAFA).
    pub const CURRENCY_FRANC: IconData = material('\u{eafa}');
    /// `currency_lira` (U+EAEF).
    pub const CURRENCY_LIRA: IconData = material('\u{eaef}');
    /// `currency_pound` (U+EAF1).
    pub const CURRENCY_POUND: IconData = material('\u{eaf1}');
    /// `currency_ruble` (U+EAEC).
    pub const CURRENCY_RUBLE: IconData = material('\u{eaec}');
    /// `currency_rupee` (U+EAF7).
    pub const CURRENCY_RUPEE: IconData = material('\u{eaf7}');
    /// `currency_yen` (U+EAFB).
    pub const CURRENCY_YEN: IconData = material('\u{eafb}');
    /// `currency_yuan` (U+EAF9).
    pub const CURRENCY_YUAN: IconData = material('\u{eaf9}');
    /// `curtains` (U+EC1E).
    pub const CURTAINS: IconData = material('\u{ec1e}');
    /// `curtains_closed` (U+EC1D).
    pub const CURTAINS_CLOSED: IconData = material('\u{ec1d}');
    /// `cyclone` (U+EBD5).
    pub const CYCLONE: IconData = material('\u{ebd5}');
    /// `dangerous` (U+E99A).
    pub const DANGEROUS: IconData = material('\u{e99a}');
    /// `dark_mode` (U+E51C).
    pub const DARK_MODE: IconData = material('\u{e51c}');
    /// `dashboard` (U+E871).
    pub const DASHBOARD: IconData = material('\u{e871}');
    /// `dashboard_customize` (U+E99B).
    pub const DASHBOARD_CUSTOMIZE: IconData = material('\u{e99b}');
    /// `data_array` (U+EAD1).
    pub const DATA_ARRAY: IconData = material('\u{ead1}');
    /// `data_exploration` (U+E76F).
    pub const DATA_EXPLORATION: IconData = material('\u{e76f}');
    /// `data_object` (U+EAD3).
    pub const DATA_OBJECT: IconData = material('\u{ead3}');
    /// `data_saver_off` (U+EFF2).
    pub const DATA_SAVER_OFF: IconData = material('\u{eff2}');
    /// `data_saver_on` (U+EFF3).
    pub const DATA_SAVER_ON: IconData = material('\u{eff3}');
    /// `data_thresholding` (U+EB9F).
    pub const DATA_THRESHOLDING: IconData = material('\u{eb9f}');
    /// `data_usage` (U+E1AF).
    pub const DATA_USAGE: IconData = material('\u{e1af}');
    /// `dataset` (U+F8EE).
    pub const DATASET: IconData = material('\u{f8ee}');
    /// `dataset_linked` (U+F8EF).
    pub const DATASET_LINKED: IconData = material('\u{f8ef}');
    /// `date_range` (U+E916).
    pub const DATE_RANGE: IconData = material('\u{e916}');
    /// `deblur` (U+EB77).
    pub const DEBLUR: IconData = material('\u{eb77}');
    /// `deck` (U+EA42).
    pub const DECK: IconData = material('\u{ea42}');
    /// `dehaze` (U+E3C7).
    pub const DEHAZE: IconData = material('\u{e3c7}');
    /// `delete` (U+E872).
    pub const DELETE: IconData = material('\u{e872}');
    /// `delete_forever` (U+E92B).
    pub const DELETE_FOREVER: IconData = material('\u{e92b}');
    /// `delete_outline` (U+E92E).
    pub const DELETE_OUTLINE: IconData = material('\u{e92e}');
    /// `delete_sweep` (U+E16C).
    pub const DELETE_SWEEP: IconData = material('\u{e16c}');
    /// `delivery_dining` (U+EA72).
    pub const DELIVERY_DINING: IconData = material('\u{ea72}');
    /// `density_large` (U+EBA9).
    pub const DENSITY_LARGE: IconData = material('\u{eba9}');
    /// `density_medium` (U+EB9E).
    pub const DENSITY_MEDIUM: IconData = material('\u{eb9e}');
    /// `density_small` (U+EBA8).
    pub const DENSITY_SMALL: IconData = material('\u{eba8}');
    /// `departure_board` (U+E576).
    pub const DEPARTURE_BOARD: IconData = material('\u{e576}');
    /// `description` (U+E873).
    pub const DESCRIPTION: IconData = material('\u{e873}');
    /// `deselect` (U+EBB6).
    pub const DESELECT: IconData = material('\u{ebb6}');
    /// `design_services` (U+F10A).
    pub const DESIGN_SERVICES: IconData = material('\u{f10a}');
    /// `desk` (U+F8F4).
    pub const DESK: IconData = material('\u{f8f4}');
    /// `desktop_access_disabled` (U+E99D).
    pub const DESKTOP_ACCESS_DISABLED: IconData = material('\u{e99d}');
    /// `desktop_mac` (U+E30B).
    pub const DESKTOP_MAC: IconData = material('\u{e30b}');
    /// `desktop_windows` (U+E30C).
    pub const DESKTOP_WINDOWS: IconData = material('\u{e30c}');
    /// `details` (U+E3C8).
    pub const DETAILS: IconData = material('\u{e3c8}');
    /// `developer_board` (U+E30D).
    pub const DEVELOPER_BOARD: IconData = material('\u{e30d}');
    /// `developer_board_off` (U+E4FF).
    pub const DEVELOPER_BOARD_OFF: IconData = material('\u{e4ff}');
    /// `developer_mode` (U+E1B0).
    pub const DEVELOPER_MODE: IconData = material('\u{e1b0}');
    /// `device_hub` (U+E335).
    pub const DEVICE_HUB: IconData = material('\u{e335}');
    /// `device_thermostat` (U+E1FF).
    pub const DEVICE_THERMOSTAT: IconData = material('\u{e1ff}');
    /// `device_unknown` (U+E339).
    pub const DEVICE_UNKNOWN: IconData = material('\u{e339}');
    /// `devices` (U+E1B1).
    pub const DEVICES: IconData = material('\u{e1b1}');
    /// `devices_fold` (U+EBDE).
    pub const DEVICES_FOLD: IconData = material('\u{ebde}');
    /// `devices_other` (U+E337).
    pub const DEVICES_OTHER: IconData = material('\u{e337}');
    /// `dew_point` (U+F879).
    pub const DEW_POINT: IconData = material('\u{f879}');
    /// `dialer_sip` (U+E0BB).
    pub const DIALER_SIP: IconData = material('\u{e0bb}');
    /// `dialpad` (U+E0BC).
    pub const DIALPAD: IconData = material('\u{e0bc}');
    /// `diamond` (U+EAD5).
    pub const DIAMOND: IconData = material('\u{ead5}');
    /// `difference` (U+EB7D).
    pub const DIFFERENCE: IconData = material('\u{eb7d}');
    /// `dining` (U+EFF4).
    pub const DINING: IconData = material('\u{eff4}');
    /// `dinner_dining` (U+EA57).
    pub const DINNER_DINING: IconData = material('\u{ea57}');
    /// `directions` (U+E52E).
    pub const DIRECTIONS: IconData = material('\u{e52e}');
    /// `directions_bike` (U+E52F).
    pub const DIRECTIONS_BIKE: IconData = material('\u{e52f}');
    /// `directions_boat` (U+E532).
    pub const DIRECTIONS_BOAT: IconData = material('\u{e532}');
    /// `directions_boat_filled` (U+EFF5).
    pub const DIRECTIONS_BOAT_FILLED: IconData = material('\u{eff5}');
    /// `directions_bus` (U+E530).
    pub const DIRECTIONS_BUS: IconData = material('\u{e530}');
    /// `directions_bus_filled` (U+EFF6).
    pub const DIRECTIONS_BUS_FILLED: IconData = material('\u{eff6}');
    /// `directions_car` (U+E531).
    pub const DIRECTIONS_CAR: IconData = material('\u{e531}');
    /// `directions_car_filled` (U+EFF7).
    pub const DIRECTIONS_CAR_FILLED: IconData = material('\u{eff7}');
    /// `directions_ferry` (U+E532).
    pub const DIRECTIONS_FERRY: IconData = material('\u{e532}');
    /// `directions_off` (U+F10F).
    pub const DIRECTIONS_OFF: IconData = material('\u{f10f}');
    /// `directions_railway` (U+E534).
    pub const DIRECTIONS_RAILWAY: IconData = material('\u{e534}');
    /// `directions_railway_filled` (U+EFF8).
    pub const DIRECTIONS_RAILWAY_FILLED: IconData = material('\u{eff8}');
    /// `directions_run` (U+E566).
    pub const DIRECTIONS_RUN: IconData = material('\u{e566}');
    /// `directions_subway` (U+E533).
    pub const DIRECTIONS_SUBWAY: IconData = material('\u{e533}');
    /// `directions_subway_filled` (U+EFF9).
    pub const DIRECTIONS_SUBWAY_FILLED: IconData = material('\u{eff9}');
    /// `directions_train` (U+E534).
    pub const DIRECTIONS_TRAIN: IconData = material('\u{e534}');
    /// `directions_transit` (U+E535).
    pub const DIRECTIONS_TRANSIT: IconData = material('\u{e535}');
    /// `directions_transit_filled` (U+EFFA).
    pub const DIRECTIONS_TRANSIT_FILLED: IconData = material('\u{effa}');
    /// `directions_walk` (U+E536).
    pub const DIRECTIONS_WALK: IconData = material('\u{e536}');
    /// `dirty_lens` (U+EF4B).
    pub const DIRTY_LENS: IconData = material('\u{ef4b}');
    /// `disabled_by_default` (U+F230).
    pub const DISABLED_BY_DEFAULT: IconData = material('\u{f230}');
    /// `disabled_visible` (U+E76E).
    pub const DISABLED_VISIBLE: IconData = material('\u{e76e}');
    /// `disc_full` (U+E610).
    pub const DISC_FULL: IconData = material('\u{e610}');
    /// `discord` (U+EA6C).
    pub const DISCORD: IconData = material('\u{ea6c}');
    /// `discount` (U+EBC9).
    pub const DISCOUNT: IconData = material('\u{ebc9}');
    /// `display_settings` (U+EB97).
    pub const DISPLAY_SETTINGS: IconData = material('\u{eb97}');
    /// `diversity_1` (U+F8D7).
    pub const DIVERSITY_1: IconData = material('\u{f8d7}');
    /// `diversity_2` (U+F8D8).
    pub const DIVERSITY_2: IconData = material('\u{f8d8}');
    /// `diversity_3` (U+F8D9).
    pub const DIVERSITY_3: IconData = material('\u{f8d9}');
    /// `dnd_forwardslash` (U+E611).
    pub const DND_FORWARDSLASH: IconData = material('\u{e611}');
    /// `dns` (U+E875).
    pub const DNS: IconData = material('\u{e875}');
    /// `do_disturb` (U+F08C).
    pub const DO_DISTURB: IconData = material('\u{f08c}');
    /// `do_disturb_alt` (U+F08D).
    pub const DO_DISTURB_ALT: IconData = material('\u{f08d}');
    /// `do_disturb_off` (U+F08E).
    pub const DO_DISTURB_OFF: IconData = material('\u{f08e}');
    /// `do_disturb_on` (U+F08F).
    pub const DO_DISTURB_ON: IconData = material('\u{f08f}');
    /// `do_not_disturb` (U+E612).
    pub const DO_NOT_DISTURB: IconData = material('\u{e612}');
    /// `do_not_disturb_alt` (U+E611).
    pub const DO_NOT_DISTURB_ALT: IconData = material('\u{e611}');
    /// `do_not_disturb_off` (U+E643).
    pub const DO_NOT_DISTURB_OFF: IconData = material('\u{e643}');
    /// `do_not_disturb_on` (U+E644).
    pub const DO_NOT_DISTURB_ON: IconData = material('\u{e644}');
    /// `do_not_disturb_on_total_silence` (U+EFFB).
    pub const DO_NOT_DISTURB_ON_TOTAL_SILENCE: IconData = material('\u{effb}');
    /// `do_not_step` (U+F19F).
    pub const DO_NOT_STEP: IconData = material('\u{f19f}');
    /// `do_not_touch` (U+F1B0).
    pub const DO_NOT_TOUCH: IconData = material('\u{f1b0}');
    /// `dock` (U+E30E).
    pub const DOCK: IconData = material('\u{e30e}');
    /// `document_scanner` (U+E5FA).
    pub const DOCUMENT_SCANNER: IconData = material('\u{e5fa}');
    /// `domain` (U+E7EE).
    pub const DOMAIN: IconData = material('\u{e7ee}');
    /// `domain_add` (U+EB62).
    pub const DOMAIN_ADD: IconData = material('\u{eb62}');
    /// `domain_disabled` (U+E0EF).
    pub const DOMAIN_DISABLED: IconData = material('\u{e0ef}');
    /// `domain_verification` (U+EF4C).
    pub const DOMAIN_VERIFICATION: IconData = material('\u{ef4c}');
    /// `done` (U+E876).
    pub const DONE: IconData = material('\u{e876}');
    /// `done_all` (U+E877).
    pub const DONE_ALL: IconData = material('\u{e877}');
    /// `done_outline` (U+E92F).
    pub const DONE_OUTLINE: IconData = material('\u{e92f}');
    /// `donut_large` (U+E917).
    pub const DONUT_LARGE: IconData = material('\u{e917}');
    /// `donut_small` (U+E918).
    pub const DONUT_SMALL: IconData = material('\u{e918}');
    /// `door_back` (U+EFFC).
    pub const DOOR_BACK: IconData = material('\u{effc}');
    /// `door_front` (U+EFFD).
    pub const DOOR_FRONT: IconData = material('\u{effd}');
    /// `door_sliding` (U+EFFE).
    pub const DOOR_SLIDING: IconData = material('\u{effe}');
    /// `doorbell` (U+EFFF).
    pub const DOORBELL: IconData = material('\u{efff}');
    /// `double_arrow` (U+EA50).
    pub const DOUBLE_ARROW: IconData = material('\u{ea50}');
    /// `downhill_skiing` (U+E509).
    pub const DOWNHILL_SKIING: IconData = material('\u{e509}');
    /// `download` (U+F090).
    pub const DOWNLOAD: IconData = material('\u{f090}');
    /// `download_done` (U+F091).
    pub const DOWNLOAD_DONE: IconData = material('\u{f091}');
    /// `download_for_offline` (U+F000).
    pub const DOWNLOAD_FOR_OFFLINE: IconData = material('\u{f000}');
    /// `downloading` (U+F001).
    pub const DOWNLOADING: IconData = material('\u{f001}');
    /// `drafts` (U+E151).
    pub const DRAFTS: IconData = material('\u{e151}');
    /// `drag_handle` (U+E25D).
    pub const DRAG_HANDLE: IconData = material('\u{e25d}');
    /// `drag_indicator` (U+E945).
    pub const DRAG_INDICATOR: IconData = material('\u{e945}');
    /// `draw` (U+E746).
    pub const DRAW: IconData = material('\u{e746}');
    /// `drive_eta` (U+E613).
    pub const DRIVE_ETA: IconData = material('\u{e613}');
    /// `drive_file_move` (U+E675).
    pub const DRIVE_FILE_MOVE: IconData = material('\u{e675}');
    /// `drive_file_move_outline` (U+E9A1).
    pub const DRIVE_FILE_MOVE_OUTLINE: IconData = material('\u{e9a1}');
    /// `drive_file_move_rtl` (U+E76D).
    pub const DRIVE_FILE_MOVE_RTL: IconData = material('\u{e76d}');
    /// `drive_file_rename_outline` (U+E9A2).
    pub const DRIVE_FILE_RENAME_OUTLINE: IconData = material('\u{e9a2}');
    /// `drive_folder_upload` (U+E9A3).
    pub const DRIVE_FOLDER_UPLOAD: IconData = material('\u{e9a3}');
    /// `dry` (U+F1B3).
    pub const DRY: IconData = material('\u{f1b3}');
    /// `dry_cleaning` (U+EA58).
    pub const DRY_CLEANING: IconData = material('\u{ea58}');
    /// `duo` (U+E9A5).
    pub const DUO: IconData = material('\u{e9a5}');
    /// `dvr` (U+E1B2).
    pub const DVR: IconData = material('\u{e1b2}');
    /// `dynamic_feed` (U+EA14).
    pub const DYNAMIC_FEED: IconData = material('\u{ea14}');
    /// `dynamic_form` (U+F1BF).
    pub const DYNAMIC_FORM: IconData = material('\u{f1bf}');
    /// `e_mobiledata` (U+F002).
    pub const E_MOBILEDATA: IconData = material('\u{f002}');
    /// `earbuds` (U+F003).
    pub const EARBUDS: IconData = material('\u{f003}');
    /// `earbuds_battery` (U+F004).
    pub const EARBUDS_BATTERY: IconData = material('\u{f004}');
    /// `east` (U+F1DF).
    pub const EAST: IconData = material('\u{f1df}');
    /// `eco` (U+EA35).
    pub const ECO: IconData = material('\u{ea35}');
    /// `edgesensor_high` (U+F005).
    pub const EDGESENSOR_HIGH: IconData = material('\u{f005}');
    /// `edgesensor_low` (U+F006).
    pub const EDGESENSOR_LOW: IconData = material('\u{f006}');
    /// `edit` (U+E3C9).
    pub const EDIT: IconData = material('\u{e3c9}');
    /// `edit_attributes` (U+E578).
    pub const EDIT_ATTRIBUTES: IconData = material('\u{e578}');
    /// `edit_calendar` (U+E742).
    pub const EDIT_CALENDAR: IconData = material('\u{e742}');
    /// `edit_document` (U+F88C).
    pub const EDIT_DOCUMENT: IconData = material('\u{f88c}');
    /// `edit_location` (U+E568).
    pub const EDIT_LOCATION: IconData = material('\u{e568}');
    /// `edit_location_alt` (U+E1C5).
    pub const EDIT_LOCATION_ALT: IconData = material('\u{e1c5}');
    /// `edit_note` (U+E745).
    pub const EDIT_NOTE: IconData = material('\u{e745}');
    /// `edit_notifications` (U+E525).
    pub const EDIT_NOTIFICATIONS: IconData = material('\u{e525}');
    /// `edit_off` (U+E950).
    pub const EDIT_OFF: IconData = material('\u{e950}');
    /// `edit_road` (U+EF4D).
    pub const EDIT_ROAD: IconData = material('\u{ef4d}');
    /// `edit_square` (U+F88D).
    pub const EDIT_SQUARE: IconData = material('\u{f88d}');
    /// `egg` (U+EACC).
    pub const EGG: IconData = material('\u{eacc}');
    /// `egg_alt` (U+EAC8).
    pub const EGG_ALT: IconData = material('\u{eac8}');
    /// `eject` (U+E8FB).
    pub const EJECT: IconData = material('\u{e8fb}');
    /// `elderly` (U+F21A).
    pub const ELDERLY: IconData = material('\u{f21a}');
    /// `elderly_woman` (U+EB69).
    pub const ELDERLY_WOMAN: IconData = material('\u{eb69}');
    /// `electric_bike` (U+EB1B).
    pub const ELECTRIC_BIKE: IconData = material('\u{eb1b}');
    /// `electric_bolt` (U+EC1C).
    pub const ELECTRIC_BOLT: IconData = material('\u{ec1c}');
    /// `electric_car` (U+EB1C).
    pub const ELECTRIC_CAR: IconData = material('\u{eb1c}');
    /// `electric_meter` (U+EC1B).
    pub const ELECTRIC_METER: IconData = material('\u{ec1b}');
    /// `electric_moped` (U+EB1D).
    pub const ELECTRIC_MOPED: IconData = material('\u{eb1d}');
    /// `electric_rickshaw` (U+EB1E).
    pub const ELECTRIC_RICKSHAW: IconData = material('\u{eb1e}');
    /// `electric_scooter` (U+EB1F).
    pub const ELECTRIC_SCOOTER: IconData = material('\u{eb1f}');
    /// `electrical_services` (U+F102).
    pub const ELECTRICAL_SERVICES: IconData = material('\u{f102}');
    /// `elevator` (U+F1A0).
    pub const ELEVATOR: IconData = material('\u{f1a0}');
    /// `email` (U+E0BE).
    pub const EMAIL: IconData = material('\u{e0be}');
    /// `emergency` (U+E1EB).
    pub const EMERGENCY: IconData = material('\u{e1eb}');
    /// `emergency_recording` (U+EBF4).
    pub const EMERGENCY_RECORDING: IconData = material('\u{ebf4}');
    /// `emergency_share` (U+EBF6).
    pub const EMERGENCY_SHARE: IconData = material('\u{ebf6}');
    /// `emoji_emotions` (U+EA22).
    pub const EMOJI_EMOTIONS: IconData = material('\u{ea22}');
    /// `emoji_events` (U+EA23).
    pub const EMOJI_EVENTS: IconData = material('\u{ea23}');
    /// `emoji_flags` (U+EA1A).
    pub const EMOJI_FLAGS: IconData = material('\u{ea1a}');
    /// `emoji_food_beverage` (U+EA1B).
    pub const EMOJI_FOOD_BEVERAGE: IconData = material('\u{ea1b}');
    /// `emoji_nature` (U+EA1C).
    pub const EMOJI_NATURE: IconData = material('\u{ea1c}');
    /// `emoji_objects` (U+EA24).
    pub const EMOJI_OBJECTS: IconData = material('\u{ea24}');
    /// `emoji_people` (U+EA1D).
    pub const EMOJI_PEOPLE: IconData = material('\u{ea1d}');
    /// `emoji_symbols` (U+EA1E).
    pub const EMOJI_SYMBOLS: IconData = material('\u{ea1e}');
    /// `emoji_transportation` (U+EA1F).
    pub const EMOJI_TRANSPORTATION: IconData = material('\u{ea1f}');
    /// `energy_savings_leaf` (U+EC1A).
    pub const ENERGY_SAVINGS_LEAF: IconData = material('\u{ec1a}');
    /// `engineering` (U+EA3D).
    pub const ENGINEERING: IconData = material('\u{ea3d}');
    /// `enhance_photo_translate` (U+E8FC).
    pub const ENHANCE_PHOTO_TRANSLATE: IconData = material('\u{e8fc}');
    /// `enhanced_encryption` (U+E63F).
    pub const ENHANCED_ENCRYPTION: IconData = material('\u{e63f}');
    /// `equalizer` (U+E01D).
    pub const EQUALIZER: IconData = material('\u{e01d}');
    /// `error` (U+E000).
    pub const ERROR: IconData = material('\u{e000}');
    /// `error_outline` (U+E001).
    pub const ERROR_OUTLINE: IconData = material('\u{e001}');
    /// `escalator` (U+F1A1).
    pub const ESCALATOR: IconData = material('\u{f1a1}');
    /// `escalator_warning` (U+F1AC).
    pub const ESCALATOR_WARNING: IconData = material('\u{f1ac}');
    /// `euro` (U+EA15).
    pub const EURO: IconData = material('\u{ea15}');
    /// `euro_symbol` (U+E926).
    pub const EURO_SYMBOL: IconData = material('\u{e926}');
    /// `ev_station` (U+E56D).
    pub const EV_STATION: IconData = material('\u{e56d}');
    /// `event` (U+E878).
    pub const EVENT: IconData = material('\u{e878}');
    /// `event_available` (U+E614).
    pub const EVENT_AVAILABLE: IconData = material('\u{e614}');
    /// `event_busy` (U+E615).
    pub const EVENT_BUSY: IconData = material('\u{e615}');
    /// `event_note` (U+E616).
    pub const EVENT_NOTE: IconData = material('\u{e616}');
    /// `event_repeat` (U+EB7B).
    pub const EVENT_REPEAT: IconData = material('\u{eb7b}');
    /// `event_seat` (U+E903).
    pub const EVENT_SEAT: IconData = material('\u{e903}');
    /// `exit_to_app` (U+E879).
    pub const EXIT_TO_APP: IconData = material('\u{e879}');
    /// `expand` (U+E94F).
    pub const EXPAND: IconData = material('\u{e94f}');
    /// `expand_circle_down` (U+E7CD).
    pub const EXPAND_CIRCLE_DOWN: IconData = material('\u{e7cd}');
    /// `expand_less` (U+E5CE).
    pub const EXPAND_LESS: IconData = material('\u{e5ce}');
    /// `expand_more` (U+E5CF).
    pub const EXPAND_MORE: IconData = material('\u{e5cf}');
    /// `explicit` (U+E01E).
    pub const EXPLICIT: IconData = material('\u{e01e}');
    /// `explore` (U+E87A).
    pub const EXPLORE: IconData = material('\u{e87a}');
    /// `explore_off` (U+E9A8).
    pub const EXPLORE_OFF: IconData = material('\u{e9a8}');
    /// `exposure` (U+E3CA).
    pub const EXPOSURE: IconData = material('\u{e3ca}');
    /// `exposure_minus_1` (U+E3CB).
    pub const EXPOSURE_MINUS_1: IconData = material('\u{e3cb}');
    /// `exposure_minus_2` (U+E3CC).
    pub const EXPOSURE_MINUS_2: IconData = material('\u{e3cc}');
    /// `exposure_neg_1` (U+E3CB).
    pub const EXPOSURE_NEG_1: IconData = material('\u{e3cb}');
    /// `exposure_neg_2` (U+E3CC).
    pub const EXPOSURE_NEG_2: IconData = material('\u{e3cc}');
    /// `exposure_plus_1` (U+E3CD).
    pub const EXPOSURE_PLUS_1: IconData = material('\u{e3cd}');
    /// `exposure_plus_2` (U+E3CE).
    pub const EXPOSURE_PLUS_2: IconData = material('\u{e3ce}');
    /// `exposure_zero` (U+E3CF).
    pub const EXPOSURE_ZERO: IconData = material('\u{e3cf}');
    /// `extension` (U+E87B).
    pub const EXTENSION: IconData = material('\u{e87b}');
    /// `extension_off` (U+E4F5).
    pub const EXTENSION_OFF: IconData = material('\u{e4f5}');
    /// `face` (U+E87C).
    pub const FACE: IconData = material('\u{e87c}');
    /// `face_2` (U+F8DA).
    pub const FACE_2: IconData = material('\u{f8da}');
    /// `face_3` (U+F8DB).
    pub const FACE_3: IconData = material('\u{f8db}');
    /// `face_4` (U+F8DC).
    pub const FACE_4: IconData = material('\u{f8dc}');
    /// `face_5` (U+F8DD).
    pub const FACE_5: IconData = material('\u{f8dd}');
    /// `face_6` (U+F8DE).
    pub const FACE_6: IconData = material('\u{f8de}');
    /// `face_retouching_natural` (U+EF4E).
    pub const FACE_RETOUCHING_NATURAL: IconData = material('\u{ef4e}');
    /// `face_retouching_off` (U+F007).
    pub const FACE_RETOUCHING_OFF: IconData = material('\u{f007}');
    /// `facebook` (U+F234).
    pub const FACEBOOK: IconData = material('\u{f234}');
    /// `fact_check` (U+F0C5).
    pub const FACT_CHECK: IconData = material('\u{f0c5}');
    /// `factory` (U+EBBC).
    pub const FACTORY: IconData = material('\u{ebbc}');
    /// `family_restroom` (U+F1A2).
    pub const FAMILY_RESTROOM: IconData = material('\u{f1a2}');
    /// `fast_forward` (U+E01F).
    pub const FAST_FORWARD: IconData = material('\u{e01f}');
    /// `fast_rewind` (U+E020).
    pub const FAST_REWIND: IconData = material('\u{e020}');
    /// `fastfood` (U+E57A).
    pub const FASTFOOD: IconData = material('\u{e57a}');
    /// `favorite` (U+E87D).
    pub const FAVORITE: IconData = material('\u{e87d}');
    /// `favorite_border` (U+E87E).
    pub const FAVORITE_BORDER: IconData = material('\u{e87e}');
    /// `favorite_outline` (U+E87E).
    pub const FAVORITE_OUTLINE: IconData = material('\u{e87e}');
    /// `fax` (U+EAD8).
    pub const FAX: IconData = material('\u{ead8}');
    /// `featured_play_list` (U+E06D).
    pub const FEATURED_PLAY_LIST: IconData = material('\u{e06d}');
    /// `featured_video` (U+E06E).
    pub const FEATURED_VIDEO: IconData = material('\u{e06e}');
    /// `feed` (U+F009).
    pub const FEED: IconData = material('\u{f009}');
    /// `feedback` (U+E87F).
    pub const FEEDBACK: IconData = material('\u{e87f}');
    /// `female` (U+E590).
    pub const FEMALE: IconData = material('\u{e590}');
    /// `fence` (U+F1F6).
    pub const FENCE: IconData = material('\u{f1f6}');
    /// `festival` (U+EA68).
    pub const FESTIVAL: IconData = material('\u{ea68}');
    /// `fiber_dvr` (U+E05D).
    pub const FIBER_DVR: IconData = material('\u{e05d}');
    /// `fiber_manual_record` (U+E061).
    pub const FIBER_MANUAL_RECORD: IconData = material('\u{e061}');
    /// `fiber_new` (U+E05E).
    pub const FIBER_NEW: IconData = material('\u{e05e}');
    /// `fiber_pin` (U+E06A).
    pub const FIBER_PIN: IconData = material('\u{e06a}');
    /// `fiber_smart_record` (U+E062).
    pub const FIBER_SMART_RECORD: IconData = material('\u{e062}');
    /// `file_copy` (U+E173).
    pub const FILE_COPY: IconData = material('\u{e173}');
    /// `file_download` (U+E2C4).
    pub const FILE_DOWNLOAD: IconData = material('\u{e2c4}');
    /// `file_download_done` (U+E9AA).
    pub const FILE_DOWNLOAD_DONE: IconData = material('\u{e9aa}');
    /// `file_download_off` (U+E4FE).
    pub const FILE_DOWNLOAD_OFF: IconData = material('\u{e4fe}');
    /// `file_open` (U+EAF3).
    pub const FILE_OPEN: IconData = material('\u{eaf3}');
    /// `file_present` (U+EA0E).
    pub const FILE_PRESENT: IconData = material('\u{ea0e}');
    /// `file_upload` (U+E2C6).
    pub const FILE_UPLOAD: IconData = material('\u{e2c6}');
    /// `file_upload_off` (U+F886).
    pub const FILE_UPLOAD_OFF: IconData = material('\u{f886}');
    /// `filter` (U+E3D3).
    pub const FILTER: IconData = material('\u{e3d3}');
    /// `filter_1` (U+E3D0).
    pub const FILTER_1: IconData = material('\u{e3d0}');
    /// `filter_2` (U+E3D1).
    pub const FILTER_2: IconData = material('\u{e3d1}');
    /// `filter_3` (U+E3D2).
    pub const FILTER_3: IconData = material('\u{e3d2}');
    /// `filter_4` (U+E3D4).
    pub const FILTER_4: IconData = material('\u{e3d4}');
    /// `filter_5` (U+E3D5).
    pub const FILTER_5: IconData = material('\u{e3d5}');
    /// `filter_6` (U+E3D6).
    pub const FILTER_6: IconData = material('\u{e3d6}');
    /// `filter_7` (U+E3D7).
    pub const FILTER_7: IconData = material('\u{e3d7}');
    /// `filter_8` (U+E3D8).
    pub const FILTER_8: IconData = material('\u{e3d8}');
    /// `filter_9` (U+E3D9).
    pub const FILTER_9: IconData = material('\u{e3d9}');
    /// `filter_9_plus` (U+E3DA).
    pub const FILTER_9_PLUS: IconData = material('\u{e3da}');
    /// `filter_alt` (U+EF4F).
    pub const FILTER_ALT: IconData = material('\u{ef4f}');
    /// `filter_alt_off` (U+EB32).
    pub const FILTER_ALT_OFF: IconData = material('\u{eb32}');
    /// `filter_b_and_w` (U+E3DB).
    pub const FILTER_B_AND_W: IconData = material('\u{e3db}');
    /// `filter_center_focus` (U+E3DC).
    pub const FILTER_CENTER_FOCUS: IconData = material('\u{e3dc}');
    /// `filter_drama` (U+E3DD).
    pub const FILTER_DRAMA: IconData = material('\u{e3dd}');
    /// `filter_frames` (U+E3DE).
    pub const FILTER_FRAMES: IconData = material('\u{e3de}');
    /// `filter_hdr` (U+E3DF).
    pub const FILTER_HDR: IconData = material('\u{e3df}');
    /// `filter_list` (U+E152).
    pub const FILTER_LIST: IconData = material('\u{e152}');
    /// `filter_list_alt` (U+E94E).
    pub const FILTER_LIST_ALT: IconData = material('\u{e94e}');
    /// `filter_list_off` (U+EB57).
    pub const FILTER_LIST_OFF: IconData = material('\u{eb57}');
    /// `filter_none` (U+E3E0).
    pub const FILTER_NONE: IconData = material('\u{e3e0}');
    /// `filter_tilt_shift` (U+E3E2).
    pub const FILTER_TILT_SHIFT: IconData = material('\u{e3e2}');
    /// `filter_vintage` (U+E3E3).
    pub const FILTER_VINTAGE: IconData = material('\u{e3e3}');
    /// `find_in_page` (U+E880).
    pub const FIND_IN_PAGE: IconData = material('\u{e880}');
    /// `find_replace` (U+E881).
    pub const FIND_REPLACE: IconData = material('\u{e881}');
    /// `fingerprint` (U+E90D).
    pub const FINGERPRINT: IconData = material('\u{e90d}');
    /// `fire_extinguisher` (U+F1D8).
    pub const FIRE_EXTINGUISHER: IconData = material('\u{f1d8}');
    /// `fire_hydrant` (U+F1A3).
    pub const FIRE_HYDRANT: IconData = material('\u{f1a3}');
    /// `fire_hydrant_alt` (U+F8F1).
    pub const FIRE_HYDRANT_ALT: IconData = material('\u{f8f1}');
    /// `fire_truck` (U+F8F2).
    pub const FIRE_TRUCK: IconData = material('\u{f8f2}');
    /// `fireplace` (U+EA43).
    pub const FIREPLACE: IconData = material('\u{ea43}');
    /// `first_page` (U+E5DC).
    pub const FIRST_PAGE: IconData = material('\u{e5dc}');
    /// `fit_screen` (U+EA10).
    pub const FIT_SCREEN: IconData = material('\u{ea10}');
    /// `fitbit` (U+E82B).
    pub const FITBIT: IconData = material('\u{e82b}');
    /// `fitness_center` (U+EB43).
    pub const FITNESS_CENTER: IconData = material('\u{eb43}');
    /// `flag` (U+E153).
    pub const FLAG: IconData = material('\u{e153}');
    /// `flag_circle` (U+EAF8).
    pub const FLAG_CIRCLE: IconData = material('\u{eaf8}');
    /// `flaky` (U+EF50).
    pub const FLAKY: IconData = material('\u{ef50}');
    /// `flare` (U+E3E4).
    pub const FLARE: IconData = material('\u{e3e4}');
    /// `flash_auto` (U+E3E5).
    pub const FLASH_AUTO: IconData = material('\u{e3e5}');
    /// `flash_off` (U+E3E6).
    pub const FLASH_OFF: IconData = material('\u{e3e6}');
    /// `flash_on` (U+E3E7).
    pub const FLASH_ON: IconData = material('\u{e3e7}');
    /// `flashlight_off` (U+F00A).
    pub const FLASHLIGHT_OFF: IconData = material('\u{f00a}');
    /// `flashlight_on` (U+F00B).
    pub const FLASHLIGHT_ON: IconData = material('\u{f00b}');
    /// `flatware` (U+F00C).
    pub const FLATWARE: IconData = material('\u{f00c}');
    /// `flight` (U+E539).
    pub const FLIGHT: IconData = material('\u{e539}');
    /// `flight_class` (U+E7CB).
    pub const FLIGHT_CLASS: IconData = material('\u{e7cb}');
    /// `flight_land` (U+E904).
    pub const FLIGHT_LAND: IconData = material('\u{e904}');
    /// `flight_takeoff` (U+E905).
    pub const FLIGHT_TAKEOFF: IconData = material('\u{e905}');
    /// `flip` (U+E3E8).
    pub const FLIP: IconData = material('\u{e3e8}');
    /// `flip_camera_android` (U+EA37).
    pub const FLIP_CAMERA_ANDROID: IconData = material('\u{ea37}');
    /// `flip_camera_ios` (U+EA38).
    pub const FLIP_CAMERA_IOS: IconData = material('\u{ea38}');
    /// `flip_to_back` (U+E882).
    pub const FLIP_TO_BACK: IconData = material('\u{e882}');
    /// `flip_to_front` (U+E883).
    pub const FLIP_TO_FRONT: IconData = material('\u{e883}');
    /// `flood` (U+EBE6).
    pub const FLOOD: IconData = material('\u{ebe6}');
    /// `flourescent` (U+EC31).
    pub const FLOURESCENT: IconData = material('\u{ec31}');
    /// `fluorescent` (U+EC31).
    pub const FLUORESCENT: IconData = material('\u{ec31}');
    /// `flutter_dash` (U+E00B).
    pub const FLUTTER_DASH: IconData = material('\u{e00b}');
    /// `fmd_bad` (U+F00E).
    pub const FMD_BAD: IconData = material('\u{f00e}');
    /// `fmd_good` (U+F00F).
    pub const FMD_GOOD: IconData = material('\u{f00f}');
    /// `foggy` (U+E818).
    pub const FOGGY: IconData = material('\u{e818}');
    /// `folder` (U+E2C7).
    pub const FOLDER: IconData = material('\u{e2c7}');
    /// `folder_copy` (U+EBBD).
    pub const FOLDER_COPY: IconData = material('\u{ebbd}');
    /// `folder_delete` (U+EB34).
    pub const FOLDER_DELETE: IconData = material('\u{eb34}');
    /// `folder_off` (U+EB83).
    pub const FOLDER_OFF: IconData = material('\u{eb83}');
    /// `folder_open` (U+E2C8).
    pub const FOLDER_OPEN: IconData = material('\u{e2c8}');
    /// `folder_shared` (U+E2C9).
    pub const FOLDER_SHARED: IconData = material('\u{e2c9}');
    /// `folder_special` (U+E617).
    pub const FOLDER_SPECIAL: IconData = material('\u{e617}');
    /// `folder_zip` (U+EB2C).
    pub const FOLDER_ZIP: IconData = material('\u{eb2c}');
    /// `follow_the_signs` (U+F222).
    pub const FOLLOW_THE_SIGNS: IconData = material('\u{f222}');
    /// `font_download` (U+E167).
    pub const FONT_DOWNLOAD: IconData = material('\u{e167}');
    /// `font_download_off` (U+E4F9).
    pub const FONT_DOWNLOAD_OFF: IconData = material('\u{e4f9}');
    /// `food_bank` (U+F1F2).
    pub const FOOD_BANK: IconData = material('\u{f1f2}');
    /// `forest` (U+EA99).
    pub const FOREST: IconData = material('\u{ea99}');
    /// `fork_left` (U+EBA0).
    pub const FORK_LEFT: IconData = material('\u{eba0}');
    /// `fork_right` (U+EBAC).
    pub const FORK_RIGHT: IconData = material('\u{ebac}');
    /// `forklift` (U+F868).
    pub const FORKLIFT: IconData = material('\u{f868}');
    /// `format_align_center` (U+E234).
    pub const FORMAT_ALIGN_CENTER: IconData = material('\u{e234}');
    /// `format_align_justify` (U+E235).
    pub const FORMAT_ALIGN_JUSTIFY: IconData = material('\u{e235}');
    /// `format_align_left` (U+E236).
    pub const FORMAT_ALIGN_LEFT: IconData = material('\u{e236}');
    /// `format_align_right` (U+E237).
    pub const FORMAT_ALIGN_RIGHT: IconData = material('\u{e237}');
    /// `format_bold` (U+E238).
    pub const FORMAT_BOLD: IconData = material('\u{e238}');
    /// `format_clear` (U+E239).
    pub const FORMAT_CLEAR: IconData = material('\u{e239}');
    /// `format_color_fill` (U+E23A).
    pub const FORMAT_COLOR_FILL: IconData = material('\u{e23a}');
    /// `format_color_reset` (U+E23B).
    pub const FORMAT_COLOR_RESET: IconData = material('\u{e23b}');
    /// `format_color_text` (U+E23C).
    pub const FORMAT_COLOR_TEXT: IconData = material('\u{e23c}');
    /// `format_indent_decrease` (U+E23D).
    pub const FORMAT_INDENT_DECREASE: IconData = material('\u{e23d}');
    /// `format_indent_increase` (U+E23E).
    pub const FORMAT_INDENT_INCREASE: IconData = material('\u{e23e}');
    /// `format_italic` (U+E23F).
    pub const FORMAT_ITALIC: IconData = material('\u{e23f}');
    /// `format_line_spacing` (U+E240).
    pub const FORMAT_LINE_SPACING: IconData = material('\u{e240}');
    /// `format_list_bulleted` (U+E241).
    pub const FORMAT_LIST_BULLETED: IconData = material('\u{e241}');
    /// `format_list_bulleted_add` (U+F849).
    pub const FORMAT_LIST_BULLETED_ADD: IconData = material('\u{f849}');
    /// `format_list_numbered` (U+E242).
    pub const FORMAT_LIST_NUMBERED: IconData = material('\u{e242}');
    /// `format_list_numbered_rtl` (U+E267).
    pub const FORMAT_LIST_NUMBERED_RTL: IconData = material('\u{e267}');
    /// `format_overline` (U+EB65).
    pub const FORMAT_OVERLINE: IconData = material('\u{eb65}');
    /// `format_paint` (U+E243).
    pub const FORMAT_PAINT: IconData = material('\u{e243}');
    /// `format_quote` (U+E244).
    pub const FORMAT_QUOTE: IconData = material('\u{e244}');
    /// `format_shapes` (U+E25E).
    pub const FORMAT_SHAPES: IconData = material('\u{e25e}');
    /// `format_size` (U+E245).
    pub const FORMAT_SIZE: IconData = material('\u{e245}');
    /// `format_strikethrough` (U+E246).
    pub const FORMAT_STRIKETHROUGH: IconData = material('\u{e246}');
    /// `format_textdirection_l_to_r` (U+E247).
    pub const FORMAT_TEXTDIRECTION_L_TO_R: IconData = material('\u{e247}');
    /// `format_textdirection_r_to_l` (U+E248).
    pub const FORMAT_TEXTDIRECTION_R_TO_L: IconData = material('\u{e248}');
    /// `format_underline` (U+E249).
    pub const FORMAT_UNDERLINE: IconData = material('\u{e249}');
    /// `format_underlined` (U+E249).
    pub const FORMAT_UNDERLINED: IconData = material('\u{e249}');
    /// `fort` (U+EAAD).
    pub const FORT: IconData = material('\u{eaad}');
    /// `forum` (U+E0BF).
    pub const FORUM: IconData = material('\u{e0bf}');
    /// `forward` (U+E154).
    pub const FORWARD: IconData = material('\u{e154}');
    /// `forward_10` (U+E056).
    pub const FORWARD_10: IconData = material('\u{e056}');
    /// `forward_30` (U+E057).
    pub const FORWARD_30: IconData = material('\u{e057}');
    /// `forward_5` (U+E058).
    pub const FORWARD_5: IconData = material('\u{e058}');
    /// `forward_to_inbox` (U+F187).
    pub const FORWARD_TO_INBOX: IconData = material('\u{f187}');
    /// `foundation` (U+F200).
    pub const FOUNDATION: IconData = material('\u{f200}');
    /// `free_breakfast` (U+EB44).
    pub const FREE_BREAKFAST: IconData = material('\u{eb44}');
    /// `free_cancellation` (U+E748).
    pub const FREE_CANCELLATION: IconData = material('\u{e748}');
    /// `front_hand` (U+E769).
    pub const FRONT_HAND: IconData = material('\u{e769}');
    /// `front_loader` (U+F869).
    pub const FRONT_LOADER: IconData = material('\u{f869}');
    /// `fullscreen` (U+E5D0).
    pub const FULLSCREEN: IconData = material('\u{e5d0}');
    /// `fullscreen_exit` (U+E5D1).
    pub const FULLSCREEN_EXIT: IconData = material('\u{e5d1}');
    /// `functions` (U+E24A).
    pub const FUNCTIONS: IconData = material('\u{e24a}');
    /// `g_mobiledata` (U+F010).
    pub const G_MOBILEDATA: IconData = material('\u{f010}');
    /// `g_translate` (U+E927).
    pub const G_TRANSLATE: IconData = material('\u{e927}');
    /// `gamepad` (U+E30F).
    pub const GAMEPAD: IconData = material('\u{e30f}');
    /// `games` (U+E021).
    pub const GAMES: IconData = material('\u{e021}');
    /// `garage` (U+F011).
    pub const GARAGE: IconData = material('\u{f011}');
    /// `gas_meter` (U+EC19).
    pub const GAS_METER: IconData = material('\u{ec19}');
    /// `gavel` (U+E90E).
    pub const GAVEL: IconData = material('\u{e90e}');
    /// `generating_tokens` (U+E749).
    pub const GENERATING_TOKENS: IconData = material('\u{e749}');
    /// `gesture` (U+E155).
    pub const GESTURE: IconData = material('\u{e155}');
    /// `get_app` (U+E884).
    pub const GET_APP: IconData = material('\u{e884}');
    /// `gif` (U+E908).
    pub const GIF: IconData = material('\u{e908}');
    /// `gif_box` (U+E7A3).
    pub const GIF_BOX: IconData = material('\u{e7a3}');
    /// `girl` (U+EB68).
    pub const GIRL: IconData = material('\u{eb68}');
    /// `gite` (U+E58B).
    pub const GITE: IconData = material('\u{e58b}');
    /// `goat` (U+10FFFD).
    pub const GOAT: IconData = material('\u{10fffd}');
    /// `golf_course` (U+EB45).
    pub const GOLF_COURSE: IconData = material('\u{eb45}');
    /// `gpp_bad` (U+F012).
    pub const GPP_BAD: IconData = material('\u{f012}');
    /// `gpp_good` (U+F013).
    pub const GPP_GOOD: IconData = material('\u{f013}');
    /// `gpp_maybe` (U+F014).
    pub const GPP_MAYBE: IconData = material('\u{f014}');
    /// `gps_fixed` (U+E1B3).
    pub const GPS_FIXED: IconData = material('\u{e1b3}');
    /// `gps_not_fixed` (U+E1B4).
    pub const GPS_NOT_FIXED: IconData = material('\u{e1b4}');
    /// `gps_off` (U+E1B5).
    pub const GPS_OFF: IconData = material('\u{e1b5}');
    /// `grade` (U+E885).
    pub const GRADE: IconData = material('\u{e885}');
    /// `gradient` (U+E3E9).
    pub const GRADIENT: IconData = material('\u{e3e9}');
    /// `grading` (U+EA4F).
    pub const GRADING: IconData = material('\u{ea4f}');
    /// `grain` (U+E3EA).
    pub const GRAIN: IconData = material('\u{e3ea}');
    /// `graphic_eq` (U+E1B8).
    pub const GRAPHIC_EQ: IconData = material('\u{e1b8}');
    /// `grass` (U+F205).
    pub const GRASS: IconData = material('\u{f205}');
    /// `grid_3x3` (U+F015).
    pub const GRID_3X3: IconData = material('\u{f015}');
    /// `grid_4x4` (U+F016).
    pub const GRID_4X4: IconData = material('\u{f016}');
    /// `grid_goldenratio` (U+F017).
    pub const GRID_GOLDENRATIO: IconData = material('\u{f017}');
    /// `grid_off` (U+E3EB).
    pub const GRID_OFF: IconData = material('\u{e3eb}');
    /// `grid_on` (U+E3EC).
    pub const GRID_ON: IconData = material('\u{e3ec}');
    /// `grid_view` (U+E9B0).
    pub const GRID_VIEW: IconData = material('\u{e9b0}');
    /// `group` (U+E7EF).
    pub const GROUP: IconData = material('\u{e7ef}');
    /// `group_add` (U+E7F0).
    pub const GROUP_ADD: IconData = material('\u{e7f0}');
    /// `group_off` (U+E747).
    pub const GROUP_OFF: IconData = material('\u{e747}');
    /// `group_remove` (U+E7AD).
    pub const GROUP_REMOVE: IconData = material('\u{e7ad}');
    /// `group_work` (U+E886).
    pub const GROUP_WORK: IconData = material('\u{e886}');
    /// `groups` (U+F233).
    pub const GROUPS: IconData = material('\u{f233}');
    /// `groups_2` (U+F8DF).
    pub const GROUPS_2: IconData = material('\u{f8df}');
    /// `groups_3` (U+F8E0).
    pub const GROUPS_3: IconData = material('\u{f8e0}');
    /// `h_mobiledata` (U+F018).
    pub const H_MOBILEDATA: IconData = material('\u{f018}');
    /// `h_plus_mobiledata` (U+F019).
    pub const H_PLUS_MOBILEDATA: IconData = material('\u{f019}');
    /// `hail` (U+E9B1).
    pub const HAIL: IconData = material('\u{e9b1}');
    /// `handshake` (U+EBCB).
    pub const HANDSHAKE: IconData = material('\u{ebcb}');
    /// `handyman` (U+F10B).
    pub const HANDYMAN: IconData = material('\u{f10b}');
    /// `hardware` (U+EA59).
    pub const HARDWARE: IconData = material('\u{ea59}');
    /// `hd` (U+E052).
    pub const HD: IconData = material('\u{e052}');
    /// `hdr_auto` (U+F01A).
    pub const HDR_AUTO: IconData = material('\u{f01a}');
    /// `hdr_auto_select` (U+F01B).
    pub const HDR_AUTO_SELECT: IconData = material('\u{f01b}');
    /// `hdr_enhanced_select` (U+EF51).
    pub const HDR_ENHANCED_SELECT: IconData = material('\u{ef51}');
    /// `hdr_off` (U+E3ED).
    pub const HDR_OFF: IconData = material('\u{e3ed}');
    /// `hdr_off_select` (U+F01C).
    pub const HDR_OFF_SELECT: IconData = material('\u{f01c}');
    /// `hdr_on` (U+E3EE).
    pub const HDR_ON: IconData = material('\u{e3ee}');
    /// `hdr_on_select` (U+F01D).
    pub const HDR_ON_SELECT: IconData = material('\u{f01d}');
    /// `hdr_plus` (U+F01E).
    pub const HDR_PLUS: IconData = material('\u{f01e}');
    /// `hdr_strong` (U+E3F1).
    pub const HDR_STRONG: IconData = material('\u{e3f1}');
    /// `hdr_weak` (U+E3F2).
    pub const HDR_WEAK: IconData = material('\u{e3f2}');
    /// `headphones` (U+F01F).
    pub const HEADPHONES: IconData = material('\u{f01f}');
    /// `headphones_battery` (U+F020).
    pub const HEADPHONES_BATTERY: IconData = material('\u{f020}');
    /// `headset` (U+E310).
    pub const HEADSET: IconData = material('\u{e310}');
    /// `headset_mic` (U+E311).
    pub const HEADSET_MIC: IconData = material('\u{e311}');
    /// `headset_off` (U+E33A).
    pub const HEADSET_OFF: IconData = material('\u{e33a}');
    /// `healing` (U+E3F3).
    pub const HEALING: IconData = material('\u{e3f3}');
    /// `health_and_safety` (U+E1D5).
    pub const HEALTH_AND_SAFETY: IconData = material('\u{e1d5}');
    /// `hearing` (U+E023).
    pub const HEARING: IconData = material('\u{e023}');
    /// `hearing_disabled` (U+F104).
    pub const HEARING_DISABLED: IconData = material('\u{f104}');
    /// `heart_broken` (U+EAC2).
    pub const HEART_BROKEN: IconData = material('\u{eac2}');
    /// `heat_pump` (U+EC18).
    pub const HEAT_PUMP: IconData = material('\u{ec18}');
    /// `height` (U+EA16).
    pub const HEIGHT: IconData = material('\u{ea16}');
    /// `help` (U+E887).
    pub const HELP: IconData = material('\u{e887}');
    /// `help_center` (U+F1C0).
    pub const HELP_CENTER: IconData = material('\u{f1c0}');
    /// `help_outline` (U+E8FD).
    pub const HELP_OUTLINE: IconData = material('\u{e8fd}');
    /// `hevc` (U+F021).
    pub const HEVC: IconData = material('\u{f021}');
    /// `hexagon` (U+EB39).
    pub const HEXAGON: IconData = material('\u{eb39}');
    /// `hide_image` (U+F022).
    pub const HIDE_IMAGE: IconData = material('\u{f022}');
    /// `hide_source` (U+F023).
    pub const HIDE_SOURCE: IconData = material('\u{f023}');
    /// `high_quality` (U+E024).
    pub const HIGH_QUALITY: IconData = material('\u{e024}');
    /// `highlight` (U+E25F).
    pub const HIGHLIGHT: IconData = material('\u{e25f}');
    /// `highlight_alt` (U+EF52).
    pub const HIGHLIGHT_ALT: IconData = material('\u{ef52}');
    /// `highlight_off` (U+E888).
    pub const HIGHLIGHT_OFF: IconData = material('\u{e888}');
    /// `highlight_remove` (U+E888).
    pub const HIGHLIGHT_REMOVE: IconData = material('\u{e888}');
    /// `hiking` (U+E50A).
    pub const HIKING: IconData = material('\u{e50a}');
    /// `history` (U+E889).
    pub const HISTORY: IconData = material('\u{e889}');
    /// `history_edu` (U+EA3E).
    pub const HISTORY_EDU: IconData = material('\u{ea3e}');
    /// `history_toggle_off` (U+F17D).
    pub const HISTORY_TOGGLE_OFF: IconData = material('\u{f17d}');
    /// `hive` (U+EAA6).
    pub const HIVE: IconData = material('\u{eaa6}');
    /// `hls` (U+EB8A).
    pub const HLS: IconData = material('\u{eb8a}');
    /// `hls_off` (U+EB8C).
    pub const HLS_OFF: IconData = material('\u{eb8c}');
    /// `holiday_village` (U+E58A).
    pub const HOLIDAY_VILLAGE: IconData = material('\u{e58a}');
    /// `home` (U+E88A).
    pub const HOME: IconData = material('\u{e88a}');
    /// `home_filled` (U+E9B2).
    pub const HOME_FILLED: IconData = material('\u{e9b2}');
    /// `home_max` (U+F024).
    pub const HOME_MAX: IconData = material('\u{f024}');
    /// `home_mini` (U+F025).
    pub const HOME_MINI: IconData = material('\u{f025}');
    /// `home_repair_service` (U+F100).
    pub const HOME_REPAIR_SERVICE: IconData = material('\u{f100}');
    /// `home_work` (U+EA09).
    pub const HOME_WORK: IconData = material('\u{ea09}');
    /// `horizontal_distribute` (U+E014).
    pub const HORIZONTAL_DISTRIBUTE: IconData = material('\u{e014}');
    /// `horizontal_rule` (U+F108).
    pub const HORIZONTAL_RULE: IconData = material('\u{f108}');
    /// `horizontal_split` (U+E947).
    pub const HORIZONTAL_SPLIT: IconData = material('\u{e947}');
    /// `hot_tub` (U+EB46).
    pub const HOT_TUB: IconData = material('\u{eb46}');
    /// `hotel` (U+E53A).
    pub const HOTEL: IconData = material('\u{e53a}');
    /// `hotel_class` (U+E743).
    pub const HOTEL_CLASS: IconData = material('\u{e743}');
    /// `hourglass_bottom` (U+EA5C).
    pub const HOURGLASS_BOTTOM: IconData = material('\u{ea5c}');
    /// `hourglass_disabled` (U+EF53).
    pub const HOURGLASS_DISABLED: IconData = material('\u{ef53}');
    /// `hourglass_empty` (U+E88B).
    pub const HOURGLASS_EMPTY: IconData = material('\u{e88b}');
    /// `hourglass_full` (U+E88C).
    pub const HOURGLASS_FULL: IconData = material('\u{e88c}');
    /// `hourglass_top` (U+EA5B).
    pub const HOURGLASS_TOP: IconData = material('\u{ea5b}');
    /// `house` (U+EA44).
    pub const HOUSE: IconData = material('\u{ea44}');
    /// `house_siding` (U+F202).
    pub const HOUSE_SIDING: IconData = material('\u{f202}');
    /// `houseboat` (U+E584).
    pub const HOUSEBOAT: IconData = material('\u{e584}');
    /// `how_to_reg` (U+E174).
    pub const HOW_TO_REG: IconData = material('\u{e174}');
    /// `how_to_vote` (U+E175).
    pub const HOW_TO_VOTE: IconData = material('\u{e175}');
    /// `html` (U+EB7E).
    pub const HTML: IconData = material('\u{eb7e}');
    /// `http` (U+E902).
    pub const HTTP: IconData = material('\u{e902}');
    /// `https` (U+E88D).
    pub const HTTPS: IconData = material('\u{e88d}');
    /// `hub` (U+E9F4).
    pub const HUB: IconData = material('\u{e9f4}');
    /// `hvac` (U+F10E).
    pub const HVAC: IconData = material('\u{f10e}');
    /// `ice_skating` (U+E50B).
    pub const ICE_SKATING: IconData = material('\u{e50b}');
    /// `icecream` (U+EA69).
    pub const ICECREAM: IconData = material('\u{ea69}');
    /// `image` (U+E3F4).
    pub const IMAGE: IconData = material('\u{e3f4}');
    /// `image_aspect_ratio` (U+E3F5).
    pub const IMAGE_ASPECT_RATIO: IconData = material('\u{e3f5}');
    /// `image_not_supported` (U+F116).
    pub const IMAGE_NOT_SUPPORTED: IconData = material('\u{f116}');
    /// `image_search` (U+E43F).
    pub const IMAGE_SEARCH: IconData = material('\u{e43f}');
    /// `imagesearch_roller` (U+E9B4).
    pub const IMAGESEARCH_ROLLER: IconData = material('\u{e9b4}');
    /// `import_contacts` (U+E0E0).
    pub const IMPORT_CONTACTS: IconData = material('\u{e0e0}');
    /// `import_export` (U+E0C3).
    pub const IMPORT_EXPORT: IconData = material('\u{e0c3}');
    /// `important_devices` (U+E912).
    pub const IMPORTANT_DEVICES: IconData = material('\u{e912}');
    /// `inbox` (U+E156).
    pub const INBOX: IconData = material('\u{e156}');
    /// `incomplete_circle` (U+E79B).
    pub const INCOMPLETE_CIRCLE: IconData = material('\u{e79b}');
    /// `indeterminate_check_box` (U+E909).
    pub const INDETERMINATE_CHECK_BOX: IconData = material('\u{e909}');
    /// `info` (U+E88E).
    pub const INFO: IconData = material('\u{e88e}');
    /// `info_outline` (U+E88F).
    pub const INFO_OUTLINE: IconData = material('\u{e88f}');
    /// `input` (U+E890).
    pub const INPUT: IconData = material('\u{e890}');
    /// `insert_chart` (U+E24B).
    pub const INSERT_CHART: IconData = material('\u{e24b}');
    /// `insert_chart_outlined` (U+E26A).
    pub const INSERT_CHART_OUTLINED: IconData = material('\u{e26a}');
    /// `insert_comment` (U+E24C).
    pub const INSERT_COMMENT: IconData = material('\u{e24c}');
    /// `insert_drive_file` (U+E24D).
    pub const INSERT_DRIVE_FILE: IconData = material('\u{e24d}');
    /// `insert_emoticon` (U+E24E).
    pub const INSERT_EMOTICON: IconData = material('\u{e24e}');
    /// `insert_invitation` (U+E24F).
    pub const INSERT_INVITATION: IconData = material('\u{e24f}');
    /// `insert_link` (U+E250).
    pub const INSERT_LINK: IconData = material('\u{e250}');
    /// `insert_page_break` (U+EACA).
    pub const INSERT_PAGE_BREAK: IconData = material('\u{eaca}');
    /// `insert_photo` (U+E251).
    pub const INSERT_PHOTO: IconData = material('\u{e251}');
    /// `insights` (U+F092).
    pub const INSIGHTS: IconData = material('\u{f092}');
    /// `install_desktop` (U+EB71).
    pub const INSTALL_DESKTOP: IconData = material('\u{eb71}');
    /// `install_mobile` (U+EB72).
    pub const INSTALL_MOBILE: IconData = material('\u{eb72}');
    /// `integration_instructions` (U+EF54).
    pub const INTEGRATION_INSTRUCTIONS: IconData = material('\u{ef54}');
    /// `interests` (U+E7C8).
    pub const INTERESTS: IconData = material('\u{e7c8}');
    /// `interpreter_mode` (U+E83B).
    pub const INTERPRETER_MODE: IconData = material('\u{e83b}');
    /// `inventory` (U+E179).
    pub const INVENTORY: IconData = material('\u{e179}');
    /// `inventory_2` (U+E1A1).
    pub const INVENTORY_2: IconData = material('\u{e1a1}');
    /// `invert_colors` (U+E891).
    pub const INVERT_COLORS: IconData = material('\u{e891}');
    /// `invert_colors_off` (U+E0C4).
    pub const INVERT_COLORS_OFF: IconData = material('\u{e0c4}');
    /// `invert_colors_on` (U+E891).
    pub const INVERT_COLORS_ON: IconData = material('\u{e891}');
    /// `ios_share` (U+E6B8).
    pub const IOS_SHARE: IconData = material('\u{e6b8}');
    /// `iron` (U+E583).
    pub const IRON: IconData = material('\u{e583}');
    /// `iso` (U+E3F6).
    pub const ISO: IconData = material('\u{e3f6}');
    /// `javascript` (U+EB7C).
    pub const JAVASCRIPT: IconData = material('\u{eb7c}');
    /// `join_full` (U+EAEB).
    pub const JOIN_FULL: IconData = material('\u{eaeb}');
    /// `join_inner` (U+EAF4).
    pub const JOIN_INNER: IconData = material('\u{eaf4}');
    /// `join_left` (U+EAF2).
    pub const JOIN_LEFT: IconData = material('\u{eaf2}');
    /// `join_right` (U+EAEA).
    pub const JOIN_RIGHT: IconData = material('\u{eaea}');
    /// `kayaking` (U+E50C).
    pub const KAYAKING: IconData = material('\u{e50c}');
    /// `kebab_dining` (U+E842).
    pub const KEBAB_DINING: IconData = material('\u{e842}');
    /// `key` (U+E73C).
    pub const KEY: IconData = material('\u{e73c}');
    /// `key_off` (U+EB84).
    pub const KEY_OFF: IconData = material('\u{eb84}');
    /// `keyboard` (U+E312).
    pub const KEYBOARD: IconData = material('\u{e312}');
    /// `keyboard_alt` (U+F028).
    pub const KEYBOARD_ALT: IconData = material('\u{f028}');
    /// `keyboard_arrow_down` (U+E313).
    pub const KEYBOARD_ARROW_DOWN: IconData = material('\u{e313}');
    /// `keyboard_arrow_left` (U+E314).
    pub const KEYBOARD_ARROW_LEFT: IconData = material('\u{e314}');
    /// `keyboard_arrow_right` (U+E315).
    pub const KEYBOARD_ARROW_RIGHT: IconData = material('\u{e315}');
    /// `keyboard_arrow_up` (U+E316).
    pub const KEYBOARD_ARROW_UP: IconData = material('\u{e316}');
    /// `keyboard_backspace` (U+E317).
    pub const KEYBOARD_BACKSPACE: IconData = material('\u{e317}');
    /// `keyboard_capslock` (U+E318).
    pub const KEYBOARD_CAPSLOCK: IconData = material('\u{e318}');
    /// `keyboard_command` (U+EAE0).
    pub const KEYBOARD_COMMAND: IconData = material('\u{eae0}');
    /// `keyboard_command_key` (U+EAE7).
    pub const KEYBOARD_COMMAND_KEY: IconData = material('\u{eae7}');
    /// `keyboard_control` (U+E5D3).
    pub const KEYBOARD_CONTROL: IconData = material('\u{e5d3}');
    /// `keyboard_control_key` (U+EAE6).
    pub const KEYBOARD_CONTROL_KEY: IconData = material('\u{eae6}');
    /// `keyboard_double_arrow_down` (U+EAD0).
    pub const KEYBOARD_DOUBLE_ARROW_DOWN: IconData = material('\u{ead0}');
    /// `keyboard_double_arrow_left` (U+EAC3).
    pub const KEYBOARD_DOUBLE_ARROW_LEFT: IconData = material('\u{eac3}');
    /// `keyboard_double_arrow_right` (U+EAC9).
    pub const KEYBOARD_DOUBLE_ARROW_RIGHT: IconData = material('\u{eac9}');
    /// `keyboard_double_arrow_up` (U+EACF).
    pub const KEYBOARD_DOUBLE_ARROW_UP: IconData = material('\u{eacf}');
    /// `keyboard_hide` (U+E31A).
    pub const KEYBOARD_HIDE: IconData = material('\u{e31a}');
    /// `keyboard_option` (U+EADF).
    pub const KEYBOARD_OPTION: IconData = material('\u{eadf}');
    /// `keyboard_option_key` (U+EAE8).
    pub const KEYBOARD_OPTION_KEY: IconData = material('\u{eae8}');
    /// `keyboard_return` (U+E31B).
    pub const KEYBOARD_RETURN: IconData = material('\u{e31b}');
    /// `keyboard_tab` (U+E31C).
    pub const KEYBOARD_TAB: IconData = material('\u{e31c}');
    /// `keyboard_voice` (U+E31D).
    pub const KEYBOARD_VOICE: IconData = material('\u{e31d}');
    /// `king_bed` (U+EA45).
    pub const KING_BED: IconData = material('\u{ea45}');
    /// `kitchen` (U+EB47).
    pub const KITCHEN: IconData = material('\u{eb47}');
    /// `kitesurfing` (U+E50D).
    pub const KITESURFING: IconData = material('\u{e50d}');
    /// `label` (U+E892).
    pub const LABEL: IconData = material('\u{e892}');
    /// `label_important` (U+E937).
    pub const LABEL_IMPORTANT: IconData = material('\u{e937}');
    /// `label_important_outline` (U+E948).
    pub const LABEL_IMPORTANT_OUTLINE: IconData = material('\u{e948}');
    /// `label_off` (U+E9B6).
    pub const LABEL_OFF: IconData = material('\u{e9b6}');
    /// `label_outline` (U+E893).
    pub const LABEL_OUTLINE: IconData = material('\u{e893}');
    /// `lan` (U+EB2F).
    pub const LAN: IconData = material('\u{eb2f}');
    /// `landscape` (U+E3F7).
    pub const LANDSCAPE: IconData = material('\u{e3f7}');
    /// `landslide` (U+EBD7).
    pub const LANDSLIDE: IconData = material('\u{ebd7}');
    /// `language` (U+E894).
    pub const LANGUAGE: IconData = material('\u{e894}');
    /// `laptop` (U+E31E).
    pub const LAPTOP: IconData = material('\u{e31e}');
    /// `laptop_chromebook` (U+E31F).
    pub const LAPTOP_CHROMEBOOK: IconData = material('\u{e31f}');
    /// `laptop_mac` (U+E320).
    pub const LAPTOP_MAC: IconData = material('\u{e320}');
    /// `laptop_windows` (U+E321).
    pub const LAPTOP_WINDOWS: IconData = material('\u{e321}');
    /// `last_page` (U+E5DD).
    pub const LAST_PAGE: IconData = material('\u{e5dd}');
    /// `launch` (U+E895).
    pub const LAUNCH: IconData = material('\u{e895}');
    /// `layers` (U+E53B).
    pub const LAYERS: IconData = material('\u{e53b}');
    /// `layers_clear` (U+E53C).
    pub const LAYERS_CLEAR: IconData = material('\u{e53c}');
    /// `leaderboard` (U+F20C).
    pub const LEADERBOARD: IconData = material('\u{f20c}');
    /// `leak_add` (U+E3F8).
    pub const LEAK_ADD: IconData = material('\u{e3f8}');
    /// `leak_remove` (U+E3F9).
    pub const LEAK_REMOVE: IconData = material('\u{e3f9}');
    /// `leave_bags_at_home` (U+F21B).
    pub const LEAVE_BAGS_AT_HOME: IconData = material('\u{f21b}');
    /// `legend_toggle` (U+F11B).
    pub const LEGEND_TOGGLE: IconData = material('\u{f11b}');
    /// `lens` (U+E3FA).
    pub const LENS: IconData = material('\u{e3fa}');
    /// `lens_blur` (U+F029).
    pub const LENS_BLUR: IconData = material('\u{f029}');
    /// `library_add` (U+E02E).
    pub const LIBRARY_ADD: IconData = material('\u{e02e}');
    /// `library_add_check` (U+E9B7).
    pub const LIBRARY_ADD_CHECK: IconData = material('\u{e9b7}');
    /// `library_books` (U+E02F).
    pub const LIBRARY_BOOKS: IconData = material('\u{e02f}');
    /// `library_music` (U+E030).
    pub const LIBRARY_MUSIC: IconData = material('\u{e030}');
    /// `light` (U+F02A).
    pub const LIGHT: IconData = material('\u{f02a}');
    /// `light_mode` (U+E518).
    pub const LIGHT_MODE: IconData = material('\u{e518}');
    /// `lightbulb` (U+E0F0).
    pub const LIGHTBULB: IconData = material('\u{e0f0}');
    /// `lightbulb_circle` (U+EBFE).
    pub const LIGHTBULB_CIRCLE: IconData = material('\u{ebfe}');
    /// `lightbulb_outline` (U+E90F).
    pub const LIGHTBULB_OUTLINE: IconData = material('\u{e90f}');
    /// `line_axis` (U+EA9A).
    pub const LINE_AXIS: IconData = material('\u{ea9a}');
    /// `line_style` (U+E919).
    pub const LINE_STYLE: IconData = material('\u{e919}');
    /// `line_weight` (U+E91A).
    pub const LINE_WEIGHT: IconData = material('\u{e91a}');
    /// `linear_scale` (U+E260).
    pub const LINEAR_SCALE: IconData = material('\u{e260}');
    /// `link` (U+E157).
    pub const LINK: IconData = material('\u{e157}');
    /// `link_off` (U+E16F).
    pub const LINK_OFF: IconData = material('\u{e16f}');
    /// `linked_camera` (U+E438).
    pub const LINKED_CAMERA: IconData = material('\u{e438}');
    /// `liquor` (U+EA60).
    pub const LIQUOR: IconData = material('\u{ea60}');
    /// `list` (U+E896).
    pub const LIST: IconData = material('\u{e896}');
    /// `list_alt` (U+E0EE).
    pub const LIST_ALT: IconData = material('\u{e0ee}');
    /// `live_help` (U+E0C6).
    pub const LIVE_HELP: IconData = material('\u{e0c6}');
    /// `live_tv` (U+E639).
    pub const LIVE_TV: IconData = material('\u{e639}');
    /// `living` (U+F02B).
    pub const LIVING: IconData = material('\u{f02b}');
    /// `local_activity` (U+E53F).
    pub const LOCAL_ACTIVITY: IconData = material('\u{e53f}');
    /// `local_airport` (U+E53D).
    pub const LOCAL_AIRPORT: IconData = material('\u{e53d}');
    /// `local_atm` (U+E53E).
    pub const LOCAL_ATM: IconData = material('\u{e53e}');
    /// `local_attraction` (U+E53F).
    pub const LOCAL_ATTRACTION: IconData = material('\u{e53f}');
    /// `local_bar` (U+E540).
    pub const LOCAL_BAR: IconData = material('\u{e540}');
    /// `local_cafe` (U+E541).
    pub const LOCAL_CAFE: IconData = material('\u{e541}');
    /// `local_car_wash` (U+E542).
    pub const LOCAL_CAR_WASH: IconData = material('\u{e542}');
    /// `local_convenience_store` (U+E543).
    pub const LOCAL_CONVENIENCE_STORE: IconData = material('\u{e543}');
    /// `local_dining` (U+E556).
    pub const LOCAL_DINING: IconData = material('\u{e556}');
    /// `local_drink` (U+E544).
    pub const LOCAL_DRINK: IconData = material('\u{e544}');
    /// `local_fire_department` (U+EF55).
    pub const LOCAL_FIRE_DEPARTMENT: IconData = material('\u{ef55}');
    /// `local_florist` (U+E545).
    pub const LOCAL_FLORIST: IconData = material('\u{e545}');
    /// `local_gas_station` (U+E546).
    pub const LOCAL_GAS_STATION: IconData = material('\u{e546}');
    /// `local_grocery_store` (U+E547).
    pub const LOCAL_GROCERY_STORE: IconData = material('\u{e547}');
    /// `local_hospital` (U+E548).
    pub const LOCAL_HOSPITAL: IconData = material('\u{e548}');
    /// `local_hotel` (U+E549).
    pub const LOCAL_HOTEL: IconData = material('\u{e549}');
    /// `local_laundry_service` (U+E54A).
    pub const LOCAL_LAUNDRY_SERVICE: IconData = material('\u{e54a}');
    /// `local_library` (U+E54B).
    pub const LOCAL_LIBRARY: IconData = material('\u{e54b}');
    /// `local_mall` (U+E54C).
    pub const LOCAL_MALL: IconData = material('\u{e54c}');
    /// `local_movies` (U+E54D).
    pub const LOCAL_MOVIES: IconData = material('\u{e54d}');
    /// `local_offer` (U+E54E).
    pub const LOCAL_OFFER: IconData = material('\u{e54e}');
    /// `local_parking` (U+E54F).
    pub const LOCAL_PARKING: IconData = material('\u{e54f}');
    /// `local_pharmacy` (U+E550).
    pub const LOCAL_PHARMACY: IconData = material('\u{e550}');
    /// `local_phone` (U+E551).
    pub const LOCAL_PHONE: IconData = material('\u{e551}');
    /// `local_pizza` (U+E552).
    pub const LOCAL_PIZZA: IconData = material('\u{e552}');
    /// `local_play` (U+E553).
    pub const LOCAL_PLAY: IconData = material('\u{e553}');
    /// `local_police` (U+EF56).
    pub const LOCAL_POLICE: IconData = material('\u{ef56}');
    /// `local_post_office` (U+E554).
    pub const LOCAL_POST_OFFICE: IconData = material('\u{e554}');
    /// `local_print_shop` (U+E555).
    pub const LOCAL_PRINT_SHOP: IconData = material('\u{e555}');
    /// `local_printshop` (U+E555).
    pub const LOCAL_PRINTSHOP: IconData = material('\u{e555}');
    /// `local_restaurant` (U+E556).
    pub const LOCAL_RESTAURANT: IconData = material('\u{e556}');
    /// `local_see` (U+E557).
    pub const LOCAL_SEE: IconData = material('\u{e557}');
    /// `local_shipping` (U+E558).
    pub const LOCAL_SHIPPING: IconData = material('\u{e558}');
    /// `local_taxi` (U+E559).
    pub const LOCAL_TAXI: IconData = material('\u{e559}');
    /// `location_city` (U+E7F1).
    pub const LOCATION_CITY: IconData = material('\u{e7f1}');
    /// `location_disabled` (U+E1B6).
    pub const LOCATION_DISABLED: IconData = material('\u{e1b6}');
    /// `location_history` (U+E55A).
    pub const LOCATION_HISTORY: IconData = material('\u{e55a}');
    /// `location_off` (U+E0C7).
    pub const LOCATION_OFF: IconData = material('\u{e0c7}');
    /// `location_on` (U+E0C8).
    pub const LOCATION_ON: IconData = material('\u{e0c8}');
    /// `location_pin` (U+F1DB).
    pub const LOCATION_PIN: IconData = material('\u{f1db}');
    /// `location_searching` (U+E1B7).
    pub const LOCATION_SEARCHING: IconData = material('\u{e1b7}');
    /// `lock` (U+E897).
    pub const LOCK: IconData = material('\u{e897}');
    /// `lock_clock` (U+EF57).
    pub const LOCK_CLOCK: IconData = material('\u{ef57}');
    /// `lock_open` (U+E898).
    pub const LOCK_OPEN: IconData = material('\u{e898}');
    /// `lock_outline` (U+E899).
    pub const LOCK_OUTLINE: IconData = material('\u{e899}');
    /// `lock_person` (U+F8F3).
    pub const LOCK_PERSON: IconData = material('\u{f8f3}');
    /// `lock_reset` (U+EADE).
    pub const LOCK_RESET: IconData = material('\u{eade}');
    /// `login` (U+EA77).
    pub const LOGIN: IconData = material('\u{ea77}');
    /// `logo_dev` (U+EAD6).
    pub const LOGO_DEV: IconData = material('\u{ead6}');
    /// `logout` (U+E9BA).
    pub const LOGOUT: IconData = material('\u{e9ba}');
    /// `looks` (U+E3FC).
    pub const LOOKS: IconData = material('\u{e3fc}');
    /// `looks_3` (U+E3FB).
    pub const LOOKS_3: IconData = material('\u{e3fb}');
    /// `looks_4` (U+E3FD).
    pub const LOOKS_4: IconData = material('\u{e3fd}');
    /// `looks_5` (U+E3FE).
    pub const LOOKS_5: IconData = material('\u{e3fe}');
    /// `looks_6` (U+E3FF).
    pub const LOOKS_6: IconData = material('\u{e3ff}');
    /// `looks_one` (U+E400).
    pub const LOOKS_ONE: IconData = material('\u{e400}');
    /// `looks_two` (U+E401).
    pub const LOOKS_TWO: IconData = material('\u{e401}');
    /// `loop` (U+E028).
    pub const LOOP: IconData = material('\u{e028}');
    /// `loupe` (U+E402).
    pub const LOUPE: IconData = material('\u{e402}');
    /// `low_priority` (U+E16D).
    pub const LOW_PRIORITY: IconData = material('\u{e16d}');
    /// `loyalty` (U+E89A).
    pub const LOYALTY: IconData = material('\u{e89a}');
    /// `lte_mobiledata` (U+F02C).
    pub const LTE_MOBILEDATA: IconData = material('\u{f02c}');
    /// `lte_plus_mobiledata` (U+F02D).
    pub const LTE_PLUS_MOBILEDATA: IconData = material('\u{f02d}');
    /// `luggage` (U+F235).
    pub const LUGGAGE: IconData = material('\u{f235}');
    /// `lunch_dining` (U+EA61).
    pub const LUNCH_DINING: IconData = material('\u{ea61}');
    /// `lyrics` (U+EC0B).
    pub const LYRICS: IconData = material('\u{ec0b}');
    /// `macro_off` (U+F8D2).
    pub const MACRO_OFF: IconData = material('\u{f8d2}');
    /// `mail` (U+E158).
    pub const MAIL: IconData = material('\u{e158}');
    /// `mail_lock` (U+EC0A).
    pub const MAIL_LOCK: IconData = material('\u{ec0a}');
    /// `mail_outline` (U+E0E1).
    pub const MAIL_OUTLINE: IconData = material('\u{e0e1}');
    /// `male` (U+E58E).
    pub const MALE: IconData = material('\u{e58e}');
    /// `man` (U+E4EB).
    pub const MAN: IconData = material('\u{e4eb}');
    /// `man_2` (U+F8E1).
    pub const MAN_2: IconData = material('\u{f8e1}');
    /// `man_3` (U+F8E2).
    pub const MAN_3: IconData = material('\u{f8e2}');
    /// `man_4` (U+F8E3).
    pub const MAN_4: IconData = material('\u{f8e3}');
    /// `manage_accounts` (U+F02E).
    pub const MANAGE_ACCOUNTS: IconData = material('\u{f02e}');
    /// `manage_history` (U+EBE7).
    pub const MANAGE_HISTORY: IconData = material('\u{ebe7}');
    /// `manage_search` (U+F02F).
    pub const MANAGE_SEARCH: IconData = material('\u{f02f}');
    /// `map` (U+E55B).
    pub const MAP: IconData = material('\u{e55b}');
    /// `maps_home_work` (U+F030).
    pub const MAPS_HOME_WORK: IconData = material('\u{f030}');
    /// `maps_ugc` (U+EF58).
    pub const MAPS_UGC: IconData = material('\u{ef58}');
    /// `margin` (U+E9BB).
    pub const MARGIN: IconData = material('\u{e9bb}');
    /// `mark_as_unread` (U+E9BC).
    pub const MARK_AS_UNREAD: IconData = material('\u{e9bc}');
    /// `mark_chat_read` (U+F18B).
    pub const MARK_CHAT_READ: IconData = material('\u{f18b}');
    /// `mark_chat_unread` (U+F189).
    pub const MARK_CHAT_UNREAD: IconData = material('\u{f189}');
    /// `mark_email_read` (U+F18C).
    pub const MARK_EMAIL_READ: IconData = material('\u{f18c}');
    /// `mark_email_unread` (U+F18A).
    pub const MARK_EMAIL_UNREAD: IconData = material('\u{f18a}');
    /// `mark_unread_chat_alt` (U+EB9D).
    pub const MARK_UNREAD_CHAT_ALT: IconData = material('\u{eb9d}');
    /// `markunread` (U+E159).
    pub const MARKUNREAD: IconData = material('\u{e159}');
    /// `markunread_mailbox` (U+E89B).
    pub const MARKUNREAD_MAILBOX: IconData = material('\u{e89b}');
    /// `masks` (U+F218).
    pub const MASKS: IconData = material('\u{f218}');
    /// `maximize` (U+E930).
    pub const MAXIMIZE: IconData = material('\u{e930}');
    /// `media_bluetooth_off` (U+F031).
    pub const MEDIA_BLUETOOTH_OFF: IconData = material('\u{f031}');
    /// `media_bluetooth_on` (U+F032).
    pub const MEDIA_BLUETOOTH_ON: IconData = material('\u{f032}');
    /// `mediation` (U+EFA7).
    pub const MEDIATION: IconData = material('\u{efa7}');
    /// `medical_information` (U+EBED).
    pub const MEDICAL_INFORMATION: IconData = material('\u{ebed}');
    /// `medical_services` (U+F109).
    pub const MEDICAL_SERVICES: IconData = material('\u{f109}');
    /// `medication` (U+F033).
    pub const MEDICATION: IconData = material('\u{f033}');
    /// `medication_liquid` (U+EA87).
    pub const MEDICATION_LIQUID: IconData = material('\u{ea87}');
    /// `meeting_room` (U+EB4F).
    pub const MEETING_ROOM: IconData = material('\u{eb4f}');
    /// `memory` (U+E322).
    pub const MEMORY: IconData = material('\u{e322}');
    /// `menu` (U+E5D2).
    pub const MENU: IconData = material('\u{e5d2}');
    /// `menu_book` (U+EA19).
    pub const MENU_BOOK: IconData = material('\u{ea19}');
    /// `menu_open` (U+E9BD).
    pub const MENU_OPEN: IconData = material('\u{e9bd}');
    /// `merge` (U+EB98).
    pub const MERGE: IconData = material('\u{eb98}');
    /// `merge_type` (U+E252).
    pub const MERGE_TYPE: IconData = material('\u{e252}');
    /// `message` (U+E0C9).
    pub const MESSAGE: IconData = material('\u{e0c9}');
    /// `messenger` (U+E0CA).
    pub const MESSENGER: IconData = material('\u{e0ca}');
    /// `messenger_outline` (U+E0CB).
    pub const MESSENGER_OUTLINE: IconData = material('\u{e0cb}');
    /// `mic` (U+E029).
    pub const MIC: IconData = material('\u{e029}');
    /// `mic_external_off` (U+EF59).
    pub const MIC_EXTERNAL_OFF: IconData = material('\u{ef59}');
    /// `mic_external_on` (U+EF5A).
    pub const MIC_EXTERNAL_ON: IconData = material('\u{ef5a}');
    /// `mic_none` (U+E02A).
    pub const MIC_NONE: IconData = material('\u{e02a}');
    /// `mic_off` (U+E02B).
    pub const MIC_OFF: IconData = material('\u{e02b}');
    /// `microwave` (U+F204).
    pub const MICROWAVE: IconData = material('\u{f204}');
    /// `military_tech` (U+EA3F).
    pub const MILITARY_TECH: IconData = material('\u{ea3f}');
    /// `minimize` (U+E931).
    pub const MINIMIZE: IconData = material('\u{e931}');
    /// `minor_crash` (U+EBF1).
    pub const MINOR_CRASH: IconData = material('\u{ebf1}');
    /// `miscellaneous_services` (U+F10C).
    pub const MISCELLANEOUS_SERVICES: IconData = material('\u{f10c}');
    /// `missed_video_call` (U+E073).
    pub const MISSED_VIDEO_CALL: IconData = material('\u{e073}');
    /// `mms` (U+E618).
    pub const MMS: IconData = material('\u{e618}');
    /// `mobile_friendly` (U+E200).
    pub const MOBILE_FRIENDLY: IconData = material('\u{e200}');
    /// `mobile_off` (U+E201).
    pub const MOBILE_OFF: IconData = material('\u{e201}');
    /// `mobile_screen_share` (U+E0E7).
    pub const MOBILE_SCREEN_SHARE: IconData = material('\u{e0e7}');
    /// `mobiledata_off` (U+F034).
    pub const MOBILEDATA_OFF: IconData = material('\u{f034}');
    /// `mode` (U+F097).
    pub const MODE: IconData = material('\u{f097}');
    /// `mode_comment` (U+E253).
    pub const MODE_COMMENT: IconData = material('\u{e253}');
    /// `mode_edit` (U+E254).
    pub const MODE_EDIT: IconData = material('\u{e254}');
    /// `mode_edit_outline` (U+F035).
    pub const MODE_EDIT_OUTLINE: IconData = material('\u{f035}');
    /// `mode_fan_off` (U+EC17).
    pub const MODE_FAN_OFF: IconData = material('\u{ec17}');
    /// `mode_night` (U+F036).
    pub const MODE_NIGHT: IconData = material('\u{f036}');
    /// `mode_of_travel` (U+E7CE).
    pub const MODE_OF_TRAVEL: IconData = material('\u{e7ce}');
    /// `mode_standby` (U+F037).
    pub const MODE_STANDBY: IconData = material('\u{f037}');
    /// `model_training` (U+F0CF).
    pub const MODEL_TRAINING: IconData = material('\u{f0cf}');
    /// `monetization_on` (U+E263).
    pub const MONETIZATION_ON: IconData = material('\u{e263}');
    /// `money` (U+E57D).
    pub const MONEY: IconData = material('\u{e57d}');
    /// `money_off` (U+E25C).
    pub const MONEY_OFF: IconData = material('\u{e25c}');
    /// `money_off_csred` (U+F038).
    pub const MONEY_OFF_CSRED: IconData = material('\u{f038}');
    /// `monitor` (U+EF5B).
    pub const MONITOR: IconData = material('\u{ef5b}');
    /// `monitor_heart` (U+EAA2).
    pub const MONITOR_HEART: IconData = material('\u{eaa2}');
    /// `monitor_weight` (U+F039).
    pub const MONITOR_WEIGHT: IconData = material('\u{f039}');
    /// `monochrome_photos` (U+E403).
    pub const MONOCHROME_PHOTOS: IconData = material('\u{e403}');
    /// `mood` (U+E7F2).
    pub const MOOD: IconData = material('\u{e7f2}');
    /// `mood_bad` (U+E7F3).
    pub const MOOD_BAD: IconData = material('\u{e7f3}');
    /// `moped` (U+EB28).
    pub const MOPED: IconData = material('\u{eb28}');
    /// `more` (U+E619).
    pub const MORE: IconData = material('\u{e619}');
    /// `more_horiz` (U+E5D3).
    pub const MORE_HORIZ: IconData = material('\u{e5d3}');
    /// `more_time` (U+EA5D).
    pub const MORE_TIME: IconData = material('\u{ea5d}');
    /// `more_vert` (U+E5D4).
    pub const MORE_VERT: IconData = material('\u{e5d4}');
    /// `mosque` (U+EAB2).
    pub const MOSQUE: IconData = material('\u{eab2}');
    /// `motion_photos_auto` (U+F03A).
    pub const MOTION_PHOTOS_AUTO: IconData = material('\u{f03a}');
    /// `motion_photos_off` (U+E9C0).
    pub const MOTION_PHOTOS_OFF: IconData = material('\u{e9c0}');
    /// `motion_photos_on` (U+E9C1).
    pub const MOTION_PHOTOS_ON: IconData = material('\u{e9c1}');
    /// `motion_photos_pause` (U+F227).
    pub const MOTION_PHOTOS_PAUSE: IconData = material('\u{f227}');
    /// `motion_photos_paused` (U+E9C2).
    pub const MOTION_PHOTOS_PAUSED: IconData = material('\u{e9c2}');
    /// `motorcycle` (U+E91B).
    pub const MOTORCYCLE: IconData = material('\u{e91b}');
    /// `mouse` (U+E323).
    pub const MOUSE: IconData = material('\u{e323}');
    /// `move_down` (U+EB61).
    pub const MOVE_DOWN: IconData = material('\u{eb61}');
    /// `move_to_inbox` (U+E168).
    pub const MOVE_TO_INBOX: IconData = material('\u{e168}');
    /// `move_up` (U+EB64).
    pub const MOVE_UP: IconData = material('\u{eb64}');
    /// `movie` (U+E02C).
    pub const MOVIE: IconData = material('\u{e02c}');
    /// `movie_creation` (U+E404).
    pub const MOVIE_CREATION: IconData = material('\u{e404}');
    /// `movie_edit` (U+F840).
    pub const MOVIE_EDIT: IconData = material('\u{f840}');
    /// `movie_filter` (U+E43A).
    pub const MOVIE_FILTER: IconData = material('\u{e43a}');
    /// `moving` (U+E501).
    pub const MOVING: IconData = material('\u{e501}');
    /// `mp` (U+E9C3).
    pub const MP: IconData = material('\u{e9c3}');
    /// `multiline_chart` (U+E6DF).
    pub const MULTILINE_CHART: IconData = material('\u{e6df}');
    /// `multiple_stop` (U+F1B9).
    pub const MULTIPLE_STOP: IconData = material('\u{f1b9}');
    /// `multitrack_audio` (U+E1B8).
    pub const MULTITRACK_AUDIO: IconData = material('\u{e1b8}');
    /// `museum` (U+EA36).
    pub const MUSEUM: IconData = material('\u{ea36}');
    /// `music_note` (U+E405).
    pub const MUSIC_NOTE: IconData = material('\u{e405}');
    /// `music_off` (U+E440).
    pub const MUSIC_OFF: IconData = material('\u{e440}');
    /// `music_video` (U+E063).
    pub const MUSIC_VIDEO: IconData = material('\u{e063}');
    /// `my_library_add` (U+E02E).
    pub const MY_LIBRARY_ADD: IconData = material('\u{e02e}');
    /// `my_library_books` (U+E02F).
    pub const MY_LIBRARY_BOOKS: IconData = material('\u{e02f}');
    /// `my_library_music` (U+E030).
    pub const MY_LIBRARY_MUSIC: IconData = material('\u{e030}');
    /// `my_location` (U+E55C).
    pub const MY_LOCATION: IconData = material('\u{e55c}');
    /// `nat` (U+EF5C).
    pub const NAT: IconData = material('\u{ef5c}');
    /// `nature` (U+E406).
    pub const NATURE: IconData = material('\u{e406}');
    /// `nature_people` (U+E407).
    pub const NATURE_PEOPLE: IconData = material('\u{e407}');
    /// `navigate_before` (U+E408).
    pub const NAVIGATE_BEFORE: IconData = material('\u{e408}');
    /// `navigate_next` (U+E409).
    pub const NAVIGATE_NEXT: IconData = material('\u{e409}');
    /// `navigation` (U+E55D).
    pub const NAVIGATION: IconData = material('\u{e55d}');
    /// `near_me` (U+E569).
    pub const NEAR_ME: IconData = material('\u{e569}');
    /// `near_me_disabled` (U+F1EF).
    pub const NEAR_ME_DISABLED: IconData = material('\u{f1ef}');
    /// `nearby_error` (U+F03B).
    pub const NEARBY_ERROR: IconData = material('\u{f03b}');
    /// `nearby_off` (U+F03C).
    pub const NEARBY_OFF: IconData = material('\u{f03c}');
    /// `nest_cam_wired_stand` (U+EC16).
    pub const NEST_CAM_WIRED_STAND: IconData = material('\u{ec16}');
    /// `network_cell` (U+E1B9).
    pub const NETWORK_CELL: IconData = material('\u{e1b9}');
    /// `network_check` (U+E640).
    pub const NETWORK_CHECK: IconData = material('\u{e640}');
    /// `network_locked` (U+E61A).
    pub const NETWORK_LOCKED: IconData = material('\u{e61a}');
    /// `network_ping` (U+EBCA).
    pub const NETWORK_PING: IconData = material('\u{ebca}');
    /// `network_wifi` (U+E1BA).
    pub const NETWORK_WIFI: IconData = material('\u{e1ba}');
    /// `network_wifi_1_bar` (U+EBE4).
    pub const NETWORK_WIFI_1_BAR: IconData = material('\u{ebe4}');
    /// `network_wifi_2_bar` (U+EBD6).
    pub const NETWORK_WIFI_2_BAR: IconData = material('\u{ebd6}');
    /// `network_wifi_3_bar` (U+EBE1).
    pub const NETWORK_WIFI_3_BAR: IconData = material('\u{ebe1}');
    /// `new_label` (U+E609).
    pub const NEW_LABEL: IconData = material('\u{e609}');
    /// `new_releases` (U+E031).
    pub const NEW_RELEASES: IconData = material('\u{e031}');
    /// `newspaper` (U+EB81).
    pub const NEWSPAPER: IconData = material('\u{eb81}');
    /// `next_plan` (U+EF5D).
    pub const NEXT_PLAN: IconData = material('\u{ef5d}');
    /// `next_week` (U+E16A).
    pub const NEXT_WEEK: IconData = material('\u{e16a}');
    /// `nfc` (U+E1BB).
    pub const NFC: IconData = material('\u{e1bb}');
    /// `night_shelter` (U+F1F1).
    pub const NIGHT_SHELTER: IconData = material('\u{f1f1}');
    /// `nightlife` (U+EA62).
    pub const NIGHTLIFE: IconData = material('\u{ea62}');
    /// `nightlight` (U+F03D).
    pub const NIGHTLIGHT: IconData = material('\u{f03d}');
    /// `nightlight_round` (U+EF5E).
    pub const NIGHTLIGHT_ROUND: IconData = material('\u{ef5e}');
    /// `nights_stay` (U+EA46).
    pub const NIGHTS_STAY: IconData = material('\u{ea46}');
    /// `no_accounts` (U+F03E).
    pub const NO_ACCOUNTS: IconData = material('\u{f03e}');
    /// `no_adult_content` (U+F8FE).
    pub const NO_ADULT_CONTENT: IconData = material('\u{f8fe}');
    /// `no_backpack` (U+F237).
    pub const NO_BACKPACK: IconData = material('\u{f237}');
    /// `no_cell` (U+F1A4).
    pub const NO_CELL: IconData = material('\u{f1a4}');
    /// `no_crash` (U+EBF0).
    pub const NO_CRASH: IconData = material('\u{ebf0}');
    /// `no_drinks` (U+F1A5).
    pub const NO_DRINKS: IconData = material('\u{f1a5}');
    /// `no_encryption` (U+E641).
    pub const NO_ENCRYPTION: IconData = material('\u{e641}');
    /// `no_encryption_gmailerrorred` (U+F03F).
    pub const NO_ENCRYPTION_GMAILERRORRED: IconData = material('\u{f03f}');
    /// `no_flash` (U+F1A6).
    pub const NO_FLASH: IconData = material('\u{f1a6}');
    /// `no_food` (U+F1A7).
    pub const NO_FOOD: IconData = material('\u{f1a7}');
    /// `no_luggage` (U+F23B).
    pub const NO_LUGGAGE: IconData = material('\u{f23b}');
    /// `no_meals` (U+F1D6).
    pub const NO_MEALS: IconData = material('\u{f1d6}');
    /// `no_meals_ouline` (U+F229).
    pub const NO_MEALS_OULINE: IconData = material('\u{f229}');
    /// `no_meeting_room` (U+EB4E).
    pub const NO_MEETING_ROOM: IconData = material('\u{eb4e}');
    /// `no_photography` (U+F1A8).
    pub const NO_PHOTOGRAPHY: IconData = material('\u{f1a8}');
    /// `no_sim` (U+E0CC).
    pub const NO_SIM: IconData = material('\u{e0cc}');
    /// `no_stroller` (U+F1AF).
    pub const NO_STROLLER: IconData = material('\u{f1af}');
    /// `no_transfer` (U+F1D5).
    pub const NO_TRANSFER: IconData = material('\u{f1d5}');
    /// `noise_aware` (U+EBEC).
    pub const NOISE_AWARE: IconData = material('\u{ebec}');
    /// `noise_control_off` (U+EBF3).
    pub const NOISE_CONTROL_OFF: IconData = material('\u{ebf3}');
    /// `nordic_walking` (U+E50E).
    pub const NORDIC_WALKING: IconData = material('\u{e50e}');
    /// `north` (U+F1E0).
    pub const NORTH: IconData = material('\u{f1e0}');
    /// `north_east` (U+F1E1).
    pub const NORTH_EAST: IconData = material('\u{f1e1}');
    /// `north_west` (U+F1E2).
    pub const NORTH_WEST: IconData = material('\u{f1e2}');
    /// `not_accessible` (U+F0FE).
    pub const NOT_ACCESSIBLE: IconData = material('\u{f0fe}');
    /// `not_interested` (U+E033).
    pub const NOT_INTERESTED: IconData = material('\u{e033}');
    /// `not_listed_location` (U+E575).
    pub const NOT_LISTED_LOCATION: IconData = material('\u{e575}');
    /// `not_started` (U+F0D1).
    pub const NOT_STARTED: IconData = material('\u{f0d1}');
    /// `note` (U+E06F).
    pub const NOTE: IconData = material('\u{e06f}');
    /// `note_add` (U+E89C).
    pub const NOTE_ADD: IconData = material('\u{e89c}');
    /// `note_alt` (U+F040).
    pub const NOTE_ALT: IconData = material('\u{f040}');
    /// `notes` (U+E26C).
    pub const NOTES: IconData = material('\u{e26c}');
    /// `notification_add` (U+E399).
    pub const NOTIFICATION_ADD: IconData = material('\u{e399}');
    /// `notification_important` (U+E004).
    pub const NOTIFICATION_IMPORTANT: IconData = material('\u{e004}');
    /// `notifications` (U+E7F4).
    pub const NOTIFICATIONS: IconData = material('\u{e7f4}');
    /// `notifications_active` (U+E7F7).
    pub const NOTIFICATIONS_ACTIVE: IconData = material('\u{e7f7}');
    /// `notifications_none` (U+E7F5).
    pub const NOTIFICATIONS_NONE: IconData = material('\u{e7f5}');
    /// `notifications_off` (U+E7F6).
    pub const NOTIFICATIONS_OFF: IconData = material('\u{e7f6}');
    /// `notifications_on` (U+E7F7).
    pub const NOTIFICATIONS_ON: IconData = material('\u{e7f7}');
    /// `notifications_paused` (U+E7F8).
    pub const NOTIFICATIONS_PAUSED: IconData = material('\u{e7f8}');
    /// `now_wallpaper` (U+E1BC).
    pub const NOW_WALLPAPER: IconData = material('\u{e1bc}');
    /// `now_widgets` (U+E1BD).
    pub const NOW_WIDGETS: IconData = material('\u{e1bd}');
    /// `numbers` (U+EAC7).
    pub const NUMBERS: IconData = material('\u{eac7}');
    /// `offline_bolt` (U+E932).
    pub const OFFLINE_BOLT: IconData = material('\u{e932}');
    /// `offline_pin` (U+E90A).
    pub const OFFLINE_PIN: IconData = material('\u{e90a}');
    /// `offline_share` (U+E9C5).
    pub const OFFLINE_SHARE: IconData = material('\u{e9c5}');
    /// `oil_barrel` (U+EC15).
    pub const OIL_BARREL: IconData = material('\u{ec15}');
    /// `on_device_training` (U+EBFD).
    pub const ON_DEVICE_TRAINING: IconData = material('\u{ebfd}');
    /// `ondemand_video` (U+E63A).
    pub const ONDEMAND_VIDEO: IconData = material('\u{e63a}');
    /// `online_prediction` (U+F0EB).
    pub const ONLINE_PREDICTION: IconData = material('\u{f0eb}');
    /// `opacity` (U+E91C).
    pub const OPACITY: IconData = material('\u{e91c}');
    /// `open_in_browser` (U+E89D).
    pub const OPEN_IN_BROWSER: IconData = material('\u{e89d}');
    /// `open_in_full` (U+F1CE).
    pub const OPEN_IN_FULL: IconData = material('\u{f1ce}');
    /// `open_in_new` (U+E89E).
    pub const OPEN_IN_NEW: IconData = material('\u{e89e}');
    /// `open_in_new_off` (U+E4F6).
    pub const OPEN_IN_NEW_OFF: IconData = material('\u{e4f6}');
    /// `open_with` (U+E89F).
    pub const OPEN_WITH: IconData = material('\u{e89f}');
    /// `other_houses` (U+E58C).
    pub const OTHER_HOUSES: IconData = material('\u{e58c}');
    /// `outbond` (U+F228).
    pub const OUTBOND: IconData = material('\u{f228}');
    /// `outbound` (U+E1CA).
    pub const OUTBOUND: IconData = material('\u{e1ca}');
    /// `outbox` (U+EF5F).
    pub const OUTBOX: IconData = material('\u{ef5f}');
    /// `outdoor_grill` (U+EA47).
    pub const OUTDOOR_GRILL: IconData = material('\u{ea47}');
    /// `outgoing_mail` (U+F0D2).
    pub const OUTGOING_MAIL: IconData = material('\u{f0d2}');
    /// `outlet` (U+F1D4).
    pub const OUTLET: IconData = material('\u{f1d4}');
    /// `outlined_flag` (U+E16E).
    pub const OUTLINED_FLAG: IconData = material('\u{e16e}');
    /// `output` (U+EBBE).
    pub const OUTPUT: IconData = material('\u{ebbe}');
    /// `padding` (U+E9C8).
    pub const PADDING: IconData = material('\u{e9c8}');
    /// `pages` (U+E7F9).
    pub const PAGES: IconData = material('\u{e7f9}');
    /// `pageview` (U+E8A0).
    pub const PAGEVIEW: IconData = material('\u{e8a0}');
    /// `paid` (U+F041).
    pub const PAID: IconData = material('\u{f041}');
    /// `palette` (U+E40A).
    pub const PALETTE: IconData = material('\u{e40a}');
    /// `pallet` (U+F86A).
    pub const PALLET: IconData = material('\u{f86a}');
    /// `pan_tool` (U+E925).
    pub const PAN_TOOL: IconData = material('\u{e925}');
    /// `pan_tool_alt` (U+EBB9).
    pub const PAN_TOOL_ALT: IconData = material('\u{ebb9}');
    /// `panorama` (U+E40B).
    pub const PANORAMA: IconData = material('\u{e40b}');
    /// `panorama_fish_eye` (U+E40C).
    pub const PANORAMA_FISH_EYE: IconData = material('\u{e40c}');
    /// `panorama_fisheye` (U+E40C).
    pub const PANORAMA_FISHEYE: IconData = material('\u{e40c}');
    /// `panorama_horizontal` (U+E40D).
    pub const PANORAMA_HORIZONTAL: IconData = material('\u{e40d}');
    /// `panorama_horizontal_select` (U+EF60).
    pub const PANORAMA_HORIZONTAL_SELECT: IconData = material('\u{ef60}');
    /// `panorama_photosphere` (U+E9C9).
    pub const PANORAMA_PHOTOSPHERE: IconData = material('\u{e9c9}');
    /// `panorama_photosphere_select` (U+E9CA).
    pub const PANORAMA_PHOTOSPHERE_SELECT: IconData = material('\u{e9ca}');
    /// `panorama_vertical` (U+E40E).
    pub const PANORAMA_VERTICAL: IconData = material('\u{e40e}');
    /// `panorama_vertical_select` (U+EF61).
    pub const PANORAMA_VERTICAL_SELECT: IconData = material('\u{ef61}');
    /// `panorama_wide_angle` (U+E40F).
    pub const PANORAMA_WIDE_ANGLE: IconData = material('\u{e40f}');
    /// `panorama_wide_angle_select` (U+EF62).
    pub const PANORAMA_WIDE_ANGLE_SELECT: IconData = material('\u{ef62}');
    /// `paragliding` (U+E50F).
    pub const PARAGLIDING: IconData = material('\u{e50f}');
    /// `park` (U+EA63).
    pub const PARK: IconData = material('\u{ea63}');
    /// `party_mode` (U+E7FA).
    pub const PARTY_MODE: IconData = material('\u{e7fa}');
    /// `password` (U+F042).
    pub const PASSWORD: IconData = material('\u{f042}');
    /// `pattern` (U+F043).
    pub const PATTERN: IconData = material('\u{f043}');
    /// `pause` (U+E034).
    pub const PAUSE: IconData = material('\u{e034}');
    /// `pause_circle` (U+E1A2).
    pub const PAUSE_CIRCLE: IconData = material('\u{e1a2}');
    /// `pause_circle_filled` (U+E035).
    pub const PAUSE_CIRCLE_FILLED: IconData = material('\u{e035}');
    /// `pause_circle_outline` (U+E036).
    pub const PAUSE_CIRCLE_OUTLINE: IconData = material('\u{e036}');
    /// `pause_presentation` (U+E0EA).
    pub const PAUSE_PRESENTATION: IconData = material('\u{e0ea}');
    /// `payment` (U+E8A1).
    pub const PAYMENT: IconData = material('\u{e8a1}');
    /// `payments` (U+EF63).
    pub const PAYMENTS: IconData = material('\u{ef63}');
    /// `paypal` (U+EA8D).
    pub const PAYPAL: IconData = material('\u{ea8d}');
    /// `pedal_bike` (U+EB29).
    pub const PEDAL_BIKE: IconData = material('\u{eb29}');
    /// `pending` (U+EF64).
    pub const PENDING: IconData = material('\u{ef64}');
    /// `pending_actions` (U+F1BB).
    pub const PENDING_ACTIONS: IconData = material('\u{f1bb}');
    /// `pentagon` (U+EB50).
    pub const PENTAGON: IconData = material('\u{eb50}');
    /// `people` (U+E7FB).
    pub const PEOPLE: IconData = material('\u{e7fb}');
    /// `people_alt` (U+EA21).
    pub const PEOPLE_ALT: IconData = material('\u{ea21}');
    /// `people_outline` (U+E7FC).
    pub const PEOPLE_OUTLINE: IconData = material('\u{e7fc}');
    /// `percent` (U+EB58).
    pub const PERCENT: IconData = material('\u{eb58}');
    /// `perm_camera_mic` (U+E8A2).
    pub const PERM_CAMERA_MIC: IconData = material('\u{e8a2}');
    /// `perm_contact_cal` (U+E8A3).
    pub const PERM_CONTACT_CAL: IconData = material('\u{e8a3}');
    /// `perm_contact_calendar` (U+E8A3).
    pub const PERM_CONTACT_CALENDAR: IconData = material('\u{e8a3}');
    /// `perm_data_setting` (U+E8A4).
    pub const PERM_DATA_SETTING: IconData = material('\u{e8a4}');
    /// `perm_device_info` (U+E8A5).
    pub const PERM_DEVICE_INFO: IconData = material('\u{e8a5}');
    /// `perm_device_information` (U+E8A5).
    pub const PERM_DEVICE_INFORMATION: IconData = material('\u{e8a5}');
    /// `perm_identity` (U+E8A6).
    pub const PERM_IDENTITY: IconData = material('\u{e8a6}');
    /// `perm_media` (U+E8A7).
    pub const PERM_MEDIA: IconData = material('\u{e8a7}');
    /// `perm_phone_msg` (U+E8A8).
    pub const PERM_PHONE_MSG: IconData = material('\u{e8a8}');
    /// `perm_scan_wifi` (U+E8A9).
    pub const PERM_SCAN_WIFI: IconData = material('\u{e8a9}');
    /// `person` (U+E7FD).
    pub const PERSON: IconData = material('\u{e7fd}');
    /// `person_2` (U+F8E4).
    pub const PERSON_2: IconData = material('\u{f8e4}');
    /// `person_3` (U+F8E5).
    pub const PERSON_3: IconData = material('\u{f8e5}');
    /// `person_4` (U+F8E6).
    pub const PERSON_4: IconData = material('\u{f8e6}');
    /// `person_add` (U+E7FE).
    pub const PERSON_ADD: IconData = material('\u{e7fe}');
    /// `person_add_alt` (U+EA4D).
    pub const PERSON_ADD_ALT: IconData = material('\u{ea4d}');
    /// `person_add_alt_1` (U+EF65).
    pub const PERSON_ADD_ALT_1: IconData = material('\u{ef65}');
    /// `person_add_disabled` (U+E9CB).
    pub const PERSON_ADD_DISABLED: IconData = material('\u{e9cb}');
    /// `person_off` (U+E510).
    pub const PERSON_OFF: IconData = material('\u{e510}');
    /// `person_outline` (U+E7FF).
    pub const PERSON_OUTLINE: IconData = material('\u{e7ff}');
    /// `person_pin` (U+E55A).
    pub const PERSON_PIN: IconData = material('\u{e55a}');
    /// `person_pin_circle` (U+E56A).
    pub const PERSON_PIN_CIRCLE: IconData = material('\u{e56a}');
    /// `person_remove` (U+EF66).
    pub const PERSON_REMOVE: IconData = material('\u{ef66}');
    /// `person_remove_alt_1` (U+EF67).
    pub const PERSON_REMOVE_ALT_1: IconData = material('\u{ef67}');
    /// `person_search` (U+F106).
    pub const PERSON_SEARCH: IconData = material('\u{f106}');
    /// `personal_injury` (U+E6DA).
    pub const PERSONAL_INJURY: IconData = material('\u{e6da}');
    /// `personal_video` (U+E63B).
    pub const PERSONAL_VIDEO: IconData = material('\u{e63b}');
    /// `pest_control` (U+F0FA).
    pub const PEST_CONTROL: IconData = material('\u{f0fa}');
    /// `pest_control_rodent` (U+F0FD).
    pub const PEST_CONTROL_RODENT: IconData = material('\u{f0fd}');
    /// `pets` (U+E91D).
    pub const PETS: IconData = material('\u{e91d}');
    /// `phishing` (U+EAD7).
    pub const PHISHING: IconData = material('\u{ead7}');
    /// `phone` (U+E0CD).
    pub const PHONE: IconData = material('\u{e0cd}');
    /// `phone_android` (U+E324).
    pub const PHONE_ANDROID: IconData = material('\u{e324}');
    /// `phone_bluetooth_speaker` (U+E61B).
    pub const PHONE_BLUETOOTH_SPEAKER: IconData = material('\u{e61b}');
    /// `phone_callback` (U+E649).
    pub const PHONE_CALLBACK: IconData = material('\u{e649}');
    /// `phone_disabled` (U+E9CC).
    pub const PHONE_DISABLED: IconData = material('\u{e9cc}');
    /// `phone_enabled` (U+E9CD).
    pub const PHONE_ENABLED: IconData = material('\u{e9cd}');
    /// `phone_forwarded` (U+E61C).
    pub const PHONE_FORWARDED: IconData = material('\u{e61c}');
    /// `phone_in_talk` (U+E61D).
    pub const PHONE_IN_TALK: IconData = material('\u{e61d}');
    /// `phone_iphone` (U+E325).
    pub const PHONE_IPHONE: IconData = material('\u{e325}');
    /// `phone_locked` (U+E61E).
    pub const PHONE_LOCKED: IconData = material('\u{e61e}');
    /// `phone_missed` (U+E61F).
    pub const PHONE_MISSED: IconData = material('\u{e61f}');
    /// `phone_paused` (U+E620).
    pub const PHONE_PAUSED: IconData = material('\u{e620}');
    /// `phonelink` (U+E326).
    pub const PHONELINK: IconData = material('\u{e326}');
    /// `phonelink_erase` (U+E0DB).
    pub const PHONELINK_ERASE: IconData = material('\u{e0db}');
    /// `phonelink_lock` (U+E0DC).
    pub const PHONELINK_LOCK: IconData = material('\u{e0dc}');
    /// `phonelink_off` (U+E327).
    pub const PHONELINK_OFF: IconData = material('\u{e327}');
    /// `phonelink_ring` (U+E0DD).
    pub const PHONELINK_RING: IconData = material('\u{e0dd}');
    /// `phonelink_setup` (U+E0DE).
    pub const PHONELINK_SETUP: IconData = material('\u{e0de}');
    /// `photo` (U+E410).
    pub const PHOTO: IconData = material('\u{e410}');
    /// `photo_album` (U+E411).
    pub const PHOTO_ALBUM: IconData = material('\u{e411}');
    /// `photo_camera` (U+E412).
    pub const PHOTO_CAMERA: IconData = material('\u{e412}');
    /// `photo_camera_back` (U+EF68).
    pub const PHOTO_CAMERA_BACK: IconData = material('\u{ef68}');
    /// `photo_camera_front` (U+EF69).
    pub const PHOTO_CAMERA_FRONT: IconData = material('\u{ef69}');
    /// `photo_filter` (U+E43B).
    pub const PHOTO_FILTER: IconData = material('\u{e43b}');
    /// `photo_library` (U+E413).
    pub const PHOTO_LIBRARY: IconData = material('\u{e413}');
    /// `photo_size_select_actual` (U+E432).
    pub const PHOTO_SIZE_SELECT_ACTUAL: IconData = material('\u{e432}');
    /// `photo_size_select_large` (U+E433).
    pub const PHOTO_SIZE_SELECT_LARGE: IconData = material('\u{e433}');
    /// `photo_size_select_small` (U+E434).
    pub const PHOTO_SIZE_SELECT_SMALL: IconData = material('\u{e434}');
    /// `php` (U+EB8F).
    pub const PHP: IconData = material('\u{eb8f}');
    /// `piano` (U+E521).
    pub const PIANO: IconData = material('\u{e521}');
    /// `piano_off` (U+E520).
    pub const PIANO_OFF: IconData = material('\u{e520}');
    /// `picture_as_pdf` (U+E415).
    pub const PICTURE_AS_PDF: IconData = material('\u{e415}');
    /// `picture_in_picture` (U+E8AA).
    pub const PICTURE_IN_PICTURE: IconData = material('\u{e8aa}');
    /// `picture_in_picture_alt` (U+E911).
    pub const PICTURE_IN_PICTURE_ALT: IconData = material('\u{e911}');
    /// `pie_chart` (U+E6C4).
    pub const PIE_CHART: IconData = material('\u{e6c4}');
    /// `pie_chart_outline` (U+F044).
    pub const PIE_CHART_OUTLINE: IconData = material('\u{f044}');
    /// `pie_chart_outlined` (U+E6C5).
    pub const PIE_CHART_OUTLINED: IconData = material('\u{e6c5}');
    /// `pin` (U+F045).
    pub const PIN: IconData = material('\u{f045}');
    /// `pin_drop` (U+E55E).
    pub const PIN_DROP: IconData = material('\u{e55e}');
    /// `pin_end` (U+E767).
    pub const PIN_END: IconData = material('\u{e767}');
    /// `pin_invoke` (U+E763).
    pub const PIN_INVOKE: IconData = material('\u{e763}');
    /// `pinch` (U+EB38).
    pub const PINCH: IconData = material('\u{eb38}');
    /// `pivot_table_chart` (U+E9CE).
    pub const PIVOT_TABLE_CHART: IconData = material('\u{e9ce}');
    /// `pix` (U+EAA3).
    pub const PIX: IconData = material('\u{eaa3}');
    /// `place` (U+E55F).
    pub const PLACE: IconData = material('\u{e55f}');
    /// `plagiarism` (U+EA5A).
    pub const PLAGIARISM: IconData = material('\u{ea5a}');
    /// `play_arrow` (U+E037).
    pub const PLAY_ARROW: IconData = material('\u{e037}');
    /// `play_circle` (U+E1C4).
    pub const PLAY_CIRCLE: IconData = material('\u{e1c4}');
    /// `play_circle_fill` (U+E038).
    pub const PLAY_CIRCLE_FILL: IconData = material('\u{e038}');
    /// `play_circle_filled` (U+E038).
    pub const PLAY_CIRCLE_FILLED: IconData = material('\u{e038}');
    /// `play_circle_outline` (U+E039).
    pub const PLAY_CIRCLE_OUTLINE: IconData = material('\u{e039}');
    /// `play_disabled` (U+EF6A).
    pub const PLAY_DISABLED: IconData = material('\u{ef6a}');
    /// `play_for_work` (U+E906).
    pub const PLAY_FOR_WORK: IconData = material('\u{e906}');
    /// `play_lesson` (U+F047).
    pub const PLAY_LESSON: IconData = material('\u{f047}');
    /// `playlist_add` (U+E03B).
    pub const PLAYLIST_ADD: IconData = material('\u{e03b}');
    /// `playlist_add_check` (U+E065).
    pub const PLAYLIST_ADD_CHECK: IconData = material('\u{e065}');
    /// `playlist_add_check_circle` (U+E7E6).
    pub const PLAYLIST_ADD_CHECK_CIRCLE: IconData = material('\u{e7e6}');
    /// `playlist_add_circle` (U+E7E5).
    pub const PLAYLIST_ADD_CIRCLE: IconData = material('\u{e7e5}');
    /// `playlist_play` (U+E05F).
    pub const PLAYLIST_PLAY: IconData = material('\u{e05f}');
    /// `playlist_remove` (U+EB80).
    pub const PLAYLIST_REMOVE: IconData = material('\u{eb80}');
    /// `plumbing` (U+F107).
    pub const PLUMBING: IconData = material('\u{f107}');
    /// `plus_one` (U+E800).
    pub const PLUS_ONE: IconData = material('\u{e800}');
    /// `podcasts` (U+F048).
    pub const PODCASTS: IconData = material('\u{f048}');
    /// `point_of_sale` (U+F17E).
    pub const POINT_OF_SALE: IconData = material('\u{f17e}');
    /// `policy` (U+EA17).
    pub const POLICY: IconData = material('\u{ea17}');
    /// `poll` (U+E801).
    pub const POLL: IconData = material('\u{e801}');
    /// `polyline` (U+EBBB).
    pub const POLYLINE: IconData = material('\u{ebbb}');
    /// `polymer` (U+E8AB).
    pub const POLYMER: IconData = material('\u{e8ab}');
    /// `pool` (U+EB48).
    pub const POOL: IconData = material('\u{eb48}');
    /// `portable_wifi_off` (U+E0CE).
    pub const PORTABLE_WIFI_OFF: IconData = material('\u{e0ce}');
    /// `portrait` (U+E416).
    pub const PORTRAIT: IconData = material('\u{e416}');
    /// `post_add` (U+EA20).
    pub const POST_ADD: IconData = material('\u{ea20}');
    /// `power` (U+E63C).
    pub const POWER: IconData = material('\u{e63c}');
    /// `power_input` (U+E336).
    pub const POWER_INPUT: IconData = material('\u{e336}');
    /// `power_off` (U+E646).
    pub const POWER_OFF: IconData = material('\u{e646}');
    /// `power_settings_new` (U+E8AC).
    pub const POWER_SETTINGS_NEW: IconData = material('\u{e8ac}');
    /// `precision_manufacturing` (U+F049).
    pub const PRECISION_MANUFACTURING: IconData = material('\u{f049}');
    /// `pregnant_woman` (U+E91E).
    pub const PREGNANT_WOMAN: IconData = material('\u{e91e}');
    /// `present_to_all` (U+E0DF).
    pub const PRESENT_TO_ALL: IconData = material('\u{e0df}');
    /// `preview` (U+F1C5).
    pub const PREVIEW: IconData = material('\u{f1c5}');
    /// `price_change` (U+F04A).
    pub const PRICE_CHANGE: IconData = material('\u{f04a}');
    /// `price_check` (U+F04B).
    pub const PRICE_CHECK: IconData = material('\u{f04b}');
    /// `print` (U+E8AD).
    pub const PRINT: IconData = material('\u{e8ad}');
    /// `print_disabled` (U+E9CF).
    pub const PRINT_DISABLED: IconData = material('\u{e9cf}');
    /// `priority_high` (U+E645).
    pub const PRIORITY_HIGH: IconData = material('\u{e645}');
    /// `privacy_tip` (U+F0DC).
    pub const PRIVACY_TIP: IconData = material('\u{f0dc}');
    /// `private_connectivity` (U+E744).
    pub const PRIVATE_CONNECTIVITY: IconData = material('\u{e744}');
    /// `production_quantity_limits` (U+E1D1).
    pub const PRODUCTION_QUANTITY_LIMITS: IconData = material('\u{e1d1}');
    /// `propane` (U+EC14).
    pub const PROPANE: IconData = material('\u{ec14}');
    /// `propane_tank` (U+EC13).
    pub const PROPANE_TANK: IconData = material('\u{ec13}');
    /// `psychology` (U+EA4A).
    pub const PSYCHOLOGY: IconData = material('\u{ea4a}');
    /// `psychology_alt` (U+F8EA).
    pub const PSYCHOLOGY_ALT: IconData = material('\u{f8ea}');
    /// `public` (U+E80B).
    pub const PUBLIC: IconData = material('\u{e80b}');
    /// `public_off` (U+F1CA).
    pub const PUBLIC_OFF: IconData = material('\u{f1ca}');
    /// `publish` (U+E255).
    pub const PUBLISH: IconData = material('\u{e255}');
    /// `published_with_changes` (U+F232).
    pub const PUBLISHED_WITH_CHANGES: IconData = material('\u{f232}');
    /// `punch_clock` (U+EAA8).
    pub const PUNCH_CLOCK: IconData = material('\u{eaa8}');
    /// `push_pin` (U+F10D).
    pub const PUSH_PIN: IconData = material('\u{f10d}');
    /// `qr_code` (U+EF6B).
    pub const QR_CODE: IconData = material('\u{ef6b}');
    /// `qr_code_2` (U+E00A).
    pub const QR_CODE_2: IconData = material('\u{e00a}');
    /// `qr_code_scanner` (U+F206).
    pub const QR_CODE_SCANNER: IconData = material('\u{f206}');
    /// `query_builder` (U+E8AE).
    pub const QUERY_BUILDER: IconData = material('\u{e8ae}');
    /// `query_stats` (U+E4FC).
    pub const QUERY_STATS: IconData = material('\u{e4fc}');
    /// `question_answer` (U+E8AF).
    pub const QUESTION_ANSWER: IconData = material('\u{e8af}');
    /// `question_mark` (U+EB8B).
    pub const QUESTION_MARK: IconData = material('\u{eb8b}');
    /// `queue` (U+E03C).
    pub const QUEUE: IconData = material('\u{e03c}');
    /// `queue_music` (U+E03D).
    pub const QUEUE_MUSIC: IconData = material('\u{e03d}');
    /// `queue_play_next` (U+E066).
    pub const QUEUE_PLAY_NEXT: IconData = material('\u{e066}');
    /// `quick_contacts_dialer` (U+E0CF).
    pub const QUICK_CONTACTS_DIALER: IconData = material('\u{e0cf}');
    /// `quick_contacts_mail` (U+E0D0).
    pub const QUICK_CONTACTS_MAIL: IconData = material('\u{e0d0}');
    /// `quickreply` (U+EF6C).
    pub const QUICKREPLY: IconData = material('\u{ef6c}');
    /// `quiz` (U+F04C).
    pub const QUIZ: IconData = material('\u{f04c}');
    /// `quora` (U+EA98).
    pub const QUORA: IconData = material('\u{ea98}');
    /// `r_mobiledata` (U+F04D).
    pub const R_MOBILEDATA: IconData = material('\u{f04d}');
    /// `radar` (U+F04E).
    pub const RADAR: IconData = material('\u{f04e}');
    /// `radio` (U+E03E).
    pub const RADIO: IconData = material('\u{e03e}');
    /// `radio_button_checked` (U+E837).
    pub const RADIO_BUTTON_CHECKED: IconData = material('\u{e837}');
    /// `radio_button_off` (U+E836).
    pub const RADIO_BUTTON_OFF: IconData = material('\u{e836}');
    /// `radio_button_on` (U+E837).
    pub const RADIO_BUTTON_ON: IconData = material('\u{e837}');
    /// `radio_button_unchecked` (U+E836).
    pub const RADIO_BUTTON_UNCHECKED: IconData = material('\u{e836}');
    /// `railway_alert` (U+E9D1).
    pub const RAILWAY_ALERT: IconData = material('\u{e9d1}');
    /// `ramen_dining` (U+EA64).
    pub const RAMEN_DINING: IconData = material('\u{ea64}');
    /// `ramp_left` (U+EB9C).
    pub const RAMP_LEFT: IconData = material('\u{eb9c}');
    /// `ramp_right` (U+EB96).
    pub const RAMP_RIGHT: IconData = material('\u{eb96}');
    /// `rate_review` (U+E560).
    pub const RATE_REVIEW: IconData = material('\u{e560}');
    /// `raw_off` (U+F04F).
    pub const RAW_OFF: IconData = material('\u{f04f}');
    /// `raw_on` (U+F050).
    pub const RAW_ON: IconData = material('\u{f050}');
    /// `read_more` (U+EF6D).
    pub const READ_MORE: IconData = material('\u{ef6d}');
    /// `real_estate_agent` (U+E73A).
    pub const REAL_ESTATE_AGENT: IconData = material('\u{e73a}');
    /// `rebase_edit` (U+F846).
    pub const REBASE_EDIT: IconData = material('\u{f846}');
    /// `receipt` (U+E8B0).
    pub const RECEIPT: IconData = material('\u{e8b0}');
    /// `receipt_long` (U+EF6E).
    pub const RECEIPT_LONG: IconData = material('\u{ef6e}');
    /// `recent_actors` (U+E03F).
    pub const RECENT_ACTORS: IconData = material('\u{e03f}');
    /// `recommend` (U+E9D2).
    pub const RECOMMEND: IconData = material('\u{e9d2}');
    /// `record_voice_over` (U+E91F).
    pub const RECORD_VOICE_OVER: IconData = material('\u{e91f}');
    /// `rectangle` (U+EB54).
    pub const RECTANGLE: IconData = material('\u{eb54}');
    /// `recycling` (U+E760).
    pub const RECYCLING: IconData = material('\u{e760}');
    /// `reddit` (U+EAA0).
    pub const REDDIT: IconData = material('\u{eaa0}');
    /// `redeem` (U+E8B1).
    pub const REDEEM: IconData = material('\u{e8b1}');
    /// `redo` (U+E15A).
    pub const REDO: IconData = material('\u{e15a}');
    /// `reduce_capacity` (U+F21C).
    pub const REDUCE_CAPACITY: IconData = material('\u{f21c}');
    /// `refresh` (U+E5D5).
    pub const REFRESH: IconData = material('\u{e5d5}');
    /// `remember_me` (U+F051).
    pub const REMEMBER_ME: IconData = material('\u{f051}');
    /// `remove` (U+E15B).
    pub const REMOVE: IconData = material('\u{e15b}');
    /// `remove_circle` (U+E15C).
    pub const REMOVE_CIRCLE: IconData = material('\u{e15c}');
    /// `remove_circle_outline` (U+E15D).
    pub const REMOVE_CIRCLE_OUTLINE: IconData = material('\u{e15d}');
    /// `remove_done` (U+E9D3).
    pub const REMOVE_DONE: IconData = material('\u{e9d3}');
    /// `remove_from_queue` (U+E067).
    pub const REMOVE_FROM_QUEUE: IconData = material('\u{e067}');
    /// `remove_moderator` (U+E9D4).
    pub const REMOVE_MODERATOR: IconData = material('\u{e9d4}');
    /// `remove_red_eye` (U+E417).
    pub const REMOVE_RED_EYE: IconData = material('\u{e417}');
    /// `remove_road` (U+EBFC).
    pub const REMOVE_ROAD: IconData = material('\u{ebfc}');
    /// `remove_shopping_cart` (U+E928).
    pub const REMOVE_SHOPPING_CART: IconData = material('\u{e928}');
    /// `reorder` (U+E8FE).
    pub const REORDER: IconData = material('\u{e8fe}');
    /// `repartition` (U+F8E8).
    pub const REPARTITION: IconData = material('\u{f8e8}');
    /// `repeat` (U+E040).
    pub const REPEAT: IconData = material('\u{e040}');
    /// `repeat_on` (U+E9D6).
    pub const REPEAT_ON: IconData = material('\u{e9d6}');
    /// `repeat_one` (U+E041).
    pub const REPEAT_ONE: IconData = material('\u{e041}');
    /// `repeat_one_on` (U+E9D7).
    pub const REPEAT_ONE_ON: IconData = material('\u{e9d7}');
    /// `replay` (U+E042).
    pub const REPLAY: IconData = material('\u{e042}');
    /// `replay_10` (U+E059).
    pub const REPLAY_10: IconData = material('\u{e059}');
    /// `replay_30` (U+E05A).
    pub const REPLAY_30: IconData = material('\u{e05a}');
    /// `replay_5` (U+E05B).
    pub const REPLAY_5: IconData = material('\u{e05b}');
    /// `replay_circle_filled` (U+E9D8).
    pub const REPLAY_CIRCLE_FILLED: IconData = material('\u{e9d8}');
    /// `reply` (U+E15E).
    pub const REPLY: IconData = material('\u{e15e}');
    /// `reply_all` (U+E15F).
    pub const REPLY_ALL: IconData = material('\u{e15f}');
    /// `report` (U+E160).
    pub const REPORT: IconData = material('\u{e160}');
    /// `report_gmailerrorred` (U+F052).
    pub const REPORT_GMAILERRORRED: IconData = material('\u{f052}');
    /// `report_off` (U+E170).
    pub const REPORT_OFF: IconData = material('\u{e170}');
    /// `report_problem` (U+E8B2).
    pub const REPORT_PROBLEM: IconData = material('\u{e8b2}');
    /// `request_page` (U+F22C).
    pub const REQUEST_PAGE: IconData = material('\u{f22c}');
    /// `request_quote` (U+F1B6).
    pub const REQUEST_QUOTE: IconData = material('\u{f1b6}');
    /// `reset_tv` (U+E9D9).
    pub const RESET_TV: IconData = material('\u{e9d9}');
    /// `restart_alt` (U+F053).
    pub const RESTART_ALT: IconData = material('\u{f053}');
    /// `restaurant` (U+E56C).
    pub const RESTAURANT: IconData = material('\u{e56c}');
    /// `restaurant_menu` (U+E561).
    pub const RESTAURANT_MENU: IconData = material('\u{e561}');
    /// `restore` (U+E8B3).
    pub const RESTORE: IconData = material('\u{e8b3}');
    /// `restore_from_trash` (U+E938).
    pub const RESTORE_FROM_TRASH: IconData = material('\u{e938}');
    /// `restore_page` (U+E929).
    pub const RESTORE_PAGE: IconData = material('\u{e929}');
    /// `reviews` (U+F054).
    pub const REVIEWS: IconData = material('\u{f054}');
    /// `rice_bowl` (U+F1F5).
    pub const RICE_BOWL: IconData = material('\u{f1f5}');
    /// `ring_volume` (U+E0D1).
    pub const RING_VOLUME: IconData = material('\u{e0d1}');
    /// `rocket` (U+EBA5).
    pub const ROCKET: IconData = material('\u{eba5}');
    /// `rocket_launch` (U+EB9B).
    pub const ROCKET_LAUNCH: IconData = material('\u{eb9b}');
    /// `roller_shades` (U+EC12).
    pub const ROLLER_SHADES: IconData = material('\u{ec12}');
    /// `roller_shades_closed` (U+EC11).
    pub const ROLLER_SHADES_CLOSED: IconData = material('\u{ec11}');
    /// `roller_skating` (U+EBCD).
    pub const ROLLER_SKATING: IconData = material('\u{ebcd}');
    /// `roofing` (U+F201).
    pub const ROOFING: IconData = material('\u{f201}');
    /// `room` (U+E8B4).
    pub const ROOM: IconData = material('\u{e8b4}');
    /// `room_preferences` (U+F1B8).
    pub const ROOM_PREFERENCES: IconData = material('\u{f1b8}');
    /// `room_service` (U+EB49).
    pub const ROOM_SERVICE: IconData = material('\u{eb49}');
    /// `rotate_90_degrees_ccw` (U+E418).
    pub const ROTATE_90_DEGREES_CCW: IconData = material('\u{e418}');
    /// `rotate_90_degrees_cw` (U+EAAB).
    pub const ROTATE_90_DEGREES_CW: IconData = material('\u{eaab}');
    /// `rotate_left` (U+E419).
    pub const ROTATE_LEFT: IconData = material('\u{e419}');
    /// `rotate_right` (U+E41A).
    pub const ROTATE_RIGHT: IconData = material('\u{e41a}');
    /// `roundabout_left` (U+EB99).
    pub const ROUNDABOUT_LEFT: IconData = material('\u{eb99}');
    /// `roundabout_right` (U+EBA3).
    pub const ROUNDABOUT_RIGHT: IconData = material('\u{eba3}');
    /// `rounded_corner` (U+E920).
    pub const ROUNDED_CORNER: IconData = material('\u{e920}');
    /// `route` (U+EACD).
    pub const ROUTE: IconData = material('\u{eacd}');
    /// `router` (U+E328).
    pub const ROUTER: IconData = material('\u{e328}');
    /// `rowing` (U+E921).
    pub const ROWING: IconData = material('\u{e921}');
    /// `rss_feed` (U+E0E5).
    pub const RSS_FEED: IconData = material('\u{e0e5}');
    /// `rsvp` (U+F055).
    pub const RSVP: IconData = material('\u{f055}');
    /// `rtt` (U+E9AD).
    pub const RTT: IconData = material('\u{e9ad}');
    /// `rule` (U+F1C2).
    pub const RULE: IconData = material('\u{f1c2}');
    /// `rule_folder` (U+F1C9).
    pub const RULE_FOLDER: IconData = material('\u{f1c9}');
    /// `run_circle` (U+EF6F).
    pub const RUN_CIRCLE: IconData = material('\u{ef6f}');
    /// `running_with_errors` (U+E51D).
    pub const RUNNING_WITH_ERRORS: IconData = material('\u{e51d}');
    /// `rv_hookup` (U+E642).
    pub const RV_HOOKUP: IconData = material('\u{e642}');
    /// `safety_check` (U+EBEF).
    pub const SAFETY_CHECK: IconData = material('\u{ebef}');
    /// `safety_divider` (U+E1CC).
    pub const SAFETY_DIVIDER: IconData = material('\u{e1cc}');
    /// `sailing` (U+E502).
    pub const SAILING: IconData = material('\u{e502}');
    /// `sanitizer` (U+F21D).
    pub const SANITIZER: IconData = material('\u{f21d}');
    /// `satellite` (U+E562).
    pub const SATELLITE: IconData = material('\u{e562}');
    /// `satellite_alt` (U+EB3A).
    pub const SATELLITE_ALT: IconData = material('\u{eb3a}');
    /// `save` (U+E161).
    pub const SAVE: IconData = material('\u{e161}');
    /// `save_alt` (U+E171).
    pub const SAVE_ALT: IconData = material('\u{e171}');
    /// `save_as` (U+EB60).
    pub const SAVE_AS: IconData = material('\u{eb60}');
    /// `saved_search` (U+EA11).
    pub const SAVED_SEARCH: IconData = material('\u{ea11}');
    /// `savings` (U+E2EB).
    pub const SAVINGS: IconData = material('\u{e2eb}');
    /// `scale` (U+EB5F).
    pub const SCALE: IconData = material('\u{eb5f}');
    /// `scanner` (U+E329).
    pub const SCANNER: IconData = material('\u{e329}');
    /// `scatter_plot` (U+E268).
    pub const SCATTER_PLOT: IconData = material('\u{e268}');
    /// `schedule` (U+E8B5).
    pub const SCHEDULE: IconData = material('\u{e8b5}');
    /// `schedule_send` (U+EA0A).
    pub const SCHEDULE_SEND: IconData = material('\u{ea0a}');
    /// `schema` (U+E4FD).
    pub const SCHEMA: IconData = material('\u{e4fd}');
    /// `school` (U+E80C).
    pub const SCHOOL: IconData = material('\u{e80c}');
    /// `science` (U+EA4B).
    pub const SCIENCE: IconData = material('\u{ea4b}');
    /// `score` (U+E269).
    pub const SCORE: IconData = material('\u{e269}');
    /// `scoreboard` (U+EBD0).
    pub const SCOREBOARD: IconData = material('\u{ebd0}');
    /// `screen_lock_landscape` (U+E1BE).
    pub const SCREEN_LOCK_LANDSCAPE: IconData = material('\u{e1be}');
    /// `screen_lock_portrait` (U+E1BF).
    pub const SCREEN_LOCK_PORTRAIT: IconData = material('\u{e1bf}');
    /// `screen_lock_rotation` (U+E1C0).
    pub const SCREEN_LOCK_ROTATION: IconData = material('\u{e1c0}');
    /// `screen_rotation` (U+E1C1).
    pub const SCREEN_ROTATION: IconData = material('\u{e1c1}');
    /// `screen_rotation_alt` (U+EBEE).
    pub const SCREEN_ROTATION_ALT: IconData = material('\u{ebee}');
    /// `screen_search_desktop` (U+EF70).
    pub const SCREEN_SEARCH_DESKTOP: IconData = material('\u{ef70}');
    /// `screen_share` (U+E0E2).
    pub const SCREEN_SHARE: IconData = material('\u{e0e2}');
    /// `screenshot` (U+F056).
    pub const SCREENSHOT: IconData = material('\u{f056}');
    /// `screenshot_monitor` (U+EC08).
    pub const SCREENSHOT_MONITOR: IconData = material('\u{ec08}');
    /// `scuba_diving` (U+EBCE).
    pub const SCUBA_DIVING: IconData = material('\u{ebce}');
    /// `sd` (U+E9DD).
    pub const SD: IconData = material('\u{e9dd}');
    /// `sd_card` (U+E623).
    pub const SD_CARD: IconData = material('\u{e623}');
    /// `sd_card_alert` (U+F057).
    pub const SD_CARD_ALERT: IconData = material('\u{f057}');
    /// `sd_storage` (U+E1C2).
    pub const SD_STORAGE: IconData = material('\u{e1c2}');
    /// `search` (U+E8B6).
    pub const SEARCH: IconData = material('\u{e8b6}');
    /// `search_off` (U+EA76).
    pub const SEARCH_OFF: IconData = material('\u{ea76}');
    /// `security` (U+E32A).
    pub const SECURITY: IconData = material('\u{e32a}');
    /// `security_update` (U+F058).
    pub const SECURITY_UPDATE: IconData = material('\u{f058}');
    /// `security_update_good` (U+F059).
    pub const SECURITY_UPDATE_GOOD: IconData = material('\u{f059}');
    /// `security_update_warning` (U+F05A).
    pub const SECURITY_UPDATE_WARNING: IconData = material('\u{f05a}');
    /// `segment` (U+E94B).
    pub const SEGMENT: IconData = material('\u{e94b}');
    /// `select_all` (U+E162).
    pub const SELECT_ALL: IconData = material('\u{e162}');
    /// `self_improvement` (U+EA78).
    pub const SELF_IMPROVEMENT: IconData = material('\u{ea78}');
    /// `sell` (U+F05B).
    pub const SELL: IconData = material('\u{f05b}');
    /// `send` (U+E163).
    pub const SEND: IconData = material('\u{e163}');
    /// `send_and_archive` (U+EA0C).
    pub const SEND_AND_ARCHIVE: IconData = material('\u{ea0c}');
    /// `send_time_extension` (U+EADB).
    pub const SEND_TIME_EXTENSION: IconData = material('\u{eadb}');
    /// `send_to_mobile` (U+F05C).
    pub const SEND_TO_MOBILE: IconData = material('\u{f05c}');
    /// `sensor_door` (U+F1B5).
    pub const SENSOR_DOOR: IconData = material('\u{f1b5}');
    /// `sensor_occupied` (U+EC10).
    pub const SENSOR_OCCUPIED: IconData = material('\u{ec10}');
    /// `sensor_window` (U+F1B4).
    pub const SENSOR_WINDOW: IconData = material('\u{f1b4}');
    /// `sensors` (U+E51E).
    pub const SENSORS: IconData = material('\u{e51e}');
    /// `sensors_off` (U+E51F).
    pub const SENSORS_OFF: IconData = material('\u{e51f}');
    /// `sentiment_dissatisfied` (U+E811).
    pub const SENTIMENT_DISSATISFIED: IconData = material('\u{e811}');
    /// `sentiment_neutral` (U+E812).
    pub const SENTIMENT_NEUTRAL: IconData = material('\u{e812}');
    /// `sentiment_satisfied` (U+E813).
    pub const SENTIMENT_SATISFIED: IconData = material('\u{e813}');
    /// `sentiment_satisfied_alt` (U+E0ED).
    pub const SENTIMENT_SATISFIED_ALT: IconData = material('\u{e0ed}');
    /// `sentiment_very_dissatisfied` (U+E814).
    pub const SENTIMENT_VERY_DISSATISFIED: IconData = material('\u{e814}');
    /// `sentiment_very_satisfied` (U+E815).
    pub const SENTIMENT_VERY_SATISFIED: IconData = material('\u{e815}');
    /// `set_meal` (U+F1EA).
    pub const SET_MEAL: IconData = material('\u{f1ea}');
    /// `settings` (U+E8B8).
    pub const SETTINGS: IconData = material('\u{e8b8}');
    /// `settings_accessibility` (U+F05D).
    pub const SETTINGS_ACCESSIBILITY: IconData = material('\u{f05d}');
    /// `settings_applications` (U+E8B9).
    pub const SETTINGS_APPLICATIONS: IconData = material('\u{e8b9}');
    /// `settings_backup_restore` (U+E8BA).
    pub const SETTINGS_BACKUP_RESTORE: IconData = material('\u{e8ba}');
    /// `settings_bluetooth` (U+E8BB).
    pub const SETTINGS_BLUETOOTH: IconData = material('\u{e8bb}');
    /// `settings_brightness` (U+E8BD).
    pub const SETTINGS_BRIGHTNESS: IconData = material('\u{e8bd}');
    /// `settings_cell` (U+E8BC).
    pub const SETTINGS_CELL: IconData = material('\u{e8bc}');
    /// `settings_display` (U+E8BD).
    pub const SETTINGS_DISPLAY: IconData = material('\u{e8bd}');
    /// `settings_ethernet` (U+E8BE).
    pub const SETTINGS_ETHERNET: IconData = material('\u{e8be}');
    /// `settings_input_antenna` (U+E8BF).
    pub const SETTINGS_INPUT_ANTENNA: IconData = material('\u{e8bf}');
    /// `settings_input_component` (U+E8C0).
    pub const SETTINGS_INPUT_COMPONENT: IconData = material('\u{e8c0}');
    /// `settings_input_composite` (U+E8C1).
    pub const SETTINGS_INPUT_COMPOSITE: IconData = material('\u{e8c1}');
    /// `settings_input_hdmi` (U+E8C2).
    pub const SETTINGS_INPUT_HDMI: IconData = material('\u{e8c2}');
    /// `settings_input_svideo` (U+E8C3).
    pub const SETTINGS_INPUT_SVIDEO: IconData = material('\u{e8c3}');
    /// `settings_overscan` (U+E8C4).
    pub const SETTINGS_OVERSCAN: IconData = material('\u{e8c4}');
    /// `settings_phone` (U+E8C5).
    pub const SETTINGS_PHONE: IconData = material('\u{e8c5}');
    /// `settings_power` (U+E8C6).
    pub const SETTINGS_POWER: IconData = material('\u{e8c6}');
    /// `settings_remote` (U+E8C7).
    pub const SETTINGS_REMOTE: IconData = material('\u{e8c7}');
    /// `settings_suggest` (U+F05E).
    pub const SETTINGS_SUGGEST: IconData = material('\u{f05e}');
    /// `settings_system_daydream` (U+E1C3).
    pub const SETTINGS_SYSTEM_DAYDREAM: IconData = material('\u{e1c3}');
    /// `settings_voice` (U+E8C8).
    pub const SETTINGS_VOICE: IconData = material('\u{e8c8}');
    /// `severe_cold` (U+EBD3).
    pub const SEVERE_COLD: IconData = material('\u{ebd3}');
    /// `shape_line` (U+F8D3).
    pub const SHAPE_LINE: IconData = material('\u{f8d3}');
    /// `share` (U+E80D).
    pub const SHARE: IconData = material('\u{e80d}');
    /// `share_arrival_time` (U+E524).
    pub const SHARE_ARRIVAL_TIME: IconData = material('\u{e524}');
    /// `share_location` (U+F05F).
    pub const SHARE_LOCATION: IconData = material('\u{f05f}');
    /// `shelves` (U+F86E).
    pub const SHELVES: IconData = material('\u{f86e}');
    /// `shield` (U+E9E0).
    pub const SHIELD: IconData = material('\u{e9e0}');
    /// `shield_moon` (U+EAA9).
    pub const SHIELD_MOON: IconData = material('\u{eaa9}');
    /// `shop` (U+E8C9).
    pub const SHOP: IconData = material('\u{e8c9}');
    /// `shop_2` (U+E19E).
    pub const SHOP_2: IconData = material('\u{e19e}');
    /// `shop_two` (U+E8CA).
    pub const SHOP_TWO: IconData = material('\u{e8ca}');
    /// `shopify` (U+EA9D).
    pub const SHOPIFY: IconData = material('\u{ea9d}');
    /// `shopping_bag` (U+F1CC).
    pub const SHOPPING_BAG: IconData = material('\u{f1cc}');
    /// `shopping_basket` (U+E8CB).
    pub const SHOPPING_BASKET: IconData = material('\u{e8cb}');
    /// `shopping_cart` (U+E8CC).
    pub const SHOPPING_CART: IconData = material('\u{e8cc}');
    /// `shopping_cart_checkout` (U+EB88).
    pub const SHOPPING_CART_CHECKOUT: IconData = material('\u{eb88}');
    /// `short_text` (U+E261).
    pub const SHORT_TEXT: IconData = material('\u{e261}');
    /// `shortcut` (U+F060).
    pub const SHORTCUT: IconData = material('\u{f060}');
    /// `show_chart` (U+E6E1).
    pub const SHOW_CHART: IconData = material('\u{e6e1}');
    /// `shower` (U+F061).
    pub const SHOWER: IconData = material('\u{f061}');
    /// `shuffle` (U+E043).
    pub const SHUFFLE: IconData = material('\u{e043}');
    /// `shuffle_on` (U+E9E1).
    pub const SHUFFLE_ON: IconData = material('\u{e9e1}');
    /// `shutter_speed` (U+E43D).
    pub const SHUTTER_SPEED: IconData = material('\u{e43d}');
    /// `sick` (U+F220).
    pub const SICK: IconData = material('\u{f220}');
    /// `sign_language` (U+EBE5).
    pub const SIGN_LANGUAGE: IconData = material('\u{ebe5}');
    /// `signal_cellular_0_bar` (U+F0A8).
    pub const SIGNAL_CELLULAR_0_BAR: IconData = material('\u{f0a8}');
    /// `signal_cellular_4_bar` (U+E1C8).
    pub const SIGNAL_CELLULAR_4_BAR: IconData = material('\u{e1c8}');
    /// `signal_cellular_alt` (U+E202).
    pub const SIGNAL_CELLULAR_ALT: IconData = material('\u{e202}');
    /// `signal_cellular_alt_1_bar` (U+EBDF).
    pub const SIGNAL_CELLULAR_ALT_1_BAR: IconData = material('\u{ebdf}');
    /// `signal_cellular_alt_2_bar` (U+EBE3).
    pub const SIGNAL_CELLULAR_ALT_2_BAR: IconData = material('\u{ebe3}');
    /// `signal_cellular_connected_no_internet_0_bar` (U+F0AC).
    pub const SIGNAL_CELLULAR_CONNECTED_NO_INTERNET_0_BAR: IconData = material('\u{f0ac}');
    /// `signal_cellular_connected_no_internet_4_bar` (U+E1CD).
    pub const SIGNAL_CELLULAR_CONNECTED_NO_INTERNET_4_BAR: IconData = material('\u{e1cd}');
    /// `signal_cellular_no_sim` (U+E1CE).
    pub const SIGNAL_CELLULAR_NO_SIM: IconData = material('\u{e1ce}');
    /// `signal_cellular_nodata` (U+F062).
    pub const SIGNAL_CELLULAR_NODATA: IconData = material('\u{f062}');
    /// `signal_cellular_null` (U+E1CF).
    pub const SIGNAL_CELLULAR_NULL: IconData = material('\u{e1cf}');
    /// `signal_cellular_off` (U+E1D0).
    pub const SIGNAL_CELLULAR_OFF: IconData = material('\u{e1d0}');
    /// `signal_wifi_0_bar` (U+F0B0).
    pub const SIGNAL_WIFI_0_BAR: IconData = material('\u{f0b0}');
    /// `signal_wifi_4_bar` (U+E1D8).
    pub const SIGNAL_WIFI_4_BAR: IconData = material('\u{e1d8}');
    /// `signal_wifi_4_bar_lock` (U+E1D9).
    pub const SIGNAL_WIFI_4_BAR_LOCK: IconData = material('\u{e1d9}');
    /// `signal_wifi_bad` (U+F063).
    pub const SIGNAL_WIFI_BAD: IconData = material('\u{f063}');
    /// `signal_wifi_connected_no_internet_4` (U+F064).
    pub const SIGNAL_WIFI_CONNECTED_NO_INTERNET_4: IconData = material('\u{f064}');
    /// `signal_wifi_off` (U+E1DA).
    pub const SIGNAL_WIFI_OFF: IconData = material('\u{e1da}');
    /// `signal_wifi_statusbar_4_bar` (U+F065).
    pub const SIGNAL_WIFI_STATUSBAR_4_BAR: IconData = material('\u{f065}');
    /// `signal_wifi_statusbar_connected_no_internet_4` (U+F066).
    pub const SIGNAL_WIFI_STATUSBAR_CONNECTED_NO_INTERNET_4: IconData = material('\u{f066}');
    /// `signal_wifi_statusbar_null` (U+F067).
    pub const SIGNAL_WIFI_STATUSBAR_NULL: IconData = material('\u{f067}');
    /// `signpost` (U+EB91).
    pub const SIGNPOST: IconData = material('\u{eb91}');
    /// `sim_card` (U+E32B).
    pub const SIM_CARD: IconData = material('\u{e32b}');
    /// `sim_card_alert` (U+E624).
    pub const SIM_CARD_ALERT: IconData = material('\u{e624}');
    /// `sim_card_download` (U+F068).
    pub const SIM_CARD_DOWNLOAD: IconData = material('\u{f068}');
    /// `single_bed` (U+EA48).
    pub const SINGLE_BED: IconData = material('\u{ea48}');
    /// `sip` (U+F069).
    pub const SIP: IconData = material('\u{f069}');
    /// `skateboarding` (U+E511).
    pub const SKATEBOARDING: IconData = material('\u{e511}');
    /// `skip_next` (U+E044).
    pub const SKIP_NEXT: IconData = material('\u{e044}');
    /// `skip_previous` (U+E045).
    pub const SKIP_PREVIOUS: IconData = material('\u{e045}');
    /// `sledding` (U+E512).
    pub const SLEDDING: IconData = material('\u{e512}');
    /// `slideshow` (U+E41B).
    pub const SLIDESHOW: IconData = material('\u{e41b}');
    /// `slow_motion_video` (U+E068).
    pub const SLOW_MOTION_VIDEO: IconData = material('\u{e068}');
    /// `smart_button` (U+F1C1).
    pub const SMART_BUTTON: IconData = material('\u{f1c1}');
    /// `smart_display` (U+F06A).
    pub const SMART_DISPLAY: IconData = material('\u{f06a}');
    /// `smart_screen` (U+F06B).
    pub const SMART_SCREEN: IconData = material('\u{f06b}');
    /// `smart_toy` (U+F06C).
    pub const SMART_TOY: IconData = material('\u{f06c}');
    /// `smartphone` (U+E32C).
    pub const SMARTPHONE: IconData = material('\u{e32c}');
    /// `smoke_free` (U+EB4A).
    pub const SMOKE_FREE: IconData = material('\u{eb4a}');
    /// `smoking_rooms` (U+EB4B).
    pub const SMOKING_ROOMS: IconData = material('\u{eb4b}');
    /// `sms` (U+E625).
    pub const SMS: IconData = material('\u{e625}');
    /// `sms_failed` (U+E626).
    pub const SMS_FAILED: IconData = material('\u{e626}');
    /// `snapchat` (U+EA6E).
    pub const SNAPCHAT: IconData = material('\u{ea6e}');
    /// `snippet_folder` (U+F1C7).
    pub const SNIPPET_FOLDER: IconData = material('\u{f1c7}');
    /// `snooze` (U+E046).
    pub const SNOOZE: IconData = material('\u{e046}');
    /// `snowboarding` (U+E513).
    pub const SNOWBOARDING: IconData = material('\u{e513}');
    /// `snowing` (U+E80F).
    pub const SNOWING: IconData = material('\u{e80f}');
    /// `snowmobile` (U+E503).
    pub const SNOWMOBILE: IconData = material('\u{e503}');
    /// `snowshoeing` (U+E514).
    pub const SNOWSHOEING: IconData = material('\u{e514}');
    /// `soap` (U+F1B2).
    pub const SOAP: IconData = material('\u{f1b2}');
    /// `social_distance` (U+E1CB).
    pub const SOCIAL_DISTANCE: IconData = material('\u{e1cb}');
    /// `solar_power` (U+EC0F).
    pub const SOLAR_POWER: IconData = material('\u{ec0f}');
    /// `sort` (U+E164).
    pub const SORT: IconData = material('\u{e164}');
    /// `sort_by_alpha` (U+E053).
    pub const SORT_BY_ALPHA: IconData = material('\u{e053}');
    /// `sos` (U+EBF7).
    pub const SOS: IconData = material('\u{ebf7}');
    /// `soup_kitchen` (U+E7D3).
    pub const SOUP_KITCHEN: IconData = material('\u{e7d3}');
    /// `source` (U+F1C4).
    pub const SOURCE: IconData = material('\u{f1c4}');
    /// `south` (U+F1E3).
    pub const SOUTH: IconData = material('\u{f1e3}');
    /// `south_america` (U+E7E4).
    pub const SOUTH_AMERICA: IconData = material('\u{e7e4}');
    /// `south_east` (U+F1E4).
    pub const SOUTH_EAST: IconData = material('\u{f1e4}');
    /// `south_west` (U+F1E5).
    pub const SOUTH_WEST: IconData = material('\u{f1e5}');
    /// `spa` (U+EB4C).
    pub const SPA: IconData = material('\u{eb4c}');
    /// `space_bar` (U+E256).
    pub const SPACE_BAR: IconData = material('\u{e256}');
    /// `space_dashboard` (U+E66B).
    pub const SPACE_DASHBOARD: IconData = material('\u{e66b}');
    /// `spatial_audio` (U+EBEB).
    pub const SPATIAL_AUDIO: IconData = material('\u{ebeb}');
    /// `spatial_audio_off` (U+EBE8).
    pub const SPATIAL_AUDIO_OFF: IconData = material('\u{ebe8}');
    /// `spatial_tracking` (U+EBEA).
    pub const SPATIAL_TRACKING: IconData = material('\u{ebea}');
    /// `speaker` (U+E32D).
    pub const SPEAKER: IconData = material('\u{e32d}');
    /// `speaker_group` (U+E32E).
    pub const SPEAKER_GROUP: IconData = material('\u{e32e}');
    /// `speaker_notes` (U+E8CD).
    pub const SPEAKER_NOTES: IconData = material('\u{e8cd}');
    /// `speaker_notes_off` (U+E92A).
    pub const SPEAKER_NOTES_OFF: IconData = material('\u{e92a}');
    /// `speaker_phone` (U+E0D2).
    pub const SPEAKER_PHONE: IconData = material('\u{e0d2}');
    /// `speed` (U+E9E4).
    pub const SPEED: IconData = material('\u{e9e4}');
    /// `spellcheck` (U+E8CE).
    pub const SPELLCHECK: IconData = material('\u{e8ce}');
    /// `splitscreen` (U+F06D).
    pub const SPLITSCREEN: IconData = material('\u{f06d}');
    /// `spoke` (U+E9A7).
    pub const SPOKE: IconData = material('\u{e9a7}');
    /// `sports` (U+EA30).
    pub const SPORTS: IconData = material('\u{ea30}');
    /// `sports_bar` (U+F1F3).
    pub const SPORTS_BAR: IconData = material('\u{f1f3}');
    /// `sports_baseball` (U+EA51).
    pub const SPORTS_BASEBALL: IconData = material('\u{ea51}');
    /// `sports_basketball` (U+EA26).
    pub const SPORTS_BASKETBALL: IconData = material('\u{ea26}');
    /// `sports_cricket` (U+EA27).
    pub const SPORTS_CRICKET: IconData = material('\u{ea27}');
    /// `sports_esports` (U+EA28).
    pub const SPORTS_ESPORTS: IconData = material('\u{ea28}');
    /// `sports_football` (U+EA29).
    pub const SPORTS_FOOTBALL: IconData = material('\u{ea29}');
    /// `sports_golf` (U+EA2A).
    pub const SPORTS_GOLF: IconData = material('\u{ea2a}');
    /// `sports_gymnastics` (U+EBC4).
    pub const SPORTS_GYMNASTICS: IconData = material('\u{ebc4}');
    /// `sports_handball` (U+EA33).
    pub const SPORTS_HANDBALL: IconData = material('\u{ea33}');
    /// `sports_hockey` (U+EA2B).
    pub const SPORTS_HOCKEY: IconData = material('\u{ea2b}');
    /// `sports_kabaddi` (U+EA34).
    pub const SPORTS_KABADDI: IconData = material('\u{ea34}');
    /// `sports_martial_arts` (U+EAE9).
    pub const SPORTS_MARTIAL_ARTS: IconData = material('\u{eae9}');
    /// `sports_mma` (U+EA2C).
    pub const SPORTS_MMA: IconData = material('\u{ea2c}');
    /// `sports_motorsports` (U+EA2D).
    pub const SPORTS_MOTORSPORTS: IconData = material('\u{ea2d}');
    /// `sports_rugby` (U+EA2E).
    pub const SPORTS_RUGBY: IconData = material('\u{ea2e}');
    /// `sports_score` (U+F06E).
    pub const SPORTS_SCORE: IconData = material('\u{f06e}');
    /// `sports_soccer` (U+EA2F).
    pub const SPORTS_SOCCER: IconData = material('\u{ea2f}');
    /// `sports_tennis` (U+EA32).
    pub const SPORTS_TENNIS: IconData = material('\u{ea32}');
    /// `sports_volleyball` (U+EA31).
    pub const SPORTS_VOLLEYBALL: IconData = material('\u{ea31}');
    /// `square` (U+EB36).
    pub const SQUARE: IconData = material('\u{eb36}');
    /// `square_foot` (U+EA49).
    pub const SQUARE_FOOT: IconData = material('\u{ea49}');
    /// `ssid_chart` (U+EB66).
    pub const SSID_CHART: IconData = material('\u{eb66}');
    /// `stacked_bar_chart` (U+E9E6).
    pub const STACKED_BAR_CHART: IconData = material('\u{e9e6}');
    /// `stacked_line_chart` (U+F22B).
    pub const STACKED_LINE_CHART: IconData = material('\u{f22b}');
    /// `stadium` (U+EB90).
    pub const STADIUM: IconData = material('\u{eb90}');
    /// `stairs` (U+F1A9).
    pub const STAIRS: IconData = material('\u{f1a9}');
    /// `star` (U+E838).
    pub const STAR: IconData = material('\u{e838}');
    /// `star_border` (U+E83A).
    pub const STAR_BORDER: IconData = material('\u{e83a}');
    /// `star_border_purple500` (U+F099).
    pub const STAR_BORDER_PURPLE500: IconData = material('\u{f099}');
    /// `star_half` (U+E839).
    pub const STAR_HALF: IconData = material('\u{e839}');
    /// `star_outline` (U+F06F).
    pub const STAR_OUTLINE: IconData = material('\u{f06f}');
    /// `star_purple500` (U+F09A).
    pub const STAR_PURPLE500: IconData = material('\u{f09a}');
    /// `star_rate` (U+F0EC).
    pub const STAR_RATE: IconData = material('\u{f0ec}');
    /// `stars` (U+E8D0).
    pub const STARS: IconData = material('\u{e8d0}');
    /// `start` (U+E089).
    pub const START: IconData = material('\u{e089}');
    /// `stay_current_landscape` (U+E0D3).
    pub const STAY_CURRENT_LANDSCAPE: IconData = material('\u{e0d3}');
    /// `stay_current_portrait` (U+E0D4).
    pub const STAY_CURRENT_PORTRAIT: IconData = material('\u{e0d4}');
    /// `stay_primary_landscape` (U+E0D5).
    pub const STAY_PRIMARY_LANDSCAPE: IconData = material('\u{e0d5}');
    /// `stay_primary_portrait` (U+E0D6).
    pub const STAY_PRIMARY_PORTRAIT: IconData = material('\u{e0d6}');
    /// `sticky_note_2` (U+F1FC).
    pub const STICKY_NOTE_2: IconData = material('\u{f1fc}');
    /// `stop` (U+E047).
    pub const STOP: IconData = material('\u{e047}');
    /// `stop_circle` (U+EF71).
    pub const STOP_CIRCLE: IconData = material('\u{ef71}');
    /// `stop_screen_share` (U+E0E3).
    pub const STOP_SCREEN_SHARE: IconData = material('\u{e0e3}');
    /// `storage` (U+E1DB).
    pub const STORAGE: IconData = material('\u{e1db}');
    /// `store` (U+E8D1).
    pub const STORE: IconData = material('\u{e8d1}');
    /// `store_mall_directory` (U+E563).
    pub const STORE_MALL_DIRECTORY: IconData = material('\u{e563}');
    /// `storefront` (U+EA12).
    pub const STOREFRONT: IconData = material('\u{ea12}');
    /// `storm` (U+F070).
    pub const STORM: IconData = material('\u{f070}');
    /// `straight` (U+EB95).
    pub const STRAIGHT: IconData = material('\u{eb95}');
    /// `straighten` (U+E41C).
    pub const STRAIGHTEN: IconData = material('\u{e41c}');
    /// `stream` (U+E9E9).
    pub const STREAM: IconData = material('\u{e9e9}');
    /// `streetview` (U+E56E).
    pub const STREETVIEW: IconData = material('\u{e56e}');
    /// `strikethrough_s` (U+E257).
    pub const STRIKETHROUGH_S: IconData = material('\u{e257}');
    /// `stroller` (U+F1AE).
    pub const STROLLER: IconData = material('\u{f1ae}');
    /// `style` (U+E41D).
    pub const STYLE: IconData = material('\u{e41d}');
    /// `subdirectory_arrow_left` (U+E5D9).
    pub const SUBDIRECTORY_ARROW_LEFT: IconData = material('\u{e5d9}');
    /// `subdirectory_arrow_right` (U+E5DA).
    pub const SUBDIRECTORY_ARROW_RIGHT: IconData = material('\u{e5da}');
    /// `subject` (U+E8D2).
    pub const SUBJECT: IconData = material('\u{e8d2}');
    /// `subscript` (U+F111).
    pub const SUBSCRIPT: IconData = material('\u{f111}');
    /// `subscriptions` (U+E064).
    pub const SUBSCRIPTIONS: IconData = material('\u{e064}');
    /// `subtitles` (U+E048).
    pub const SUBTITLES: IconData = material('\u{e048}');
    /// `subtitles_off` (U+EF72).
    pub const SUBTITLES_OFF: IconData = material('\u{ef72}');
    /// `subway` (U+E56F).
    pub const SUBWAY: IconData = material('\u{e56f}');
    /// `summarize` (U+F071).
    pub const SUMMARIZE: IconData = material('\u{f071}');
    /// `sunny` (U+E81A).
    pub const SUNNY: IconData = material('\u{e81a}');
    /// `sunny_snowing` (U+E819).
    pub const SUNNY_SNOWING: IconData = material('\u{e819}');
    /// `superscript` (U+F112).
    pub const SUPERSCRIPT: IconData = material('\u{f112}');
    /// `supervised_user_circle` (U+E939).
    pub const SUPERVISED_USER_CIRCLE: IconData = material('\u{e939}');
    /// `supervisor_account` (U+E8D3).
    pub const SUPERVISOR_ACCOUNT: IconData = material('\u{e8d3}');
    /// `support` (U+EF73).
    pub const SUPPORT: IconData = material('\u{ef73}');
    /// `support_agent` (U+F0E2).
    pub const SUPPORT_AGENT: IconData = material('\u{f0e2}');
    /// `surfing` (U+E515).
    pub const SURFING: IconData = material('\u{e515}');
    /// `surround_sound` (U+E049).
    pub const SURROUND_SOUND: IconData = material('\u{e049}');
    /// `swap_calls` (U+E0D7).
    pub const SWAP_CALLS: IconData = material('\u{e0d7}');
    /// `swap_horiz` (U+E8D4).
    pub const SWAP_HORIZ: IconData = material('\u{e8d4}');
    /// `swap_horizontal_circle` (U+E933).
    pub const SWAP_HORIZONTAL_CIRCLE: IconData = material('\u{e933}');
    /// `swap_vert` (U+E8D5).
    pub const SWAP_VERT: IconData = material('\u{e8d5}');
    /// `swap_vert_circle` (U+E8D6).
    pub const SWAP_VERT_CIRCLE: IconData = material('\u{e8d6}');
    /// `swap_vertical_circle` (U+E8D6).
    pub const SWAP_VERTICAL_CIRCLE: IconData = material('\u{e8d6}');
    /// `swipe` (U+E9EC).
    pub const SWIPE: IconData = material('\u{e9ec}');
    /// `swipe_down` (U+EB53).
    pub const SWIPE_DOWN: IconData = material('\u{eb53}');
    /// `swipe_down_alt` (U+EB30).
    pub const SWIPE_DOWN_ALT: IconData = material('\u{eb30}');
    /// `swipe_left` (U+EB59).
    pub const SWIPE_LEFT: IconData = material('\u{eb59}');
    /// `swipe_left_alt` (U+EB33).
    pub const SWIPE_LEFT_ALT: IconData = material('\u{eb33}');
    /// `swipe_right` (U+EB52).
    pub const SWIPE_RIGHT: IconData = material('\u{eb52}');
    /// `swipe_right_alt` (U+EB56).
    pub const SWIPE_RIGHT_ALT: IconData = material('\u{eb56}');
    /// `swipe_up` (U+EB2E).
    pub const SWIPE_UP: IconData = material('\u{eb2e}');
    /// `swipe_up_alt` (U+EB35).
    pub const SWIPE_UP_ALT: IconData = material('\u{eb35}');
    /// `swipe_vertical` (U+EB51).
    pub const SWIPE_VERTICAL: IconData = material('\u{eb51}');
    /// `switch_access_shortcut` (U+E7E1).
    pub const SWITCH_ACCESS_SHORTCUT: IconData = material('\u{e7e1}');
    /// `switch_access_shortcut_add` (U+E7E2).
    pub const SWITCH_ACCESS_SHORTCUT_ADD: IconData = material('\u{e7e2}');
    /// `switch_account` (U+E9ED).
    pub const SWITCH_ACCOUNT: IconData = material('\u{e9ed}');
    /// `switch_camera` (U+E41E).
    pub const SWITCH_CAMERA: IconData = material('\u{e41e}');
    /// `switch_left` (U+F1D1).
    pub const SWITCH_LEFT: IconData = material('\u{f1d1}');
    /// `switch_right` (U+F1D2).
    pub const SWITCH_RIGHT: IconData = material('\u{f1d2}');
    /// `switch_video` (U+E41F).
    pub const SWITCH_VIDEO: IconData = material('\u{e41f}');
    /// `synagogue` (U+EAB0).
    pub const SYNAGOGUE: IconData = material('\u{eab0}');
    /// `sync` (U+E627).
    pub const SYNC: IconData = material('\u{e627}');
    /// `sync_alt` (U+EA18).
    pub const SYNC_ALT: IconData = material('\u{ea18}');
    /// `sync_disabled` (U+E628).
    pub const SYNC_DISABLED: IconData = material('\u{e628}');
    /// `sync_lock` (U+EAEE).
    pub const SYNC_LOCK: IconData = material('\u{eaee}');
    /// `sync_problem` (U+E629).
    pub const SYNC_PROBLEM: IconData = material('\u{e629}');
    /// `system_security_update` (U+F072).
    pub const SYSTEM_SECURITY_UPDATE: IconData = material('\u{f072}');
    /// `system_security_update_good` (U+F073).
    pub const SYSTEM_SECURITY_UPDATE_GOOD: IconData = material('\u{f073}');
    /// `system_security_update_warning` (U+F074).
    pub const SYSTEM_SECURITY_UPDATE_WARNING: IconData = material('\u{f074}');
    /// `system_update` (U+E62A).
    pub const SYSTEM_UPDATE: IconData = material('\u{e62a}');
    /// `system_update_alt` (U+E8D7).
    pub const SYSTEM_UPDATE_ALT: IconData = material('\u{e8d7}');
    /// `system_update_tv` (U+E8D7).
    pub const SYSTEM_UPDATE_TV: IconData = material('\u{e8d7}');
    /// `tab` (U+E8D8).
    pub const TAB: IconData = material('\u{e8d8}');
    /// `tab_unselected` (U+E8D9).
    pub const TAB_UNSELECTED: IconData = material('\u{e8d9}');
    /// `table_bar` (U+EAD2).
    pub const TABLE_BAR: IconData = material('\u{ead2}');
    /// `table_chart` (U+E265).
    pub const TABLE_CHART: IconData = material('\u{e265}');
    /// `table_restaurant` (U+EAC6).
    pub const TABLE_RESTAURANT: IconData = material('\u{eac6}');
    /// `table_rows` (U+F101).
    pub const TABLE_ROWS: IconData = material('\u{f101}');
    /// `table_view` (U+F1BE).
    pub const TABLE_VIEW: IconData = material('\u{f1be}');
    /// `tablet` (U+E32F).
    pub const TABLET: IconData = material('\u{e32f}');
    /// `tablet_android` (U+E330).
    pub const TABLET_ANDROID: IconData = material('\u{e330}');
    /// `tablet_mac` (U+E331).
    pub const TABLET_MAC: IconData = material('\u{e331}');
    /// `tag` (U+E9EF).
    pub const TAG: IconData = material('\u{e9ef}');
    /// `tag_faces` (U+E420).
    pub const TAG_FACES: IconData = material('\u{e420}');
    /// `takeout_dining` (U+EA74).
    pub const TAKEOUT_DINING: IconData = material('\u{ea74}');
    /// `tap_and_play` (U+E62B).
    pub const TAP_AND_PLAY: IconData = material('\u{e62b}');
    /// `tapas` (U+F1E9).
    pub const TAPAS: IconData = material('\u{f1e9}');
    /// `task` (U+F075).
    pub const TASK: IconData = material('\u{f075}');
    /// `task_alt` (U+E2E6).
    pub const TASK_ALT: IconData = material('\u{e2e6}');
    /// `taxi_alert` (U+EF74).
    pub const TAXI_ALERT: IconData = material('\u{ef74}');
    /// `telegram` (U+EA6B).
    pub const TELEGRAM: IconData = material('\u{ea6b}');
    /// `temple_buddhist` (U+EAB3).
    pub const TEMPLE_BUDDHIST: IconData = material('\u{eab3}');
    /// `temple_hindu` (U+EAAF).
    pub const TEMPLE_HINDU: IconData = material('\u{eaaf}');
    /// `terminal` (U+EB8E).
    pub const TERMINAL: IconData = material('\u{eb8e}');
    /// `terrain` (U+E564).
    pub const TERRAIN: IconData = material('\u{e564}');
    /// `text_decrease` (U+EADD).
    pub const TEXT_DECREASE: IconData = material('\u{eadd}');
    /// `text_fields` (U+E262).
    pub const TEXT_FIELDS: IconData = material('\u{e262}');
    /// `text_format` (U+E165).
    pub const TEXT_FORMAT: IconData = material('\u{e165}');
    /// `text_increase` (U+EAE2).
    pub const TEXT_INCREASE: IconData = material('\u{eae2}');
    /// `text_rotate_up` (U+E93A).
    pub const TEXT_ROTATE_UP: IconData = material('\u{e93a}');
    /// `text_rotate_vertical` (U+E93B).
    pub const TEXT_ROTATE_VERTICAL: IconData = material('\u{e93b}');
    /// `text_rotation_angledown` (U+E93C).
    pub const TEXT_ROTATION_ANGLEDOWN: IconData = material('\u{e93c}');
    /// `text_rotation_angleup` (U+E93D).
    pub const TEXT_ROTATION_ANGLEUP: IconData = material('\u{e93d}');
    /// `text_rotation_down` (U+E93E).
    pub const TEXT_ROTATION_DOWN: IconData = material('\u{e93e}');
    /// `text_rotation_none` (U+E93F).
    pub const TEXT_ROTATION_NONE: IconData = material('\u{e93f}');
    /// `text_snippet` (U+F1C6).
    pub const TEXT_SNIPPET: IconData = material('\u{f1c6}');
    /// `textsms` (U+E0D8).
    pub const TEXTSMS: IconData = material('\u{e0d8}');
    /// `texture` (U+E421).
    pub const TEXTURE: IconData = material('\u{e421}');
    /// `theater_comedy` (U+EA66).
    pub const THEATER_COMEDY: IconData = material('\u{ea66}');
    /// `theaters` (U+E8DA).
    pub const THEATERS: IconData = material('\u{e8da}');
    /// `thermostat` (U+F076).
    pub const THERMOSTAT: IconData = material('\u{f076}');
    /// `thermostat_auto` (U+F077).
    pub const THERMOSTAT_AUTO: IconData = material('\u{f077}');
    /// `thumb_down` (U+E8DB).
    pub const THUMB_DOWN: IconData = material('\u{e8db}');
    /// `thumb_down_alt` (U+E816).
    pub const THUMB_DOWN_ALT: IconData = material('\u{e816}');
    /// `thumb_down_off_alt` (U+E9F2).
    pub const THUMB_DOWN_OFF_ALT: IconData = material('\u{e9f2}');
    /// `thumb_up` (U+E8DC).
    pub const THUMB_UP: IconData = material('\u{e8dc}');
    /// `thumb_up_alt` (U+E817).
    pub const THUMB_UP_ALT: IconData = material('\u{e817}');
    /// `thumb_up_off_alt` (U+E9F3).
    pub const THUMB_UP_OFF_ALT: IconData = material('\u{e9f3}');
    /// `thumbs_up_down` (U+E8DD).
    pub const THUMBS_UP_DOWN: IconData = material('\u{e8dd}');
    /// `thunderstorm` (U+EBDB).
    pub const THUNDERSTORM: IconData = material('\u{ebdb}');
    /// `tiktok` (U+EA7E).
    pub const TIKTOK: IconData = material('\u{ea7e}');
    /// `time_to_leave` (U+E62C).
    pub const TIME_TO_LEAVE: IconData = material('\u{e62c}');
    /// `timelapse` (U+E422).
    pub const TIMELAPSE: IconData = material('\u{e422}');
    /// `timeline` (U+E922).
    pub const TIMELINE: IconData = material('\u{e922}');
    /// `timer` (U+E425).
    pub const TIMER: IconData = material('\u{e425}');
    /// `timer_10` (U+E423).
    pub const TIMER_10: IconData = material('\u{e423}');
    /// `timer_10_select` (U+F07A).
    pub const TIMER_10_SELECT: IconData = material('\u{f07a}');
    /// `timer_3` (U+E424).
    pub const TIMER_3: IconData = material('\u{e424}');
    /// `timer_3_select` (U+F07B).
    pub const TIMER_3_SELECT: IconData = material('\u{f07b}');
    /// `timer_off` (U+E426).
    pub const TIMER_OFF: IconData = material('\u{e426}');
    /// `tips_and_updates` (U+E79A).
    pub const TIPS_AND_UPDATES: IconData = material('\u{e79a}');
    /// `tire_repair` (U+EBC8).
    pub const TIRE_REPAIR: IconData = material('\u{ebc8}');
    /// `title` (U+E264).
    pub const TITLE: IconData = material('\u{e264}');
    /// `toc` (U+E8DE).
    pub const TOC: IconData = material('\u{e8de}');
    /// `today` (U+E8DF).
    pub const TODAY: IconData = material('\u{e8df}');
    /// `toggle_off` (U+E9F5).
    pub const TOGGLE_OFF: IconData = material('\u{e9f5}');
    /// `toggle_on` (U+E9F6).
    pub const TOGGLE_ON: IconData = material('\u{e9f6}');
    /// `token` (U+EA25).
    pub const TOKEN: IconData = material('\u{ea25}');
    /// `toll` (U+E8E0).
    pub const TOLL: IconData = material('\u{e8e0}');
    /// `tonality` (U+E427).
    pub const TONALITY: IconData = material('\u{e427}');
    /// `topic` (U+F1C8).
    pub const TOPIC: IconData = material('\u{f1c8}');
    /// `tornado` (U+E199).
    pub const TORNADO: IconData = material('\u{e199}');
    /// `touch_app` (U+E913).
    pub const TOUCH_APP: IconData = material('\u{e913}');
    /// `tour` (U+EF75).
    pub const TOUR: IconData = material('\u{ef75}');
    /// `toys` (U+E332).
    pub const TOYS: IconData = material('\u{e332}');
    /// `track_changes` (U+E8E1).
    pub const TRACK_CHANGES: IconData = material('\u{e8e1}');
    /// `traffic` (U+E565).
    pub const TRAFFIC: IconData = material('\u{e565}');
    /// `train` (U+E570).
    pub const TRAIN: IconData = material('\u{e570}');
    /// `tram` (U+E571).
    pub const TRAM: IconData = material('\u{e571}');
    /// `transcribe` (U+F8EC).
    pub const TRANSCRIBE: IconData = material('\u{f8ec}');
    /// `transfer_within_a_station` (U+E572).
    pub const TRANSFER_WITHIN_A_STATION: IconData = material('\u{e572}');
    /// `transform` (U+E428).
    pub const TRANSFORM: IconData = material('\u{e428}');
    /// `transgender` (U+E58D).
    pub const TRANSGENDER: IconData = material('\u{e58d}');
    /// `transit_enterexit` (U+E579).
    pub const TRANSIT_ENTEREXIT: IconData = material('\u{e579}');
    /// `translate` (U+E8E2).
    pub const TRANSLATE: IconData = material('\u{e8e2}');
    /// `travel_explore` (U+E2DB).
    pub const TRAVEL_EXPLORE: IconData = material('\u{e2db}');
    /// `trending_down` (U+E8E3).
    pub const TRENDING_DOWN: IconData = material('\u{e8e3}');
    /// `trending_flat` (U+E8E4).
    pub const TRENDING_FLAT: IconData = material('\u{e8e4}');
    /// `trending_neutral` (U+E8E4).
    pub const TRENDING_NEUTRAL: IconData = material('\u{e8e4}');
    /// `trending_up` (U+E8E5).
    pub const TRENDING_UP: IconData = material('\u{e8e5}');
    /// `trip_origin` (U+E57B).
    pub const TRIP_ORIGIN: IconData = material('\u{e57b}');
    /// `trolley` (U+F86B).
    pub const TROLLEY: IconData = material('\u{f86b}');
    /// `troubleshoot` (U+E1D2).
    pub const TROUBLESHOOT: IconData = material('\u{e1d2}');
    /// `try` (U+F07C).
    pub const TRY: IconData = material('\u{f07c}');
    /// `tsunami` (U+EBD8).
    pub const TSUNAMI: IconData = material('\u{ebd8}');
    /// `tty` (U+F1AA).
    pub const TTY: IconData = material('\u{f1aa}');
    /// `tune` (U+E429).
    pub const TUNE: IconData = material('\u{e429}');
    /// `tungsten` (U+F07D).
    pub const TUNGSTEN: IconData = material('\u{f07d}');
    /// `turn_left` (U+EBA6).
    pub const TURN_LEFT: IconData = material('\u{eba6}');
    /// `turn_right` (U+EBAB).
    pub const TURN_RIGHT: IconData = material('\u{ebab}');
    /// `turn_sharp_left` (U+EBA7).
    pub const TURN_SHARP_LEFT: IconData = material('\u{eba7}');
    /// `turn_sharp_right` (U+EBAA).
    pub const TURN_SHARP_RIGHT: IconData = material('\u{ebaa}');
    /// `turn_slight_left` (U+EBA4).
    pub const TURN_SLIGHT_LEFT: IconData = material('\u{eba4}');
    /// `turn_slight_right` (U+EB9A).
    pub const TURN_SLIGHT_RIGHT: IconData = material('\u{eb9a}');
    /// `turned_in` (U+E8E6).
    pub const TURNED_IN: IconData = material('\u{e8e6}');
    /// `turned_in_not` (U+E8E7).
    pub const TURNED_IN_NOT: IconData = material('\u{e8e7}');
    /// `tv` (U+E333).
    pub const TV: IconData = material('\u{e333}');
    /// `tv_off` (U+E647).
    pub const TV_OFF: IconData = material('\u{e647}');
    /// `two_wheeler` (U+E9F9).
    pub const TWO_WHEELER: IconData = material('\u{e9f9}');
    /// `type_specimen` (U+F8F0).
    pub const TYPE_SPECIMEN: IconData = material('\u{f8f0}');
    /// `u_turn_left` (U+EBA1).
    pub const U_TURN_LEFT: IconData = material('\u{eba1}');
    /// `u_turn_right` (U+EBA2).
    pub const U_TURN_RIGHT: IconData = material('\u{eba2}');
    /// `umbrella` (U+F1AD).
    pub const UMBRELLA: IconData = material('\u{f1ad}');
    /// `unarchive` (U+E169).
    pub const UNARCHIVE: IconData = material('\u{e169}');
    /// `undo` (U+E166).
    pub const UNDO: IconData = material('\u{e166}');
    /// `unfold_less` (U+E5D6).
    pub const UNFOLD_LESS: IconData = material('\u{e5d6}');
    /// `unfold_less_double` (U+F8CF).
    pub const UNFOLD_LESS_DOUBLE: IconData = material('\u{f8cf}');
    /// `unfold_more` (U+E5D7).
    pub const UNFOLD_MORE: IconData = material('\u{e5d7}');
    /// `unfold_more_double` (U+F8D0).
    pub const UNFOLD_MORE_DOUBLE: IconData = material('\u{f8d0}');
    /// `unpublished` (U+F236).
    pub const UNPUBLISHED: IconData = material('\u{f236}');
    /// `unsubscribe` (U+E0EB).
    pub const UNSUBSCRIBE: IconData = material('\u{e0eb}');
    /// `upcoming` (U+F07E).
    pub const UPCOMING: IconData = material('\u{f07e}');
    /// `update` (U+E923).
    pub const UPDATE: IconData = material('\u{e923}');
    /// `update_disabled` (U+E075).
    pub const UPDATE_DISABLED: IconData = material('\u{e075}');
    /// `upgrade` (U+F0FB).
    pub const UPGRADE: IconData = material('\u{f0fb}');
    /// `upload` (U+F09B).
    pub const UPLOAD: IconData = material('\u{f09b}');
    /// `upload_file` (U+E9FC).
    pub const UPLOAD_FILE: IconData = material('\u{e9fc}');
    /// `usb` (U+E1E0).
    pub const USB: IconData = material('\u{e1e0}');
    /// `usb_off` (U+E4FA).
    pub const USB_OFF: IconData = material('\u{e4fa}');
    /// `vaccines` (U+E138).
    pub const VACCINES: IconData = material('\u{e138}');
    /// `vape_free` (U+EBC6).
    pub const VAPE_FREE: IconData = material('\u{ebc6}');
    /// `vaping_rooms` (U+EBCF).
    pub const VAPING_ROOMS: IconData = material('\u{ebcf}');
    /// `verified` (U+EF76).
    pub const VERIFIED: IconData = material('\u{ef76}');
    /// `verified_user` (U+E8E8).
    pub const VERIFIED_USER: IconData = material('\u{e8e8}');
    /// `vertical_align_bottom` (U+E258).
    pub const VERTICAL_ALIGN_BOTTOM: IconData = material('\u{e258}');
    /// `vertical_align_center` (U+E259).
    pub const VERTICAL_ALIGN_CENTER: IconData = material('\u{e259}');
    /// `vertical_align_top` (U+E25A).
    pub const VERTICAL_ALIGN_TOP: IconData = material('\u{e25a}');
    /// `vertical_distribute` (U+E076).
    pub const VERTICAL_DISTRIBUTE: IconData = material('\u{e076}');
    /// `vertical_shades` (U+EC0E).
    pub const VERTICAL_SHADES: IconData = material('\u{ec0e}');
    /// `vertical_shades_closed` (U+EC0D).
    pub const VERTICAL_SHADES_CLOSED: IconData = material('\u{ec0d}');
    /// `vertical_split` (U+E949).
    pub const VERTICAL_SPLIT: IconData = material('\u{e949}');
    /// `vibration` (U+E62D).
    pub const VIBRATION: IconData = material('\u{e62d}');
    /// `video_call` (U+E070).
    pub const VIDEO_CALL: IconData = material('\u{e070}');
    /// `video_camera_back` (U+F07F).
    pub const VIDEO_CAMERA_BACK: IconData = material('\u{f07f}');
    /// `video_camera_front` (U+F080).
    pub const VIDEO_CAMERA_FRONT: IconData = material('\u{f080}');
    /// `video_chat` (U+F8A0).
    pub const VIDEO_CHAT: IconData = material('\u{f8a0}');
    /// `video_collection` (U+E04A).
    pub const VIDEO_COLLECTION: IconData = material('\u{e04a}');
    /// `video_file` (U+EB87).
    pub const VIDEO_FILE: IconData = material('\u{eb87}');
    /// `video_label` (U+E071).
    pub const VIDEO_LABEL: IconData = material('\u{e071}');
    /// `video_library` (U+E04A).
    pub const VIDEO_LIBRARY: IconData = material('\u{e04a}');
    /// `video_settings` (U+EA75).
    pub const VIDEO_SETTINGS: IconData = material('\u{ea75}');
    /// `video_stable` (U+F081).
    pub const VIDEO_STABLE: IconData = material('\u{f081}');
    /// `videocam` (U+E04B).
    pub const VIDEOCAM: IconData = material('\u{e04b}');
    /// `videocam_off` (U+E04C).
    pub const VIDEOCAM_OFF: IconData = material('\u{e04c}');
    /// `videogame_asset` (U+E338).
    pub const VIDEOGAME_ASSET: IconData = material('\u{e338}');
    /// `videogame_asset_off` (U+E500).
    pub const VIDEOGAME_ASSET_OFF: IconData = material('\u{e500}');
    /// `view_agenda` (U+E8E9).
    pub const VIEW_AGENDA: IconData = material('\u{e8e9}');
    /// `view_array` (U+E8EA).
    pub const VIEW_ARRAY: IconData = material('\u{e8ea}');
    /// `view_carousel` (U+E8EB).
    pub const VIEW_CAROUSEL: IconData = material('\u{e8eb}');
    /// `view_column` (U+E8EC).
    pub const VIEW_COLUMN: IconData = material('\u{e8ec}');
    /// `view_comfortable` (U+E42A).
    pub const VIEW_COMFORTABLE: IconData = material('\u{e42a}');
    /// `view_comfy` (U+E42A).
    pub const VIEW_COMFY: IconData = material('\u{e42a}');
    /// `view_comfy_alt` (U+EB73).
    pub const VIEW_COMFY_ALT: IconData = material('\u{eb73}');
    /// `view_compact` (U+E42B).
    pub const VIEW_COMPACT: IconData = material('\u{e42b}');
    /// `view_compact_alt` (U+EB74).
    pub const VIEW_COMPACT_ALT: IconData = material('\u{eb74}');
    /// `view_cozy` (U+EB75).
    pub const VIEW_COZY: IconData = material('\u{eb75}');
    /// `view_day` (U+E8ED).
    pub const VIEW_DAY: IconData = material('\u{e8ed}');
    /// `view_headline` (U+E8EE).
    pub const VIEW_HEADLINE: IconData = material('\u{e8ee}');
    /// `view_in_ar` (U+E9FE).
    pub const VIEW_IN_AR: IconData = material('\u{e9fe}');
    /// `view_kanban` (U+EB7F).
    pub const VIEW_KANBAN: IconData = material('\u{eb7f}');
    /// `view_list` (U+E8EF).
    pub const VIEW_LIST: IconData = material('\u{e8ef}');
    /// `view_module` (U+E8F0).
    pub const VIEW_MODULE: IconData = material('\u{e8f0}');
    /// `view_quilt` (U+E8F1).
    pub const VIEW_QUILT: IconData = material('\u{e8f1}');
    /// `view_sidebar` (U+F114).
    pub const VIEW_SIDEBAR: IconData = material('\u{f114}');
    /// `view_stream` (U+E8F2).
    pub const VIEW_STREAM: IconData = material('\u{e8f2}');
    /// `view_timeline` (U+EB85).
    pub const VIEW_TIMELINE: IconData = material('\u{eb85}');
    /// `view_week` (U+E8F3).
    pub const VIEW_WEEK: IconData = material('\u{e8f3}');
    /// `vignette` (U+E435).
    pub const VIGNETTE: IconData = material('\u{e435}');
    /// `villa` (U+E586).
    pub const VILLA: IconData = material('\u{e586}');
    /// `visibility` (U+E8F4).
    pub const VISIBILITY: IconData = material('\u{e8f4}');
    /// `visibility_off` (U+E8F5).
    pub const VISIBILITY_OFF: IconData = material('\u{e8f5}');
    /// `voice_chat` (U+E62E).
    pub const VOICE_CHAT: IconData = material('\u{e62e}');
    /// `voice_over_off` (U+E94A).
    pub const VOICE_OVER_OFF: IconData = material('\u{e94a}');
    /// `voicemail` (U+E0D9).
    pub const VOICEMAIL: IconData = material('\u{e0d9}');
    /// `volcano` (U+EBDA).
    pub const VOLCANO: IconData = material('\u{ebda}');
    /// `volume_down` (U+E04D).
    pub const VOLUME_DOWN: IconData = material('\u{e04d}');
    /// `volume_down_alt` (U+E79C).
    pub const VOLUME_DOWN_ALT: IconData = material('\u{e79c}');
    /// `volume_mute` (U+E04E).
    pub const VOLUME_MUTE: IconData = material('\u{e04e}');
    /// `volume_off` (U+E04F).
    pub const VOLUME_OFF: IconData = material('\u{e04f}');
    /// `volume_up` (U+E050).
    pub const VOLUME_UP: IconData = material('\u{e050}');
    /// `volunteer_activism` (U+EA70).
    pub const VOLUNTEER_ACTIVISM: IconData = material('\u{ea70}');
    /// `vpn_key` (U+E0DA).
    pub const VPN_KEY: IconData = material('\u{e0da}');
    /// `vpn_key_off` (U+EB7A).
    pub const VPN_KEY_OFF: IconData = material('\u{eb7a}');
    /// `vpn_lock` (U+E62F).
    pub const VPN_LOCK: IconData = material('\u{e62f}');
    /// `vrpano` (U+F082).
    pub const VRPANO: IconData = material('\u{f082}');
    /// `wallet` (U+F8FF).
    pub const WALLET: IconData = material('\u{f8ff}');
    /// `wallet_giftcard` (U+E8F6).
    pub const WALLET_GIFTCARD: IconData = material('\u{e8f6}');
    /// `wallet_membership` (U+E8F7).
    pub const WALLET_MEMBERSHIP: IconData = material('\u{e8f7}');
    /// `wallet_travel` (U+E8F8).
    pub const WALLET_TRAVEL: IconData = material('\u{e8f8}');
    /// `wallpaper` (U+E1BC).
    pub const WALLPAPER: IconData = material('\u{e1bc}');
    /// `warehouse` (U+EBB8).
    pub const WAREHOUSE: IconData = material('\u{ebb8}');
    /// `warning` (U+E002).
    pub const WARNING: IconData = material('\u{e002}');
    /// `warning_amber` (U+F083).
    pub const WARNING_AMBER: IconData = material('\u{f083}');
    /// `wash` (U+F1B1).
    pub const WASH: IconData = material('\u{f1b1}');
    /// `watch` (U+E334).
    pub const WATCH: IconData = material('\u{e334}');
    /// `watch_later` (U+E924).
    pub const WATCH_LATER: IconData = material('\u{e924}');
    /// `watch_off` (U+EAE3).
    pub const WATCH_OFF: IconData = material('\u{eae3}');
    /// `water` (U+F084).
    pub const WATER: IconData = material('\u{f084}');
    /// `water_damage` (U+F203).
    pub const WATER_DAMAGE: IconData = material('\u{f203}');
    /// `water_drop` (U+E798).
    pub const WATER_DROP: IconData = material('\u{e798}');
    /// `waterfall_chart` (U+EA00).
    pub const WATERFALL_CHART: IconData = material('\u{ea00}');
    /// `waves` (U+E176).
    pub const WAVES: IconData = material('\u{e176}');
    /// `waving_hand` (U+E766).
    pub const WAVING_HAND: IconData = material('\u{e766}');
    /// `wb_auto` (U+E42C).
    pub const WB_AUTO: IconData = material('\u{e42c}');
    /// `wb_cloudy` (U+E42D).
    pub const WB_CLOUDY: IconData = material('\u{e42d}');
    /// `wb_incandescent` (U+E42E).
    pub const WB_INCANDESCENT: IconData = material('\u{e42e}');
    /// `wb_iridescent` (U+E436).
    pub const WB_IRIDESCENT: IconData = material('\u{e436}');
    /// `wb_shade` (U+EA01).
    pub const WB_SHADE: IconData = material('\u{ea01}');
    /// `wb_sunny` (U+E430).
    pub const WB_SUNNY: IconData = material('\u{e430}');
    /// `wb_twighlight` (U+EA02).
    pub const WB_TWIGHLIGHT: IconData = material('\u{ea02}');
    /// `wb_twilight` (U+E1C6).
    pub const WB_TWILIGHT: IconData = material('\u{e1c6}');
    /// `wc` (U+E63D).
    pub const WC: IconData = material('\u{e63d}');
    /// `web` (U+E051).
    pub const WEB: IconData = material('\u{e051}');
    /// `web_asset` (U+E069).
    pub const WEB_ASSET: IconData = material('\u{e069}');
    /// `web_asset_off` (U+E4F7).
    pub const WEB_ASSET_OFF: IconData = material('\u{e4f7}');
    /// `web_stories` (U+E595).
    pub const WEB_STORIES: IconData = material('\u{e595}');
    /// `webhook` (U+EB92).
    pub const WEBHOOK: IconData = material('\u{eb92}');
    /// `wechat` (U+EA81).
    pub const WECHAT: IconData = material('\u{ea81}');
    /// `weekend` (U+E16B).
    pub const WEEKEND: IconData = material('\u{e16b}');
    /// `west` (U+F1E6).
    pub const WEST: IconData = material('\u{f1e6}');
    /// `whatshot` (U+E80E).
    pub const WHATSHOT: IconData = material('\u{e80e}');
    /// `wheelchair_pickup` (U+F1AB).
    pub const WHEELCHAIR_PICKUP: IconData = material('\u{f1ab}');
    /// `where_to_vote` (U+E177).
    pub const WHERE_TO_VOTE: IconData = material('\u{e177}');
    /// `widgets` (U+E1BD).
    pub const WIDGETS: IconData = material('\u{e1bd}');
    /// `width_full` (U+F8F5).
    pub const WIDTH_FULL: IconData = material('\u{f8f5}');
    /// `width_normal` (U+F8F6).
    pub const WIDTH_NORMAL: IconData = material('\u{f8f6}');
    /// `width_wide` (U+F8F7).
    pub const WIDTH_WIDE: IconData = material('\u{f8f7}');
    /// `wifi` (U+E63E).
    pub const WIFI: IconData = material('\u{e63e}');
    /// `wifi_1_bar` (U+E4CA).
    pub const WIFI_1_BAR: IconData = material('\u{e4ca}');
    /// `wifi_2_bar` (U+E4D9).
    pub const WIFI_2_BAR: IconData = material('\u{e4d9}');
    /// `wifi_calling` (U+EF77).
    pub const WIFI_CALLING: IconData = material('\u{ef77}');
    /// `wifi_calling_3` (U+F085).
    pub const WIFI_CALLING_3: IconData = material('\u{f085}');
    /// `wifi_channel` (U+EB6A).
    pub const WIFI_CHANNEL: IconData = material('\u{eb6a}');
    /// `wifi_find` (U+EB31).
    pub const WIFI_FIND: IconData = material('\u{eb31}');
    /// `wifi_lock` (U+E1E1).
    pub const WIFI_LOCK: IconData = material('\u{e1e1}');
    /// `wifi_off` (U+E648).
    pub const WIFI_OFF: IconData = material('\u{e648}');
    /// `wifi_password` (U+EB6B).
    pub const WIFI_PASSWORD: IconData = material('\u{eb6b}');
    /// `wifi_protected_setup` (U+F0FC).
    pub const WIFI_PROTECTED_SETUP: IconData = material('\u{f0fc}');
    /// `wifi_tethering` (U+E1E2).
    pub const WIFI_TETHERING: IconData = material('\u{e1e2}');
    /// `wifi_tethering_error` (U+EAD9).
    pub const WIFI_TETHERING_ERROR: IconData = material('\u{ead9}');
    /// `wifi_tethering_error_rounded` (U+F086).
    pub const WIFI_TETHERING_ERROR_ROUNDED: IconData = material('\u{f086}');
    /// `wifi_tethering_off` (U+F087).
    pub const WIFI_TETHERING_OFF: IconData = material('\u{f087}');
    /// `wind_power` (U+EC0C).
    pub const WIND_POWER: IconData = material('\u{ec0c}');
    /// `window` (U+F088).
    pub const WINDOW: IconData = material('\u{f088}');
    /// `wine_bar` (U+F1E8).
    pub const WINE_BAR: IconData = material('\u{f1e8}');
    /// `woman` (U+E13E).
    pub const WOMAN: IconData = material('\u{e13e}');
    /// `woman_2` (U+F8E7).
    pub const WOMAN_2: IconData = material('\u{f8e7}');
    /// `woo_commerce` (U+EA6D).
    pub const WOO_COMMERCE: IconData = material('\u{ea6d}');
    /// `wordpress` (U+EA9F).
    pub const WORDPRESS: IconData = material('\u{ea9f}');
    /// `work` (U+E8F9).
    pub const WORK: IconData = material('\u{e8f9}');
    /// `work_history` (U+EC09).
    pub const WORK_HISTORY: IconData = material('\u{ec09}');
    /// `work_off` (U+E942).
    pub const WORK_OFF: IconData = material('\u{e942}');
    /// `work_outline` (U+E943).
    pub const WORK_OUTLINE: IconData = material('\u{e943}');
    /// `workspace_premium` (U+E7AF).
    pub const WORKSPACE_PREMIUM: IconData = material('\u{e7af}');
    /// `workspaces` (U+E1A0).
    pub const WORKSPACES: IconData = material('\u{e1a0}');
    /// `workspaces_filled` (U+EA0D).
    pub const WORKSPACES_FILLED: IconData = material('\u{ea0d}');
    /// `workspaces_outline` (U+EA0F).
    pub const WORKSPACES_OUTLINE: IconData = material('\u{ea0f}');
    /// `wrap_text` (U+E25B).
    pub const WRAP_TEXT: IconData = material('\u{e25b}');
    /// `wrong_location` (U+EF78).
    pub const WRONG_LOCATION: IconData = material('\u{ef78}');
    /// `wysiwyg` (U+F1C3).
    pub const WYSIWYG: IconData = material('\u{f1c3}');
    /// `yard` (U+F089).
    pub const YARD: IconData = material('\u{f089}');
    /// `youtube_searched_for` (U+E8FA).
    pub const YOUTUBE_SEARCHED_FOR: IconData = material('\u{e8fa}');
    /// `zoom_in` (U+E8FF).
    pub const ZOOM_IN: IconData = material('\u{e8ff}');
    /// `zoom_in_map` (U+EB2D).
    pub const ZOOM_IN_MAP: IconData = material('\u{eb2d}');
    /// `zoom_out` (U+E900).
    pub const ZOOM_OUT: IconData = material('\u{e900}');
    /// `zoom_out_map` (U+E56B).
    pub const ZOOM_OUT_MAP: IconData = material('\u{e56b}');
}
