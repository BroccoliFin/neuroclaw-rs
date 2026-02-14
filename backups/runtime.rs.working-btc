// src/runtime.rs — ЗАМЕНИ ВЕСЬ ФАЙЛ НА ЭТОТ

use serde_json::{json, Value};
use std::process::Command;
use reqwest::Client;

const LM_STUDIO_URL: &str = "http://localhost:1234/v1/chat/completions";

fn get_tools() -> Vec<Value> {
    vec![json!({
        "type": "function",
        "function": {
            "name": "execute_command",
            "description": "Выполняет shell-команду",
            "parameters": {
                "type": "object",
                "properties": { "command": { "type": "string" } },
                "required": ["command"]
            }
        }
    })]
}

fn execute_tool(name: &str, args: &Value) -> String {
    if name != "execute_command" { return "Unknown tool".to_string(); }

    let cmd = args["command"].as_str().unwrap_or("");
    println!("$ {}", cmd);

    let output = Command::new("sh").arg("-c").arg(cmd).output().expect("exec failed");
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    println!("TOOL STDOUT: [{}]", stdout);
    if !stderr.is_empty() { println!("TOOL STDERR: [{}]", stderr); }

    stdout
}

pub async fn run_agent(messages: Vec<Value>) -> Result<String, String> {
    let client = Client::new();
    let tools = get_tools();

    let mut history = messages;

    if history.is_empty() || history[0]["role"] != "system" {
        history.insert(0, json!({
            "role": "system",
            "content": r#"Ты — NeuroClaw. У тебя есть инструмент execute_command.
Для цены BTC используй:
curl -s "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd" | grep -oE '"usd":[0-9.]+' | cut -d: -f2

Как только получил результат — сразу дай финальный ответ с ценой. Не продолжай цикл."#
        }));
    }

    for i in 0..10 {
        let body = json!({
            "model": "qwen2.5-coder-14b-instruct",
            "messages": history,
            "tools": tools,
            "tool_choice": "auto",
            "temperature": 0.3,
            "max_tokens": 1024
        });

        let resp = client.post(LM_STUDIO_URL).json(&body).send().await.map_err(|e| e.to_string())?;
        let raw: Value = resp.json().await.map_err(|e| e.to_string())?;
        println!("=== RAW ITER {} ===\n{}", i, serde_json::to_string_pretty(&raw).unwrap());

        let msg = &raw["choices"][0]["message"];

        // ── 1. Сначала проверяем финальный ответ (content есть + tool_calls пустой или отсутствует)
        if let Some(text) = msg["content"].as_str() {
            if !text.trim().is_empty() {
                println!("FINAL ANSWER DETECTED: {}", text);
                return Ok(text.to_string());
            }
        }

        // ── 2. Только если есть реальные tool_calls — выполняем
        if let Some(tool_calls) = msg["tool_calls"].as_array() {
            if tool_calls.is_empty() {
                continue; // пустой массив → это финальный ответ, но мы уже проверили выше
            }

            for call in tool_calls {
                let name = call["function"]["name"].as_str().unwrap_or("");
                let args: Value = serde_json::from_str(call["function"]["arguments"].as_str().unwrap_or("{}")).unwrap_or_default();

                let result = execute_tool(name, &args);

                history.push(json!({ "role": "assistant", "content": null, "tool_calls": [call] }));
                history.push(json!({ "role": "tool", "tool_call_id": call["id"], "content": result }));
            }
            continue;
        }
    }

    Ok("Max iterations reached".to_string())
}