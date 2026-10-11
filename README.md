
## Анализируемые приложения

[broken-app](https://code.s3.yandex.net/middle-rust-blockchain/broken-app.zip)
[reference-app](https://code.s3.yandex.net/middle-rust-blockchain/reference-app.zip)

## 📑 Отчёты

Все подробные отчёты о проделанной работе собраны в папке [`reports/`](reports/):

| # | Документ | Описание |
|---|----------|----------|
| 1 | [Краткая сводка](reports/01_summary.md) | Обзор результатов: исправленные баги, ускорение, использованные инструменты |
| 2 | [Поиск и исправление багов](reports/02_bugs_and_fixes.md) | Детальное описание 6 найденных багов с примерами кода и регрессионными тестами |
| 3 | [Подтверждение корректности](reports/03_correctness_proof.md) | Логи прогонов `cargo test`, Miri, Valgrind, ASan, TSan |
| 4 | [Профилирование](reports/04_profiling.md) | Анализ узких мест с помощью `samply` и flamegraph |
| 5 | [Бенчмарки до/после](reports/05_benchmarks.md) | Сравнение производительности с расчётом фактора ускорения |
| 6 | [Чек-лист самопроверки](reports/06_self_check.md) | Финальная проверка перед сдачей на ревью |

### Быстрый старт

```bash
# Проверка корректности
cargo test
cargo +nightly miri test
valgrind --leak-check=full ./target/debug/demo

# Бенчмарки
cargo bench

# Профилирование
samply record cargo bench
```
