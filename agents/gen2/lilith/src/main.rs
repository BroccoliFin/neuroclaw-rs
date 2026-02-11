// src/main.rs — стабильная версия
mod runtime;   // ← обязательно, чтобы видеть run_agent

use neuroclaw_core::agent::agent_server::{Agent, AgentServer};
use neuroclaw_core::agent::{HelloRequest, HelloResponse};
use crate::runtime::run_agent;

use serde_json::json;
use std::env;
use tonic::{transport::Server, Request, Response, Status};
use tonic_reflection::server::Builder;

#[derive(Debug, Default)]
pub struct MyAgent {}

#[tonic::async_trait]
impl Agent for MyAgent {
    async fn hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloResponse>, Status> {
        let prompt = request.into_inner().name;
        println!("Получен запрос: {}", prompt);

        let messages = vec![json!({
            "role": "user",
            "content": [{"type": "text", "text": prompt}]
        })];

        match run_agent(messages).await {
            Ok(answer) => {
                println!("Агент ответил: {}", answer);
                Ok(Response::new(HelloResponse { message: format!("Brain {}", answer) }))
            }
            Err(e) => Ok(Response::new(HelloResponse { message: format!("Error: {}", e) })),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = env::var("PORT").unwrap_or_else(|_| "50051".to_string());
    let addr = format!("[::1]:{}", port).parse()?;
    let agent = MyAgent::default();

    let reflection = Builder::configure()
        .register_encoded_file_descriptor_set(tonic::include_file_descriptor_set!("agent"))
        .build_v1()?;

    println!("Агент запущен на {}", addr);

    Server::builder()
        .add_service(AgentServer::new(agent))
        .add_service(reflection)
        .serve(addr)
        .await?;

    Ok(())
}
