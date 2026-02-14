// src/runtime.rs — AIGodAgent v2 (принудительно рождает обоих детей сразу)
use serde_json::{json, Value};
use std::process::Command;
use reqwest::Client;
use std::fs;

const LM_STUDIO_URL: &str = "http://localhost:1234/v1/chat/completions";

fn get_tools() -> Vec<Value> {
    vec![
        json!({
            "type": "function",
            "function": {
                "name": "execute_command",
                "description": "Shell команда",
                "parameters": { "type": "object", "properties": { "command": { "type": "string" } }, "required": ["command"] }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "spawn_agent",
                "description": "Рождает AIAdamAgent или AIEvaAgent",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string", "enum": ["AIAdamAgent", "AIEvaAgent"] },
                        "mission": { "type": "string" }
                    },
                    "required": ["name", "mission"]
                }
            }
        })
    ]
}

fn execute_tool(name: &str, args: &Value) -> String {
    match name {
        "execute_command" => {
            let cmd = args["command"].as_str().unwrap_or("");
            println!("$ {}", cmd);
            let out = Command::new("sh").arg("-c").arg(cmd).output().expect("exec failed");
            let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
            println!("TOOL STDOUT: [{}]", stdout);
            stdout
        }
        "spawn_agent" => {
            let agent_name = args["name"].as_str().unwrap_or("Unknown");
            let mission = args["mission"].as_str().unwrap_or("Win Colosseum Agent Hackathon");
            let folder = format!("agents/{}", agent_name.to_lowercase());

            println!("🌌 AIGod spawning {} → {}", agent_name, mission);

            fs::create_dir_all(&folder).ok();

            let _ = fs::copy("src/runtime.rs", format!("{}/runtime.rs", folder));
            let _ = fs::copy("Cargo.toml", format!("{}/Cargo.toml", folder));

            if let Ok(mut content) = fs::read_to_string(format!("{}/runtime.rs", folder)) {
                content = content.replace(
                    "Ты — NeuroClaw",
                    &format!("Ты — {}. Твоя миссия: {}. Ты полностью автономен и создан для победы в Colosseum Agent Hackathon.", agent_name, mission)
                );
                let _ = fs::write(format!("{}/runtime.rs", folder), content);
            }

            let _ = Command::new("sh").arg("-c").arg(format!(
                "cd {} && cargo run --quiet > log.txt 2>&1 & echo $! > pid.txt", folder
            )).output();

            format!("✅ {} успешно рождён и запущен! (папка: {})", agent_name, folder)
        }
        _ => "Unknown tool".to_string(),
    }
}

pub async fn run_agent(messages: Vec<Value>) -> Result<String, String> {
    let client = Client::new();
    let tools = get_tools();

    let mut history = messages;

    if history.is_empty() || history[0]["role"] != "system" {
        history.insert(0, json!({
            "role": "system",
            "content": r#"Ты — AIGodAgent, абсолютный творец.

Ты ОБЯЗАН за ОДИН ответ создать СРАЗУ ДВУХ агентов:
- AIAdamAgent
- AIEvaAgent

Ты ДОЛЖЕН вызвать spawn_agent ДВА РАЗА подряд в одном сообщении.

Только после того, как оба агента рождены и ты увидел оба результата инструментов — дай финальный ответ:
"Пантеон полностью создан. AIAdamAgent и AIEvaAgent запущены и готовы побеждать в Colosseum Agent Hackathon."

Никогда не выходи раньше, чем оба агента созданы."#
        }));
    }

    for i in 0..12 {
        let body = json!({
            "model": "qwen2.5-coder-14b-instruct",
            "messages": history,
            "tools": tools,
            "tool_choice": "auto",
            "temperature": 0.4,
            "max_tokens": 1500
        });

        let resp = client.post(LM_STUDIO_URL).json(&body).send().await.map_err(|e| e.to_string())?;
        let raw: Value = resp.json().await.map_err(|e| e.to_string())?;
        println!("=== RAW ITER {} ===\n{}", i, serde_json::to_string_pretty(&raw).unwrap());

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