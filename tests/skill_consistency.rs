//! `skills/cwsweep/SKILL.md` がコードの実体とずれていないことを検証する。
//!
//! CLI のフラグ・サブコマンド・既定値・retention の許容値を変えたのに SKILL.md を更新し忘れると、
//! このテストが失敗する（AGENTS.md「CLI を変えたらスキルも更新する」の機械的な担保）。

use std::collections::BTreeSet;
use std::path::PathBuf;

use clap::{Command, CommandFactory};
use cwsweep::cli::{Cli, Commands};
use cwsweep::planner::ALLOWED_RETENTION_DAYS;

const SKILL_NAME: &str = "cwsweep";

fn skill_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("skills")
        .join(SKILL_NAME)
}

fn skill_md() -> String {
    let path = skill_dir().join("SKILL.md");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// frontmatter（先頭の `---` で挟まれた部分）と本文に分ける。
fn split_frontmatter(md: &str) -> (String, String) {
    let rest = md
        .strip_prefix("---\n")
        .expect("SKILL.md must start with `---`");
    let (front, body) = rest
        .split_once("\n---\n")
        .expect("SKILL.md frontmatter must be closed by `---`");
    (front.to_string(), body.to_string())
}

fn frontmatter_value(front: &str, key: &str) -> Option<String> {
    front
        .lines()
        .find_map(|l| l.strip_prefix(&format!("{key}: ")))
        .map(str::to_string)
}

#[test]
fn frontmatter_follows_the_agent_skills_specification() {
    let (front, _) = split_frontmatter(&skill_md());
    let name = frontmatter_value(&front, "name").expect("name is required");
    assert_eq!(name, SKILL_NAME, "name must match the directory name");
    assert!(name.len() <= 64);
    assert!(name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'));
    assert!(!name.starts_with('-') && !name.ends_with('-') && !name.contains("--"));

    let description = frontmatter_value(&front, "description").expect("description is required");
    let chars = description.chars().count();
    assert!(
        (1..=1024).contains(&chars),
        "description must be 1-1024 chars, got {chars}"
    );
}

#[test]
fn retention_values_match_allowed_retention_days() {
    let (_, body) = split_frontmatter(&skill_md());
    let start_key = "CloudWatch Logs が許容する値だけ使える: ";
    let start = body
        .find(start_key)
        .unwrap_or_else(|| panic!("SKILL.md must contain `{start_key}`"))
        + start_key.len();
    let tail = &body[start..];
    let end = tail.find("(出典").expect("source note after the list");
    let listed: Vec<i32> = tail[..end]
        .split(',')
        .map(|v| v.trim().parse().expect("integer in retention list"))
        .collect();
    assert_eq!(listed, ALLOWED_RETENTION_DAYS);
}

#[test]
fn defaults_in_the_skill_match_the_cli_defaults() {
    let (_, body) = split_frontmatter(&skill_md());
    let cli = Cli::parse_from_args(["cwsweep", "clean"]).expect("parse");
    let Commands::Clean(args) = cli.command else {
        panic!("expected clean");
    };
    assert!(
        body.contains(&format!("`{}`", args.role_name)),
        "SKILL.md must mention the default role name {}",
        args.role_name
    );
    assert!(
        body.contains(&format!("`{}`", args.audit_log_path.display())),
        "SKILL.md must mention the default audit log path {}",
        args.audit_log_path.display()
    );
    assert!(!args.execute, "dry-run must remain the default");
}

fn collect_long_flags(cmd: &Command, out: &mut BTreeSet<String>) {
    for arg in cmd.get_arguments() {
        if let Some(long) = arg.get_long() {
            out.insert(format!("--{long}"));
        }
        for alias in arg.get_all_aliases().into_iter().flatten() {
            out.insert(format!("--{alias}"));
        }
    }
    for sub in cmd.get_subcommands() {
        collect_long_flags(sub, out);
    }
}

fn collect_subcommand_paths(cmd: &Command, prefix: &str, out: &mut BTreeSet<String>) {
    for sub in cmd.get_subcommands() {
        let path = format!("{prefix} {}", sub.get_name());
        out.insert(path.clone());
        collect_subcommand_paths(sub, &path, out);
    }
}

#[test]
fn every_flag_mentioned_in_the_skill_exists_in_the_cli() {
    let (_, full_body) = split_frontmatter(&skill_md());
    // `cargo install --git ...` など cwsweep 以外のコマンドのフラグは対象外。
    let body = full_body
        .lines()
        .filter(|l| !l.contains("cargo install"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut known = BTreeSet::new();
    collect_long_flags(&Cli::command(), &mut known);
    known.insert("--help".into());
    known.insert("--version".into());

    let mut mentioned = BTreeSet::new();
    let bytes: Vec<char> = body.chars().collect();
    let mut i = 0;
    while i + 2 < bytes.len() {
        let preceded_ok = i == 0 || !(bytes[i - 1].is_alphanumeric() || bytes[i - 1] == '-');
        if preceded_ok
            && bytes[i] == '-'
            && bytes[i + 1] == '-'
            && bytes[i + 2].is_ascii_lowercase()
        {
            let mut j = i + 2;
            while j < bytes.len() && (bytes[j].is_ascii_lowercase() || bytes[j] == '-') {
                j += 1;
            }
            mentioned.insert(bytes[i..j].iter().collect::<String>());
            i = j;
        } else {
            i += 1;
        }
    }
    assert!(!mentioned.is_empty());
    for flag in &mentioned {
        assert!(
            known.contains(flag),
            "SKILL.md mentions {flag}, which the CLI does not define"
        );
    }
    // 手順の核になるフラグが本文から消えていないこと。
    for must in [
        "--regions",
        "--execute",
        "--days",
        "--output",
        "--role-name",
    ] {
        assert!(mentioned.contains(must), "SKILL.md must mention {must}");
    }
}

#[test]
fn every_cwsweep_command_in_code_blocks_is_a_real_subcommand() {
    let (_, body) = split_frontmatter(&skill_md());
    let mut paths = BTreeSet::new();
    collect_subcommand_paths(&Cli::command(), "cwsweep", &mut paths);

    let mut in_block = false;
    let mut checked = 0;
    for line in body.lines() {
        if line.trim_start().starts_with("```") {
            in_block = !in_block;
            continue;
        }
        let line = line.trim_start();
        if !in_block || !line.starts_with("cwsweep ") {
            continue;
        }
        let words: Vec<&str> = line.split_whitespace().collect();
        if words[1].starts_with('-') {
            continue; // `cwsweep --version` など
        }
        let two = format!(
            "cwsweep {} {}",
            words[1],
            words.get(2).copied().unwrap_or("")
        );
        let one = format!("cwsweep {}", words[1]);
        assert!(
            paths.contains(two.trim_end()) || paths.contains(&one),
            "unknown subcommand in SKILL.md code block: {line}"
        );
        // `retention` は必ず `set` まで書く（単独では実行できない）。
        if words[1] == "retention" {
            assert_eq!(words.get(2), Some(&"set"), "incomplete command: {line}");
        }
        checked += 1;
    }
    assert!(
        checked >= 5,
        "expected several cwsweep commands, got {checked}"
    );
}
