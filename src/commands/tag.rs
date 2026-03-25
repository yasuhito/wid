use std::fs;
use std::path::Path;

use anyhow::{Result, anyhow};

use crate::commands::show::print_log_if_changed;
use crate::log::{paths::default_log_path, store};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagAction {
    Add,
    Rm,
}

pub fn run(action: TagAction, id: String, tags: Vec<String>) -> Result<()> {
    let path = default_log_path()?;
    let before = fs::read_to_string(&path).unwrap_or_default();
    run_at_path(&path, action, &id, tags)?;
    print_log_if_changed(&path, &before)
}

pub fn run_at_path(path: &Path, action: TagAction, id: &str, tags: Vec<String>) -> Result<()> {
    let tags = normalize_tags(tags)?;

    match action {
        TagAction::Add => store::add_tags_by_transient_id(path, id, &tags),
        TagAction::Rm => store::remove_tags_by_transient_id(path, id, &tags),
    }
}

fn normalize_tags(tags: Vec<String>) -> Result<Vec<String>> {
    if tags.is_empty() {
        return Err(anyhow!("at least one tag is required"));
    }

    let mut normalized = Vec::new();
    for tag in tags {
        if !tag.starts_with('@') || tag.len() <= 1 {
            return Err(anyhow!("tags must start with @"));
        }

        let bare = &tag[1..];
        if !bare
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
        {
            return Err(anyhow!("tags may only contain letters, numbers, - and _"));
        }

        if !normalized.iter().any(|existing| existing == bare) {
            normalized.push(bare.to_string());
        }
    }

    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::{TagAction, normalize_tags};

    #[test]
    fn normalize_tags_strips_prefix_and_deduplicates() {
        assert_eq!(
            normalize_tags(vec!["@wid".into(), "@agent".into(), "@wid".into()]).unwrap(),
            vec!["wid".to_string(), "agent".to_string()]
        );
    }

    #[test]
    fn normalize_tags_rejects_missing_prefix() {
        let error = normalize_tags(vec!["wid".into()]).unwrap_err();
        assert!(format!("{error:#}").contains("must start with @"));
    }

    #[test]
    fn tag_action_is_copyable() {
        let action = TagAction::Add;
        assert_eq!(action, TagAction::Add);
    }
}
