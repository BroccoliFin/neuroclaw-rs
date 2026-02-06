use reqwest;
use serde_json::{json, Value};
use std::process::Command;

const LM_STUDIO_URL: &str = "http://localhost:1234/v1/messages";

pub fn get_tools() -> Vec<Value> {
    vec![json!({
        "name": "execute_command",
        "description": "Выполняет команду в терминале на этом компьютере. Будь осторожен.",
        "input_schema": {
            "type": "object",
            "properties": {
                "command": { "type": "string", "description": "Команда для выполнения" }
            },
            "required": ["command"]
        }
    })]
}

fn execute_tool(name: &str, input: &Value) -> String {
    if name == "execute_command" {
        if let Some(cmd) = input["command"].as_str() {
            let output = Command::new("sh").arg("-c").arg(cmd).output()
                .unwrap_or_else(|e| panic!("Command failed: {}", e));

            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

            if !stderr.is_empty() {
                format!("STDOUT:\n{}\nSTDERR:\n{}", stdout, stderr)
            } else {
                stdout
            }
        } else {
            "Error: no command".to_string()
        }
    } else {
        format!("Unknown tool: {}", name)
    }
}

pub async fn run_agent(messages: Vec<Value>) -> Result<String, String> {
    let client = reqwest::Client::new();
    let tools = get_tools();

    let model_name = "qwen2.5-coder-14b-instruct";   // ← твоя модель, которая работала

    let body = json!({
        "model": model_name,
        "max_tokens": 2048,
        "temperature": 0.7,
        "messages": messages,
        "tools": tools,
        "tool_choice": { "type": "auto" },
        "stream": false
    });

    let resp = client.post(LM_STUDIO_URL)
        .header("anthropic-version", "2023-06-01")
        .header("x-api-key", "lmstudio")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let raw: Value = resp.json().await.map_err(|e| format!("JSON error: {}", e))?;

    println!("=== RAW FROM LM STUDIO ===\n{:#}", raw);

    if let Some(content) = raw["content"].as_array() {
        for block in content {
            if block["type"] == "tool_use" {
                let name = block["name"].as_str().unwrap_or("");
                let input = &block["input"];
                let result = execute_tool(name, input);
                return Ok(format!("✅ Tool executed: {}\nResult:\n{}", name, result));
            }
        }
        if let Some(text) = content.iter().find(|b| b["type"] == "text")
            .and_then(|b| b["text"].as_str())
        {
            return Ok(text.to_string());
        }
    }
    Ok(format!("Unexpected: {:#}", raw))
}