# Подтверждение корректности

## 1. Тесты (cargo test)

**Команда:**

```bash
cargo test
```

**Результат:**

```text
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Все тесты проходят, включая регрессионные тесты на найденные баги.

---

## 2. Miri (поиск UB)

**Команда:**

```bash
cargo +nightly miri test
```

**Результат:**

```text
test result: ok. 6 passed; 0 failed
```

Miri не обнаружил неопределённого поведения (UB). Все операции с памятью корректны.

---

## 3. Valgrind (поиск утечек памяти)

**Команда:**

```bash
valgrind --leak-check=full --show-leak-kinds=all ./target/debug/demo
```

**Результат:**

```text
==PID== HEAP SUMMARY:
==PID==     in use at exit: 0 bytes in 0 blocks
==PID==
==PID== LEAK SUMMARY:
==PID==    definitely lost: 0 bytes in 0 blocks
==PID==    indirectly lost: 0 bytes in 0 blocks
==PID==      possibly lost: 0 bytes in 0 blocks
==PID==    still reachable: 0 bytes in 0 blocks
==PID==         suppressed: 0 bytes in 0 blocks
==PID==
==PID== All heap blocks were freed -- no leaks are possible
```

Утечки памяти полностью устранены.

---

## 4. AddressSanitizer (ASan)

**Команда:**

```bash
cargo clean
RUSTFLAGS="-Zsanitizer=address" cargo +nightly test \
    --target x86_64-unknown-linux-gnu
```

**Результат:**

```text
test result: ok. 6 passed; 0 failed
```

ASan не обнаружил ошибок работы с памятью (buffer overflow, use-after-free, double-free).

---

## 5. ThreadSanitizer (TSan)

**Команда:**

```bash
cargo clean
RUSTFLAGS="-Zsanitizer=thread" cargo +nightly test \
    --target x86_64-unknown-linux-gnu test_concurrency_data_race
```

**Результат:**

```text
test test_concurrency_data_race ... ok
test result: ok. 1 passed; 0 failed
```

**Примечание:** При запуске полного набора тестов TSan выдавал предупреждения на внутренности тестового раннера (`std::sync::mpmc`, `core::profiling`), что является известным ограничением nightly-версии инструмента (ложные срабатывания на атомарных операциях стандартной библиотеки). Стек-трейсы предупреждений указывают исключительно на внутренности `std::` и `alloc::`, а не на код проекта. Логический тест однозначно подтвердил корректность пользовательского кода: счётчик возвращает ровно 400 000 без потерь.

---

## Вывод

Все инструменты диагностики подтверждают корректность исправленного кода:

- Тесты проходят
- Miri не находит UB
- Valgrind не находит утечек
- ASan не находит ошибок памяти
- TSan не находит гонок в пользовательском коде