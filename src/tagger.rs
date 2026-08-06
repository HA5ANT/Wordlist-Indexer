use std::path::{Component, Path};

struct Rule {
    keyword: &'static str,
    tags: &'static [&'static str],
}

const RULES: &[Rule] = &[
    Rule {
        keyword: "discovery",
        tags: &["discovery"],
    },
    Rule {
        keyword: "webcontent",
        tags: &["webcontent", "web", "discovery"],
    },
    Rule {
        keyword: "webshells",
        tags: &["webshells", "web"],
    },
    Rule {
        keyword: "dns",
        tags: &["dns", "discovery"],
    },
    Rule {
        keyword: "infra",
        tags: &["infra", "discovery"],
    },
    Rule {
        keyword: "infrastructure",
        tags: &["infra", "discovery"],
    },
    Rule {
        keyword: "snmp",
        tags: &["snmp", "infra"],
    },
    Rule {
        keyword: "mainframe",
        tags: &["mainframe", "infra"],
    },
    Rule {
        keyword: "iot",
        tags: &["iot", "infra"],
    },
    Rule {
        keyword: "ports",
        tags: &["ports", "infra"],
    },
    Rule {
        keyword: "fuzzing",
        tags: &["fuzzing"],
    },
    Rule {
        keyword: "xss",
        tags: &["xss", "fuzzing", "injection"],
    },
    Rule {
        keyword: "sqli",
        tags: &["sqli", "fuzzing", "injection"],
    },
    Rule {
        keyword: "sql",
        tags: &["sqli", "fuzzing", "injection"],
    },
    Rule {
        keyword: "lfi",
        tags: &["lfi", "fuzzing", "injection"],
    },
    Rule {
        keyword: "ssrf",
        tags: &["ssrf", "fuzzing", "injection"],
    },
    Rule {
        keyword: "ssti",
        tags: &["ssti", "fuzzing", "injection"],
    },
    Rule {
        keyword: "xxe",
        tags: &["xxe", "fuzzing", "injection"],
    },
    Rule {
        keyword: "cmdi",
        tags: &["cmdi", "fuzzing", "injection"],
    },
    Rule {
        keyword: "commandinjection",
        tags: &["cmdi", "fuzzing", "injection"],
    },
    Rule {
        keyword: "passwords",
        tags: &["passwords"],
    },
    Rule {
        keyword: "password",
        tags: &["passwords"],
    },
    Rule {
        keyword: "creds",
        tags: &["creds", "passwords"],
    },
    Rule {
        keyword: "credentials",
        tags: &["creds", "passwords"],
    },
    Rule {
        keyword: "credential",
        tags: &["creds", "passwords"],
    },
    Rule {
        keyword: "leaked",
        tags: &["leaked", "passwords"],
    },
    Rule {
        keyword: "leak",
        tags: &["leaked", "passwords"],
    },
    Rule {
        keyword: "leaks",
        tags: &["leaked", "passwords"],
    },
    Rule {
        keyword: "permutations",
        tags: &["permutations", "passwords"],
    },
    Rule {
        keyword: "permutation",
        tags: &["permutations", "passwords"],
    },
    Rule {
        keyword: "usernames",
        tags: &["usernames"],
    },
    Rule {
        keyword: "username",
        tags: &["usernames"],
    },
    Rule {
        keyword: "patterns",
        tags: &["patterns"],
    },
    Rule {
        keyword: "pattern",
        tags: &["patterns"],
    },
    Rule {
        keyword: "payloads",
        tags: &["payloads"],
    },
    Rule {
        keyword: "payload",
        tags: &["payloads"],
    },
    Rule {
        keyword: "ai",
        tags: &["ai"],
    },
    Rule {
        keyword: "llm",
        tags: &["ai"],
    },
];

pub fn get_tags_for_path(path: &Path) -> Vec<String> {
    let mut tags = Vec::new();

    for comp in path.components() {
        let Component::Normal(os_str) = comp else {
            continue;
        };
        let lower = os_str.to_string_lossy().to_ascii_lowercase();
        let squashed: String = lower.chars().filter(|c| c.is_alphanumeric()).collect();
        let tokens: Vec<&str> = lower
            .split(|c: char| !c.is_alphanumeric())
            .filter(|t| !t.is_empty())
            .collect();

        for rule in RULES {
            if rule.keyword == squashed || tokens.contains(&rule.keyword) {
                for tag in rule.tags {
                    add_tag(&mut tags, tag);
                }
            }
        }
    }

    tags
}

fn add_tag(tags: &mut Vec<String>, tag: &str) {
    if !tags.contains(&tag.to_string()) {
        tags.push(tag.to_string());
    }
}
