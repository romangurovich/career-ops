use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineJob {
    pub id: i32,
    pub url: String,
    pub company: String,
    pub role: String,
    pub location: String,
    pub posted_on: String,
    pub status: String,
    pub is_pending: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurveyQuestion {
    pub id: i32,
    pub category: String,
    pub prompt: String,
    pub placeholder: String,
    pub default_answer: String,
}

pub fn resolve_pipeline_path(repo_root: &Path) -> PathBuf {
    if let Ok(env_p) = std::env::var("CAREER_OPS_PIPELINE") {
        let trimmed = env_p.trim();
        if !trimmed.is_empty() {
            let p = PathBuf::from(trimmed);
            if p.is_absolute() {
                return p;
            } else {
                return repo_root.join(p);
            }
        }
    }

    let p1 = repo_root.join("data").join("pipeline.md");
    if p1.exists() {
        return p1;
    }
    repo_root.join("pipeline.md")
}

pub fn parse_pipeline(repo_root: &Path) -> Vec<PipelineJob> {
    let path = resolve_pipeline_path(repo_root);
    let content = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let mut jobs = Vec::new();
    let mut current_id = 1;

    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("- [") {
            continue;
        }

        let is_pending = trimmed.starts_with("- [ ]");
        let is_processed = trimmed.starts_with("- [x]");
        let status = if is_pending {
            "Pending"
        } else if is_processed {
            "Processed"
        } else {
            "Note"
        };

        // Strip the checkbox prefix "- [ ] " or "- [x] "
        let payload = if trimmed.len() > 6 {
            trimmed[5..].trim()
        } else {
            continue
        };

        let parts: Vec<&str> = payload.split('|').map(|s| s.trim()).collect();
        let mut url = String::new();
        let mut company = String::new();
        let mut role = String::new();
        let mut location = String::new();
        let mut posted_on = String::new();

        if !parts.is_empty() {
            url = parts[0].to_string();
            // Clean markdown link formatting if present: [text](url) -> url
            if let (Some(start), Some(end)) = (url.find('('), url.find(')')) {
                if start < end {
                    url = url[start + 1..end].to_string();
                }
            }
        }

        if parts.len() > 1 {
            company = parts[1].to_string();
        }
        if parts.len() > 2 {
            role = parts[2].to_string();
        }
        if parts.len() > 3 {
            location = parts[3].to_string();
        }
        if parts.len() > 4 {
            let p4 = parts[4];
            if let Some(stripped) = p4.strip_prefix("posted:") {
                posted_on = stripped.trim().to_string();
            } else {
                posted_on = p4.to_string();
            }
        }

        // Fallback for company name if not separated by |
        if company.is_empty() && !url.is_empty() {
            if let Some(pos) = url.find("://") {
                let rest = &url[pos + 3..];
                let host = rest.split('/').next().unwrap_or(rest);
                let domain_parts: Vec<&str> = host.split('.').collect();
                if domain_parts.len() >= 2 {
                    company = domain_parts[domain_parts.len() - 2].to_string();
                }
            }
            if company.is_empty() {
                company = "Target Employer".to_string();
            }
        }

        if role.is_empty() {
            role = "Software Engineer".to_string();
        }

        jobs.push(PipelineJob {
            id: current_id,
            url,
            company,
            role,
            location,
            posted_on,
            status: status.to_string(),
            is_pending,
        });

        current_id += 1;
    }

    jobs
}

pub fn generate_survey(job: &PipelineJob) -> Vec<SurveyQuestion> {
    let role_lower = job.role.to_lowercase();
    let company = if job.company.is_empty() { "this company" } else { &job.company };
    let role = if job.role.is_empty() { "Software Engineer" } else { &job.role };

    let is_backend = role_lower.contains("backend")
        || role_lower.contains("back-end")
        || role_lower.contains("infra")
        || role_lower.contains("platform")
        || role_lower.contains("system")
        || role_lower.contains("sre");

    let is_fullstack = role_lower.contains("full stack")
        || role_lower.contains("fullstack")
        || role_lower.contains("full-stack")
        || role_lower.contains("web")
        || role_lower.contains("frontend");

    let is_ai = role_lower.contains("ai")
        || role_lower.contains("ml")
        || role_lower.contains("machine learning")
        || role_lower.contains("agent")
        || role_lower.contains("llm")
        || role_lower.contains("intelligence");

    let mut questions = Vec::new();

    // 1. Tech Stack
    if is_ai {
        questions.push(SurveyQuestion {
            id: 1,
            category: "CORE AI TECH STACK".to_string(),
            prompt: format!("What programming languages, deep learning frameworks, and LLM APIs have you used in production relevant to {}?", company),
            placeholder: "e.g. Python, PyTorch, LangChain, OpenAI / Anthropic APIs, Hugging Face, vLLM, PostgreSQL / pgvector".to_string(),
            default_answer: String::new(),
        });
    } else if is_backend {
        questions.push(SurveyQuestion {
            id: 1,
            category: "CORE TECH STACK".to_string(),
            prompt: format!("What backend languages, databases, and message brokers have you built production systems with for {}?", company),
            placeholder: "e.g. Go, Rust, Python, PostgreSQL, Redis, Kafka, gRPC, Docker, AWS (ECS, RDS, SQS)".to_string(),
            default_answer: String::new(),
        });
    } else if is_fullstack {
        questions.push(SurveyQuestion {
            id: 1,
            category: "FULL-STACK STACK".to_string(),
            prompt: format!("What frontend and backend technologies do you use for building end-to-end applications at {}?", company),
            placeholder: "e.g. TypeScript, React / Next.js, Node.js, Go, GraphQL, Tailwind CSS, PostgreSQL, Redis".to_string(),
            default_answer: String::new(),
        });
    } else {
        questions.push(SurveyQuestion {
            id: 1,
            category: "PRIMARY TECH STACK".to_string(),
            prompt: format!("What primary technologies, languages, and tools have you mastered relevant to the {} position?", role),
            placeholder: "e.g. TypeScript, Python, Go, Docker, Kubernetes, AWS, SQL databases".to_string(),
            default_answer: String::new(),
        });
    }

    // 2. Architecture & Design
    if is_ai {
        questions.push(SurveyQuestion {
            id: 2,
            category: "AI ARCHITECTURE & RAG".to_string(),
            prompt: format!("Describe an Applied AI architecture (e.g. RAG, Autonomous Agents, Model Serving) you engineered."),
            placeholder: "e.g. Built multi-stage RAG pipeline with hybrid keyword + semantic search, chunking strategy, and automated hallucination guardrails.".to_string(),
            default_answer: String::new(),
        });
    } else if is_backend {
        questions.push(SurveyQuestion {
            id: 2,
            category: "SYSTEM ARCHITECTURE & SCALE".to_string(),
            prompt: format!("Describe a high-throughput or distributed service you designed, and how you ensured low latency and resilience."),
            placeholder: "e.g. Architected distributed event processor with Kafka + Go workers handling 12,000 req/sec at <40ms p99 latency with zero data loss.".to_string(),
            default_answer: String::new(),
        });
    } else {
        questions.push(SurveyQuestion {
            id: 2,
            category: "SYSTEM ARCHITECTURE".to_string(),
            prompt: format!("How did you architect a scalable, maintainable application architecture for a complex feature or product?"),
            placeholder: "e.g. Designed clean hexagonal architecture separating business logic from datastores and third-party APIs.".to_string(),
            default_answer: String::new(),
        });
    }

    // 3. Domain Specialization / Problem Solved
    questions.push(SurveyQuestion {
        id: 3,
        category: "TECHNICAL CHALLENGE".to_string(),
        prompt: format!("What was the most difficult technical bottleneck (concurrency, memory, API limits, drift, data synchronization) you resolved?"),
        placeholder: "e.g. Solved Redis cache stampede by implementing single-flight request coalescing and jittered TTLs, reducing DB load by 70%.".to_string(),
        default_answer: String::new(),
    });

    // 4. STAR Project Highlight
    questions.push(SurveyQuestion {
        id: 4,
        category: "STAR PROJECT HIGHLIGHT".to_string(),
        prompt: format!("Describe your flagship project highlight for {}: Situation, Task, your specific Actions, and the Result.", company),
        placeholder: "e.g. Designed and deployed an automated data ingestion pipeline using Go and AWS Lambda that processed 50M daily events with 99.99% reliability.".to_string(),
        default_answer: String::new(),
    });

    // 5. Quantified Metrics & Impact
    questions.push(SurveyQuestion {
        id: 5,
        category: "QUANTIFIABLE METRICS".to_string(),
        prompt: "What specific metrics demonstrate your impact (e.g. % latency decrease, $ cost savings, user growth, uptime, delivery speed)?".to_string(),
        placeholder: "e.g. Reduced compute infrastructure spend by $45,000/yr; accelerated deployment cycle from bi-weekly to multiple times per day.".to_string(),
        default_answer: String::new(),
    });

    questions
}
