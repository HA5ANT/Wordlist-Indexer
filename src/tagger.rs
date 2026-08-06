use std::path::Path;

pub fn get_tags_for_path(path: &Path) -> Vec<String> {
    let path_str = path.to_string_lossy().to_lowercase();
    let mut tags = Vec::new();

    // Taxonomy rules
    if path_str.contains("discovery") {
        tags.push("discovery".to_string());
    }
    if path_str.contains("webcontent") {
        add_tag(&mut tags, "webcontent");
        add_tag(&mut tags, "web");
        add_tag(&mut tags, "discovery");
    }
    if path_str.contains("webshells") {
        add_tag(&mut tags, "webshells");
        add_tag(&mut tags, "web");
    }
    if path_str.contains("dns") {
        add_tag(&mut tags, "dns");
        add_tag(&mut tags, "discovery");
    }
    if path_str.contains("infra") {
        add_tag(&mut tags, "infra");
        add_tag(&mut tags, "discovery");
    }
    if path_str.contains("snmp") {
        add_tag(&mut tags, "snmp");
        add_tag(&mut tags, "infra");
    }
    if path_str.contains("mainframe") {
        add_tag(&mut tags, "mainframe");
        add_tag(&mut tags, "infra");
    }
    if path_str.contains("iot") {
        add_tag(&mut tags, "iot");
        add_tag(&mut tags, "infra");
    }
    if path_str.contains("ports") {
        add_tag(&mut tags, "ports");
        add_tag(&mut tags, "infra");
    }
    if path_str.contains("fuzzing") {
        add_tag(&mut tags, "fuzzing");
    }
    if path_str.contains("xss") {
        add_tag(&mut tags, "xss");
        add_tag(&mut tags, "fuzzing");
        add_tag(&mut tags, "injection");
    }
    if path_str.contains("sqli") {
        add_tag(&mut tags, "sqli");
        add_tag(&mut tags, "fuzzing");
        add_tag(&mut tags, "injection");
    }
    if path_str.contains("lfi") {
        add_tag(&mut tags, "lfi");
        add_tag(&mut tags, "fuzzing");
        add_tag(&mut tags, "injection");
    }
    if path_str.contains("ssrf") {
        add_tag(&mut tags, "ssrf");
        add_tag(&mut tags, "fuzzing");
        add_tag(&mut tags, "injection");
    }
    if path_str.contains("ssti") {
        add_tag(&mut tags, "ssti");
        add_tag(&mut tags, "fuzzing");
        add_tag(&mut tags, "injection");
    }
    if path_str.contains("xxe") {
        add_tag(&mut tags, "xxe");
        add_tag(&mut tags, "fuzzing");
        add_tag(&mut tags, "injection");
    }
    if path_str.contains("cmdi") {
        add_tag(&mut tags, "cmdi");
        add_tag(&mut tags, "fuzzing");
        add_tag(&mut tags, "injection");
    }
    if path_str.contains("passwords") {
        add_tag(&mut tags, "passwords");
    }
    if path_str.contains("creds") {
        add_tag(&mut tags, "creds");
        add_tag(&mut tags, "passwords");
    }
    if path_str.contains("leaked") {
        add_tag(&mut tags, "leaked");
        add_tag(&mut tags, "passwords");
    }
    if path_str.contains("permutations") {
        add_tag(&mut tags, "permutations");
        add_tag(&mut tags, "passwords");
    }
    if path_str.contains("usernames") {
        add_tag(&mut tags, "usernames");
    }
    if path_str.contains("patterns") {
        add_tag(&mut tags, "patterns");
    }
    if path_str.contains("payloads") {
        add_tag(&mut tags, "payloads");
    }
    if path_str.contains("ai") {
        add_tag(&mut tags, "ai");
    }

    tags
}

fn add_tag(tags: &mut Vec<String>, tag: &str) {
    if !tags.contains(&tag.to_string()) {
        tags.push(tag.to_string());
    }
}
