//! JSON-RPC 2.0 as MCP uses it: `initialize`, `ping`, `tools/list` and
//! `tools/call`. Notifications get no answer.

use std::sync::Mutex;
use std::time::Instant;

use base64::Engine;
use serde_json::{Value, json};

use crate::bridge::{Bridge, Outcome};
use crate::status::Status;
use crate::tools;

/// Protocol versions this server speaks, newest first.
pub const PROTOCOL_VERSIONS: [&str; 4] = ["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"];

pub const PARSE_ERROR: i64 = -32700;
pub const INVALID_REQUEST: i64 = -32600;
pub const METHOD_NOT_FOUND: i64 = -32601;
pub const INVALID_PARAMS: i64 = -32602;

/// What a message body gets back: `None` for notifications only (HTTP 202).
pub fn handle_body(body: &[u8], bridge: &Bridge, status: &Mutex<Status>) -> Option<Value> {
    let message: Value = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(err) => {
            return Some(error(
                Value::Null,
                PARSE_ERROR,
                &format!("parse error: {err}"),
            ));
        }
    };
    match message {
        Value::Array(batch) => {
            let answers: Vec<Value> = batch
                .iter()
                .filter_map(|m| handle_message(m, bridge, status))
                .collect();
            (!answers.is_empty()).then_some(Value::Array(answers))
        }
        single => handle_message(&single, bridge, status),
    }
}

/// The answer to one message (`None` for a notification or a response).
pub fn handle_message(message: &Value, bridge: &Bridge, status: &Mutex<Status>) -> Option<Value> {
    let Some(method) = message.get("method").and_then(Value::as_str) else {
        // A response from the client, or junk.
        return message
            .get("id")
            .filter(|_| message.get("result").is_none() && message.get("error").is_none())
            .map(|id| error(id.clone(), INVALID_REQUEST, "not a request"));
    };
    let id = message.get("id")?.clone();
    let params = message.get("params").cloned().unwrap_or(Value::Null);
    Some(match method {
        "initialize" => result(id, initialize(&params)),
        "ping" => result(id, json!({})),
        "tools/list" => result(id, json!({"tools": tools::list()})),
        "tools/call" => match params.get("name").and_then(Value::as_str) {
            Some(name) => {
                let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                result(id, call_tool(name, &arguments, bridge, status))
            }
            None => error(id, INVALID_PARAMS, "tools/call needs a tool `name`"),
        },
        other => error(id, METHOD_NOT_FOUND, &format!("unknown method `{other}`")),
    })
}

fn initialize(params: &Value) -> Value {
    let asked = params.get("protocolVersion").and_then(Value::as_str);
    let version = asked
        .filter(|v| PROTOCOL_VERSIONS.contains(v))
        .unwrap_or(PROTOCOL_VERSIONS[0]);
    json!({
        "protocolVersion": version,
        "capabilities": {"tools": {"listChanged": false}},
        "serverInfo": {"name": "truckpaint", "title": "TruckPaint", "version": tp_core::VERSION},
        "instructions": tools::INSTRUCTIONS,
    })
}

/// Runs a tool and wraps its result as MCP content; failures are tool
/// results with `isError`, so the model reads them.
pub fn call_tool(name: &str, arguments: &Value, bridge: &Bridge, status: &Mutex<Status>) -> Value {
    let call = match tools::parse(name, arguments) {
        Ok(call) => call,
        Err(err) => {
            note(status, false);
            return tool_error(&err);
        }
    };
    let write = call.is_write();
    let reply = bridge.call(call);
    note(status, write && reply.is_ok());
    match reply {
        Ok(Outcome::Json(v)) => json!({
            "content": [{"type": "text", "text": serde_json::to_string_pretty(&v).unwrap_or_default()}],
            "structuredContent": if v.is_object() { v.clone() } else { json!({"result": v}) },
            "isError": false,
        }),
        Ok(Outcome::Image { png, text }) => json!({
            "content": [
                {"type": "image", "data": base64::engine::general_purpose::STANDARD.encode(png), "mimeType": "image/png"},
                {"type": "text", "text": text},
            ],
            "isError": false,
        }),
        Ok(Outcome::Deferred(_)) => tool_error("internal error: unfinished result"),
        Err(err) => tool_error(&err),
    }
}

fn note(status: &Mutex<Status>, applied_write: bool) {
    if let Ok(mut s) = status.lock() {
        s.note_call(applied_write, Instant::now());
    }
}

fn tool_error(text: &str) -> Value {
    json!({"content": [{"type": "text", "text": text}], "isError": true})
}

fn result(id: Value, result: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "result": result})
}

fn error(id: Value, code: i64, message: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::bridge;

    fn rig() -> (Bridge, bridge::Inbox, Mutex<Status>) {
        let (b, inbox) = bridge::bridge(|| {}, Duration::from_millis(50));
        (b, inbox, Mutex::new(Status::default()))
    }

    #[test]
    fn initialize_then_list() {
        let (b, _inbox, s) = rig();
        let init = handle_body(
            br#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"t","version":"1"}}}"#,
            &b,
            &s,
        )
        .unwrap();
        assert_eq!(init["id"], 1);
        assert_eq!(init["result"]["protocolVersion"], "2025-06-18");
        assert!(
            init["result"]["instructions"]
                .as_str()
                .unwrap()
                .contains("template")
        );
        assert!(
            handle_body(
                br#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
                &b,
                &s
            )
            .is_none()
        );
        let list = handle_body(
            br#"{"jsonrpc":"2.0","id":"a","method":"tools/list"}"#,
            &b,
            &s,
        )
        .unwrap();
        assert_eq!(list["id"], "a");
        assert!(list["result"]["tools"].as_array().unwrap().len() > 10);
    }

    #[test]
    fn unknown_version_gets_the_newest() {
        let (b, _inbox, s) = rig();
        let init = handle_body(br#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"1999-01-01"}}"#, &b, &s).unwrap();
        assert_eq!(init["result"]["protocolVersion"], PROTOCOL_VERSIONS[0]);
    }

    #[test]
    fn errors() {
        let (b, _inbox, s) = rig();
        let bad = handle_body(b"{not json", &b, &s).unwrap();
        assert_eq!(bad["error"]["code"], PARSE_ERROR);
        let unknown = handle_body(
            br#"{"jsonrpc":"2.0","id":2,"method":"resources/list"}"#,
            &b,
            &s,
        )
        .unwrap();
        assert_eq!(unknown["error"]["code"], METHOD_NOT_FOUND);
    }

    #[test]
    fn invalid_arguments_are_tool_errors() {
        let (b, _inbox, s) = rig();
        let r = handle_body(
            br#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"add_path","arguments":{"texture":"t","d":"M 0 0 L foo"}}}"#,
            &b,
            &s,
        )
        .unwrap();
        assert_eq!(r["result"]["isError"], true);
        assert!(
            r["result"]["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("`d`")
        );
    }

    #[test]
    fn busy_app() {
        let (b, _inbox, s) = rig();
        let r = handle_body(
            br#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"get_project"}}"#,
            &b,
            &s,
        )
        .unwrap();
        assert_eq!(r["result"]["isError"], true);
        assert_eq!(r["result"]["content"][0]["text"], bridge::BUSY);
    }
}
