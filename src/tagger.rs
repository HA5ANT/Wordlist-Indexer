use std::path::{Path, Component};

pub fn get_tags_for_path(path: &Path) -> Vec<String> {
    let mut tags = Vec::new();
    let components: Vec<String> = path.components()
        .filter_map(|c| {
            if let Component::Normal(os_str) = c {
                Some(os_str.to_string_lossy().to_lowercase())
            } else {
                None
            }
        })
        .collect();

    // Taxonomy rules (matching against components)
    for comp in &components {
        match comp.as_str() {
            "discovery" => add_tag(&mut tags, "discovery"),
            "webcontent" => {
                add_tag(&mut tags, "webcontent");
                add_tag(&mut tags, "web");
                add_tag(&mut tags, "discovery");
            },
            "webshells" => {
                add_tag(&mut tags, "webshells");
                add_tag(&mut tags, "web");
            },
            "dns" => {
                add_tag(&mut tags, "dns");
                add_tag(&mut tags, "discovery");
            },
            "infra" => {
                add_tag(&mut tags, "infra");
                add_tag(&mut tags, "discovery");
            },
            "snmp" => {
                add_tag(&mut tags, "snmp");
                add_tag(&mut tags, "infra");
            },
            "mainframe" => {
                add_tag(&mut tags, "mainframe");
                add_tag(&mut tags, "infra");
            },
            "iot" => {
                add_tag(&mut tags, "iot");
                add_tag(&mut tags, "infra");
            },
            "ports" => {
                add_tag(&mut tags, "ports");
                add_tag(&mut tags, "infra");
            },
            "fuzzing" => add_tag(&mut tags, "fuzzing"),
            "xss" => {
                add_tag(&mut tags, "xss");
                add_tag(&mut tags, "fuzzing");
                add_tag(&mut tags, "injection");
            },
            "sqli" => {
                add_tag(&mut tags, "sqli");
                add_tag(&mut tags, "fuzzing");
                add_tag(&mut tags, "injection");
            },
            "lfi" => {
                add_tag(&mut tags, "lfi");
                add_tag(&mut tags, "fuzzing");
                add_tag(&mut tags, "injection");
            },
            "ssrf" => {
                add_tag(&mut tags, "ssrf");
                add_tag(&mut tags, "fuzzing");
                add_tag(&mut tags, "injection");
            },
            "ssti" => {
                add_tag(&mut tags, "ssti");
                add_tag(&mut tags, "fuzzing");
                add_tag(&mut tags, "injection");
            },
            "xxe" => {
                add_tag(&mut tags, "xxe");
                add_tag(&mut tags, "fuzzing");
                add_tag(&mut tags, "injection");
            },
            "cmdi" => {
                add_tag(&mut tags, "cmdi");
                add_tag(&mut tags, "fuzzing");
                add_tag(&mut tags, "injection");
            },
            "passwords" => add_tag(&mut tags, "passwords"),
            "creds" => {
                add_tag(&mut tags, "creds");
                add_tag(&mut tags, "passwords");
            },
            "leaked" => {
                add_tag(&mut tags, "leaked");
                add_tag(&mut tags, "passwords");
            },
            "permutations" => {
                add_tag(&mut tags, "permutations");
                add_tag(&mut tags, "passwords");
            },
            "usernames" => add_tag(&mut tags, "usernames"),
            "patterns" => add_tag(&mut tags, "patterns"),
            "payloads" => add_tag(&mut tags, "payloads"),
            "ai" => add_tag(&mut tags, "ai"),
            _ => {}
        }
    }

    tags
}

fn add_tag(tags: &mut Vec<String>, tag: &str) {
    if !tags.contains(&tag.to_string()) {
        tags.push(tag.to_string());
    }
}
