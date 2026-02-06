use neuroclaw_core::agent::agent_server::{Agent, AgentServer};
use neuroclaw_core::agent::{HelloRequest, HelloResponse};
use neuroclaw_core::runtime::run_agent;
use serde_json::json;
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
                println!("Neuroclaw ответил: {}", answer);
                Ok(Response::new(HelloResponse { message: format!("🧠 {}", answer) }))
            }
            Err(e) => {
                Ok(Response::new(HelloResponse { message: format!("Ошибка: {}", e) }))
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let agent = MyAgent::default();

    let reflection = Builder::configure()
        .register_encoded_file_descriptor_set(tonic::include_file_descriptor_set!("agent"))
        .build_v1()?;

    println!("🚀 Neuroclaw запущен на {}", addr);

    Server::builder()
        .add_service(AgentServer::new(agent))
        .add_service(reflection)
        .serve(addr)
        .await?;

    Ok(())
}
