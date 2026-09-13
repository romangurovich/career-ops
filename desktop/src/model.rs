use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CareerApplication {
    pub number: i32,
    pub date: String,
    pub company: String,
    pub role: String,
    pub status: String,
    pub score: f64,
    pub score_raw: String,
    pub has_pdf: bool,
    pub report_path: String,
    pub report_number: String,
    pub notes: String,
    pub job_url: String,

    // Derived from Notes free-text
    pub location: String,
    pub work_mode: String, // "Remote" | "Hybrid" | "Full" (onsite) | ""
    pub pay_range: String,
    pub pay_max: f64,
    pub pay_source: String, // "POSTED" | "est" | ""
    pub posted_on: String,
    pub last_contact: String,

    // Report enrichment
    pub archetype: String,
    pub tldr: String,
    pub remote: String,
    pub comp_estimate: String,
}

impl Default for CareerApplication {
    fn default() -> Self {
        Self {
            number: 0,
            date: String::new(),
            company: String::new(),
            role: String::new(),
            status: String::new(),
            score: 0.0,
            score_raw: String::new(),
            has_pdf: false,
            report_path: String::new(),
            report_number: String::new(),
            notes: String::new(),
            job_url: String::new(),
            location: String::new(),
            work_mode: String::new(),
            pay_range: String::new(),
            pay_max: 0.0,
            pay_source: String::new(),
            posted_on: String::new(),
            last_contact: String::new(),
            archetype: String::new(),
            tldr: String::new(),
            remote: String::new(),
            comp_estimate: String::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PipelineMetrics {
    pub total: usize,
    pub evaluated: usize,
    pub applied: usize,
    pub interview: usize,
    pub responded: usize,
    pub offer: usize,
    pub hired: usize,
    pub discarded: usize,
    pub rejected: usize,
    pub avg_score: f64,
    pub top_score: f64,
    pub with_pdf: usize,
}
