use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DangerLevel {
    Safe,
    Caution,
    Dangerous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetType {
    None,
    Text,
    Branch,
    File,
    Remote,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuilderFocus {
    Options,
    Target,
}

#[derive(Debug, Clone)]
pub struct CommandDoc {
    pub description: &'static str,
    pub when_to_use: &'static str,
    pub examples: Vec<&'static str>,
    pub related: Vec<&'static str>,
    pub danger_level: DangerLevel,
    pub danger_note: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub struct CommandOption {
    pub key: &'static str,
    pub label: &'static str,
    pub cli_flag: &'static str,
    pub help: &'static str,
}

#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub id: &'static str,
    pub category: &'static str,
    pub base: &'static str,
    pub target_label: Option<&'static str>,
    pub target_type: TargetType,
    pub target_format: Option<&'static str>, // e.g. "-m \"{}\""
    pub docs: CommandDoc,
    pub toggles: Vec<CommandOption>,
}

#[derive(Debug, Clone)]
pub struct BuilderState {
    pub selected_command: usize,
    pub selected_option: usize,
    pub enabled_options: BTreeSet<String>,
    pub target_input: String,
    pub selected_suggestion: usize, 
    pub focus: BuilderFocus,
}

impl BuilderState {
    pub fn new() -> Self {
        Self {
            selected_command: 0,
            selected_option: 0,
            enabled_options: BTreeSet::new(),
            target_input: String::new(),
            selected_suggestion: 0,
            focus: BuilderFocus::Options,
        }
    }

    pub fn reset_for_command(&mut self) {
        self.selected_option = 0;
        self.enabled_options.clear();
        self.target_input.clear();
        self.selected_suggestion = 0;
        self.focus = BuilderFocus::Options;
    }

    pub fn toggle_option(&mut self, key: &str) {
        if !self.enabled_options.remove(key) {
            self.enabled_options.insert(key.to_string());
        }
    }

    pub fn option_enabled(&self, key: &str) -> bool {
        self.enabled_options.contains(key)
    }

    pub fn validate(&self, spec: &CommandSpec) -> Vec<String> {
        let mut problems = Vec::new();
        let by_key: BTreeMap<_, _> = spec.toggles.iter().map(|o| (o.key, o)).collect();

        let has = |k: &str| self.option_enabled(k);

        if spec.id == "pull" && has("rebase") && has("ff_only") {
            problems.push("--rebase and --ff-only are usually conflicting intents".to_string());
        }

        if spec.id == "push" && has("force") && has("force_with_lease") {
            problems.push("Use either --force OR --force-with-lease, not both".to_string());
        }

        if spec.target_type != TargetType::None && self.target_input.trim().is_empty() {
            if spec.id == "switch" || spec.id == "branch" || spec.id == "commit" {
                problems.push("A target (branch name, message, etc.) is required".to_string());
            }
        }

        for enabled in &self.enabled_options {
            if !by_key.contains_key(enabled.as_str()) {
                problems.push(format!("Unknown option key: {enabled}"));
            }
        }

        problems
    }

    pub fn preview_command(&self, spec: &CommandSpec) -> String {
        let mut parts: Vec<String> = vec!["git".to_string()];
        parts.extend(spec.base.split_whitespace().map(ToString::to_string));

        for option in &spec.toggles {
            if self.option_enabled(option.key) {
                if spec.target_format.is_none() || !option.cli_flag.contains("{}") {
                    parts.push(option.cli_flag.to_string());
                }
            }
        }

        if spec.target_type != TargetType::None {
            let input = self.target_input.trim();
            if !input.is_empty() {
                if let Some(fmt) = spec.target_format {
                    parts.push(fmt.replace("{}", input));
                } else {
                    parts.push(input.to_string());
                }
            } else if spec.id == "status" && spec.target_type == TargetType::File {
                parts.push(".".to_string());
            }
        }

        parts.join(" ")
    }
}
