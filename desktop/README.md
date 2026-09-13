# Career-Ops Desktop

Standalone, native GUI application for the career-ops pipeline built with [Slint](https://slint.dev) and Rust.

## Features

- **Triple-View Workflow**:
  - **Table View (`⊞ Table`)**: Full-featured data grid showing Application `#`, Company, Role, Score badge, Lifecycle status pill, Location / Work Mode, Compensation range, and Date.
  - **Kanban Board (`▥ Kanban`)**: Visual workflow columns (`Evaluated / Inbox` → `Applied` → `Interview` → `Offer / Hired` → `Discarded / Rejected`).
  - **Pipeline & Survey (`⚡ Pipeline & Survey`)**:
    - Browse discovered roles directly from `data/pipeline.md`.
    - Generates dynamic, targeted technical surveys based on the job role (Backend, Full-Stack, Applied AI/ML).
    - Answer questions to systematically extract verified skills and STAR project highlights (with concrete metrics).
    - Persists inventory to `data/skills-inventory.json` / `data/skills-inventory.md`.
    - One-click synthesis and merge into your master CV (`cv.md`).
- **Live Search & Filter Tabs**:
  - Real-time instant query filter matching company names, job roles, locations, notes, and archetypes.
  - Tab filters: `All`, `Evaluated`, `Interview`, `Responded`, `Applied`, `Top (★≥4.0)`, `Skip / Discard`, `Rejected`.
- **KPI Metrics Dashboard**:
  - Summary cards in the header for Total Tracked, Applied, Active Interviews, Received Offers, and Top Score.
- **Detail Inspector Drawer**:
  - Shows metadata chips (Location, Work Mode, Pay Range, Dates).
  - Previews evaluation report TL;DR summary and notes.
  - Direct actions:
    - 🔗 **Open Job Posting**: Opens URL in your default desktop browser.
    - 📄 **Open Tailored CV PDF**: Opens matching PDF in your default PDF viewer.
    - **Quick Status Transitions**: Change status to Applied, Interview, Offer, or Discarded directly.
- **Theme**:
  - Built-in Catppuccin Mocha dark theme.
- **Data Contract Compliant**:
  - Reads `data/applications.md`, `data/pipeline.md`, and `reports/*.md`.
  - Fully decoupled in `desktop/` as part of the System Layer, never modifying or overwriting your personal files unexpectedly.

## Running

From the repository root:

```bash
# Launch the desktop app
npm run desktop

# Or directly with Cargo
cd desktop && cargo run

# Point to an explicit data directory or fixture
cargo run -- --path /path/to/custom-career-ops
```

To build a release binary:

```bash
npm run build:desktop
# Binary generated at desktop/target/release/career-ops-desktop
```
