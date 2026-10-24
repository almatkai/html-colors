use crate::{sections, ColorResult, Session, View};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};

#[derive(Debug, PartialEq)]
pub enum Effect {
    Render,
    Copy(String),
}
#[derive(Debug, Clone, Copy)]
pub enum CopyFormat {
    Hex,
    Rgb,
    Name,
}
impl CopyFormat {
    pub fn text(self, row: &ColorResult) -> &str {
        match self {
            Self::Hex => &row.color.hex,
            Self::Rgb => &row.color.rgb,
            Self::Name => &row.color.id,
        }
    }
}
fn node(kind: &str, props: Value, children: Vec<Value>) -> Value {
    json!({"type":kind,"props":props,"children":children})
}
fn capital(text: &str) -> String {
    let mut c = text.chars();
    c.next()
        .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
        .unwrap_or_default()
}
fn icon(category: &str) -> &'static str {
    if category == "basic" {
        "Icon.Circle"
    } else {
        "Icon.CircleEllipsis"
    }
}
pub fn swatch(row: &ColorResult) -> String {
    let svg=format!("<svg width=\"16\" height=\"16\" viewBox=\"0 0 16 16\" fill=\"none\" xmlns=\"http://www.w3.org/2000/svg\"><circle cx=\"8\" cy=\"8\" r=\"8\" fill=\"{}\"/></svg>",row.color.rgb);
    format!("data:image/svg+xml;base64,{}", STANDARD.encode(svg))
}
pub fn preview(row: &ColorResult) -> String {
    let svg=format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg width=\"200\" height=\"150\" viewBox=\"0 0 200 150\" xmlns=\"http://www.w3.org/2000/svg\">\n  <rect width=\"200\" height=\"150\" fill=\"{}\" rx=\"8\" ry=\"8\"/>\n  <rect width=\"200\" height=\"150\" fill=\"none\" stroke=\"#ccc\" stroke-width=\"1\" rx=\"8\" ry=\"8\"/>\n</svg>",row.color.hex);
    format!(
        "![{} Color Preview](data:image/svg+xml;base64,{})",
        row.color.name,
        STANDARD.encode(svg)
    )
}
fn detail(row: &ColorResult) -> Value {
    let sep = || node("List.Item.Detail.Metadata.Separator", json!({}), vec![]);
    let label = |title: &str, text: &str| {
        node(
            "List.Item.Detail.Metadata.Label",
            json!({"title":title,"text":text}),
            vec![],
        )
    };
    let cats = row
        .categories
        .iter()
        .map(|c| capital(c))
        .collect::<Vec<_>>()
        .join(", ");
    let mut cat = label("Categories", &cats);
    cat["props"]["icon"] = json!(if row.categories.iter().any(|c| c == "basic") {
        "Icon.Circle"
    } else {
        "Icon.CircleEllipsis"
    });
    let meta = node(
        "List.Item.Detail.Metadata",
        json!({}),
        vec![
            sep(),
            cat,
            sep(),
            label("Name", &row.color.name),
            label("Named color", &row.color.id),
            label("HEX", &row.color.hex),
            label("RGB", &row.color.rgb),
        ],
    );
    node(
        "List.Item.Detail",
        json!({"markdown":preview(row),"metadata":meta}),
        vec![],
    )
}
pub fn render(session: &Session) -> Result<Value, String> {
    session.ensure_active()?;
    let rows = session.rows();
    let mut actions = Vec::new();
    let mut nodes = Vec::new();
    for (i, row) in rows.iter().enumerate() {
        let mut ids = Vec::new();
        for (op, title) in [
            ("hex", "Copy HEX to Clipboard"),
            ("rgb", "Copy RGB to Clipboard"),
            ("name", "Copy Name to Clipboard"),
            (
                "format",
                if session.show_hex {
                    "Show RGB"
                } else {
                    "Show HEX"
                },
            ),
            (
                "detail",
                if session.details {
                    "Hide Details"
                } else {
                    "Show Details"
                },
            ),
        ] {
            let id = session.action_id(&format!("{i}:{op}"));
            ids.push(id.clone());
            let mut action = json!({"id":id,"title":title,"kind":"action"});
            // Preserved descriptors. The pinned beta overlay has no arbitrary shortcut handler.
            if op == "format" || op == "detail" {
                action["shortcut"] =
                    json!({"modifiers":["cmd"],"key":if op=="format" {"t"} else {"i"}})
            }
            actions.push(action);
        }
        let mut categories = row.categories.clone();
        categories.sort_by_key(|c| c != "basic");
        let accessories: Vec<_> = categories
            .iter()
            .map(|c| json!({"icon":icon(c),"tooltip":format!("{} Color",capital(c))}))
            .collect();
        let mut props = json!({"id":format!("{}-{}-{i}",row.color.category,row.color.id),"title":row.color.name,"subtitle":if session.show_hex {&row.color.hex} else {&row.color.rgb},"icon":{"source":swatch(row),"mask":"circle"},"accessories":accessories,"actionIds":ids});
        if session.details {
            props["detail"] = detail(row);
        }
        nodes.push(node("List.Item", props, vec![]));
    }
    let children = if session.view == View::Grouped {
        sections(&rows, &session.query)
            .into_iter()
            .map(|(shade, _)| {
                node(
                    "List.Section",
                    json!({"title":capital(&shade)}),
                    rows.iter()
                        .zip(&nodes)
                        .filter(|(r, _)| r.color.shade == shade)
                        .map(|(_, n)| n.clone())
                        .collect(),
                )
            })
            .collect()
    } else {
        nodes
    };
    let dropdown = node(
        "List.Dropdown",
        json!({"tooltip":"View Options","value":session.view.name(),"actionId":session.action_id("view")}),
        vec![
            node(
                "List.Dropdown.Item",
                json!({"title":"All Colors","value":"all","icon":"Icon.StackedBars3"}),
                vec![],
            ),
            node(
                "List.Dropdown.Item",
                json!({"title":"Basic Colors","value":"basic","icon":"Icon.Circle"}),
                vec![],
            ),
            node(
                "List.Dropdown.Item",
                json!({"title":"Extended Colors","value":"extended","icon":"Icon.CircleEllipsis"}),
                vec![],
            ),
            node(
                "List.Dropdown.Section",
                json!({"title":"Grouping"}),
                vec![node(
                    "List.Dropdown.Item",
                    json!({"title":"Group by Shade","value":"grouped","icon":"Icon.Layers"}),
                    vec![],
                )],
            ),
        ],
    );
    let root = node(
        "List",
        json!({"searchText":session.query,"searchBarPlaceholder":"Name, hex, or RGB...","__hasServerSearch":true,"isShowingDetail":session.details,"isLoading":false,"searchBarAccessory":dropdown}),
        children,
    );
    Ok(json!({"ok":true,"mode":"view","title":"Search HTML Colors","root":root,"actions":actions}))
}
