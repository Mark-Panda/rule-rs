use axum::{
    debug_handler,
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use hyper::Server;
use rulego_rs::{
    types::{NodeDescriptor, RuleChain},
    Message, RuleEngine,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

// API 响应格式
#[derive(Debug, Serialize)]
struct ApiResponse<T> {
    code: i32,
    message: String,
    data: Option<T>,
}

impl<T> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            code: 0,
            message: "success".to_string(),
            data: Some(data),
        }
    }

    fn error(code: i32, message: &str) -> Self {
        Self {
            code,
            message: message.to_string(),
            data: None,
        }
    }
}

// 规则链请求体
#[derive(Debug, Deserialize)]
struct RuleChainRequest {
    name: String,
    root: bool,
    nodes: Vec<NodeRequest>,
    connections: Vec<ConnectionRequest>,
}

#[derive(Debug, Deserialize)]
struct NodeRequest {
    type_name: String,
    config: serde_json::Value,
    layout: Position,
}

#[derive(Debug, Deserialize)]
struct ConnectionRequest {
    from_id: Uuid,
    to_id: Uuid,
    type_name: String,
}

#[derive(Debug, Deserialize)]
struct Position {
    x: f32,
    y: f32,
}

// 应用状态
#[derive(Clone)]
struct AppState {
    engine: Arc<RuleEngine>,
}

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // 创建规则引擎
    let engine = Arc::new(RuleEngine::new().await);

    // 创建路由
    let app = Router::new()
        .route("/api/components", get(list_components))
        .route("/api/chains", post(create_chain))
        .route("/api/chains/:id", get(get_chain))
        .route("/api/chains/:id", put(update_chain))
        .route("/api/chains/:id", delete(delete_chain))
        .layer(TraceLayer::new_for_http())
        .with_state(AppState {
            engine: engine.clone(),
        });

    // 启动服务器
    let addr = "127.0.0.1:3000";
    println!("Server running on http://{}", addr);

    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// 获取所有组件
#[debug_handler]
async fn list_components(State(state): State<AppState>) -> Json<ApiResponse<Vec<NodeDescriptor>>> {
    let components = state.engine.get_registered_components().await;
    Json(ApiResponse::success(components))
}

// 创建规则链
#[debug_handler]
async fn create_chain(
    State(state): State<AppState>,
    Json(req): Json<RuleChainRequest>,
) -> Result<Json<ApiResponse<Uuid>>, (StatusCode, Json<ApiResponse<()>>)> {
    // 构造规则链配置
    let chain = json!({
        "id": Uuid::new_v4(),
        "name": req.name,
        "root": req.root,
        "nodes": req.nodes,
        "connections": req.connections,
        "metadata": {
            "version": 1,
            "created_at": chrono::Utc::now().timestamp_millis(),
            "updated_at": chrono::Utc::now().timestamp_millis()
        }
    });

    // 加载规则链
    match state.engine.load_chain(&chain.to_string()).await {
        Ok(id) => Ok(Json(ApiResponse::success(id))),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(400, &e.to_string())),
        )),
    }
}

// 获取规则链
#[debug_handler]
async fn get_chain(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Arc<RuleChain>>>, (StatusCode, Json<ApiResponse<()>>)> {
    match state.engine.get_chain(id).await {
        Some(chain) => Ok(Json(ApiResponse::success(chain))),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error(404, "Rule chain not found")),
        )),
    }
}

// 更新规则链
#[debug_handler]
async fn update_chain(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<RuleChainRequest>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    // 检查规则链是否存在
    if state.engine.get_chain(id).await.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error(404, "Rule chain not found")),
        ));
    }

    // 构造更新后的规则链配置
    let chain = json!({
        "id": id,
        "name": req.name,
        "root": req.root,
        "nodes": req.nodes,
        "connections": req.connections,
        "metadata": {
            "version": state.engine.get_current_version().await + 1,
            "created_at": chrono::Utc::now().timestamp_millis(),
            "updated_at": chrono::Utc::now().timestamp_millis()
        }
    });

    // 更新规则链
    match state.engine.load_chain(&chain.to_string()).await {
        Ok(_) => Ok(Json(ApiResponse::success(()))),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(400, &e.to_string())),
        )),
    }
}

// 删除规则链
#[debug_handler]
async fn delete_chain(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    match state.engine.remove_chain(id).await {
        Ok(_) => Ok(Json(ApiResponse::success(()))),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(400, &e.to_string())),
        )),
    }
}
