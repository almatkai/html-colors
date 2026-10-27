//! Exact production adapter, SDK and common; only Tauri's unused handle is stubbed.
#![allow(dead_code)]
extern crate self as tauri;
#[derive(Clone)]
pub struct AppHandle;
pub trait Emitter {
    fn emit(&self, channel: &str, value: serde_json::Value) -> Result<(), String>;
}
impl Emitter for AppHandle {
    fn emit(&self, _: &str, _: serde_json::Value) -> Result<(), String> {
        panic!("OS event route must not run in tests")
    }
}
#[rustfmt::skip]
#[allow(clippy::all)]
#[path="fixtures/beta/src-tauri/src/native_core/common.rs"]
pub mod beta_common;
#[rustfmt::skip]
#[allow(clippy::all)]
#[path="fixtures/beta/src-tauri/src/native_core/extensions/v2_sdk.rs"]
pub mod beta_sdk;
pub mod native_core {
    pub use crate::beta_common as common;
    pub mod extensions {
        pub use crate::beta_sdk as v2_sdk;
    }
}
#[path = "../src/tezbar_adapter.rs"]
mod adapter;
use beta_common::{CoreContext, CorePaths, CoreResult};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
type CapturedCalls = Arc<Mutex<Vec<(String, Vec<Value>)>>>;
fn context(reply: CoreResult) -> (tempfile::TempDir, CoreContext, CapturedCalls) {
    let dir = tempfile::tempdir().unwrap();
    let ctx = CoreContext::headless(CorePaths {
        data_dir: dir.path().join("data"),
        config_dir: dir.path().join("config"),
        resource_dir: dir.path().join("resources"),
        temp_dir: dir.path().join("tmp"),
        home_dir: dir.path().to_path_buf(),
    });
    let calls = Arc::new(Mutex::new(Vec::new()));
    let captured = calls.clone();
    ctx.set_router(Arc::new(move |channel, args| {
        captured.lock().unwrap().push((channel, args));
        let reply = reply.clone();
        Box::pin(async move { reply })
    }))
    .unwrap();
    (dir, ctx, calls)
}
async fn call(ctx: &CoreContext, op: &str, p: Value, s: &mut Value) -> CoreResult {
    adapter::handle(ctx, "html-color", op, &p, s).await
}
async fn run(ctx: &CoreContext, s: &mut Value) -> Value {
    call(ctx, "extension:run-command", json!({}), s)
        .await
        .unwrap()
}
fn action(view: &Value, title: &str) -> String {
    view["actions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["title"] == title)
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .into()
}
#[tokio::test]
async fn actual_sdk_copy_routes_all_formats_and_envelopes_retry_without_os_effects() {
    let replies = vec![
        Ok(json!({"ok":true})),
        Ok(json!({"ok":true,"error":null})),
        Ok(json!({"ok":true,"success":true})),
        Ok(json!({"ok":true,"success":false})),
        Ok(json!({"ok":true,"error":"denied"})),
        Ok(json!({"ok":true,"error":false})),
        Ok(json!({"success":true})),
        Ok(json!(true)),
        Ok(Value::Null),
        Ok(json!({})),
        Ok(json!({"ok":"true"})),
        Ok(json!({"ok":false})),
        Err("transport failure".into()),
    ];
    for (i, reply) in replies.into_iter().enumerate() {
        let (dir, ctx, calls) = context(reply);
        let mut state = json!({});
        let view = run(&ctx, &mut state).await;
        let before = state.clone();
        for (title, text) in [
            ("Copy HEX to Clipboard", "#000000"),
            ("Copy RGB to Clipboard", "rgb(0, 0, 0)"),
            ("Copy Name to Clipboard", "black"),
        ] {
            let id = action(&view, title);
            for _ in 0..2 {
                let response = call(
                    &ctx,
                    "extension:invoke-action",
                    json!({"actionId":id}),
                    &mut state,
                )
                .await
                .unwrap();
                assert_eq!(response["ok"], i < 3, "reply {i}");
                assert!(response["message"]
                    .as_str()
                    .is_some_and(|m| m.contains(if i < 3 {
                        "Color copied"
                    } else {
                        "Failed to copy"
                    })));
                assert_eq!(response["root"], view["root"]);
                assert_eq!(state, before);
                assert_eq!(
                    calls.lock().unwrap().last().unwrap(),
                    &("clipboard:write".into(), vec![json!(text)])
                );
            }
        }
        assert!(!dir.path().join("config").exists());
        assert!(!dir.path().join("data").exists());
    }
}
#[tokio::test]
async fn stale_fabricated_cross_session_and_disposed_actions_fail() {
    let (_dir, ctx, calls) = context(Ok(json!({"ok":true})));
    let mut state = json!({});
    let first = run(&ctx, &mut state).await;
    let old = action(&first, "Copy HEX to Clipboard");
    let refresh = call(&ctx, "extension:refresh-session", json!({}), &mut state)
        .await
        .unwrap();
    assert_eq!(refresh["mode"], "unchanged");
    let before = state.clone();
    let mut staged = state.clone();
    let dropped = call(
        &ctx,
        "extension:search-text-changed",
        json!({"searchText":"blue"}),
        &mut staged,
    )
    .await
    .unwrap();
    // Models native.rs dropping a staged/superseded search before commit.
    assert!(call(
        &ctx,
        "extension:invoke-action",
        json!({"actionId":action(&dropped,"Copy HEX to Clipboard")}),
        &mut state
    )
    .await
    .is_err());
    assert_eq!(state, before);
    let searched = call(
        &ctx,
        "extension:search-text-changed",
        json!({"searchText":"Red"}),
        &mut state,
    )
    .await
    .unwrap();
    assert_eq!(searched["root"]["props"]["searchText"], "Red");
    assert!(call(
        &ctx,
        "extension:invoke-action",
        json!({"actionId":old}),
        &mut state
    )
    .await
    .is_err());
    let dropid = searched["root"]["props"]["searchBarAccessory"]["props"]["actionId"].clone();
    let grouped = call(
        &ctx,
        "extension:invoke-action",
        json!({"actionId":dropid,"formValues":{"value":"grouped"}}),
        &mut state,
    )
    .await
    .unwrap();
    assert_eq!(grouped["root"]["props"]["searchText"], "Red");
    assert!(call(
        &ctx,
        "extension:invoke-action",
        json!({"actionId":dropid,"formValues":{"value":"basic"}}),
        &mut state
    )
    .await
    .is_err());
    let mut other = json!({});
    run(&ctx, &mut other).await;
    assert!(call(
        &ctx,
        "extension:invoke-action",
        json!({"actionId":action(&grouped,"Copy HEX to Clipboard")}),
        &mut other
    )
    .await
    .is_err());
    let current = action(&grouped, "Copy HEX to Clipboard");
    for payload in [
        json!({"actionId":"forged"}),
        json!({"actionId":format!("{current}:extra")}),
        json!({"actionId":current,"text":"evil"}),
        json!({"actionId":current,"color":"#000"}),
        json!({"actionId":current,"url":"https://example.com"}),
        json!({"actionId":current,"formValues":{"text":"evil"}}),
    ] {
        assert!(call(&ctx, "extension:invoke-action", payload, &mut state)
            .await
            .is_err());
    }
    assert!(calls.lock().unwrap().is_empty());
    call(&ctx, "extension:dispose-session", json!({}), &mut state)
        .await
        .unwrap();
    assert!(call(
        &ctx,
        "extension:invoke-action",
        json!({"actionId":current}),
        &mut state
    )
    .await
    .is_err());
    let relaunched = run(&ctx, &mut state).await;
    assert_ne!(action(&relaunched, "Copy HEX to Clipboard"), old);
    assert!(call(
        &ctx,
        "extension:invoke-action",
        json!({"actionId":old}),
        &mut state
    )
    .await
    .is_err());
    assert!(adapter::handle(
        &ctx,
        "other",
        "extension:run-command",
        &json!({}),
        &mut state
    )
    .await
    .is_err());
    for op in ["extension:load-more", "clipboard:write", "bad"] {
        assert!(call(&ctx, op, json!({}), &mut state).await.is_err());
    }
}
#[tokio::test]
async fn toggles_preserve_query_and_invalidate_prior_ids() {
    let (_d, ctx, _) = context(Ok(json!({"ok":true})));
    let mut state = json!({});
    run(&ctx, &mut state).await;
    let v = call(
        &ctx,
        "extension:search-text-changed",
        json!({"searchText":"blu"}),
        &mut state,
    )
    .await
    .unwrap();
    let format = action(&v, "Show RGB");
    let old = action(&v, "Copy Name to Clipboard");
    let v = call(
        &ctx,
        "extension:invoke-action",
        json!({"actionId":format}),
        &mut state,
    )
    .await
    .unwrap();
    assert_eq!(v["root"]["props"]["searchText"], "blu");
    assert!(v["root"]["children"][0]["props"]["subtitle"]
        .as_str()
        .unwrap()
        .starts_with("rgb("));
    assert!(call(
        &ctx,
        "extension:invoke-action",
        json!({"actionId":old}),
        &mut state
    )
    .await
    .is_err());
    let v = call(
        &ctx,
        "extension:invoke-action",
        json!({"actionId":action(&v,"Show Details")}),
        &mut state,
    )
    .await
    .unwrap();
    assert_eq!(v["root"]["props"]["isShowingDetail"], true);
    assert!(v["root"]["children"][0]["props"]["detail"].is_object());
}
