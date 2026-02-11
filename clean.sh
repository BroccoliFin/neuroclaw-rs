#!/bin/bash

echo "🧹 Полная очистка пантеона..."

pkill -9 -f neuroclaw 2>/dev/null || true
pkill -9 -f "target/debug/neuroclaw" 2>/dev/null || true
pkill -9 -f "5005[0-9]" 2>/dev/null || true

rm -rf agents/
rm -f memory_*.json memory_god.json

echo "✅ Очистка завершена! (все memory_*.json удалены)"
