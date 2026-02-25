use std::{collections::BTreeMap, fmt::Write as _, fs, io, path::Path};

use super::{
    schema::{RouteDefinition, RouteRule, SceneProgressionGraph, SceneRef},
    validate::{validate_graph, GraphValidationError},
};

const CAMPAIGN_GRAPH_JSON: &str = include_str!("./content/campaign_graph.json");

#[derive(Debug, Clone, Copy)]
pub struct CampaignGraphExportSummary {
    pub route_count: usize,
    pub validation_error_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RouteFilter {
    All,
    Lab,
    Inaction,
    Deontological,
    Utilitarian,
}

impl RouteFilter {
    const ORDER: [Self; 5] = [
        Self::All,
        Self::Lab,
        Self::Inaction,
        Self::Deontological,
        Self::Utilitarian,
    ];

    fn key(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Lab => "lab",
            Self::Inaction => "inaction",
            Self::Deontological => "deontological",
            Self::Utilitarian => "utilitarian",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::All => "All Routes",
            Self::Lab => "Lab Routes",
            Self::Inaction => "Inaction Path Routes",
            Self::Deontological => "Deontological Path Routes",
            Self::Utilitarian => "Utilitarian Path Routes",
        }
    }

    fn matches_route(self, route: &RouteDefinition) -> bool {
        if self == Self::All {
            return true;
        }

        let SceneRef::Dilemma { id } = &route.from else {
            return false;
        };

        match self {
            Self::All => true,
            Self::Lab => id.starts_with("lab_"),
            Self::Inaction => id.starts_with("path_inaction."),
            Self::Deontological => id.starts_with("path_deontological."),
            Self::Utilitarian => id.starts_with("path_utilitarian."),
        }
    }
}

#[derive(Debug, Clone)]
struct EdgeLine {
    from: SceneRef,
    to: SceneRef,
    label: String,
}

pub fn export_campaign_graph_html(
    output_path: impl AsRef<Path>,
) -> io::Result<CampaignGraphExportSummary> {
    let path = output_path.as_ref();

    let (html, summary) = match serde_json::from_str::<SceneProgressionGraph>(CAMPAIGN_GRAPH_JSON) {
        Ok(graph) => {
            let validation_errors = validate_graph(&graph);
            let html = build_html(&graph, &validation_errors);
            let summary = CampaignGraphExportSummary {
                route_count: graph.routes.len(),
                validation_error_count: validation_errors.len(),
            };
            (html, summary)
        }
        Err(error) => {
            let html = build_parse_error_html(&error.to_string());
            let summary = CampaignGraphExportSummary {
                route_count: 0,
                validation_error_count: 1,
            };
            (html, summary)
        }
    };

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, html)?;
    Ok(summary)
}

fn build_html(graph: &SceneProgressionGraph, validation_errors: &[GraphValidationError]) -> String {
    let diagrams = RouteFilter::ORDER
        .iter()
        .map(|filter| (*filter, build_mermaid_diagram(graph, *filter)))
        .collect::<Vec<_>>();

    let has_errors = !validation_errors.is_empty();
    let status_text = if has_errors {
        format!("Validation: FAIL ({} errors)", validation_errors.len())
    } else {
        String::from("Validation: PASS")
    };
    let status_class = if has_errors { "fail" } else { "pass" };

    let mut tabs_html = String::new();
    let mut panels_html = String::new();
    for (index, (filter, diagram)) in diagrams.iter().enumerate() {
        let active = index == 0;
        let active_class = if active { "active" } else { "" };
        let _ = writeln!(
            tabs_html,
            r#"<button class="tab {active_class}" data-target="{key}">{title}</button>"#,
            key = filter.key(),
            title = filter.title()
        );

        let _ = writeln!(
            panels_html,
            r#"<section id="panel-{key}" class="diagram-panel {active_class}" data-key="{key}" data-rendered="false"><pre class="diagram-source">{diagram}</pre><div class="diagram-canvas"></div></section>"#,
            key = filter.key(),
            diagram = diagram
        );
    }

    let mut validation_list = String::new();
    if has_errors {
        validation_list.push_str("<ul>\n");
        for error in validation_errors {
            let _ = writeln!(
                validation_list,
                "<li><code>{}</code></li>",
                escape_html(&describe_validation_error(error))
            );
        }
        validation_list.push_str("</ul>\n");
    } else {
        validation_list.push_str("<p>No validation errors.</p>\n");
    }

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Campaign Graph Viewer</title>
  <style>
    :root {{
      --bg: #0f1117;
      --panel: #181c25;
      --text: #e5e9f0;
      --muted: #9aa6bd;
      --ok: #66d17a;
      --err: #ff6b6b;
      --accent: #6ea8ff;
      --border: #2b3240;
    }}
    body {{
      margin: 0;
      background: var(--bg);
      color: var(--text);
      font-family: ui-sans-serif, system-ui, -apple-system, Segoe UI, Roboto, Arial, sans-serif;
    }}
    main {{
      max-width: 1400px;
      margin: 0 auto;
      padding: 20px;
      display: grid;
      gap: 16px;
    }}
    .card {{
      background: var(--panel);
      border: 1px solid var(--border);
      border-radius: 12px;
      padding: 14px 16px;
    }}
    .status {{
      font-weight: 700;
    }}
    .status.pass {{ color: var(--ok); }}
    .status.fail {{ color: var(--err); }}
    .meta {{
      color: var(--muted);
      margin-top: 8px;
      font-size: 0.95rem;
    }}
    .tabs {{
      display: flex;
      flex-wrap: wrap;
      gap: 8px;
      margin-bottom: 12px;
    }}
    .tab {{
      border: 1px solid var(--border);
      color: var(--text);
      background: transparent;
      border-radius: 8px;
      padding: 8px 12px;
      cursor: pointer;
      font-weight: 600;
    }}
    .tab.active {{
      border-color: var(--accent);
      color: var(--accent);
      background: rgba(110, 168, 255, 0.12);
    }}
    .diagram-panel {{ display: none; }}
    .diagram-panel.active {{ display: block; }}
    .diagram-source {{
      display: none;
    }}
    .diagram-canvas {{
      background: #111520;
      border: 1px solid var(--border);
      border-radius: 8px;
      padding: 8px;
      overflow: auto;
      min-height: 160px;
    }}
    .diagram-canvas > svg {{
      width: 100%;
      height: auto;
    }}
    pre.parse-error {{
      background: #111520;
      border: 1px solid var(--border);
      border-radius: 8px;
      padding: 8px;
      overflow: auto;
      color: #ff6b6b;
    }}
    code {{
      color: #f7d37a;
      white-space: pre-wrap;
      word-break: break-word;
    }}
  </style>
</head>
<body>
  <main>
    <section class="card">
      <h1>Campaign Graph Viewer</h1>
      <div class="status {status_class}">{status_text}</div>
      <div class="meta">
        Source: <code>src/scenes/flow/content/campaign_graph.json</code><br/>
        Routes: <strong>{route_count}</strong>
      </div>
    </section>

    <section class="card">
      <h2>Validation Details</h2>
      {validation_list}
    </section>

    <section class="card">
      <h2>Route Diagrams</h2>
      <div class="tabs">
        {tabs_html}
      </div>
      {panels_html}
    </section>
  </main>

  <script src="https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js"></script>
  <script>
    mermaid.initialize({{ startOnLoad: false, securityLevel: "loose", theme: "dark" }});

    const tabs = Array.from(document.querySelectorAll(".tab"));
    const panels = Array.from(document.querySelectorAll(".diagram-panel"));

    async function renderPanel(key) {{
      const panel = document.getElementById(`panel-${{key}}`);
      if (!panel || panel.dataset.rendered === "true") {{
        return;
      }}

      const source = panel.querySelector(".diagram-source")?.textContent ?? "";
      const canvas = panel.querySelector(".diagram-canvas");
      if (!canvas) {{
        panel.dataset.rendered = "true";
        return;
      }}

      try {{
        const renderId = `campaign-${{key}}-${{Math.random().toString(36).slice(2)}}`;
        const result = await mermaid.render(renderId, source);
        canvas.innerHTML = result.svg;
        if (result.bindFunctions) {{
          result.bindFunctions(canvas);
        }}
      }} catch (error) {{
        console.error("Mermaid parse failed", key, error, source);
        const message = error && typeof error.message === "string"
          ? error.message
          : String(error);
        const pre = document.createElement("pre");
        pre.className = "parse-error";
        pre.textContent = `Mermaid parse failed in panel '${{key}}': ${{message}}\n\n${{source}}`;
        canvas.replaceChildren(pre);
      }}

      panel.dataset.rendered = "true";
    }}

    function activate(key) {{
      for (const tab of tabs) {{
        tab.classList.toggle("active", tab.dataset.target === key);
      }}
      for (const panel of panels) {{
        panel.classList.toggle("active", panel.id === `panel-${{key}}`);
      }}
      void renderPanel(key);
    }}

    for (const tab of tabs) {{
      tab.addEventListener("click", () => activate(tab.dataset.target));
    }}

    activate("all");
  </script>
</body>
</html>
"#,
        status_class = status_class,
        status_text = status_text,
        route_count = graph.routes.len(),
        validation_list = validation_list,
        tabs_html = tabs_html,
        panels_html = panels_html,
    )
}

fn build_parse_error_html(error: &str) -> String {
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Campaign Graph Viewer</title>
  <style>
    body {{
      background: #0f1117;
      color: #e5e9f0;
      font-family: ui-sans-serif, system-ui, -apple-system, Segoe UI, Roboto, Arial, sans-serif;
      margin: 0;
      padding: 24px;
    }}
    .card {{
      max-width: 1000px;
      margin: 0 auto;
      background: #181c25;
      border: 1px solid #2b3240;
      border-radius: 12px;
      padding: 16px;
    }}
    .error {{
      color: #ff6b6b;
      font-weight: 700;
    }}
    code {{
      color: #f7d37a;
      white-space: pre-wrap;
    }}
  </style>
</head>
<body>
  <section class="card">
    <h1>Campaign Graph Viewer</h1>
    <p class="error">Validation: FAIL (parse error)</p>
    <p><code>{}</code></p>
  </section>
</body>
</html>"#,
        escape_html(error)
    )
}

fn build_mermaid_diagram(graph: &SceneProgressionGraph, filter: RouteFilter) -> String {
    let mut edges = Vec::new();
    for route in graph
        .routes
        .iter()
        .filter(|route| filter.matches_route(route))
    {
        build_route_edges(route, &mut edges);
    }

    if edges.is_empty() {
        return String::from("flowchart TD\n    empty[\"No routes for this filter\"]");
    }

    let mut scene_by_key = BTreeMap::<String, SceneRef>::new();
    for edge in &edges {
        scene_by_key.insert(scene_key(&edge.from), edge.from.clone());
        scene_by_key.insert(scene_key(&edge.to), edge.to.clone());
    }

    let mut node_id_by_key = BTreeMap::<String, String>::new();
    for (index, key) in scene_by_key.keys().enumerate() {
        node_id_by_key.insert(key.clone(), format!("N{index}"));
    }

    let mut output = String::from("flowchart TD\n");
    for (key, scene) in &scene_by_key {
        let node_id = &node_id_by_key[key];
        let label = sanitize_mermaid_node_label(&scene_label(scene));
        let class_name = scene_css_class(scene);
        let _ = writeln!(output, "    {node_id}[\"{label}\"]:::{class_name}");
    }

    for edge in edges {
        let from_id = &node_id_by_key[&scene_key(&edge.from)];
        let to_id = &node_id_by_key[&scene_key(&edge.to)];
        let label = sanitize_mermaid_edge_label(&edge.label);
        if label.is_empty() {
            let _ = writeln!(output, "    {from_id} --> {to_id}");
        } else {
            let _ = writeln!(output, "    {from_id} -->|{label}| {to_id}");
        }
    }

    output.push('\n');
    output.push_str("    classDef scene_menu fill:#6fa8dc,stroke:#1b3b5a,color:#0f1117;\n");
    output.push_str("    classDef scene_loading fill:#ffe599,stroke:#8f6c00,color:#0f1117;\n");
    output.push_str("    classDef scene_dialogue fill:#b4a7d6,stroke:#4d3f71,color:#0f1117;\n");
    output.push_str("    classDef scene_dilemma fill:#93c47d,stroke:#2f5b23,color:#0f1117;\n");
    output.push_str("    classDef scene_ending fill:#ea9999,stroke:#7a2f2f,color:#0f1117;\n");

    output
}

fn build_route_edges(route: &RouteDefinition, edges: &mut Vec<EdgeLine>) {
    for rule in &route.rules {
        let first_label = rule_entry_label(rule);
        push_sequence_edges(route.from.clone(), &rule.then, first_label, edges);
    }

    push_sequence_edges(
        route.from.clone(),
        &route.default_then,
        String::from("default"),
        edges,
    );
}

fn push_sequence_edges(
    from: SceneRef,
    then: &[SceneRef],
    first_label: String,
    edges: &mut Vec<EdgeLine>,
) {
    if then.is_empty() {
        return;
    }

    edges.push(EdgeLine {
        from: from.clone(),
        to: then[0].clone(),
        label: first_label,
    });

    for pair in then.windows(2) {
        edges.push(EdgeLine {
            from: pair[0].clone(),
            to: pair[1].clone(),
            label: String::from("then"),
        });
    }
}

fn rule_entry_label(rule: &RouteRule) -> String {
    let name = sanitize_mermaid_label_text(&rule.name).replace('_', " ");
    format!("rule {name}")
}

fn scene_key(scene: &SceneRef) -> String {
    match scene {
        SceneRef::Menu => String::from("menu"),
        SceneRef::Loading => String::from("loading"),
        SceneRef::Dialogue { id } => format!("dialogue:{id}"),
        SceneRef::Dilemma { id } => format!("dilemma:{id}"),
        SceneRef::Ending { id } => format!("ending:{id}"),
    }
}

fn scene_label(scene: &SceneRef) -> String {
    match scene {
        SceneRef::Menu => String::from("Menu"),
        SceneRef::Loading => String::from("Loading"),
        SceneRef::Dialogue { id } => format!("Dialogue: {id}"),
        SceneRef::Dilemma { id } => format!("Dilemma: {id}"),
        SceneRef::Ending { id } => format!("Ending: {id}"),
    }
}

fn scene_css_class(scene: &SceneRef) -> &'static str {
    match scene {
        SceneRef::Menu => "scene_menu",
        SceneRef::Loading => "scene_loading",
        SceneRef::Dialogue { .. } => "scene_dialogue",
        SceneRef::Dilemma { .. } => "scene_dilemma",
        SceneRef::Ending { .. } => "scene_ending",
    }
}

fn sanitize_mermaid_node_label(input: &str) -> String {
    sanitize_mermaid_label_text(input)
}

fn sanitize_mermaid_edge_label(input: &str) -> String {
    sanitize_mermaid_label_text(input)
}

fn sanitize_mermaid_label_text(input: &str) -> String {
    input
        .replace('&', " and ")
        .replace('<', " lt ")
        .replace('>', " gt ")
        .replace('|', "/")
        .replace(['"', '`'], "")
}

fn describe_validation_error(error: &GraphValidationError) -> String {
    match error {
        GraphValidationError::DuplicateRouteSource { from } => {
            format!("duplicate route source: {}", scene_key(from))
        }
        GraphValidationError::UnsupportedRouteSource { from } => {
            format!(
                "unsupported route source (must be dilemma): {}",
                scene_key(from)
            )
        }
        GraphValidationError::EmptyDefaultRoute { from } => {
            format!("empty default route for source: {}", scene_key(from))
        }
        GraphValidationError::EmptyRuleRoute { from, rule_name } => format!(
            "empty rule route for source {} rule `{rule_name}`",
            scene_key(from)
        ),
        GraphValidationError::DuplicateRuleName { from, rule_name } => format!(
            "duplicate rule name for source {} rule `{rule_name}`",
            scene_key(from)
        ),
        GraphValidationError::UnknownSceneId { context, error } => {
            format!("unknown scene id at {context}: {error}")
        }
    }
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_GRAPH_JSON: &str = include_str!("./content/campaign_graph.example.json");

    #[test]
    fn generates_mermaid_with_rule_and_default_labels() {
        let graph: SceneProgressionGraph =
            serde_json::from_str(EXAMPLE_GRAPH_JSON).expect("example graph should parse");
        let diagram = build_mermaid_diagram(&graph, RouteFilter::All);

        assert!(diagram.contains("rule "));
        assert!(diagram.contains("default"));
    }

    #[test]
    fn utilitarian_filter_only_keeps_utilitarian_routes() {
        let graph = SceneProgressionGraph {
            version: 1,
            routes: vec![
                RouteDefinition {
                    from: SceneRef::Dilemma {
                        id: String::from("path_utilitarian.1"),
                    },
                    rules: vec![],
                    default_then: vec![SceneRef::Dialogue {
                        id: String::from("path_utilitarian.1.pass"),
                    }],
                },
                RouteDefinition {
                    from: SceneRef::Dilemma {
                        id: String::from("path_inaction.1"),
                    },
                    rules: vec![],
                    default_then: vec![SceneRef::Dialogue {
                        id: String::from("path_inaction.1.fail"),
                    }],
                },
            ],
        };

        let utilitarian = build_mermaid_diagram(&graph, RouteFilter::Utilitarian);
        assert!(utilitarian.contains("path_utilitarian"));
        assert!(!utilitarian.contains("path_inaction"));
    }
}
