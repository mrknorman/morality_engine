use std::{collections::HashSet, error::Error, fmt};

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
    if schema.layout.container.trim().is_empty() {
        return Err(MenuSchemaError::Validation(
            "layout.container must not be empty".to_string(),
        ));
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
        let command =
            command_resolver(&option.command).map_err(|reason| MenuSchemaError::CommandResolution {
                option_id: option.id.clone(),
                command_id: option.command.clone(),
                reason,
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

#[cfg(test)]
mod tests {
    use super::{load_and_resolve_menu_schema, parse_menu_schema};

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
}
