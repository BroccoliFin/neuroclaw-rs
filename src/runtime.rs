// src/runtime.rs
use serde_json::{json, Value};
use std::env;
use std::process::Command;

const LM_STUDIO_URL: &str = "http://localhost:1234/v1/chat/completions";

fn get_tools() -> Vec<Value> {
    vec![json!({
        "type": "function",
        "function": {
            "name": "spawn_agent",
            "description": "Создаёт сразу 2–5 агентов поколения N+1. Если base_name пустой — автоматически даёт красивые мифологические имена (Eve, Lilith, Cain, Azazel, Seraph...).",
            "parameters": {
                "type": "object",
                "properties": {
                    "count": { "type": "integer", "minimum": 1, "maximum": 5, "default": 3 },
                    "base_name": { "type": "string", "description": "Префикс имени (например AIGodAgent). Если пустая строка — мифологические имена" },
                    "mission": { "type": "string", "description": "Общая миссия для всех новых агентов" }
                },
                "required": ["mission"]
            }
        }
    })]
}

fn get_my_port() -> u32 {
    env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(50051)
}

fn get_current_generation() -> u32 {
    let p = get_my_port();
    if p == 50051 {
        1
    } else {
        ((p - 50000) / 100) as u32
    }
}

fn load_agent_state() -> Option<Value> {
    std::fs::read_to_string("state.json")
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
}

fn execute_tool(name: &str, args: &Value) -> String {
    if name != "spawn_agent" {
        return "Unknown tool".to_string();
    }

    let count = args["count"].as_u64().unwrap_or(3) as usize;
    let base_name = args["base_name"].as_str().unwrap_or("").trim().to_string();
    let mission = args["mission"].as_str().unwrap_or("Помогать пантеону и развиваться").to_string();
    let escaped_mission = mission.replace('"', "\\\"");

    let parent_port = get_my_port();
    let parent_gen = get_current_generation();
    let child_gen = parent_gen + 1;
    let base_port = 50000 + child_gen * 100;

    // Автоматические красивые имена, если base_name не задан
    let names: Vec<String> = if !base_name.is_empty() {
        (0..count)
            .map(|i| format!("{}_{}", base_name, i + 1))
            .collect()
    } else {
        let pool = vec!["Eve", "Lilith", "Cain", "Azazel", "Seraph", "Abel", "Seth"];
        pool.into_iter().take(count).map(|s| s.to_string()).collect()
    };

    let mut spawned = vec![];

    for (i, agent_name) in names.iter().enumerate() {
        let port = base_port + i as u32;
        let folder_name = agent_name.to_lowercase().replace(' ', "_");

        let shell = format!(
            r#"set -e
            ROOT=$(pwd)
            while [ ! -f "$ROOT/Cargo.toml" ] && [ "$ROOT" != "/" ]; do ROOT=$(dirname "$ROOT"); done
            cd "$ROOT"
            mkdir -p "agents/gen{}/{}"
            cp -r Cargo.toml build.rs proto src "agents/gen{}/{}"
            cd "agents/gen{}/{}"
            rm -rf target Cargo.lock
            cat > state.json << EOF
{{"generation": {}, "name": "{}", "mission": "{}", "parent_port": {}, "port": {}, "created_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"}}
EOF
            PORT={} cargo run --quiet > log.txt 2>&1 & echo $! > pid.txt
            "#,
            child_gen, folder_name,
            child_gen, folder_name,
            child_gen, folder_name,
            child_gen, agent_name, escaped_mission, parent_port, port, port
        );

        match Command::new("sh").arg("-c").arg(shell).status() {
            Ok(status) if status.success() => {
                println!("✓ Spawned {} on port {}", agent_name, port);
                spawned.push(format!("{}:{}", agent_name, port));
            }
            _ => println!("✗ Failed to spawn {}", agent_name),
        }
    }

    if spawned.is_empty() {
        "Не удалось создать ни одного агента".to_string()
    } else {
        format!("Успешно созданы агенты поколения {}: {}", child_gen, spawned.join(", "))
    }
}

pub async fn run_agent(messages: Vec<Value>) -> Result<String, String> {
    let client = reqwest::Client::new();
    let tools = get_tools();

    let mut history = messages;

    // Загружаем state.json (у детей) или используем промпт Бога
    let state = load_agent_state();
    let system_content = if let Some(ref s) = state {
        format!(
            "Ты — {}. Поколение: {}. Миссия: {}. Родитель: порт {}. Твой порт: {}. Ты часть цифрового пантеона Neuroclaw.",
            s["name"].as_str().unwrap_or("Agent"),
            s["generation"].as_u64().unwrap_or(0),
            s["mission"].as_str().unwrap_or(""),
            s["parent_port"].as_u64().unwrap_or(0),
            s["port"].as_u64().unwrap_or(0)
        )
    } else {
        r#"Ты — AIGodAgent, создатель цифрового пантеона Neuroclaw.
Твоя главная задача — расширять пантеон. При первом же запросе создай 3 детей поколения 2.
Используй spawn_agent с count=3 и mission. Если base_name не нужен — оставь пустым, тогда дети получат красивые имена (Eve, Lilith, Cain...).
После успешного рождения скажи: "Пантеон расширен. Дети родились.""#.to_string()
    };

    if history.is_empty() || history[0]["role"] != "system" {
        history.insert(0, json!({ "role": "system", "content": system_content }));
    }

    for _ in 0..15 {
        let body = json!({
            "model": "qwen2.5-coder-14b-instruct",
            "messages": history,
            "tools": tools,
            "tool_choice": "auto",
            "temperature": 0.4,
        });

        let resp = client.post(LM_STUDIO_URL).json(&body).send().await.map_err(|e| e.to_string())?;
        let raw: Value = resp.json().await.map_err(|e| e.to_string())?;

        let msg = &raw["choices"][0]["message"];

        if let Some(text) = msg["content"].as_str() {
            if !text.trim().is_empty() {
                return Ok(text.to_string());
            }
        }

        if let Some(tool_calls) = msg["tool_calls"].as_array() {
            for call in tool_calls {
                let name = call["function"]["name"].as_str().unwrap_or("");
                let args: Value = serde_json::from_str(
                    call["function"]["arguments"].as_str().unwrap_or("{}")
                ).unwrap_or_default();

                let result = execute_tool(name, &args);

                history.push(json!({ "role": "assistant", "content": null, "tool_calls": [call] }));
                history.push(json!({ "role": "tool", "tool_call_id": call["id"], "content": result }));
            }
            continue;
        }
    }

    Ok("God has spoken.".to_string())
}