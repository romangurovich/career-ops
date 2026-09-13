use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

static RE_ARCHETYPE_PIPE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\*\*(?:Arquetipo|Archetype)(?:\s+(?:detectado|detected))?\*\*\s*\|\s*(.+)").unwrap()
});
static RE_ARCHETYPE_COLON: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\*\*(?:Arquetipo|Archetype):\*\*\s*(.+)").unwrap()
});
static RE_ARCHETYPE_YAML: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?m)^archetype:\s*"?([^"\n]+)"?\s*$"#).unwrap()
});

static RE_TLDR_PIPE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\*\*TL;DR\*\*\s*\|\s*(.+)").unwrap()
});
static RE_TLDR_COLON: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\*\*TL;DR:\*\*\s*(.+)").unwrap()
});

static RE_REMOTE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\*\*Remote\*\*\s*\|\s*(.+)").unwrap()
});

static RE_COMP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\*\*Comp\*\*\s*\|\s*(.+)").unwrap()
});

static RE_URL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\*\*URL:\*\*\s*(https?://\S+)").unwrap()
});

#[derive(Debug, Clone, Default)]
pub struct ReportSummary {
    pub archetype: String,
    pub tldr: String,
    pub remote: String,
    pub comp: String,
    pub job_url: String,
    pub full_content: String,
}

pub fn load_report_summary(career_ops_root: &Path, rel_report_path: &str) -> ReportSummary {
    let path = if Path::new(rel_report_path).is_absolute() {
        PathBuf::from(rel_report_path)
    } else {
        career_ops_root.join(rel_report_path)
    };

    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => {
            // Also try joining with "reports" if relative
            let alt_path = career_ops_root.join("reports").join(rel_report_path);
            match fs::read_to_string(&alt_path) {
                Ok(c) => c,
                Err(_) => return ReportSummary::default(),
            }
        }
    };

    let mut summary = ReportSummary {
        full_content: content.clone(),
        ..Default::default()
    };

    // Archetype
    if let Some(caps) = RE_ARCHETYPE_PIPE.captures(&content) {
        if let Some(m) = caps.get(1) {
            summary.archetype = m.as_str().trim().to_string();
        }
    } else if let Some(caps) = RE_ARCHETYPE_COLON.captures(&content) {
        if let Some(m) = caps.get(1) {
            summary.archetype = m.as_str().trim().to_string();
        }
    } else if let Some(caps) = RE_ARCHETYPE_YAML.captures(&content) {
        if let Some(m) = caps.get(1) {
            summary.archetype = m.as_str().trim().to_string();
        }
    }

    // TL;DR
    if let Some(caps) = RE_TLDR_PIPE.captures(&content) {
        if let Some(m) = caps.get(1) {
            summary.tldr = m.as_str().trim().to_string();
        }
    } else if let Some(caps) = RE_TLDR_COLON.captures(&content) {
        if let Some(m) = caps.get(1) {
            summary.tldr = m.as_str().trim().to_string();
        }
    }

    // Remote
    if let Some(caps) = RE_REMOTE.captures(&content) {
        if let Some(m) = caps.get(1) {
            summary.remote = m.as_str().trim().to_string();
        }
    }

    // Comp
    if let Some(caps) = RE_COMP.captures(&content) {
        if let Some(m) = caps.get(1) {
            summary.comp = m.as_str().trim().to_string();
        }
    }

    // URL
    if let Some(caps) = RE_URL.captures(&content) {
        if let Some(m) = caps.get(1) {
            summary.job_url = m.as_str().trim().to_string();
        }
    }

    summary
}
