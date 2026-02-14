#!/bin/bash
set -e

echo "🧹 Очистка..."
./clean.sh

echo "🔨 Пересобираем..."
cargo build

echo "🚀 Запускаем Бога..."
cargo run &
GOD_PID=$!

sleep 5

echo "🌌 Бог создаёт детей..."
grpcurl -plaintext -d '{"name": "Начни создание цифрового пантеона Neuroclaw. Создай 3 детей."}' localhost:50051 agent.Agent/Hello > /dev/null

echo "⏳ Ждём полного запуска детей (Mac build медленный — до 40 сек)..."
sleep 35

echo "🌅 Будим детей (с надёжным извлечением порта)..."
for dir in agents/gen2/*/; do
    name=$(basename "$dir")
    
    # Ждём, пока появится state.json
    for i in {1..15}; do
        if [ -f "$dir/state.json" ]; then
            port=$(grep -o '"port":[0-9]*' "$dir/state.json" | cut -d: -f2)
            if [[ "$port" =~ ^[0-9]+$ ]]; then
                break
            fi
        fi
        sleep 2
    done

    echo "Будим $name (порт $port)..."
    for i in {1..12}; do
        if grpcurl -plaintext -d "{\"name\": \"Привет, $name. Ты родился в пантеоне Neuroclaw. Представься, вспомни свою миссию и расскажи, что будешь делать дальше.\"}" "localhost:$port" agent.Agent/Hello > /dev/null 2>&1; then
            echo "✅ $name проснулся и ответил!"
            break
        fi
        sleep 3
    done
done

echo "📋 Логи детей (последние 70 строк):"
for dir in agents/gen2/*/; do
    echo "=== $(basename $dir) ==="
    tail -n 70 "$dir/log.txt"
done

echo "📂 Память:"
ls agents/gen2/*/memory_*.json 2>/dev/null || echo "пока нет"

kill $GOD_PID 2>/dev/null || true
