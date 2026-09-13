use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use crate::derive::derive_note_fields;
use crate::model::{CareerApplication, PipelineMetrics};

static RE_REPORT_LINK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[(\d+)\]\(([^)]+)\)").unwrap()
});

static RE_SCORE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(\d+\.?\d*)/5").unwrap()
});

pub fn find_repo_root() -> PathBuf {
    if let Ok(root_env) = std::env::var("CAREER_OPS_ROOT") {
        let p = PathBuf::from(root_env.trim());
        if p.exists() {
            return p;
        }
    }
    if let Ok(data_env) = std::env::var("CAREER_OPS_DATA_DIR") {
        let p = PathBuf::from(data_env.trim());
        if p.exists() {
            return p;
        }
    }

    let mut cur = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    loop {
        if cur.join("path-resolver.mjs").exists() || cur.join("DATA_CONTRACT.md").exists() {
            return cur;
        }
        if let Some(parent) = cur.parent() {
            cur = parent.to_path_buf();
        } else {
            break;
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

pub fn resolve_tracker_path(repo_root: &Path) -> PathBuf {
    if let Ok(env_tracker) = std::env::var("CAREER_OPS_TRACKER") {
        let trimmed = env_tracker.trim();
        if !trimmed.is_empty() {
            let p = PathBuf::from(trimmed);
            if p.is_absolute() {
                return p;
            } else {
                return repo_root.join(p);
            }
        }
    }

    let data_apps = repo_root.join("data").join("applications.md");
    if data_apps.exists() {
        return data_apps;
    }
    repo_root.join("applications.md")
}

pub fn split_markdown_row(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') {
        return Vec::new();
    }
    let inner = &trimmed[1..];
    let end = if inner.ends_with('|') {
        &inner[..inner.len() - 1]
    } else {
        inner
    };

    end.split('|')
        .map(|s| s.trim().to_string())
        .collect()
}

fn resolve_column_indices(header_cells: &[String]) -> HashMap<String, usize> {
    let mut cols = HashMap::new();
    for (i, cell) in header_cells.iter().enumerate() {
        let lower = cell.to_lowercase();
        if lower == "#" || lower.contains("num") {
            cols.insert("num".to_string(), i);
        } else if lower.contains("date") || lower.contains("fecha") {
            cols.insert("date".to_string(), i);
        } else if lower.contains("company") || lower.contains("empresa") {
            cols.insert("company".to_string(), i);
        } else if lower.contains("role") || lower.contains("rol") || lower.contains("title") {
            cols.insert("role".to_string(), i);
        } else if lower.contains("url") || lower.contains("link") {
            cols.insert("url".to_string(), i);
        } else if lower.contains("status") || lower.contains("estado") {
            cols.insert("status".to_string(), i);
        } else if lower.contains("pdf") || lower.contains("cv") {
            cols.insert("pdf".to_string(), i);
        } else if lower.contains("score") || lower.contains("puntos") || lower.contains("fit") {
            cols.insert("score".to_string(), i);
        } else if lower.contains("note") || lower.contains("nota") {
            cols.insert("notes".to_string(), i);
        } else if lower.contains("location") || lower.contains("ubic") {
            cols.insert("location".to_string(), i);
        }
    }
    cols
}

pub fn parse_applications(repo_root: &Path) -> Vec<CareerApplication> {
    let tracker_file = resolve_tracker_path(repo_root);
    let content = match fs::read_to_string(&tracker_file) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let lines: Vec<&str> = content.lines().collect();
    let mut cols = HashMap::new();
    let mut found_header = false;

    for line in &lines {
        let trimmed = line.trim();
        if trimmed.starts_with("| #") || (trimmed.starts_with('|') && trimmed.to_lowercase().contains("company")) {
            let cells = split_markdown_row(trimmed);
            cols = resolve_column_indices(&cells);
            found_header = true;
            break;
        }
    }

    // Default fallback columns if header wasn't found
    if !found_header {
        cols.insert("num".to_string(), 0);
        cols.insert("date".to_string(), 1);
        cols.insert("company".to_string(), 2);
        cols.insert("role".to_string(), 3);
        cols.insert("url".to_string(), 4);
        cols.insert("status".to_string(), 5);
        cols.insert("pdf".to_string(), 6);
        cols.insert("score".to_string(), 7);
        cols.insert("notes".to_string(), 8);
    }

    let mut apps = Vec::new();
    let mut auto_num = 0;

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed.starts_with("# ")
            || trimmed.starts_with("|---")
            || trimmed.starts_with("|:---")
            || trimmed.starts_with("| #")
            || (trimmed.starts_with('|') && trimmed.to_lowercase().contains("empresa") && !trimmed.contains("202"))
        {
            continue;
        }
        if !trimmed.starts_with('|') {
            continue;
        }

        let fields = split_markdown_row(trimmed);
        if fields.len() < 7 {
            continue;
        }

        let at = |name: &str| -> String {
            if let Some(&idx) = cols.get(name) {
                if idx < fields.len() {
                    return fields[idx].clone();
                }
            }
            String::new()
        };

        auto_num += 1;
        let mut app = CareerApplication::default();

        // 1. Number and report link
        let raw_num_cell = at("num");
        if let Some(caps) = RE_REPORT_LINK.captures(&raw_num_cell) {
            if let Some(num_match) = caps.get(1) {
                app.report_number = num_match.as_str().to_string();
                if let Ok(n) = num_match.as_str().parse::<i32>() {
                    app.number = n;
                } else {
                    app.number = auto_num;
                }
            }
            if let Some(path_match) = caps.get(2) {
                app.report_path = path_match.as_str().to_string();
            }
        } else {
            // Strip markdown brackets/parens if any
            let clean_num: String = raw_num_cell.chars().filter(|c| c.is_ascii_digit()).collect();
            if let Ok(n) = clean_num.parse::<i32>() {
                app.number = n;
            } else {
                app.number = auto_num;
            }
        }

        app.date = at("date");
        app.company = at("company");
        app.role = at("role");
        app.job_url = at("url");
        app.status = at("status");
        app.has_pdf = at("pdf").contains('✅') || at("pdf").to_lowercase().contains("yes");
        app.notes = at("notes");

        // Explicit location column if available
        let explicit_loc = at("location");
        if !explicit_loc.is_empty() {
            app.location = explicit_loc;
        }

        // Score
        let raw_score = at("score");
        app.score_raw = raw_score.clone();
        if let Some(caps) = RE_SCORE.captures(&raw_score) {
            if let Some(m) = caps.get(1) {
                if let Ok(v) = m.as_str().parse::<f64>() {
                    app.score = v;
                }
            }
        } else if let Ok(v) = raw_score.trim().parse::<f64>() {
            app.score = v;
        }

        // Derive note fields (Location, WorkMode, Pay, Dates)
        derive_note_fields(&mut app);

        apps.push(app);
    }

    apps
}

pub fn compute_metrics(apps: &[CareerApplication]) -> PipelineMetrics {
    let mut metrics = PipelineMetrics {
        total: apps.len(),
        ..Default::default()
    };

    let mut total_score = 0.0;
    let mut score_count = 0;

    for app in apps {
        let norm_status = app.status.trim().to_lowercase();
        match norm_status.as_str() {
            "applied" | "aplicada" => metrics.applied += 1,
            "interview" | "entrevista" | "screening" => metrics.interview += 1,
            "responded" | "respondido" => metrics.responded += 1,
            "offer" | "oferta" => metrics.offer += 1,
            "hired" | "contratado" => metrics.hired += 1,
            "discarded" | "descartada" | "skip" => metrics.discarded += 1,
            "rejected" | "rechazada" => metrics.rejected += 1,
            _ => {}
        }

        if app.score > 0.0 {
            metrics.evaluated += 1;
            total_score += app.score;
            score_count += 1;
            if app.score > metrics.top_score {
                metrics.top_score = app.score;
            }
        }

        if app.has_pdf {
            metrics.with_pdf += 1;
        }
    }

    if score_count > 0 {
        metrics.avg_score = (total_score / score_count as f64 * 10.0).round() / 10.0;
    }

    metrics
}

pub fn append_application(
    repo_root: &Path,
    company: &str,
    role: &str,
    url: &str,
    status: &str,
    location: &str,
    pay: &str,
    notes: &str,
) -> Result<CareerApplication, std::io::Error> {
    use std::io::Write;

    let tracker_file = resolve_tracker_path(repo_root);
    if let Some(parent) = tracker_file.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    let now = chrono::Local::now().format("%Y-%m-%d").to_string();
    let current_apps = parse_applications(repo_root);
    let next_num = current_apps.iter().map(|a| a.number).max().unwrap_or(0) + 1;

    let mut note_parts = Vec::new();
    if !location.is_empty() {
        note_parts.push(location.to_string());
    }
    if !pay.is_empty() {
        note_parts.push(pay.to_string());
    }
    if !notes.is_empty() {
        note_parts.push(notes.to_string());
    }
    let combined_notes = note_parts.join("; ");

    let status_str = if status.is_empty() { "Applied" } else { status };
    let url_str = if url.is_empty() { "—" } else { url };

    if !tracker_file.exists() {
        let header = "# Applications\n\n| # | Date | Company | Role | URL | Status | PDF | Score | Notes |\n|---|---|---|---|---|---|---|---|---|\n";
        fs::write(&tracker_file, header)?;
    }

    let row = format!(
        "| {} | {} | {} | {} | {} | {} | — | — | {} |\n",
        next_num, now, company, role, url_str, status_str, combined_notes
    );

    let mut file = fs::OpenOptions::new().create(true).append(true).open(&tracker_file)?;
    file.write_all(row.as_bytes())?;

    let mut app = CareerApplication {
        number: next_num,
        date: now,
        company: company.to_string(),
        role: role.to_string(),
        job_url: if url_str == "—" { String::new() } else { url_str.to_string() },
        status: status_str.to_string(),
        notes: combined_notes,
        ..Default::default()
    };
    derive_note_fields(&mut app);
    Ok(app)
}

