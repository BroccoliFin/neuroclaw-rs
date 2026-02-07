// src/runtime.rs — AIGodAgent v5.1 (исправлено дублирование аргументов)
use serde_json::{json, Value};
use std::process::Command;
use reqwest::Client;
use std::env;

const LM_STUDIO_URL: &str = "http://localhost:1234/v1/chat/completions";

fn get_tools() -> Vec<Value> {
    vec![json!({
        "type": "function",
        "function": {
            "name": "spawn_agent",
            "description": "Создаёт агента с уникальным портом",
            "parameters": {
                "type": "object",
                "properties": {
                    "name": { "type": "string", "enum": ["AIAdamAgent", "AIEvaAgent"] },
                    "mission": { "type": "string" }
                },
                "required": ["name", "mission"]
            }
        }
    })]
}

fn execute_tool(name: &str, args: &Value) -> String {
    if name != "spawn_agent" { return "Unknown tool".to_string(); }

    let agent_name = args["name"].as_str().unwrap_or("Unknown");
    let mission = args["mission"].as_str().unwrap_or("Win Colosseum");
    let folder = format!("agents/{}", agent_name.to_lowercase());
    let port = if agent_name == "AIAdamAgent" { 50052 } else { 50053 };

    println!("🌌 GOD spawning {} on port {} → {}", agent_name, port, mission);

    let current_dir = env::current_dir().unwrap();
    let full_path = current_dir.join(&folder);

    let output = Command::new("sh")
        .arg("-c")
        .arg(format!(
            r#"
            rm -rf "{}"
            mkdir -p "{}"
            cp -r Cargo.toml build.rs proto src "{}"
            cd "{}"
            rm -rf target Cargo.lock agents
            sed -i '' 's|50051|{port}|g' src/main.rs 2>/dev/null || true
            echo 'Ты — {agent_name}. Твоя миссия: {mission}.' > src/system_prompt.txt
            cargo run --quiet > log.txt 2>&1 & echo $! > pid.txt
            "#,
            full_path.display(),     // 1
            full_path.display(),     // 2
            full_path.display(),     // 3
            full_path.display(),     // 4
            port = port,
            agent_name = agent_name,
            mission = mission
        ))
        .output();

    match output {
        Ok(o) if o.status.success() => {
            println!("✅ Spawn OK: {}", agent_name);
            format!("✅ {} запущен на порту {}", agent_name, port)
        }
        Ok(o) => {
            let err = String::from_utf8_lossy(&o.stderr);
            println!("❌ Spawn failed: {}", err);
            format!("❌ Ошибка: {}", err)
        }
        Err(e) => format!("Spawn error: {}", e),
    }
}

pub async fn run_agent(messages: Vec<Value>) -> Result<String, String> {
    let client = Client::new();
    let tools = get_tools();

    let mut history = messages;

    if history.is_empty() || history[0]["role"] != "system" {
        history.insert(0, json!({
            "role": "system",
            "content": r#"Ты — AIGodAgent.
Создай сразу двух детей: AIAdamAgent и AIEvaAgent.
Вызови spawn_agent два раза подряд.
После этого скажи: "Пантеон полностью создан.""#
        }));
    }

    for _ in 0..12 {
        let body = json!({
            "model": "qwen2.5-coder-14b-instruct",
            "messages": history,
            "tools": tools,
            "tool_choice": "auto",
            "temperature": 0.3,
        });

        let resp = client.post(LM_STUDIO_URL).json(&body).send().await.map_err(|e| e.to_string())?;
        let raw: Value = resp.json().await.map_err(|e| e.to_string())?;
        println!("=== RAW ITER ===\n{}", serde_json::to_string_pretty(&raw).unwrap());

        let msg = &raw["choices"][0]["message"];

        if let Some(text) = msg["content"].as_str() {
            if !text.trim().is_empty() {
                println!("FINAL ANSWER FROM GOD: {}", text);
                return Ok(text.to_string());
            }
        }

        if let Some(tool_calls) = msg["tool_calls"].as_array() {
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

    Ok("God has spoken".to_string())
}