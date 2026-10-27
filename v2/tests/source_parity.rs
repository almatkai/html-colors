use html_colors_native_v2::{colors, ranked, search, sections, Color, ColorResult, View};
use serde::Deserialize;
#[derive(Deserialize)]
struct Corpus {
    colors: Vec<Color>,
    cases: Vec<Case>,
}
#[derive(Deserialize)]
struct Case {
    view: String,
    query: String,
    rows: Vec<ColorResult>,
    sections: Vec<(String, Vec<ColorResult>)>,
    ranked: Vec<(usize, f64)>,
}
#[test]
fn offline_original_source_oracle() {
    let corpus: Corpus = serde_json::from_str(include_str!("fixtures/oracle.json")).unwrap();
    assert_eq!(colors(), corpus.colors);
    for case in corpus.cases {
        let view = View::parse(&case.view).unwrap();
        let rows = search(view, &case.query);
        assert_eq!(rows, case.rows, "{} {:?}", case.view, case.query);
        assert_eq!(
            sections(&rows, &case.query),
            case.sections,
            "{} {:?}",
            case.view,
            case.query
        );
        let actual = ranked(view, &case.query);
        assert_eq!(actual.len(), case.ranked.len());
        for ((index, score), (expected_index, expected_score)) in actual.iter().zip(case.ranked) {
            assert_eq!(*index, expected_index, "{} {:?}", case.view, case.query);
            assert!((score - expected_score).abs() < 2e-14);
        }
    }
}
#[test]
fn beta_json_boundary_rejects_unpaired_surrogates() {
    assert!(serde_json::from_str::<serde_json::Value>(r#"{"searchText":"\ud800"}"#).is_err());
    assert!(serde_json::from_str::<serde_json::Value>(r#"{"searchText":"\udc00"}"#).is_err());
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(r#"{"searchText":"\ud83d\ude00"}"#).unwrap()
            ["searchText"],
        "😀"
    );
}

#[test]
fn every_row_copy_and_current_wire_are_canonical() {
    use html_colors_native_v2::{render, Effect, Session};
    use serde_json::{json, Value};
    fn rows<'a>(node: &'a Value, out: &mut Vec<&'a Value>) {
        if node["type"] == "List.Item" {
            out.push(node);
        } else if let Some(children) = node["children"].as_array() {
            for child in children {
                rows(child, out);
            }
        }
    }
    for view in [View::All, View::Basic, View::Extended, View::Grouped] {
        for query in ["", " ", "Red", "CornflwerBlue", "rgb(255, 0, 0)"] {
            let mut s = Session::default();
            s.view = view;
            s.set_query(query.into()).unwrap();
            let canonical = s.rows();
            let wire = render(&s).unwrap();
            let mut wire_rows = Vec::new();
            rows(&wire["root"], &mut wire_rows);
            assert_eq!(wire_rows.len(), canonical.len());
            for row in wire_rows {
                for (slot, field) in ["hex", "rgb", "name"].iter().enumerate() {
                    let id = row["props"]["actionIds"][slot].as_str().unwrap();
                    let index: usize = id.rsplit(':').nth(1).unwrap().parse().unwrap();
                    let c = &canonical[index].color;
                    assert_eq!(row["props"]["title"], c.name);
                    let text = match *field {
                        "hex" => &c.hex,
                        "rgb" => &c.rgb,
                        _ => &c.id,
                    };
                    assert_eq!(
                        s.invoke(id, &Value::Null).unwrap(),
                        Effect::Copy(text.clone())
                    );
                }
            }
            let view_action = wire["root"]["props"]["searchBarAccessory"]["props"]["actionId"]
                .as_str()
                .unwrap();
            for values in [
                json!({"value":"invalid"}),
                json!({"value":"all","text":"injected"}),
                json!({"value":42}),
            ] {
                assert!(s.invoke(view_action, &values).is_err());
            }
        }
    }
}
