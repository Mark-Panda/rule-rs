use axum::{
    debug_handler,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use rule_rs::{
    engine::rule::RuleEngineTrait,
    types::Message,
    types::{NodeDescriptor, RuleChain},
    RuleEngine,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid; // 引入 async_trait 宏

// API 响应格式
#[derive(Debug, Serialize)]
struct ApiResponse<T: Serialize> {
    code: i32,
    message: String,
    data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
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
#[derive(Debug, Deserialize, Serialize)]
struct RuleChainRequest {
    id: Uuid,
    name: String,
    root: bool,
    nodes: Vec<NodeRequest>,
    connections: Vec<ConnectionRequest>,
    metadata: Metadata,
}

#[derive(Debug, Deserialize, Serialize)]
struct Metadata {
    version: u32,
    created_at: u64,
    updated_at: u64,
}

#[derive(Debug, Deserialize, Serialize)]
struct NodeRequest {
    id: Uuid,
    type_name: String,
    chain_id: Uuid,
    config: serde_json::Value,
    layout: Position,
}

#[derive(Debug, Deserialize, Serialize)]
struct ConnectionRequest {
    from_id: Uuid,
    to_id: Uuid,
    type_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
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
        .route("/api/chains/execute/:id", post(execute_chain))
        // .layer(TraceLayer::new_for_http())
        .with_state(AppState {
            engine: engine.clone(),
        });

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// 获取所有组件
// #[debug_handler]
async fn list_components(State(state): State<AppState>) -> Json<ApiResponse<Vec<NodeDescriptor>>> {
    let components = state.engine.get_registered_components().await;
    Json(ApiResponse::success(components))
}

// 创建规则链
async fn create_chain(
    State(state): State<AppState>,
    Json(req): Json<RuleChainRequest>,
) -> Result<Json<ApiResponse<Uuid>>, Json<ApiResponse<()>>> {
    let chain = serde_json::to_string(&req).unwrap();
    println!("chain: {}", chain);
    match state.engine.load_chain(&chain).await {
        Ok(id) => Ok(Json(ApiResponse::success(id))),
        Err(e) => Err(Json(ApiResponse::error(400, &e.to_string()))),
    }
}

// 获取规则链
#[debug_handler]
async fn get_chain(State(state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match state.engine.get_chain(id).await {
        Some(chain) => {
            let chain = (*chain).clone(); // 解引用并克隆
            Json(ApiResponse::success(chain)).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<RuleChain>::error(404, "Rule chain not found")),
        )
            .into_response(),
    }
}

// 更新规则链
#[debug_handler]
async fn update_chain(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<RuleChainRequest>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    println!("update_chain: {:?}", req);
    println!("id: {:?}", id);
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
    // 检查规则链是否存在
    if state.engine.get_chain(id).await.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error(404, "Rule chain not found")),
        ));
    }
    match state.engine.remove_chain(id).await {
        Ok(_) => Ok(Json(ApiResponse::success(()))),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(400, &e.to_string())),
        )),
    }
}

// 执行规则链
#[debug_handler]
async fn execute_chain(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(msg): Json<Message>,
) -> impl IntoResponse {
    // 启动异步任务处理消息
    tokio::spawn({
        let engine = state.engine.clone();
        async move {
            if let Err(e) = engine.process_msg(id, msg).await {
                tracing::error!("处理消息失败: {}", e);
            }
        }
    });

    // 立即返回成功响应
    Json(ApiResponse::success("消息已接收并开始处理")).into_response()
}

// // 执行规则链
// #[debug_handler]
// async fn execute_chain(
//     State(state): State<AppState>,
//     Path(id): Path<Uuid>,
//     Json(msg): Json<Message>,
// ) -> impl IntoResponse {
//     match state.engine.process_msg(id, msg).await {
//         Ok(result) => Json(ApiResponse::success(result)).into_response(),
//         Err(e) => (
//             StatusCode::INTERNAL_SERVER_ERROR,
//             Json(ApiResponse::<()>::error(500, &e.to_string())),
//         )
//             .into_response(),
//     }
// }
