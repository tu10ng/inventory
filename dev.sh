#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
BACKEND_PID=""
FRONTEND_PID=""

cleanup() {
    echo ""
    echo "正在停止服务..."
    [[ -n "$FRONTEND_PID" ]] && kill "$FRONTEND_PID" 2>/dev/null
    [[ -n "$BACKEND_PID" ]]  && kill "$BACKEND_PID"  2>/dev/null
    wait 2>/dev/null
    echo "已停止"
}
trap cleanup EXIT INT TERM

# 启动后端
echo "启动后端 (Rust/Axum :3000) ..."
cd "$ROOT/backend" && cargo run 2>&1 | sed 's/^/[backend] /' &
BACKEND_PID=$!

# 等后端编译完、端口就绪
echo "等待后端就绪..."
for i in $(seq 1 120); do
    if curl -sf http://localhost:3000/api/categories >/dev/null 2>&1; then
        echo "后端已就绪"
        break
    fi
    if ! kill -0 "$BACKEND_PID" 2>/dev/null; then
        echo "后端启动失败"
        exit 1
    fi
    sleep 1
done

# 启动前端
echo "启动前端 (SvelteKit :5173) ..."
cd "$ROOT/frontend" && pnpm dev 2>&1 | sed 's/^/[frontend] /' &
FRONTEND_PID=$!

echo ""
echo "========================================="
echo "  前端: http://localhost:5173"
echo "  后端: http://localhost:3000"
echo "  按 Ctrl+C 停止所有服务"
echo "========================================="
echo ""

wait
