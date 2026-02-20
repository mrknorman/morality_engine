use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt,
};

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MenuSchema {
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub hint: Option<String>,
    pub layout: MenuLayoutBindings,
    pub options: Vec<MenuOptionSchema>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MenuLayoutBindings {
    pub container: String,
    #[serde(default)]
    pub group: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MenuOptionSchema {
    pub id: String,
    pub label: String,
    pub command: String,
    #[serde(default)]
    pub shortcut: Option<String>,
    pub y: f32,
}

#[derive(Debug, Clone)]
pub struct ResolvedMenuSchema<C> {
    pub id: String,
    pub title: Option<String>,
    pub hint: Option<String>,
    pub layout: MenuLayoutBindings,
    pub options: Vec<ResolvedMenuOption<C>>,
}

#[derive(Debug, Clone)]
pub struct ResolvedMenuOption<C> {
    pub id: String,
    pub label: String,
    pub command: C,
    pub shortcut: Option<String>,
    pub y: f32,
}

#[derive(Debug, Clone)]
pub enum MenuSchemaError {
    Parse(String),
    Validation(String),
    CommandResolution {
        option_id: String,
        command_id: String,
        reason: String,
    },
}

#[derive(Debug, Clone)]
pub struct CommandRegistry<C> {
    by_id: HashMap<String, C>,
}

impl<C: Clone> CommandRegistry<C> {
    pub fn from_entries<I, S>(entries: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = (S, C)>,
        S: Into<String>,
    {
        let mut by_id = HashMap::new();
        for (id, command) in entries {
            let id = id.into();
            if by_id.insert(id.clone(), command).is_some() {
                return Err(format!("duplicate command id `{id}` in command registry"));
            }
        }
        Ok(Self { by_id })
    }

    pub fn resolve(&self, command_id: &str) -> Result<C, String> {
        self.by_id
            .get(command_id)
            .cloned()
            .ok_or_else(|| format!("unknown command `{command_id}`"))
    }
}

impl fmt::Display for MenuSchemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(err) => write!(f, "schema parse error: {err}"),
            Self::Validation(err) => write!(f, "schema validation error: {err}"),
            Self::CommandResolution {
                option_id,
                command_id,
                reason,
            } => write!(
                f,
                "command resolution error for option `{option_id}` / command `{command_id}`: {reason}"
            ),
        }
    }
}

impl Error for MenuSchemaError {}

fn validate_schema(schema: &MenuSchema) -> Result<(), MenuSchemaError> {
    if schema.id.trim().is_empty() {
        return Err(MenuSchemaError::Validation(
            "menu id must not be empty".to_string(),
        ));
    }
    if let Some(title) = &schema.title {
        if title.trim().is_empty() {
            return Err(MenuSchemaError::Validation(
                "menu title must not be blank when provided".to_string(),
            ));
        }
    }
    if let Some(hint) = &schema.hint {
        if hint.trim().is_empty() {
            return Err(MenuSchemaError::Validation(
                "menu hint must not be blank when provided".to_string(),
            ));
        }
    }
    if schema.layout.container.trim().is_empty() {
        return Err(MenuSchemaError::Validation(
            "layout.container must not be empty".to_string(),
        ));
    }
    if let Some(group) = &schema.layout.group {
        if group.trim().is_empty() {
            return Err(MenuSchemaError::Validation(
                "layout.group must not be blank when provided".to_string(),
            ));
        }
    }
    if schema.options.is_empty() {
        return Err(MenuSchemaError::Validation(
            "menu must define at least one option".to_string(),
        ));
    }

    let mut seen_ids = HashSet::new();
    for option in &schema.options {
        if option.id.trim().is_empty() {
            return Err(MenuSchemaError::Validation(
                "option id must not be empty".to_string(),
            ));
        }
        if option.label.trim().is_empty() {
            return Err(MenuSchemaError::Validation(format!(
                "option `{}` label must not be empty",
                option.id
            )));
        }
        if option.command.trim().is_empty() {
            return Err(MenuSchemaError::Validation(format!(
                "option `{}` command must not be empty",
                option.id
            )));
        }
        if let Some(shortcut) = &option.shortcut {
            if shortcut.trim().is_empty() {
                return Err(MenuSchemaError::Validation(format!(
                    "option `{}` shortcut must not be blank",
                    option.id
                )));
            }
        }
        if !seen_ids.insert(option.id.clone()) {
            return Err(MenuSchemaError::Validation(format!(
                "duplicate option id `{}`",
                option.id
            )));
        }
        if !option.y.is_finite() {
            return Err(MenuSchemaError::Validation(format!(
                "option `{}` y must be finite",
                option.id
            )));
        }
    }

    Ok(())
}

pub fn parse_menu_schema(json: &str) -> Result<MenuSchema, MenuSchemaError> {
    let schema: MenuSchema =
        serde_json::from_str(json).map_err(|err| MenuSchemaError::Parse(err.to_string()))?;
    validate_schema(&schema)?;
    Ok(schema)
}

pub fn resolve_menu_schema<C, F>(
    schema: MenuSchema,
    mut command_resolver: F,
) -> Result<ResolvedMenuSchema<C>, MenuSchemaError>
where
    F: FnMut(&str) -> Result<C, String>,
{
    let mut resolved_options = Vec::with_capacity(schema.options.len());
    for option in schema.options {
        let command = command_resolver(&option.command).map_err(|reason| {
            MenuSchemaError::CommandResolution {
                option_id: option.id.clone(),
                command_id: option.command.clone(),
                reason,
            }
        })?;

        resolved_options.push(ResolvedMenuOption {
            id: option.id,
            label: option.label,
            command,
            shortcut: option.shortcut,
            y: option.y,
        });
    }

    Ok(ResolvedMenuSchema {
        id: schema.id,
        title: schema.title,
        hint: schema.hint,
        layout: schema.layout,
        options: resolved_options,
    })
}

pub fn load_and_resolve_menu_schema<C, F>(
    json: &str,
    command_resolver: F,
) -> Result<ResolvedMenuSchema<C>, MenuSchemaError>
where
    F: FnMut(&str) -> Result<C, String>,
{
    let schema = parse_menu_schema(json)?;
    resolve_menu_schema(schema, command_resolver)
}

pub fn load_and_resolve_menu_schema_with_registry<C: Clone>(
    json: &str,
    registry: &CommandRegistry<C>,
) -> Result<ResolvedMenuSchema<C>, MenuSchemaError> {
    load_and_resolve_menu_schema(json, |command| registry.resolve(command))
}

#[cfg(test)]
mod tests {
    use super::{
        load_and_resolve_menu_schema, load_and_resolve_menu_schema_with_registry,
        parse_menu_schema, CommandRegistry,
    };

    const VALID_SCHEMA: &str = r#"
{
  "id": "main_menu",
  "title": "Main",
  "layout": { "container": "menu_selectable_list", "group": "vertical" },
  "options": [
    { "id": "start", "label": "Start", "command": "start", "y": 10.0 },
    { "id": "exit", "label": "Exit", "command": "exit", "y": -10.0 }
  ]
}
"#;

    #[test]
    fn parses_and_validates_schema() {
        let schema = parse_menu_schema(VALID_SCHEMA).expect("valid schema should parse");
        assert_eq!(schema.options.len(), 2);
        assert_eq!(schema.layout.container, "menu_selectable_list");
    }

    #[test]
    fn command_resolution_errors_are_explicit() {
        let error = load_and_resolve_menu_schema(VALID_SCHEMA, |command| match command {
            "start" => Ok(1usize),
            _ => Err("unknown".to_string()),
        })
        .expect_err("missing command mapping should fail");
        let message = format!("{error}");
        assert!(message.contains("command resolution error"));
        assert!(message.contains("exit"));
    }

    #[test]
    fn command_registry_maps_known_ids_and_reports_unknown() {
        let registry =
            CommandRegistry::from_entries([("start", 1usize), ("exit", 2usize)]).expect("valid");
        assert_eq!(
            registry.resolve("start").expect("start should resolve"),
            1usize
        );
        let error = registry
            .resolve("invalid")
            .expect_err("unknown command should fail");
        assert!(error.contains("unknown command"));
    }

    #[test]
    fn command_registry_rejects_duplicate_ids() {
        let error = CommandRegistry::from_entries([("start", 1usize), ("start", 2usize)])
            .expect_err("duplicate ids should fail");
        assert!(error.contains("duplicate command id"));
    }

    #[test]
    fn schema_can_resolve_commands_from_registry() {
        let registry =
            CommandRegistry::from_entries([("start", 10usize), ("exit", 20usize)]).expect("valid");
        let resolved = load_and_resolve_menu_schema_with_registry(VALID_SCHEMA, &registry)
            .expect("schema should resolve");
        assert_eq!(resolved.options.len(), 2);
        assert_eq!(resolved.options[0].command, 10usize);
        assert_eq!(resolved.options[1].command, 20usize);
    }

    #[test]
    fn rejects_blank_optional_fields() {
        let blank_title = r#"
{
  "id": "menu",
  "title": "   ",
  "layout": { "container": "menu_selectable_list", "group": "vertical" },
  "options": [
    { "id": "start", "label": "Start", "command": "start", "y": 10.0 }
  ]
}
"#;
        let error = parse_menu_schema(blank_title).expect_err("blank title should fail");
        assert!(error.to_string().contains("title must not be blank"));

        let blank_group = r#"
{
  "id": "menu",
  "title": "Main",
  "layout": { "container": "menu_selectable_list", "group": "  " },
  "options": [
    { "id": "start", "label": "Start", "command": "start", "y": 10.0 }
  ]
}
"#;
        let error = parse_menu_schema(blank_group).expect_err("blank group should fail");
        assert!(error.to_string().contains("layout.group must not be blank"));
    }
}
