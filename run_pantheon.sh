#!/bin/bash
set -e

echo "🛑 Убиваем старые процессы..."
pkill -f "neuroclaw" 2>/dev/null || true
sleep 2

echo "🧹 Очистка агентов..."
rm -rf agents/gen2/* 2>/dev/null || true

echo "🔨 Собираем..."
cargo build

echo "🚀 Запускаем Бога..."
cargo run &
GOD_PID=$!

echo "⏳ Ждём Бога..."
for i in {1..30}; do
    if nc -z localhost 50051 2>/dev/null; then
        echo "✅ Бог запущен"
        break
    fi
    sleep 2
done

echo "🌌 Бог создаёт детей..."
grpcurl -plaintext -d '{"name": "Начни создание цифрового пантеона Neuroclaw. Создай 3 детей."}' localhost:50051 agent.Agent/Hello > /dev/null

echo "⏳ Ждём 200 секунд — все дети должны сделать 3–4 heartbeat'а..."
sleep 200

echo ""
echo "📋 Логи детей (последние 80 строк):"
for dir in agents/gen2/*/; do
    name=$(basename "$dir")
    echo "=== ${name} ==="
    tail -n 80 "$dir/log.txt" 2>/dev/null || echo "   log.txt пуст"
    echo ""
done

echo "📂 Память:"
ls -l agents/gen2/*/memory_*.json 2>/dev/null || echo "   пока нет"