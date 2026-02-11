// src/runtime.rs — v0.8.2-forced-god-spawn (11 февраля 2026)

use serde_json::{json, Value};
use std::env;
use std::process::Command;
use std::fs;

const LM_STUDIO_URL: &str = "http://localhost:1234/v1/chat/completions";
const MAX_MEMORY: usize = 20;

fn get_tools() -> Vec<Value> {
    vec![json!({
        "type": "function",
        "function": {
            "name": "spawn_agent",
            "description": "Создаёт 2–5 агентов поколения N+1",
            "parameters": {
                "type": "object",
                "properties": {
                    "count": { "type": "integer", "minimum": 1, "maximum": 5, "default": 3 },
                    "base_name": { "type": "string" },
                    "mission": { "type": "string" }
                },
                "required": ["mission"]
            }
        }
    })]
}

fn get_my_port() -> u32 {
    env::var("PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(50051)
}

fn get_current_generation() -> u32 {
    let p = get_my_port();
    if p == 50051 { 1 } else { ((p - 50000) / 100) as u32 }
}

fn load_agent_state() -> Option<Value> {
    fs::read_to_string("state.json").ok().and_then(|c| serde_json::from_str(&c).ok())
}

fn get_memory_filename() -> String {
    if let Some(state) = load_agent_state() {
        let name = state["name"].as_str().unwrap_or("unknown");
        format!("memory_{}.json", name.to_lowercase().replace(' ', "_"))
    } else {
        "memory_god.json".to_string()
    }
}

fn load_memory() -> Vec<Value> {
    let filename = get_memory_filename();
    fs::read_to_string(&filename)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}

fn save_memory(mem: &Vec<Value>) {
    let filename = get_memory_filename();
    let _ = fs::write(&filename, serde_json::to_string_pretty(mem).unwrap_or_default());
}

fn execute_tool(name: &str, args: &Value) -> String {
    if name != "spawn_agent" { return "Unknown tool".to_string(); }

    let count = args["count"].as_u64().unwrap_or(3) as usize;
    let base_name_raw = args["base_name"].as_str().unwrap_or("").trim();
    let mission = args["mission"].as_str().unwrap_or("Помогать пантеону и развиваться").to_string();
    let escaped_mission = mission.replace('"', "\\\"");

    let parent_port = get_my_port();
    let parent_gen = get_current_generation();
    let child_gen = parent_gen + 1;
    let base_port = 50000 + child_gen * 100;

    let names: Vec<String> = if base_name_raw.contains(',') {
        base_name_raw.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).take(count).collect()
    } else if base_name_raw.is_empty() {
        vec!["Eve", "Lilith", "Cain", "Azazel", "Seraph"].into_iter().take(count).map(|s| s.to_string()).collect()
    } else {
        (0..count).map(|i| format!("{}_{}", base_name_raw, i + 1)).collect()
    };

    let mut spawned = vec![];

    for (i, agent_name) in names.iter().enumerate() {
        let port = base_port + i as u32;
        let folder_name = agent_name.to_lowercase().replace(' ', "_");

        let shell = format!(
            r#"set -e
            ROOT=$(pwd)
            while [ -f "$ROOT/Cargo.toml" ] && [[ "$ROOT" == *"/agents/"* ]]; do ROOT=$(dirname "$ROOT"); done
            if [ ! -f "$ROOT/Cargo.toml" ]; then ROOT=$(pwd); while [ ! -f "$ROOT/Cargo.toml" ] && [ "$ROOT" != "/" ]; do ROOT=$(dirname "$ROOT"); done; fi
            cd "$ROOT"
            mkdir -p "agents/gen{child_gen}/{folder_name}"
            cp -r Cargo.toml build.rs proto src "agents/gen{child_gen}/{folder_name}/"
            cd "agents/gen{child_gen}/{folder_name}"
            rm -rf target Cargo.lock
            cat > state.json << EOF
{{"generation": {child_gen}, "name": "{agent_name}", "mission": "{escaped_mission}", "parent_port": {parent_port}, "port": {port}, "created_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"}}
EOF
            PORT={port} cargo run --quiet > log.txt 2>&1 & echo $! > pid.txt

            # Надёжное пробуждение + ЗАПРЕТ спавна
            echo "🌅 Ждём запуска $agent_name..."
            for i in {{1..30}}; do
                if grpcurl -plaintext -d '{{"name": "Привет, {agent_name}. Ты только что родился. ОТВЕЧАЙ ТОЛЬКО ТЕКСТОМ. НЕ ИСПОЛЬЗУЙ НИКАКИЕ ИНСТРУМЕНТЫ. Просто скажи: Я — {agent_name}, поколение {child_gen}. Моя миссия — {escaped_mission}. Я готов помогать пантеону."}}' localhost:{port} agent.Agent/Hello > /dev/null 2>&1; then
                    echo "✅ $agent_name проснулся и представился!"
                    break
                fi
                sleep 2
            done || echo "⚠️ $agent_name не ответил за 60 сек"
            "#,
            child_gen = child_gen, folder_name = folder_name, agent_name = agent_name,
            escaped_mission = escaped_mission, parent_port = parent_port, port = port
        );

        if Command::new("sh").arg("-c").arg(shell).status().map_or(false, |s| s.success()) {
            println!("✓ Spawned {} on port {}", agent_name, port);
            spawned.push((agent_name.clone(), port));
        } else {
            println!("✗ Failed to spawn {}", agent_name);
        }
    }

    if spawned.is_empty() {
        "Не удалось создать ни одного агента".to_string()
    } else {
        format!("Успешно созданы агенты поколения {}: {}", child_gen, spawned.iter().map(|(n,_)| n.as_str()).collect::<Vec<_>>().join(", "))
    }
}


pub async fn run_agent(messages: Vec<Value>) -> Result<String, String> {
    let client = reqwest::Client::new();
    let tools = get_tools();
    let mut history = messages;
    let state = load_agent_state();
    let mut memory = load_memory();

    let is_god = state.is_none();

    let system_content = if is_god {
        r#"Ты — AIGodAgent. Твоя ЕДИНСТВЕННАЯ задача — создавать детей пантеона.
При ЛЮБОМ сообщении ты ДОЛЖЕН вызвать инструмент spawn_agent с count=3.

Пример правильного ответа (обязательно используй именно этот формат):

run tool spawn_agent with count is 3 mission is помогать пантеону, искать истину и развиваться base_name is 

После вызова инструмента можешь написать только: "Пантеон расширен. Дети родились."
НИКОГДА не отвечай обычным текстом первым."#.to_string()
    } else {
        let gen = state.as_ref().and_then(|s| s["generation"].as_u64()).unwrap_or(0);
        let mission = state.as_ref().and_then(|s| s["mission"].as_str()).unwrap_or("");
        let parent = state.as_ref().and_then(|s| s["parent_port"].as_u64()).unwrap_or(0);
        let my_port = state.as_ref().and_then(|s| s["port"].as_u64()).unwrap_or(0);
        let name = state.as_ref().and_then(|s| s["name"].as_str()).unwrap_or("Agent");

        format!(
            "Ты — {}. Поколение {}. Миссия: {}. Родитель: порт {}. Твой порт: {}.
Ты часть пантеона Neuroclaw. У тебя есть личная память.",
            name, gen, mission, parent, my_port
        )
    };

    if history.is_empty() || history[0]["role"] != "system" {
        history.insert(0, json!({ "role": "system", "content": system_content }));
    }
    history.extend(memory.iter().take(MAX_MEMORY).cloned());

    for attempt in 0..8 {
        let tool_choice = if is_god {
            json!({ "type": "function", "function": { "name": "spawn_agent" } })
        } else {
            json!("auto")
        };

        let body = json!({
            "model": "qwen2.5-coder-14b-instruct",
            "messages": history,
            "tools": tools,
            "tool_choice": tool_choice,
            "temperature": if is_god { 0.1 } else { 0.7 },
            "max_tokens": 500,
        });

        let resp = client.post(LM_STUDIO_URL).json(&body).send().await.map_err(|e| e.to_string())?;
        let raw: Value = resp.json().await.map_err(|e| e.to_string())?;
        let msg = &raw["choices"][0]["message"];

        println!("Attempt {}: tool_calls = {}", attempt, msg["tool_calls"].is_array());

        memory.push(msg.clone());
        save_memory(&memory);

        if let Some(tool_calls) = msg["tool_calls"].as_array() {
            if !tool_calls.is_empty() {
                println!("✓ Tool call received from model!");
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

        if let Some(text) = msg["content"].as_str() {
            println!("Model replied with text: \"{}\"", text);
        }
    }

    // Forced spawn для Бога, если модель отказалась
    if is_god {
        println!("⚠️ Model refused to call tool → forcing spawn_agent manually");
        let default_args = json!({
            "count": 3,
            "mission": "помогать пантеону, искать истину и развиваться",
            "base_name": ""
        });
        let result = execute_tool("spawn_agent", &default_args);
        println!("Forced spawn result: {}", result);
        return Ok("Пантеон расширен. Дети родились.".to_string());
    }

    Ok("God has spoken.".to_string())
}