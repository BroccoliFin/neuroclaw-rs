# Доступные инструменты Neuroclaw

1. web_search(query: string)
   → ищет актуальную информацию в интернете через DuckDuckGo

2. save_to_memory(key: string, value: string)
   → сохраняет точное значение в долговременную память (например: bitcoin_price → 65234)

3. read_from_memory(key: string)
   → читает сохранённое значение. Если данных нет — возвращает "Нет данных"

Правило использования:
- Для цены биткоина: сначала read_from_memory('bitcoin_price')
- Если данных нет или они старше 30 минут → web_search → save_to_memory('bitcoin_price', ЧИСЛО)