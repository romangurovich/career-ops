use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use slint::VecModel;

use career_ops_desktop::inventory::{load_inventory, merge_into_master_cv, record_survey_answers, MasterInventory};
use career_ops_desktop::model::CareerApplication;
use career_ops_desktop::pipeline::{generate_survey, parse_pipeline, PipelineJob, SurveyQuestion};
use career_ops_desktop::report::load_report_summary;
use career_ops_desktop::tracker::{compute_metrics, find_repo_root, parse_applications};

slint::include_modules!();

struct AppState {
    #[allow(dead_code)]
    repo_root: PathBuf,
    all_apps: Vec<CareerApplication>,
    search_query: String,
    current_tab: String,
    view_mode: String,
    selected_id: i32,

    // Pipeline & Survey State
    pipeline_jobs: Vec<PipelineJob>,
    selected_pipeline_id: i32,
    survey_questions: Vec<SurveyQuestion>,
    survey_answers: HashMap<i32, String>,
    survey_status_msg: String,
    inventory: MasterInventory,
}

fn to_pipeline_job_item(job: &PipelineJob) -> PipelineJobItem {
    PipelineJobItem {
        id: job.id,
        url: job.url.clone().into(),
        company: job.company.clone().into(),
        role: job.role.clone().into(),
        location: job.location.clone().into(),
        posted_on: job.posted_on.clone().into(),
        status: job.status.clone().into(),
        is_pending: job.is_pending,
    }
}

fn to_survey_question_item(q: &SurveyQuestion, current_ans: &str) -> SurveyQuestionItem {
    SurveyQuestionItem {
        id: q.id,
        category: q.category.clone().into(),
        prompt: q.prompt.clone().into(),
        placeholder: q.placeholder.clone().into(),
        answer: current_ans.to_string().into(),
    }
}

fn to_app_item(app: &CareerApplication) -> AppItem {
    AppItem {
        id: app.number,
        date: app.date.clone().into(),
        company: app.company.clone().into(),
        role: app.role.clone().into(),
        status: app.status.clone().into(),
        score: app.score as f32,
        score_str: app.score_raw.clone().into(),
        has_pdf: app.has_pdf,
        has_report: !app.report_path.is_empty(),
        report_path: app.report_path.clone().into(),
        report_number: app.report_number.clone().into(),
        job_url: app.job_url.clone().into(),
        notes: app.notes.clone().into(),
        location: app.location.clone().into(),
        work_mode: app.work_mode.clone().into(),
        pay_range: app.pay_range.clone().into(),
        pay_max: app.pay_max as f32,
        posted_on: app.posted_on.clone().into(),
        last_contact: app.last_contact.clone().into(),
        archetype: app.archetype.clone().into(),
        tldr: app.tldr.clone().into(),
    }
}

fn open_target(target: &str) {
    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("cmd")
        .args(["/C", "start", "", target])
        .spawn();

    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(target).spawn();

    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(target).spawn();
}

fn filter_apps<'a>(
    apps: &'a [CareerApplication],
    query: &str,
    tab: &str,
) -> Vec<&'a CareerApplication> {
    let lower_q = query.trim().to_lowercase();
    apps.iter()
        .filter(|app| {
            // Search filter
            if !lower_q.is_empty() {
                let haystack = format!(
                    "{} {} {} {} {}",
                    app.company, app.role, app.location, app.notes, app.archetype
                )
                .to_lowercase();
                if !haystack.contains(&lower_q) {
                    return false;
                }
            }

            // Tab filter
            let st = app.status.trim().to_lowercase();
            match tab {
                "all" => true,
                "evaluated" => app.score > 0.0,
                "interview" => st.contains("interview") || st.contains("entrevista") || st.contains("screening"),
                "responded" => st.contains("responded") || st.contains("respondido"),
                "applied" => st == "applied" || st == "aplicada",
                "top" => app.score >= 4.0,
                "discarded" => st.contains("discard") || st.contains("descartada") || st == "skip",
                "rejected" => st.contains("reject") || st.contains("rechazada"),
                _ => true,
            }
        })
        .collect()
}

fn update_ui(ui: &AppWindow, state: &AppState) {
    // 1. Update KPIs
    let metrics = compute_metrics(&state.all_apps);
    ui.set_stats(MetricStats {
        total: metrics.total as i32,
        evaluated: metrics.evaluated as i32,
        applied: metrics.applied as i32,
        interview: metrics.interview as i32,
        responded: metrics.responded as i32,
        offer: metrics.offer as i32,
        hired: metrics.hired as i32,
        discarded: metrics.discarded as i32,
        rejected: metrics.rejected as i32,
        avg_score: format!("{:.1}", metrics.avg_score).into(),
        top_score: format!("{:.1}", metrics.top_score).into(),
    });

    // 2. Filter apps for main table
    let filtered = filter_apps(&state.all_apps, &state.search_query, &state.current_tab);
    let items: Vec<AppItem> = filtered.into_iter().map(to_app_item).collect();
    let model = Rc::new(VecModel::from(items));
    ui.set_apps(model.into());

    // 3. Kanban columns
    let q = &state.search_query;
    let col_eval: Vec<AppItem> = state
        .all_apps
        .iter()
        .filter(|a| {
            let st = a.status.trim().to_lowercase();
            (a.score > 0.0 || st.is_empty())
                && !st.contains("applied")
                && !st.contains("interview")
                && !st.contains("offer")
                && !st.contains("hired")
                && !st.contains("reject")
                && !st.contains("discard")
        })
        .filter(|a| q.is_empty() || format!("{} {}", a.company, a.role).to_lowercase().contains(&q.to_lowercase()))
        .map(to_app_item)
        .collect();

    let col_app: Vec<AppItem> = state
        .all_apps
        .iter()
        .filter(|a| {
            let st = a.status.trim().to_lowercase();
            st == "applied" || st == "aplicada"
        })
        .filter(|a| q.is_empty() || format!("{} {}", a.company, a.role).to_lowercase().contains(&q.to_lowercase()))
        .map(to_app_item)
        .collect();

    let col_int: Vec<AppItem> = state
        .all_apps
        .iter()
        .filter(|a| {
            let st = a.status.trim().to_lowercase();
            st.contains("interview") || st.contains("entrevista") || st.contains("screening")
        })
        .filter(|a| q.is_empty() || format!("{} {}", a.company, a.role).to_lowercase().contains(&q.to_lowercase()))
        .map(to_app_item)
        .collect();

    let col_off: Vec<AppItem> = state
        .all_apps
        .iter()
        .filter(|a| {
            let st = a.status.trim().to_lowercase();
            st.contains("offer") || st.contains("oferta") || st.contains("hired") || st.contains("contratado")
        })
        .filter(|a| q.is_empty() || format!("{} {}", a.company, a.role).to_lowercase().contains(&q.to_lowercase()))
        .map(to_app_item)
        .collect();

    let col_rej: Vec<AppItem> = state
        .all_apps
        .iter()
        .filter(|a| {
            let st = a.status.trim().to_lowercase();
            st.contains("reject") || st.contains("rechazada") || st.contains("discard") || st.contains("descartada") || st == "skip"
        })
        .filter(|a| q.is_empty() || format!("{} {}", a.company, a.role).to_lowercase().contains(&q.to_lowercase()))
        .map(to_app_item)
        .collect();

    ui.set_col_evaluated(Rc::new(VecModel::from(col_eval)).into());
    ui.set_col_applied(Rc::new(VecModel::from(col_app)).into());
    ui.set_col_interview(Rc::new(VecModel::from(col_int)).into());
    ui.set_col_offer(Rc::new(VecModel::from(col_off)).into());
    ui.set_col_rejected(Rc::new(VecModel::from(col_rej)).into());

    // 4. Selected item
    if state.selected_id >= 0 {
        if let Some(found) = state.all_apps.iter().find(|a| a.number == state.selected_id) {
            ui.set_selected_app(to_app_item(found));
            ui.set_has_selection(true);
            ui.set_selected_id(state.selected_id);
        } else {
            ui.set_has_selection(false);
        }
    } else if let Some(first) = state.all_apps.first() {
        ui.set_selected_app(to_app_item(first));
        ui.set_has_selection(true);
        ui.set_selected_id(first.number);
    } else {
        ui.set_has_selection(false);
    }

    // 5. Pipeline & Survey
    let p_items: Vec<PipelineJobItem> = state.pipeline_jobs.iter().map(to_pipeline_job_item).collect();
    ui.set_pipeline_jobs(Rc::new(VecModel::from(p_items)).into());

    if state.selected_pipeline_id > 0 {
        if let Some(p_job) = state.pipeline_jobs.iter().find(|j| j.id == state.selected_pipeline_id) {
            ui.set_selected_pipeline_job(to_pipeline_job_item(p_job));
            ui.set_has_selected_pipeline_job(true);
            ui.set_selected_pipeline_job_id(state.selected_pipeline_id);

            let q_items: Vec<SurveyQuestionItem> = state
                .survey_questions
                .iter()
                .map(|q| {
                    let ans = state.survey_answers.get(&q.id).map(|s| s.as_str()).unwrap_or("");
                    to_survey_question_item(q, ans)
                })
                .collect();
            ui.set_survey_questions(Rc::new(VecModel::from(q_items)).into());
        } else {
            ui.set_has_selected_pipeline_job(false);
            ui.set_selected_pipeline_job_id(-1);
            ui.set_survey_questions(Rc::new(VecModel::from(Vec::<SurveyQuestionItem>::new())).into());
        }
    } else if let Some(first_p) = state.pipeline_jobs.first() {
        ui.set_selected_pipeline_job(to_pipeline_job_item(first_p));
        ui.set_has_selected_pipeline_job(true);
        ui.set_selected_pipeline_job_id(first_p.id);

        let q_items: Vec<SurveyQuestionItem> = state
            .survey_questions
            .iter()
            .map(|q| {
                let ans = state.survey_answers.get(&q.id).map(|s| s.as_str()).unwrap_or("");
                to_survey_question_item(q, ans)
            })
            .collect();
        ui.set_survey_questions(Rc::new(VecModel::from(q_items)).into());
    } else {
        ui.set_has_selected_pipeline_job(false);
        ui.set_selected_pipeline_job_id(-1);
        ui.set_survey_questions(Rc::new(VecModel::from(Vec::<SurveyQuestionItem>::new())).into());
    }

    ui.set_inventory_skills_count(state.inventory.skills.len() as i32);
    ui.set_inventory_highlights_count(state.inventory.highlights.len() as i32);
    ui.set_survey_status_message(state.survey_status_msg.clone().into());

    ui.set_current_tab(state.current_tab.clone().into());
    ui.set_view_mode(state.view_mode.clone().into());
}

fn load_all_data(repo_root: &Path) -> Vec<CareerApplication> {
    let mut apps = parse_applications(repo_root);
    // Enrich with reports
    for app in &mut apps {
        if !app.report_path.is_empty() {
            let summary = load_report_summary(repo_root, &app.report_path);
            if !summary.archetype.is_empty() {
                app.archetype = summary.archetype;
            }
            if !summary.tldr.is_empty() {
                app.tldr = summary.tldr;
            }
            if !summary.remote.is_empty() {
                app.remote = summary.remote;
            }
            if !summary.comp.is_empty() {
                app.comp_estimate = summary.comp;
            }
            if app.job_url.is_empty() && !summary.job_url.is_empty() {
                app.job_url = summary.job_url;
            }
        }
    }
    apps
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut repo_root = find_repo_root();
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    let mut explicit_path = false;
    while i < args.len() {
        if args[i] == "--path" && i + 1 < args.len() {
            repo_root = PathBuf::from(&args[i + 1]);
            explicit_path = true;
            i += 2;
        } else if args[i].starts_with("--path=") {
            repo_root = PathBuf::from(&args[i]["--path=".len()..]);
            explicit_path = true;
            i += 1;
        } else {
            i += 1;
        }
    }

    println!("Career-Ops Desktop launching for repo: {}", repo_root.display());

    let mut apps = load_all_data(&repo_root);
    if apps.is_empty() && !explicit_path {
        let fixture_path = repo_root.join("test-fixtures").join("upgrade").join("state-v1.18");
        if fixture_path.exists() {
            println!("No user applications.md found; previewing fixture from {}", fixture_path.display());
            apps = load_all_data(&fixture_path);
        }
    }

    println!("Loaded {} applications", apps.len());

    let initial_selected = apps.first().map(|a| a.number).unwrap_or(-1);

    // Initialize Pipeline and Skills Inventory
    let pipeline_jobs = parse_pipeline(&repo_root);
    let initial_pipeline_id = pipeline_jobs.first().map(|j| j.id).unwrap_or(-1);
    let initial_questions = if let Some(first_job) = pipeline_jobs.first() {
        generate_survey(first_job)
    } else {
        Vec::new()
    };
    let inventory = load_inventory(&repo_root);

    let state = Rc::new(RefCell::new(AppState {
        repo_root: repo_root.clone(),
        all_apps: apps,
        search_query: String::new(),
        current_tab: "all".to_string(),
        view_mode: "table".to_string(),
        selected_id: initial_selected,

        pipeline_jobs,
        selected_pipeline_id: initial_pipeline_id,
        survey_questions: initial_questions,
        survey_answers: HashMap::new(),
        survey_status_msg: String::new(),
        inventory,
    }));

    let ui = AppWindow::new()?;
    update_ui(&ui, &state.borrow());

    // Callbacks
    {
        let ui_handle = ui.as_weak();
        let state_rc = Rc::clone(&state);
        ui.on_tab_changed(move |tab| {
            if let Some(ui) = ui_handle.upgrade() {
                state_rc.borrow_mut().current_tab = tab.to_string();
                update_ui(&ui, &state_rc.borrow());
            }
        });
    }

    {
        let ui_handle = ui.as_weak();
        let state_rc = Rc::clone(&state);
        ui.on_search_changed(move |query| {
            if let Some(ui) = ui_handle.upgrade() {
                state_rc.borrow_mut().search_query = query.to_string();
                update_ui(&ui, &state_rc.borrow());
            }
        });
    }

    {
        let ui_handle = ui.as_weak();
        let state_rc = Rc::clone(&state);
        ui.on_view_mode_changed(move |mode| {
            if let Some(ui) = ui_handle.upgrade() {
                state_rc.borrow_mut().view_mode = mode.to_string();
                update_ui(&ui, &state_rc.borrow());
            }
        });
    }

    {
        let ui_handle = ui.as_weak();
        let state_rc = Rc::clone(&state);
        ui.on_select_app(move |id| {
            if let Some(ui) = ui_handle.upgrade() {
                state_rc.borrow_mut().selected_id = id;
                update_ui(&ui, &state_rc.borrow());
            }
        });
    }

    {
        let ui_handle = ui.as_weak();
        let state_rc = Rc::clone(&state);
        let root = repo_root.clone();
        ui.on_refresh_data(move || {
            if let Some(ui) = ui_handle.upgrade() {
                let fresh = load_all_data(&root);
                let fresh_pipeline = parse_pipeline(&root);
                let fresh_inventory = load_inventory(&root);
                let mut st = state_rc.borrow_mut();
                st.all_apps = fresh;
                st.pipeline_jobs = fresh_pipeline;
                st.inventory = fresh_inventory;
                update_ui(&ui, &st);
            }
        });
    }

    // Pipeline & Survey Callbacks
    {
        let ui_handle = ui.as_weak();
        let state_rc = Rc::clone(&state);
        ui.on_select_pipeline_job(move |id| {
            if let Some(ui) = ui_handle.upgrade() {
                let mut st = state_rc.borrow_mut();
                st.selected_pipeline_id = id;
                if let Some(job) = st.pipeline_jobs.iter().find(|j| j.id == id) {
                    st.survey_questions = generate_survey(job);
                    st.survey_answers.clear();
                    st.survey_status_msg.clear();
                }
                update_ui(&ui, &st);
            }
        });
    }

    {
        let state_rc = Rc::clone(&state);
        ui.on_survey_answer_changed(move |qid, text| {
            state_rc.borrow_mut().survey_answers.insert(qid, text.to_string());
        });
    }

    {
        let ui_handle = ui.as_weak();
        let state_rc = Rc::clone(&state);
        let root = repo_root.clone();
        ui.on_save_survey(move |jid| {
            if let Some(ui) = ui_handle.upgrade() {
                let mut st = state_rc.borrow_mut();
                if let Some(job) = st.pipeline_jobs.iter().find(|j| j.id == jid).cloned() {
                    let mut answers_ordered = Vec::new();
                    for q in &st.survey_questions {
                        let ans = st.survey_answers.get(&q.id).cloned().unwrap_or_default();
                        answers_ordered.push(ans);
                    }
                    match record_survey_answers(&root, &job.company, &job.role, &answers_ordered) {
                        Ok((added_skills, added_hl)) => {
                            st.inventory = load_inventory(&root);
                            st.survey_status_msg = format!("✓ Saved {} skills & {} STAR highlight to inventory!", added_skills, added_hl);
                        }
                        Err(e) => {
                            st.survey_status_msg = format!("Error saving answers: {}", e);
                        }
                    }
                }
                update_ui(&ui, &st);
            }
        });
    }

    {
        let ui_handle = ui.as_weak();
        let state_rc = Rc::clone(&state);
        let root = repo_root.clone();
        ui.on_merge_survey_to_cv(move || {
            if let Some(ui) = ui_handle.upgrade() {
                let mut st = state_rc.borrow_mut();
                match merge_into_master_cv(&root, &st.inventory) {
                    Ok(msg) => {
                        st.survey_status_msg = format!("✓ {}", msg);
                    }
                    Err(e) => {
                        st.survey_status_msg = format!("Error merging to CV: {}", e);
                    }
                }
                update_ui(&ui, &st);
            }
        });
    }

    // URL opening callback
    ui.on_open_url(move |url| {
        let u = url.to_string();
        if !u.is_empty() {
            println!("Opening job URL: {}", u);
            open_target(&u);
        }
    });

    // PDF opening callback
    {
        let root = repo_root.clone();
        ui.on_open_pdf(move |rep_num| {
            let num_str = rep_num.to_string();
            // Look for matching PDF in output/
            let output_dir = root.join("output");
            if output_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&output_dir) {
                    for entry in entries.flatten() {
                        let fname = entry.file_name().to_string_lossy().to_string();
                        if fname.ends_with(".pdf") && (fname.starts_with(&num_str) || fname.contains(&format!("-{}", num_str))) {
                            let pdf_path = entry.path();
                            println!("Opening PDF: {}", pdf_path.display());
                            open_target(&pdf_path.to_string_lossy());
                            return;
                        }
                    }
                }
            }
            println!("No PDF found for report {}", num_str);
        });
    }

    // Status change callback
    {
        let ui_handle = ui.as_weak();
        let state_rc = Rc::clone(&state);
        ui.on_change_status(move |id, new_status| {
            if let Some(ui) = ui_handle.upgrade() {
                let st = new_status.to_string();
                println!("Changing status of app #{} to {}", id, st);
                if let Some(app) = state_rc.borrow_mut().all_apps.iter_mut().find(|a| a.number == id) {
                    app.status = st;
                }
                update_ui(&ui, &state_rc.borrow());
            }
        });
    }

    // Add Company callback
    {
        let ui_handle = ui.as_weak();
        let state_rc = Rc::clone(&state);
        let root = repo_root.clone();
        ui.on_add_company(move |co, ro, ur, st, loc, pay, not| {
            if let Some(ui) = ui_handle.upgrade() {
                let company = co.to_string();
                let role = ro.to_string();
                let url = ur.to_string();
                let status = st.to_string();
                let location = loc.to_string();
                let compensation = pay.to_string();
                let notes = not.to_string();

                println!("Adding application: {} - {}", company, role);
                match career_ops_desktop::tracker::append_application(
                    &root,
                    &company,
                    &role,
                    &url,
                    &status,
                    &location,
                    &compensation,
                    &notes,
                ) {
                    Ok(new_app) => {
                        let new_id = new_app.number;
                        state_rc.borrow_mut().all_apps.push(new_app);
                        state_rc.borrow_mut().selected_id = new_id;
                        update_ui(&ui, &state_rc.borrow());
                    }
                    Err(e) => {
                        eprintln!("Failed to append application: {}", e);
                    }
                }
            }
        });
    }

    ui.run()?;
    Ok(())
}
