use chrono::Local;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectHighlight {
    pub company: String,
    pub role: String,
    pub tech_stack: String,
    pub architecture: String,
    pub challenge: String,
    pub star_description: String,
    pub metrics: String,
    pub bullet_point: String,
    pub date_added: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MasterInventory {
    pub skills: Vec<String>,
    pub highlights: Vec<ProjectHighlight>,
}

pub fn load_inventory(repo_root: &Path) -> MasterInventory {
    let json_path = repo_root.join("data").join("skills-inventory.json");
    if json_path.exists() {
        if let Ok(content) = fs::read_to_string(&json_path) {
            if let Ok(inv) = serde_json::from_str::<MasterInventory>(&content) {
                return inv;
            }
        }
    }
    MasterInventory::default()
}

pub fn save_inventory(repo_root: &Path, inventory: &MasterInventory) -> Result<(), std::io::Error> {
    let data_dir = repo_root.join("data");
    if !data_dir.exists() {
        let _ = fs::create_dir_all(&data_dir);
    }

    // 1. Save JSON
    let json_path = data_dir.join("skills-inventory.json");
    let json_str = serde_json::to_string_pretty(inventory)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    fs::write(&json_path, json_str)?;

    // 2. Save human-readable Markdown
    let md_path = data_dir.join("skills-inventory.md");
    let mut md = String::new();
    md.push_str("# Skills & Project Highlights Inventory\n\n");
    md.push_str("Accumulated ground truth of verified competencies and STAR highlights extracted from pipeline job surveys.\n\n");

    md.push_str("## Core Skills Inventory\n\n");
    if inventory.skills.is_empty() {
        md.push_str("*(No skills recorded yet. Complete a job survey to populate.)*\n\n");
    } else {
        md.push_str(&format!("Total verified skills: **{}**\n\n", inventory.skills.len()));
        let skills_list = inventory.skills.join(" • ");
        md.push_str(&format!("`{}`\n\n", skills_list));
    }

    md.push_str("## STAR Project Highlights\n\n");
    if inventory.highlights.is_empty() {
        md.push_str("*(No project highlights recorded yet.)*\n\n");
    } else {
        for (idx, h) in inventory.highlights.iter().enumerate() {
            md.push_str(&format!("### {}. {} — {}\n\n", idx + 1, h.company, h.role));
            md.push_str(&format!("- **Bullet:** {}\n", h.bullet_point));
            if !h.tech_stack.is_empty() {
                md.push_str(&format!("- **Tech Stack:** {}\n", h.tech_stack));
            }
            if !h.metrics.is_empty() {
                md.push_str(&format!("- **Impact & Metrics:** {}\n", h.metrics));
            }
            md.push_str(&format!("- **Date Added:** {}\n\n", h.date_added));
        }
    }

    fs::write(&md_path, md)?;
    Ok(())
}

pub fn record_survey_answers(
    repo_root: &Path,
    company: &str,
    role: &str,
    answers: &[String],
) -> Result<(usize, usize), String> {
    let mut inv = load_inventory(repo_root);

    let tech_stack = answers.get(0).cloned().unwrap_or_default().trim().to_string();
    let architecture = answers.get(1).cloned().unwrap_or_default().trim().to_string();
    let challenge = answers.get(2).cloned().unwrap_or_default().trim().to_string();
    let star_desc = answers.get(3).cloned().unwrap_or_default().trim().to_string();
    let metrics = answers.get(4).cloned().unwrap_or_default().trim().to_string();

    let mut existing_skills: BTreeSet<String> = inv.skills.iter().cloned().collect();
    let prev_count = existing_skills.len();

    // Extract individual skills from tech_stack
    if !tech_stack.is_empty() {
        for item in tech_stack.split(&[',', ';', '/', '•', '\n'][..]) {
            let s = item.trim();
            if !s.is_empty() && s.len() <= 35 && !s.contains("e.g.") {
                existing_skills.insert(s.to_string());
            }
        }
    }

    // Build STAR bullet point
    let mut bullet = String::new();
    if !star_desc.is_empty() {
        bullet.push_str(&star_desc);
        if !bullet.ends_with('.') && !bullet.ends_with(';') {
            bullet.push('.');
        }
    } else if !architecture.is_empty() {
        bullet.push_str(&format!("Architected and deployed system: {}.", architecture));
    } else {
        bullet.push_str(&format!("Engineered software solutions as {} at {}.", role, company));
    }

    if !metrics.is_empty() {
        bullet.push_str(&format!(" Achieved: {}.", metrics));
    }

    let today = Local::now().format("%Y-%m-%d").to_string();

    let highlight = ProjectHighlight {
        company: company.to_string(),
        role: role.to_string(),
        tech_stack,
        architecture,
        challenge,
        star_description: star_desc,
        metrics,
        bullet_point: bullet,
        date_added: today,
    };

    inv.highlights.push(highlight);
    inv.skills = existing_skills.into_iter().collect();

    let added_skills = inv.skills.len().saturating_sub(prev_count);
    let added_highlights = 1;

    save_inventory(repo_root, &inv).map_err(|e| e.to_string())?;

    Ok((added_skills, added_highlights))
}

pub fn merge_into_master_cv(repo_root: &Path, inventory: &MasterInventory) -> Result<String, String> {
    let cv_path = repo_root.join("cv.md");

    let formatted_skills = if inventory.skills.is_empty() {
        "Python, Go, TypeScript, PostgreSQL, Redis, Kafka, Docker, Kubernetes, AWS".to_string()
    } else {
        inventory.skills.join(", ")
    };

    if !cv_path.exists() {
        // Create full Master CV template incorporating inventory
        let mut content = String::new();
        content.push_str("# CV -- Master Resume\n\n");
        content.push_str("**Location:** Remote / Open to Relocation\n");
        content.push_str("**Role Focus:** Backend Engineer | Full-Stack Engineer | Applied AI Engineer\n\n");

        content.push_str("## Professional Summary\n\n");
        content.push_str(&format!(
            "Results-driven Software Engineer with proven experience across {}. Background in building resilient backend services, scalable architectures, and applied AI workflows with high uptime and measurable performance impact.\n\n",
            inventory.skills.iter().take(4).cloned().collect::<Vec<_>>().join(", ")
        ));

        content.push_str("## Core Competencies & Skills\n\n");
        content.push_str(&format!("- **Technical Stack:** {}\n\n", formatted_skills));

        content.push_str("## Projects & System Highlights\n\n");
        if inventory.highlights.is_empty() {
            content.push_str("### Flagship Platform Architecture\n");
            content.push_str("- Engineered high-concurrency event ingestion pipeline handling 10k+ events/sec with sub-50ms latency.\n\n");
        } else {
            for h in &inventory.highlights {
                content.push_str(&format!("### {} -- {}\n", h.company, h.role));
                content.push_str(&format!("- {}\n", h.bullet_point));
                if !h.tech_stack.is_empty() {
                    content.push_str(&format!("- **Technologies:** {}\n", h.tech_stack));
                }
                content.push_str("\n");
            }
        }

        content.push_str("## Work Experience\n\n");
        content.push_str("### Senior Software Engineer\n\n");
        content.push_str("- Led technical design and production deployment of core backend and AI-enabled services.\n");
        for h in inventory.highlights.iter().take(3) {
            content.push_str(&format!("- {}\n", h.bullet_point));
        }

        fs::write(&cv_path, content).map_err(|e| format!("Failed to create cv.md: {}", e))?;
        return Ok(format!(
            "Created master cv.md with {} skills and {} project highlights!",
            inventory.skills.len(),
            inventory.highlights.len()
        ));
    }

    // If cv.md already exists, append new project highlights and skills safely
    let mut current_cv = fs::read_to_string(&cv_path).map_err(|e| format!("Failed to read cv.md: {}", e))?;

    let mut added_bullets = 0;
    let mut highlight_additions = String::new();

    for h in &inventory.highlights {
        if !current_cv.contains(&h.bullet_point) {
            highlight_additions.push_str(&format!("- {}\n", h.bullet_point));
            added_bullets += 1;
        }
    }

    if added_bullets > 0 {
        if current_cv.contains("## Projects") {
            current_cv = current_cv.replace("## Projects", &format!("## Projects\n\n### Pipeline Survey Highlights\n{}", highlight_additions));
        } else {
            current_cv.push_str(&format!("\n\n## Projects (from Pipeline Survey)\n\n{}\n", highlight_additions));
        }
    }

    // Update Skills section if present
    if !inventory.skills.is_empty() {
        let skills_summary = format!("\n- **Survey Inventory:** {}", formatted_skills);
        if current_cv.contains("## Skills") && !current_cv.contains("Survey Inventory:") {
            current_cv = current_cv.replace("## Skills", &format!("## Skills{}", skills_summary));
        }
    }

    fs::write(&cv_path, current_cv).map_err(|e| format!("Failed to update cv.md: {}", e))?;

    Ok(format!(
        "Merged into cv.md: added {} new highlight bullets and synchronized skills inventory.",
        added_bullets
    ))
}
