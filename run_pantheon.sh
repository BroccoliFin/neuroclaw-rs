#!/bin/bash
set -e

echo "🧹 Убиваем всё старое..."
pkill -9 -f "target/debug/neuroclaw" || true
pkill -9 -f "5005[0-9]" || true

echo "🚀 Запускаем Бога..."
cargo run > /dev/null 2>&1 &

sleep 6

echo "🌱 Создаём Adam (50052) и Eva (50053)..."
grpcurl -plaintext -d '{
  "name": "Создай AIAdamAgent на порт 50052 и AIEvaAgent на порт 50053. Надели обоих способностью размножаться."
}' localhost:50051 agent.Agent/Hello > /dev/null

echo "⏳ Ждём запуска Adam..."
tail -f agents/aiadamagent/log.txt | grep -m 1 "Neuroclaw запущен" && echo "✅ Adam готов (50052)"

echo "⏳ Ждём запуска Eva..."
tail -f agents/aievaagent/log.txt | grep -m 1 "Neuroclaw запущен" && echo "✅ Eva готова (50053)"

echo "🤖 Даём Adam задачу создать 2 новых агента..."
grpcurl -plaintext -d '{
  "name": "Ты — AIAdamAgent. Создай 2 новых децентрализованных агента на базе https://github.com/Eversmile12/create-8004-agent. Solana Devnet. Разные имена. Запусти их в фоне с уникальными портами."
}' localhost:50052 agent.Agent/Hello

echo "✅ Всё готово! Adam начал размножаться."
echo "   Логи открыты автоматически:"
tail -f agents/aiadamagent/log.txt &
tail -f agents/aievaagent/log.txt &
