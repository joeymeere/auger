use std::collections::HashMap;

pub fn normalize_project_name(project: &str, main_project: &str) -> String {
    if project == main_project || project.is_empty() || main_project.is_empty() {
        return project.to_string();
    }
    let prefixes = ["rs", "zero", "value", "lib"];
    for prefix in prefixes {
        let prefix_with_project = format!("{}{}", prefix, main_project);
        if project == prefix_with_project {
            return main_project.to_string();
        }
    }
    let suffixes = ["_v2", "_v3", "_v4", "_lib", "_core"];
    for suffix in suffixes {
        let project_with_suffix = format!("{}{}", main_project, suffix);
        if project == project_with_suffix {
            return main_project.to_string();
        }
    }
    if project.len() > 20 && project.contains(main_project) {
        return main_project.to_string();
    }
    if project.ends_with(main_project) && project.len() > main_project.len() {
        return main_project.to_string();
    }

    project.to_string()
}

pub fn count_projects_by_name<T>(
    files: &[T],
    project_getter: impl Fn(&T) -> &str,
) -> HashMap<String, usize> {
    let mut project_counts = HashMap::new();
    for file in files {
        let project = project_getter(file).to_string();
        *project_counts.entry(project).or_insert(0) += 1;
    }

    project_counts
}

pub fn find_main_project<T>(files: &[T], project_getter: impl Fn(&T) -> &str) -> Option<String> {
    let project_counts = count_projects_by_name(files, project_getter);
    let mut filtered_counts = project_counts.clone();
    for std_lib in crate::consts::STD_LIB_NAMES {
        filtered_counts.remove(*std_lib);
    }

    filtered_counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(project, _)| project)
}

pub fn normalize_source_path(path: &str) -> String {
    let mut normalized = path.to_string();
    while normalized.contains("//") {
        normalized = normalized.replace("//", "/");
    }
    if !normalized.starts_with("programs/") && !normalized.contains("/src/") {
        if normalized.starts_with("src/") {
            normalized = format!(
                "src/{}",
                normalized.strip_prefix("src/").unwrap_or(&normalized)
            );
        } else {
            normalized = format!("src/{}", normalized);
        }
    }
    if normalized.contains("/src/src/") {
        normalized = normalized.replace("/src/src/", "/src/");
    }
    if !normalized.ends_with(".rs") && !normalized.contains(".") {
        normalized = format!("{}.rs", normalized);
    }

    normalized
}

pub fn should_use_custom_parser(linker_info: Option<&str>) -> bool {
    if let Some(linker) = linker_info {
        // TODO: understand linker patterns better
        if linker.contains("LLD") || linker.contains("LLVM") {
            return true;
        }
    }

    false
}
