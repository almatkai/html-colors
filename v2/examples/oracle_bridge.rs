use html_colors_native_v2::{colors, ranked, render, search, sections, Session, View};
use serde_json::{json, Value};
use std::io::{self, BufRead};
fn main() {
    for line in io::stdin().lock().lines() {
        let value: Value = serde_json::from_str(&line.unwrap()).unwrap();
        let view = View::parse(value["view"].as_str().unwrap_or("all")).unwrap();
        let query = value["query"].as_str().unwrap_or("");
        let rows = search(view, query);
        let mut s = Session::default();
        s.view = view;
        s.query = query.into();
        s.details = value["details"] == true;
        s.show_hex = value["showHex"].as_bool().unwrap_or(true);
        println!(
            "{}",
            json!({"colors":if value["colors"]==true {json!(colors())} else {Value::Null},"rows":rows,"ranked":ranked(view,query),"sections":sections(&rows,query),"wire":if value["wire"]==true {render(&s).unwrap()} else {Value::Null}})
        );
    }
}
