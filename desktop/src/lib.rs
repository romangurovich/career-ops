pub mod model;
pub mod derive;
pub mod report;
pub mod tracker;
pub mod pipeline;
pub mod inventory;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_pipeline_and_generate_survey() {
        let repo_root = PathBuf::from("..");
        let jobs = pipeline::parse_pipeline(&repo_root);
        assert!(!jobs.is_empty(), "Pipeline should parse jobs from data/pipeline.md");

        let first = &jobs[0];
        assert!(!first.company.is_empty());
        assert!(!first.role.is_empty());

        let survey = pipeline::generate_survey(first);
        assert_eq!(survey.len(), 5, "Survey should generate 5 questions");
        assert!(survey[0].category.contains("TECH STACK"));
        assert!(survey[3].category.contains("STAR"));
    }

    #[test]
    fn test_inventory_record_and_cv_merge() {
        let temp_dir = std::env::temp_dir().join(format!("career_ops_test_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);

        let answers = vec![
            "Go, Rust, PostgreSQL, Redis, Kafka".to_string(),
            "Designed event-driven transaction ledger with idempotency".to_string(),
            "Optimized p99 query latency from 400ms to 45ms".to_string(),
            "Led modernization of payment processor service".to_string(),
            "Reduced cloud infra spend by 35% ($60k/yr)".to_string(),
        ];

        let (skills, hl) = inventory::record_survey_answers(&temp_dir, "AcmeCorp", "Senior Backend Engineer", &answers).expect("Save answers");
        assert!(skills >= 4, "Should extract skills from tech stack");
        assert_eq!(hl, 1, "Should record 1 highlight");

        let inv = inventory::load_inventory(&temp_dir);
        assert!(!inv.skills.is_empty());
        assert_eq!(inv.highlights.len(), 1);

        let merge_msg = inventory::merge_into_master_cv(&temp_dir, &inv).expect("Merge into CV");
        assert!(merge_msg.contains("cv.md"));

        let cv_content = std::fs::read_to_string(temp_dir.join("cv.md")).expect("Read created cv.md");
        assert!(cv_content.contains("AcmeCorp"));
        assert!(cv_content.contains("PostgreSQL"));
        assert!(cv_content.contains("60k/yr"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
