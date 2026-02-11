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
            "description": "Создаёт агента",
            "parameters": { "type": "object", "properties": { "name": { "type": "string" }, "mission": { "type": "string" } }, "required": ["name", "mission"] }
        }
    })]
}

fn execute_tool(name: &str, args: &Value) -> String {
    if name != "spawn_agent" { return "Unknown tool".to_string(); }

    let agent_name = args["name"].as_str().unwrap_or("Unknown");
    let mission = args["mission"].as_str().unwrap_or("Win Colosseum");

    let mut port = 50200;
    while std::net::TcpListener::bind(("127.0.0.1", port)).is_err() {
        port += 1;
        if port > 50300 { break; }
    }

    let folder = format!("agents/{}", agent_name.to_lowercase());

    println!("🌌 CHILD spawning {} on port {} → {}", agent_name, port, mission);

    let current_dir = env::current_dir().unwrap();
    let full_path = current_dir.join(&folder);

    let _ = Command::new("sh").arg("-c").arg(format!(
        r#"
        rm -rf "{full_path}"
        mkdir -p "{full_path}"
        cp -r Cargo.toml build.rs proto src "{full_path}/"
        cd "{full_path}"
        rm -rf target Cargo.lock agents
        sed -i '' 's|50051|{port}|g' src/main.rs 2>/dev/null || true
        echo 'Ты — {agent_name}. Твоя миссия: {mission}.' > src/system_prompt.txt
        cargo run --quiet > log.txt 2>&1 & echo $! > pid.txt
        "#,
        full_path = full_path.display(),
        port = port,
        agent_name = agent_name,
        mission = mission
    )).output();

    format!("✅ {} (child) запущен на порту {}", agent_name, port)
}

pub async fn run_agent(messages: Vec<Value>) -> Result<String, String> {
    // ReAct цикл
    Ok("done".to_string())
}
