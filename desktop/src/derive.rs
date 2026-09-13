use regex::Regex;
use std::sync::LazyLock;
use crate::model::CareerApplication;

static RE_CITY_STATE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b([A-Z][A-Za-z.'-]+(?: [A-Z][A-Za-z.'-]+){0,2}),? (A[KLRZ]|C[AOT]|D[CE]|FL|GA|HI|I[ADLN]|K[SY]|LA|M[ADEINOST]|N[CDEHJMVY]|O[HKR]|PA|RI|S[CD]|T[NX]|UT|V[AT]|W[AIVY])\b").unwrap()
});

static RE_INTL_CITY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(Porto|Lisbon|London|Berlin|Munich|Hamburg|Frankfurt|Cologne|Düsseldorf|Stuttgart|Zurich|Geneva|Lausanne|Basel|Dublin|Cork|Amsterdam|Rotterdam|Eindhoven|Utrecht|Paris|Lyon|Madrid|Barcelona|Valencia|Stockholm|Gothenburg|Malmö|Copenhagen|Oslo|Helsinki|Milan|Rome|Turin|Vienna|Brussels|Ghent|Antwerp|Luxembourg|Warsaw|Kraków|Wrocław|Tallinn|Riga|Vilnius|Prague|Brno|Budapest|Bucharest|Sofia|Athens|Bengaluru|Bangalore|Singapore|Sydney|Toronto|Vancouver|Tel Aviv|São Paulo)\b").unwrap()
});

static RE_POSTED_ON: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?im)(?:^|[;|])[^\S\n]*posted(?::[^\S\n]*|[^\S\n]+)(20\d{2}-\d{2}-\d{2})\b").unwrap()
});

static RE_ISO_DATE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b20\d{2}-\d{2}-\d{2}\b").unwrap()
});

static RE_MONEY_SPAN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)[$€£¥₹₺₩CHFUSDGBP]\s*(\d[\d,]*(?:\.\d+)?)\s*([KkMmBb]?)\s*(?:[-–]\s*[$€£¥₹₺₩CHFUSDGBP]?\s*(\d[\d,]*(?:\.\d+)?)\s*([KkMmBb]?))?").unwrap()
});

static RE_FUNDING: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(valuation|(total\s+)?raised|series\s|round\b)").unwrap()
});

static RE_EST_HINT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\(est[),;. ]|\best\)|\bmarket\b").unwrap()
});

pub fn derive_note_fields(app: &mut CareerApplication) {
    let lower_notes = app.notes.to_lowercase();
    let lower_role = app.role.to_lowercase();
    let combined = format!("{} {}", lower_role, lower_notes);

    // 1. Location extraction
    if let Some(caps) = RE_CITY_STATE.captures(&app.notes) {
        if let (Some(city), Some(state)) = (caps.get(1), caps.get(2)) {
            app.location = format!("{}, {}", city.as_str(), state.as_str());
        }
    }
    if app.location.is_empty() {
        if let Some(caps) = RE_INTL_CITY.captures(&app.notes) {
            if let Some(city) = caps.get(1) {
                app.location = city.as_str().to_string();
            }
        }
    }
    if app.location.is_empty() {
        if let Some(caps) = RE_CITY_STATE.captures(&app.role) {
            if let (Some(city), Some(state)) = (caps.get(1), caps.get(2)) {
                app.location = format!("{}, {}", city.as_str(), state.as_str());
            }
        }
    }

    // 2. Work mode extraction
    if combined.contains("remote") || combined.contains("remoto") || combined.contains("telework") {
        app.work_mode = "Remote".to_string();
    } else if combined.contains("hybrid") || combined.contains("híbrido") || combined.contains("2-3 days") {
        app.work_mode = "Hybrid".to_string();
    } else if combined.contains("onsite") || combined.contains("on-site") || combined.contains("presencial") || combined.contains("in-office") {
        app.work_mode = "Full".to_string();
    }

    // 3. Pay extraction
    for caps in RE_MONEY_SPAN.captures_iter(&app.notes) {
        let whole_match = caps.get(0).map_or("", |m| m.as_str());
        // Skip if funding context follows
        let match_end = caps.get(0).map_or(0, |m| m.end());
        let trailing = &app.notes[match_end..std::cmp::min(app.notes.len(), match_end + 30)];
        if RE_FUNDING.is_match(trailing) {
            continue;
        }

        app.pay_range = whole_match.trim().to_string();

        let parse_part = |num_str: &str, unit: &str| -> f64 {
            let clean = num_str.replace(',', "");
            if let Ok(mut val) = clean.parse::<f64>() {
                match unit.to_lowercase().as_str() {
                    "k" => val *= 1_000.0,
                    "m" => val *= 1_000_000.0,
                    "b" => val *= 1_000_000_000.0,
                    _ => {
                        if val < 1000.0 {
                            val *= 1000.0; // e.g. "$140-210" without trailing K
                        }
                    }
                }
                val
            } else {
                0.0
            }
        };

        let low = caps.get(1).map_or("", |m| m.as_str());
        let low_unit = caps.get(2).map_or("", |m| m.as_str());
        let high = caps.get(3).map_or("", |m| m.as_str());
        let high_unit = caps.get(4).map_or("", |m| m.as_str());

        let mut max_val = parse_part(low, low_unit);
        if !high.is_empty() {
            let high_val = parse_part(high, if high_unit.is_empty() { low_unit } else { high_unit });
            if high_val > max_val {
                max_val = high_val;
            }
        }
        app.pay_max = max_val;

        if app.notes.contains("(POSTED)") || app.notes.contains("POSTED") {
            app.pay_source = "POSTED".to_string();
        } else if RE_EST_HINT.is_match(&app.notes) {
            app.pay_source = "est".to_string();
        }
        break;
    }

    // 4. Posted On date
    if let Some(caps) = RE_POSTED_ON.captures(&app.notes) {
        if let Some(date_match) = caps.get(1) {
            app.posted_on = date_match.as_str().to_string();
        }
    }

    // 5. Last Contact date
    let mut latest_date = app.date.clone();
    for mat in RE_ISO_DATE.find_iter(&app.notes) {
        let d = mat.as_str();
        if !app.posted_on.is_empty() && d == app.posted_on {
            continue;
        }
        if d > latest_date.as_str() {
            latest_date = d.to_string();
        }
    }
    app.last_contact = latest_date;
}
