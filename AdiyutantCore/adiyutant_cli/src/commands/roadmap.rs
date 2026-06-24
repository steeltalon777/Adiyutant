use adiyutant_core::service::AdiyutantCoreService;

pub fn handle_list(facade: &AdiyutantCoreService, project_slug: &Option<String>) {
    match facade.list_roadmaps(project_slug.as_deref()) {
        Ok(roadmaps) => {
            if roadmaps.is_empty() {
                println!("No roadmaps found.");
                return;
            }
            println!("Roadmaps:");
            for r in &roadmaps {
                println!(
                    "  {}/{}  {}  [{}]  horizon: {}",
                    r.project_slug, r.slug, r.title, r.status, r.horizon
                );
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

pub fn handle_show(facade: &AdiyutantCoreService, selector: &str) {
    match facade.get_roadmap(selector) {
        Ok(detail) => {
            println!(
                "Roadmap: {} ({}/{})",
                detail.roadmap.title, detail.project_slug, detail.roadmap.slug
            );
            println!("  Status: {}", detail.roadmap.status);
            println!("  Horizon: {}", detail.roadmap.horizon);
            if !detail.roadmap.description.is_empty() {
                println!("  Description: {}", detail.roadmap.description);
            }
            println!("  Phases:");
            for pw in &detail.phases {
                println!(
                    "    [{}] {}  ({} items)",
                    pw.phase.status,
                    pw.phase.title,
                    pw.items.len()
                );
                for item in &pw.items {
                    println!("      - {}  {}  [{}]", item.slug, item.title, item.status);
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

pub fn handle_items(facade: &AdiyutantCoreService, selector: &str) {
    match facade.list_roadmap_items(selector) {
        Ok(items) => {
            if items.is_empty() {
                println!("No items found.");
                return;
            }
            println!("Roadmap Items:");
            for item in &items {
                println!(
                    "  {}  {}  [{}]  priority: {}",
                    item.slug, item.title, item.status, item.priority
                );
                if !item.description.is_empty() {
                    println!("    Description: {}", item.description);
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

pub fn handle_take_item(facade: &AdiyutantCoreService, selector: &str, date: &str) {
    let target_date = match date {
        "today" => chrono::Utc::now().date_naive(),
        "tomorrow" => chrono::Utc::now().date_naive() + chrono::Duration::days(1),
        _ => {
            eprintln!("Error: --date must be 'today' or 'tomorrow'");
            std::process::exit(1);
        }
    };

    match facade.take_roadmap_item(selector, target_date) {
        Ok(result) => {
            println!("Take-item result:");
            println!(
                "  Roadmap item: {}  ({})",
                result.roadmap_item.title, result.roadmap_item.slug
            );
            println!("  Plan item id: {}", result.plan_item.id);
            println!("  Link id: {}", result.link.id);
            println!("  Target date: {}", result.target_date);
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}
