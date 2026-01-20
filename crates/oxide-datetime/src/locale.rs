//! Locale handling for date and time components
//!
//! Provides locale-aware formatting for month names, day names, and date formats.

use chrono::{Datelike, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};

/// Supported locales for date/time formatting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Locale {
    /// English (United States)
    #[default]
    EnUs,
    /// English (United Kingdom)
    EnGb,
    /// German (Germany)
    DeDe,
    /// French (France)
    FrFr,
    /// Spanish (Spain)
    EsEs,
    /// Italian (Italy)
    ItIt,
    /// Portuguese (Brazil)
    PtBr,
    /// Dutch (Netherlands)
    NlNl,
    /// Japanese (Japan)
    JaJp,
    /// Chinese (Simplified)
    ZhCn,
    /// Korean (Korea)
    KoKr,
    /// Russian (Russia)
    RuRu,
    /// Arabic (Saudi Arabia)
    ArSa,
    /// Hebrew (Israel)
    HeIl,
}

impl Locale {
    /// Get locale code string (e.g., "en-US")
    pub fn code(&self) -> &'static str {
        match self {
            Locale::EnUs => "en-US",
            Locale::EnGb => "en-GB",
            Locale::DeDe => "de-DE",
            Locale::FrFr => "fr-FR",
            Locale::EsEs => "es-ES",
            Locale::ItIt => "it-IT",
            Locale::PtBr => "pt-BR",
            Locale::NlNl => "nl-NL",
            Locale::JaJp => "ja-JP",
            Locale::ZhCn => "zh-CN",
            Locale::KoKr => "ko-KR",
            Locale::RuRu => "ru-RU",
            Locale::ArSa => "ar-SA",
            Locale::HeIl => "he-IL",
        }
    }

    /// Parse locale from string
    pub fn from_code(code: &str) -> Option<Self> {
        match code.to_lowercase().replace('_', "-").as_str() {
            "en-us" | "en" => Some(Locale::EnUs),
            "en-gb" => Some(Locale::EnGb),
            "de-de" | "de" => Some(Locale::DeDe),
            "fr-fr" | "fr" => Some(Locale::FrFr),
            "es-es" | "es" => Some(Locale::EsEs),
            "it-it" | "it" => Some(Locale::ItIt),
            "pt-br" | "pt" => Some(Locale::PtBr),
            "nl-nl" | "nl" => Some(Locale::NlNl),
            "ja-jp" | "ja" => Some(Locale::JaJp),
            "zh-cn" | "zh" => Some(Locale::ZhCn),
            "ko-kr" | "ko" => Some(Locale::KoKr),
            "ru-ru" | "ru" => Some(Locale::RuRu),
            "ar-sa" | "ar" => Some(Locale::ArSa),
            "he-il" | "he" => Some(Locale::HeIl),
            _ => None,
        }
    }

    /// Check if this locale uses right-to-left text direction
    pub fn is_rtl(&self) -> bool {
        matches!(self, Locale::ArSa | Locale::HeIl)
    }

    /// Get the default week start for this locale
    pub fn default_week_start(&self) -> crate::utils::WeekStart {
        match self {
            // US, Japan, and most Asian countries start on Sunday
            Locale::EnUs | Locale::JaJp | Locale::ZhCn | Locale::KoKr | Locale::ArSa | Locale::HeIl => {
                crate::utils::WeekStart::Sunday
            }
            // Most of Europe, UK, and Latin America start on Monday
            _ => crate::utils::WeekStart::Monday,
        }
    }
}

/// Locale-aware month names
pub struct MonthNames {
    /// Full month names
    pub long: [&'static str; 12],
    /// Abbreviated month names (3-letter)
    pub short: [&'static str; 12],
}

impl Locale {
    /// Get month names for this locale
    pub fn month_names(&self) -> MonthNames {
        match self {
            Locale::EnUs | Locale::EnGb => MonthNames {
                long: [
                    "January", "February", "March", "April", "May", "June",
                    "July", "August", "September", "October", "November", "December",
                ],
                short: [
                    "Jan", "Feb", "Mar", "Apr", "May", "Jun",
                    "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
                ],
            },
            Locale::DeDe => MonthNames {
                long: [
                    "Januar", "Februar", "Marz", "April", "Mai", "Juni",
                    "Juli", "August", "September", "Oktober", "November", "Dezember",
                ],
                short: [
                    "Jan", "Feb", "Mar", "Apr", "Mai", "Jun",
                    "Jul", "Aug", "Sep", "Okt", "Nov", "Dez",
                ],
            },
            Locale::FrFr => MonthNames {
                long: [
                    "janvier", "fevrier", "mars", "avril", "mai", "juin",
                    "juillet", "aout", "septembre", "octobre", "novembre", "decembre",
                ],
                short: [
                    "janv.", "fevr.", "mars", "avr.", "mai", "juin",
                    "juil.", "aout", "sept.", "oct.", "nov.", "dec.",
                ],
            },
            Locale::EsEs => MonthNames {
                long: [
                    "enero", "febrero", "marzo", "abril", "mayo", "junio",
                    "julio", "agosto", "septiembre", "octubre", "noviembre", "diciembre",
                ],
                short: [
                    "ene.", "feb.", "mar.", "abr.", "may.", "jun.",
                    "jul.", "ago.", "sept.", "oct.", "nov.", "dic.",
                ],
            },
            Locale::ItIt => MonthNames {
                long: [
                    "gennaio", "febbraio", "marzo", "aprile", "maggio", "giugno",
                    "luglio", "agosto", "settembre", "ottobre", "novembre", "dicembre",
                ],
                short: [
                    "gen", "feb", "mar", "apr", "mag", "giu",
                    "lug", "ago", "set", "ott", "nov", "dic",
                ],
            },
            Locale::PtBr => MonthNames {
                long: [
                    "janeiro", "fevereiro", "marco", "abril", "maio", "junho",
                    "julho", "agosto", "setembro", "outubro", "novembro", "dezembro",
                ],
                short: [
                    "jan", "fev", "mar", "abr", "mai", "jun",
                    "jul", "ago", "set", "out", "nov", "dez",
                ],
            },
            Locale::NlNl => MonthNames {
                long: [
                    "januari", "februari", "maart", "april", "mei", "juni",
                    "juli", "augustus", "september", "oktober", "november", "december",
                ],
                short: [
                    "jan", "feb", "mrt", "apr", "mei", "jun",
                    "jul", "aug", "sep", "okt", "nov", "dec",
                ],
            },
            Locale::JaJp => MonthNames {
                long: [
                    "1月", "2月", "3月", "4月", "5月", "6月",
                    "7月", "8月", "9月", "10月", "11月", "12月",
                ],
                short: [
                    "1月", "2月", "3月", "4月", "5月", "6月",
                    "7月", "8月", "9月", "10月", "11月", "12月",
                ],
            },
            Locale::ZhCn => MonthNames {
                long: [
                    "一月", "二月", "三月", "四月", "五月", "六月",
                    "七月", "八月", "九月", "十月", "十一月", "十二月",
                ],
                short: [
                    "1月", "2月", "3月", "4月", "5月", "6月",
                    "7月", "8月", "9月", "10月", "11月", "12月",
                ],
            },
            Locale::KoKr => MonthNames {
                long: [
                    "1월", "2월", "3월", "4월", "5월", "6월",
                    "7월", "8월", "9월", "10월", "11월", "12월",
                ],
                short: [
                    "1월", "2월", "3월", "4월", "5월", "6월",
                    "7월", "8월", "9월", "10월", "11월", "12월",
                ],
            },
            Locale::RuRu => MonthNames {
                long: [
                    "Январь", "Февраль", "Март", "Апрель", "Май", "Июнь",
                    "Июль", "Август", "Сентябрь", "Октябрь", "Ноябрь", "Декабрь",
                ],
                short: [
                    "янв", "фев", "мар", "апр", "май", "июн",
                    "июл", "авг", "сен", "окт", "ноя", "дек",
                ],
            },
            Locale::ArSa => MonthNames {
                long: [
                    "يناير", "فبراير", "مارس", "أبريل", "مايو", "يونيو",
                    "يوليو", "أغسطس", "سبتمبر", "أكتوبر", "نوفمبر", "ديسمبر",
                ],
                short: [
                    "ينا", "فبر", "مار", "أبر", "ماي", "يون",
                    "يول", "أغس", "سبت", "أكت", "نوف", "ديس",
                ],
            },
            Locale::HeIl => MonthNames {
                long: [
                    "ינואר", "פברואר", "מרץ", "אפריל", "מאי", "יוני",
                    "יולי", "אוגוסט", "ספטמבר", "אוקטובר", "נובמבר", "דצמבר",
                ],
                short: [
                    "ינו", "פבר", "מרץ", "אפר", "מאי", "יונ",
                    "יול", "אוג", "ספט", "אוק", "נוב", "דצמ",
                ],
            },
        }
    }

    /// Get the month name for a given month (1-12)
    pub fn month_name(&self, month: u32, short: bool) -> &'static str {
        let names = self.month_names();
        let idx = (month.saturating_sub(1) as usize).min(11);
        if short {
            names.short[idx]
        } else {
            names.long[idx]
        }
    }
}

/// Locale-aware weekday names
pub struct WeekdayNames {
    /// Full weekday names
    pub long: [&'static str; 7],
    /// Short weekday names (2-3 letters)
    pub short: [&'static str; 7],
    /// Narrow weekday names (1-2 letters)
    pub narrow: [&'static str; 7],
}

impl Locale {
    /// Get weekday names for this locale (starting from Sunday)
    pub fn weekday_names(&self) -> WeekdayNames {
        match self {
            Locale::EnUs | Locale::EnGb => WeekdayNames {
                long: ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"],
                short: ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"],
                narrow: ["S", "M", "T", "W", "T", "F", "S"],
            },
            Locale::DeDe => WeekdayNames {
                long: ["Sonntag", "Montag", "Dienstag", "Mittwoch", "Donnerstag", "Freitag", "Samstag"],
                short: ["So", "Mo", "Di", "Mi", "Do", "Fr", "Sa"],
                narrow: ["S", "M", "D", "M", "D", "F", "S"],
            },
            Locale::FrFr => WeekdayNames {
                long: ["dimanche", "lundi", "mardi", "mercredi", "jeudi", "vendredi", "samedi"],
                short: ["dim.", "lun.", "mar.", "mer.", "jeu.", "ven.", "sam."],
                narrow: ["D", "L", "M", "M", "J", "V", "S"],
            },
            Locale::EsEs => WeekdayNames {
                long: ["domingo", "lunes", "martes", "miercoles", "jueves", "viernes", "sabado"],
                short: ["dom.", "lun.", "mar.", "mie.", "jue.", "vie.", "sab."],
                narrow: ["D", "L", "M", "X", "J", "V", "S"],
            },
            Locale::ItIt => WeekdayNames {
                long: ["domenica", "lunedi", "martedi", "mercoledi", "giovedi", "venerdi", "sabato"],
                short: ["dom", "lun", "mar", "mer", "gio", "ven", "sab"],
                narrow: ["D", "L", "M", "M", "G", "V", "S"],
            },
            Locale::PtBr => WeekdayNames {
                long: ["domingo", "segunda-feira", "terca-feira", "quarta-feira", "quinta-feira", "sexta-feira", "sabado"],
                short: ["dom", "seg", "ter", "qua", "qui", "sex", "sab"],
                narrow: ["D", "S", "T", "Q", "Q", "S", "S"],
            },
            Locale::NlNl => WeekdayNames {
                long: ["zondag", "maandag", "dinsdag", "woensdag", "donderdag", "vrijdag", "zaterdag"],
                short: ["zo", "ma", "di", "wo", "do", "vr", "za"],
                narrow: ["Z", "M", "D", "W", "D", "V", "Z"],
            },
            Locale::JaJp => WeekdayNames {
                long: ["日曜日", "月曜日", "火曜日", "水曜日", "木曜日", "金曜日", "土曜日"],
                short: ["日", "月", "火", "水", "木", "金", "土"],
                narrow: ["日", "月", "火", "水", "木", "金", "土"],
            },
            Locale::ZhCn => WeekdayNames {
                long: ["星期日", "星期一", "星期二", "星期三", "星期四", "星期五", "星期六"],
                short: ["周日", "周一", "周二", "周三", "周四", "周五", "周六"],
                narrow: ["日", "一", "二", "三", "四", "五", "六"],
            },
            Locale::KoKr => WeekdayNames {
                long: ["일요일", "월요일", "화요일", "수요일", "목요일", "금요일", "토요일"],
                short: ["일", "월", "화", "수", "목", "금", "토"],
                narrow: ["일", "월", "화", "수", "목", "금", "토"],
            },
            Locale::RuRu => WeekdayNames {
                long: ["воскресенье", "понедельник", "вторник", "среда", "четверг", "пятница", "суббота"],
                short: ["вс", "пн", "вт", "ср", "чт", "пт", "сб"],
                narrow: ["В", "П", "В", "С", "Ч", "П", "С"],
            },
            Locale::ArSa => WeekdayNames {
                long: ["الأحد", "الاثنين", "الثلاثاء", "الأربعاء", "الخميس", "الجمعة", "السبت"],
                short: ["أحد", "إثن", "ثلا", "أرب", "خمي", "جمع", "سبت"],
                narrow: ["ح", "ن", "ث", "ر", "خ", "ج", "س"],
            },
            Locale::HeIl => WeekdayNames {
                long: ["יום ראשון", "יום שני", "יום שלישי", "יום רביעי", "יום חמישי", "יום שישי", "יום שבת"],
                short: ["א׳", "ב׳", "ג׳", "ד׳", "ה׳", "ו׳", "ש׳"],
                narrow: ["א", "ב", "ג", "ד", "ה", "ו", "ש"],
            },
        }
    }

    /// Get the weekday name for a given weekday
    pub fn weekday_name(&self, weekday: Weekday, format: WeekdayFormat) -> &'static str {
        let names = self.weekday_names();
        let idx = weekday.num_days_from_sunday() as usize;
        match format {
            WeekdayFormat::Long => names.long[idx],
            WeekdayFormat::Short => names.short[idx],
            WeekdayFormat::Narrow => names.narrow[idx],
        }
    }

    /// Get weekday names in the order for calendar display
    pub fn weekday_names_for_calendar(&self, week_start: crate::utils::WeekStart) -> Vec<&'static str> {
        let names = self.weekday_names();
        let start_idx = match week_start {
            crate::utils::WeekStart::Sunday => 0,
            crate::utils::WeekStart::Monday => 1,
        };
        let mut result = Vec::with_capacity(7);
        for i in 0..7 {
            result.push(names.short[(start_idx + i) % 7]);
        }
        result
    }
}

/// Format style for weekday names
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeekdayFormat {
    /// Full name (e.g., "Monday")
    Long,
    /// Short name (e.g., "Mon")
    Short,
    /// Narrow name (e.g., "M")
    Narrow,
}

/// Date format pattern for different locales
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DateFormatPattern {
    /// MM/DD/YYYY (US style)
    #[default]
    MonthDayYear,
    /// DD/MM/YYYY (European style)
    DayMonthYear,
    /// YYYY/MM/DD (ISO style)
    YearMonthDay,
}

impl Locale {
    /// Get the preferred date format pattern for this locale
    pub fn date_format_pattern(&self) -> DateFormatPattern {
        match self {
            Locale::EnUs => DateFormatPattern::MonthDayYear,
            Locale::JaJp | Locale::ZhCn | Locale::KoKr => DateFormatPattern::YearMonthDay,
            _ => DateFormatPattern::DayMonthYear,
        }
    }

    /// Format a date according to locale conventions
    pub fn format_date(&self, date: NaiveDate, short: bool) -> String {
        let pattern = self.date_format_pattern();
        let month_name = self.month_name(date.month(), short);

        match pattern {
            DateFormatPattern::MonthDayYear => {
                if short {
                    format!("{}/{}/{}", date.month(), date.day(), date.year())
                } else {
                    format!("{} {}, {}", month_name, date.day(), date.year())
                }
            }
            DateFormatPattern::DayMonthYear => {
                if short {
                    format!("{}/{}/{}", date.day(), date.month(), date.year())
                } else {
                    format!("{} {} {}", date.day(), month_name, date.year())
                }
            }
            DateFormatPattern::YearMonthDay => {
                if short {
                    format!("{}/{}/{}", date.year(), date.month(), date.day())
                } else {
                    format!("{}年{}月{}日", date.year(), date.month(), date.day())
                }
            }
        }
    }
}

/// Labels used in date/time picker UI
#[derive(Debug, Clone)]
pub struct DateTimeLabels {
    pub today: &'static str,
    pub clear: &'static str,
    pub cancel: &'static str,
    pub ok: &'static str,
    pub select_date: &'static str,
    pub select_time: &'static str,
    pub select_month: &'static str,
    pub select_year: &'static str,
    pub previous_month: &'static str,
    pub next_month: &'static str,
    pub previous_year: &'static str,
    pub next_year: &'static str,
    pub am: &'static str,
    pub pm: &'static str,
    pub start_date: &'static str,
    pub end_date: &'static str,
    pub preset_today: &'static str,
    pub preset_yesterday: &'static str,
    pub preset_last_7_days: &'static str,
    pub preset_last_30_days: &'static str,
    pub preset_this_month: &'static str,
    pub preset_last_month: &'static str,
    pub preset_custom: &'static str,
}

impl Locale {
    /// Get UI labels for this locale
    pub fn labels(&self) -> DateTimeLabels {
        match self {
            Locale::EnUs | Locale::EnGb => DateTimeLabels {
                today: "Today",
                clear: "Clear",
                cancel: "Cancel",
                ok: "OK",
                select_date: "Select date",
                select_time: "Select time",
                select_month: "Select month",
                select_year: "Select year",
                previous_month: "Previous month",
                next_month: "Next month",
                previous_year: "Previous year",
                next_year: "Next year",
                am: "AM",
                pm: "PM",
                start_date: "Start date",
                end_date: "End date",
                preset_today: "Today",
                preset_yesterday: "Yesterday",
                preset_last_7_days: "Last 7 days",
                preset_last_30_days: "Last 30 days",
                preset_this_month: "This month",
                preset_last_month: "Last month",
                preset_custom: "Custom",
            },
            Locale::DeDe => DateTimeLabels {
                today: "Heute",
                clear: "Loschen",
                cancel: "Abbrechen",
                ok: "OK",
                select_date: "Datum auswahlen",
                select_time: "Uhrzeit auswahlen",
                select_month: "Monat auswahlen",
                select_year: "Jahr auswahlen",
                previous_month: "Vorheriger Monat",
                next_month: "Nachster Monat",
                previous_year: "Vorheriges Jahr",
                next_year: "Nachstes Jahr",
                am: "AM",
                pm: "PM",
                start_date: "Startdatum",
                end_date: "Enddatum",
                preset_today: "Heute",
                preset_yesterday: "Gestern",
                preset_last_7_days: "Letzte 7 Tage",
                preset_last_30_days: "Letzte 30 Tage",
                preset_this_month: "Dieser Monat",
                preset_last_month: "Letzter Monat",
                preset_custom: "Benutzerdefiniert",
            },
            Locale::FrFr => DateTimeLabels {
                today: "Aujourd'hui",
                clear: "Effacer",
                cancel: "Annuler",
                ok: "OK",
                select_date: "Selectionner une date",
                select_time: "Selectionner l'heure",
                select_month: "Selectionner le mois",
                select_year: "Selectionner l'annee",
                previous_month: "Mois precedent",
                next_month: "Mois suivant",
                previous_year: "Annee precedente",
                next_year: "Annee suivante",
                am: "AM",
                pm: "PM",
                start_date: "Date de debut",
                end_date: "Date de fin",
                preset_today: "Aujourd'hui",
                preset_yesterday: "Hier",
                preset_last_7_days: "7 derniers jours",
                preset_last_30_days: "30 derniers jours",
                preset_this_month: "Ce mois",
                preset_last_month: "Mois dernier",
                preset_custom: "Personnalise",
            },
            Locale::EsEs => DateTimeLabels {
                today: "Hoy",
                clear: "Borrar",
                cancel: "Cancelar",
                ok: "OK",
                select_date: "Seleccionar fecha",
                select_time: "Seleccionar hora",
                select_month: "Seleccionar mes",
                select_year: "Seleccionar ano",
                previous_month: "Mes anterior",
                next_month: "Mes siguiente",
                previous_year: "Ano anterior",
                next_year: "Ano siguiente",
                am: "AM",
                pm: "PM",
                start_date: "Fecha de inicio",
                end_date: "Fecha de fin",
                preset_today: "Hoy",
                preset_yesterday: "Ayer",
                preset_last_7_days: "Ultimos 7 dias",
                preset_last_30_days: "Ultimos 30 dias",
                preset_this_month: "Este mes",
                preset_last_month: "Mes pasado",
                preset_custom: "Personalizado",
            },
            // For other locales, fall back to English
            _ => DateTimeLabels {
                today: "Today",
                clear: "Clear",
                cancel: "Cancel",
                ok: "OK",
                select_date: "Select date",
                select_time: "Select time",
                select_month: "Select month",
                select_year: "Select year",
                previous_month: "Previous month",
                next_month: "Next month",
                previous_year: "Previous year",
                next_year: "Next year",
                am: "AM",
                pm: "PM",
                start_date: "Start date",
                end_date: "End date",
                preset_today: "Today",
                preset_yesterday: "Yesterday",
                preset_last_7_days: "Last 7 days",
                preset_last_30_days: "Last 30 days",
                preset_this_month: "This month",
                preset_last_month: "Last month",
                preset_custom: "Custom",
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locale_from_code() {
        assert_eq!(Locale::from_code("en-US"), Some(Locale::EnUs));
        assert_eq!(Locale::from_code("en_US"), Some(Locale::EnUs));
        assert_eq!(Locale::from_code("de-DE"), Some(Locale::DeDe));
        assert_eq!(Locale::from_code("de"), Some(Locale::DeDe));
        assert_eq!(Locale::from_code("unknown"), None);
    }

    #[test]
    fn test_locale_code() {
        assert_eq!(Locale::EnUs.code(), "en-US");
        assert_eq!(Locale::DeDe.code(), "de-DE");
    }

    #[test]
    fn test_is_rtl() {
        assert!(!Locale::EnUs.is_rtl());
        assert!(!Locale::DeDe.is_rtl());
        assert!(Locale::ArSa.is_rtl());
        assert!(Locale::HeIl.is_rtl());
    }

    #[test]
    fn test_month_names() {
        assert_eq!(Locale::EnUs.month_name(1, false), "January");
        assert_eq!(Locale::EnUs.month_name(1, true), "Jan");
        assert_eq!(Locale::DeDe.month_name(1, false), "Januar");
        assert_eq!(Locale::JaJp.month_name(1, false), "1月");
    }

    #[test]
    fn test_weekday_names() {
        assert_eq!(
            Locale::EnUs.weekday_name(Weekday::Mon, WeekdayFormat::Long),
            "Monday"
        );
        assert_eq!(
            Locale::EnUs.weekday_name(Weekday::Mon, WeekdayFormat::Short),
            "Mon"
        );
        assert_eq!(
            Locale::EnUs.weekday_name(Weekday::Mon, WeekdayFormat::Narrow),
            "M"
        );
    }

    #[test]
    fn test_weekday_names_for_calendar() {
        let names_sunday = Locale::EnUs.weekday_names_for_calendar(crate::utils::WeekStart::Sunday);
        assert_eq!(names_sunday[0], "Sun");
        assert_eq!(names_sunday[1], "Mon");

        let names_monday = Locale::EnUs.weekday_names_for_calendar(crate::utils::WeekStart::Monday);
        assert_eq!(names_monday[0], "Mon");
        assert_eq!(names_monday[6], "Sun");
    }

    #[test]
    fn test_format_date() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        // US format (MM/DD/YYYY)
        assert_eq!(Locale::EnUs.format_date(date, true), "1/15/2024");
        assert_eq!(Locale::EnUs.format_date(date, false), "January 15, 2024");

        // European format (DD/MM/YYYY)
        assert_eq!(Locale::DeDe.format_date(date, true), "15/1/2024");

        // Japanese format (YYYY/MM/DD)
        assert_eq!(Locale::JaJp.format_date(date, true), "2024/1/15");
    }

    #[test]
    fn test_default_week_start() {
        assert_eq!(Locale::EnUs.default_week_start(), crate::utils::WeekStart::Sunday);
        assert_eq!(Locale::EnGb.default_week_start(), crate::utils::WeekStart::Monday);
        assert_eq!(Locale::DeDe.default_week_start(), crate::utils::WeekStart::Monday);
    }

    #[test]
    fn test_labels() {
        let labels = Locale::EnUs.labels();
        assert_eq!(labels.today, "Today");
        assert_eq!(labels.clear, "Clear");

        let labels_de = Locale::DeDe.labels();
        assert_eq!(labels_de.today, "Heute");
    }
}
