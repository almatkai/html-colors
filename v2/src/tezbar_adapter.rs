//! Production integration: compile this file as native_core::extensions::v2_html_colors.
//! Requires the html-colors-native-v2 path dependency and beta's existing SDK.
use crate::native_core::{
    common::{CoreContext, CoreResult},
    extensions::v2_sdk as sdk,
};
use html_colors_native_v2::{render, Effect, Session};
use serde_json::{json, Value};

pub fn commands() -> &'static [&'static str] {
    &["html-color"]
}

fn copy_succeeded(value: &Value) -> bool {
    value.is_object()
        && value.get("ok") == Some(&Value::Bool(true))
        && value.get("error").is_none_or(Value::is_null)
        && value.get("success").is_none_or(|v| v == &Value::Bool(true))
}
fn session(state: &Value) -> Result<Session, String> {
    serde_json::from_value(state.clone())
        .map_err(|_| "HTML Colors session is not initialized; reopen the command".into())
}
fn validate_action_payload(payload: &Value) -> Result<(), String> {
    let obj = payload.as_object().ok_or("Invalid action payload")?;
    for key in obj.keys() {
        if ![
            "actionId",
            "formValues",
            "sessionId",
            "extensionId",
            "commandName",
            "preferences",
        ]
        .contains(&key.as_str())
        {
            return Err(format!("Unexpected action payload field: {key}"));
        }
    }
    Ok(())
}
pub async fn handle(
    ctx: &CoreContext,
    command: &str,
    operation: &str,
    payload: &Value,
    state: &mut Value,
) -> CoreResult {
    if !commands().contains(&command) {
        return Err(format!("Unknown HTML Colors command: {command}"));
    }
    if operation == "extension:dispose-session" {
        if let Ok(mut current) = session(state) {
            current.dispose();
            *state = serde_json::to_value(current).map_err(|e| e.to_string())?;
        }
        return Ok(sdk::no_view("HTML Colors session disposed"));
    }
    if operation == "extension:run-command" {
        if session(state).is_ok_and(|s| s.active) {
            return Err("HTML Colors session is already running".into());
        }
        let next = Session::default();
        let view = render(&next)?;
        *state = serde_json::to_value(next).map_err(|e| e.to_string())?;
        return Ok(sdk::view(
            "Search HTML Colors",
            view["root"].clone(),
            view["actions"].as_array().cloned().unwrap_or_default(),
        ));
    }
    let mut next = session(state)?;
    next.ensure_active()?;
    match operation {
        // Never generate new capabilities during refresh. Search runs against staged
        // state in beta native.rs; only its returned view receives the committed IDs.
        "extension:refresh-session" => Ok(sdk::unchanged()),
        "extension:search-text-changed" => {
            let query = payload["searchText"].as_str().ok_or("Missing searchText")?;
            next.set_query(query.to_string())?;
            let view = render(&next)?;
            *state = serde_json::to_value(next).map_err(|e| e.to_string())?;
            Ok(view)
        }
        "extension:invoke-action" => {
            validate_action_payload(payload)?;
            let id = payload["actionId"].as_str().ok_or("Missing actionId")?;
            let effect = next.invoke(id, &payload["formValues"])?;
            match effect {
                Effect::Render => {
                    let view = render(&next)?;
                    *state = serde_json::to_value(next).map_err(|e| e.to_string())?;
                    Ok(view)
                }
                Effect::Copy(text) => {
                    let result = sdk::copy(ctx, &text).await;
                    let mut view = render(&next)?;
                    match result {
                        Ok(value) if copy_succeeded(&value) => {
                            view["message"] = json!(format!("Color copied to clipboard: {text}"));
                        }
                        Ok(_) => {
                            view["ok"] = json!(false);
                            view["error"] = json!(
                                "Failed to copy color: clipboard did not acknowledge success"
                            );
                        }
                        Err(error) => {
                            view["ok"] = json!(false);
                            view["error"] = json!(format!("Failed to copy color: {error}"));
                        }
                    }
                    if view["ok"] == false {
                        view["message"] = view["error"].clone();
                    }
                    // Copy errors leave the exact current result/actions retryable.
                    Ok(view)
                }
            }
        }
        _ => Err(format!("Unsupported HTML Colors operation: {operation}")),
    }
}
