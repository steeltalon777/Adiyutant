use adiyutant_core::service::AdiyutantCoreService;

pub fn handle_list(facade: &AdiyutantCoreService) {
    match facade.list_projects() {
        Ok(projects) => {
            if projects.is_empty() {
                println!("No projects found.");
                return;
            }
            println!("Projects:");
            for p in &projects {
                println!("  {}  {}  [{}]", p.slug, p.title, p.status);
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

pub fn handle_show(facade: &AdiyutantCoreService, slug: &str) {
    match facade.get_project(slug) {
        Ok(detail) => {
            println!(
                "Project: {} ({})",
                detail.project.title, detail.project.slug
            );
            println!("  Status: {}", detail.project.status);
            println!("  Priority: {}", detail.project.priority);
            if !detail.project.description.is_empty() {
                println!("  Description: {}", detail.project.description);
            }
            if !detail.project.why.is_empty() {
                println!("  Why: {}", detail.project.why);
            }
            println!("  Roadmaps:");
            for r in &detail.roadmaps {
                println!("    {}  {}  [{}]", r.slug, r.title, r.status);
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}
