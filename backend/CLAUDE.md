# Backend — Rust + Axum + SQLite

## 技术栈

- **Axum 0.8** — HTTP 框架，路径参数用 `{id}` 语法
- **SQLite + sqlx 0.8** — 数据库（运行时查询，非编译期检查）
- **tokio** — 异步运行时
- **tower-http** — CORS + 静态文件服务（serve `../frontend/build`）
- **serde** — JSON 序列化/反序列化
- **anyhow** — 错误处理

## 模块结构

| 文件 | 职责 |
|------|------|
| `main.rs` | 入口：初始化 DB 连接池 → 构建 router → 启动 0.0.0.0:3000 |
| `db.rs` | `init_pool()` 创建 SQLite 连接池（WAL 模式），`run_migrations()` 执行 SQL |
| `error.rs` | `AppError` 类型：`anyhow::Error` → HTTP 500，所有 handler 的 `Result` 统一用它 |
| `models.rs` | 所有数据结构（查询模型 + 创建/更新 DTO） |
| `handlers/` | 路由处理函数，每个资源一个文件，`mod.rs` 注册所有路由 |

## 模式约定

### Handler 签名
```rust
pub async fn handler_name(
    State(pool): State<SqlitePool>,    // 连接池
    Path(id): Path<i64>,               // 路径参数（可选）
    Json(body): Json<SomeType>,        // 请求体（可选）
) -> Result<Json<ReturnType>, AppError>
```

### 模型命名
- 查询模型：`Item`、`Trip`（derive `Serialize, Deserialize, sqlx::FromRow`）
- 创建 DTO：`CreateItem`、`CreateTrip`（derive `Deserialize`，字段可选用 `#[serde(default)]`）
- 更新 DTO：`UpdateTrip`（所有字段 `Option<T>`，handler 里 merge with existing）

### 路由注册
```rust
.route("/api/resource", get(handler::list).post(handler::create))
.route("/api/resource/{id}", get(handler::get).put(handler::update).delete(handler::delete))
```

## 迁移

- 所有 SQL 在 `migrations/001_initial.sql` 单文件
- `db.rs` 用 `split(';')` 逐条执行 — **SQL 字符串值不能含分号**
- 增量迁移用 `ALTER TABLE ADD COLUMN`，重复执行时 "duplicate column name" 错误会被 warn 而非 panic
- 幂等写法：`CREATE TABLE IF NOT EXISTS`、`INSERT OR IGNORE`、`ALTER TABLE`（容错）

## 已知陷阱

- **路由歧义**：Axum `{id}` 会匹配所有单段路径，`/api/items/stats` 会被 `/api/items/{id}` 抢先匹配。解决方案：使用独立前缀（如 `/api/item-stats`）
- **DELETE 返回空**：handler 返回 `Result<(), AppError>`，前端需处理 204/空响应
