# Отчёт по поиску и исправлению багов

## Баг 1: Выход за верхний индекс массива в `sum_even` (UB)

**Исходный код:**

```rust
for idx in 0..=values.len() {  // BUG: ..= включает верхнюю границу
    let v = *values.get_unchecked(idx);
    if v % 2 == 0 { acc += v; }
}
```

**Обнаружение:**

```text
thread 'sums_even_numbers' panicked at src/lib.rs:11:29: unsafe precondition(s) violated: slice::get_unchecked requires that the index is within the slice
```

**Исправление:**

Добавлен регрессионный тест sums_even_numbers, исправлен выход за границу массива.

---

## Баг 2: Утечка памяти + incorrect layout в `leak_buffer`

**Исходный код:**

```rust
let raw = Box::into_raw(boxed) as *mut u8;  // BUG: потеря длины
// ... нет Box::from_raw -> утечка
```

**Обнаружение (Miri, первый проход):**

```text
error: memory leaked: alloc48431 (Rust heap, size: 5, align: 1)
```

**Обнаружение (Miri, второй проход после добавления drop):**

```text
error: Undefined Behavior: incorrect layout on deallocation:
alloc48431 has size 5 and alignment 1, but gave size 1 and alignment 1
```

**Корневая причина:** Приведение `*mut [u8]` (fat pointer, хранит адрес + длину) к `*mut u8` (thin pointer, только адрес) безвозвратно теряет информацию о размере выделения. При последующем `Box::from_raw(raw)` Rust считает, что нужно освободить 1 байт вместо 5.

**Исправление:**

Баг исправлен, добавлен leak_check.rs, который считает аллокации без использования сторонних инструментов

**Подтверждение (Valgrind):**

```text
==PID== LEAK SUMMARY:
==PID==    definitely lost: 0 bytes in 0 blocks
==PID==    indirectly lost: 0 bytes in 0 blocks
==PID==      possibly lost: 0 bytes in 0 blocks
==PID==    still reachable: 0 bytes in 0 blocks
==PID== All heap blocks were freed -- no leaks are possible
```

---

## Баг 3: Use-After-Free в `use_after_free`

**Исходный код:**

```rust
let val = *raw;
drop(Box::from_raw(raw));
val + *raw  // BUG: чтение после освобождения
```

**Исправление:**

```rust
let val = unsafe { *raw };
unsafe { drop(Box::from_raw(raw)); }
val  // FIX: возвращаем сохранённое значение, без повторного чтения
```

---

## Баг 4: Логическая ошибка в `average_positive`

**Исходный код:**

```rust
let sum: i64 = values.iter().sum();  // BUG: суммирует ВСЕ числа
sum as f64 / values.len() as f64     // BUG: делит на общую длину
```

**Обнаружение:**

```text
assertion failed: (broken_app::average_positive(&nums) - 10.0).abs() < f64::EPSILON
```

**Исправление:**

```rust
let (sum, count) = values.iter()
    .filter(|&&x| x > 0)
    .fold((0i64, 0usize), |(s, c), &x| (s + x, c + 1));
if count == 0 { return 0.0; }
sum as f64 / count as f64  // FIX: делим на количество положительных
```

---

## Баг 5: Логическая ошибка в `normalize`

**Исходный код:**

```rust
input.replace(' ', "").to_lowercase()  // BUG: только обычные пробелы
```

**Исправление:**

```rust
input
    .split_whitespace()  // FIX: любые пробельные символы
    .collect::<Vec<&str>>()
    .join(" ")           // FIX: схлопывание в один пробел
    .to_lowercase()
```

**Регрессионные тесты:**

```rust
#[test]
fn normalize_basic() {
    assert_eq!(
        broken_app::normalize("  Привет \t мир \n Rust  "),
        "привет мир rust"
    );
}

#[test]
fn normalize_only_whitespace() {
    assert_eq!(broken_app::normalize("   \t\n\r  "), "");
}

#[test]
fn normalize_empty_string() {
    assert_eq!(broken_app::normalize(""), "");
}
```

---

## Баг 6: Data race в `race_increment`

**Исходный код:**

```rust
static mut COUNTER: u64 = 0;

pub fn race_increment(iterations: usize, threads: usize) -> u64 {
    unsafe { COUNTER = 0; }
    let mut handles = Vec::new();
    for _ in 0..threads {
        handles.push(thread::spawn(move || {
            for _ in 0..iterations {
                unsafe { COUNTER += 1; }  // BUG: не атомарно
            }
        }));
    }
    for h in handles { let _ = h.join(); }
    unsafe { COUNTER }
}
```

**Обнаружение (логический тест):**

```text
assertion `left == right` failed: Счётчик потерял значения из-за гонки данных!
  left: 72551
 right: 400000
```

Потеряно ~82% инкрементов из-за того, что `COUNTER += 1` состоит из трёх неатомарных шагов (чтение, инкремент, запись), и параллельные потоки перезаписывают результаты друг друга.

**Исправление:**

```rust
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn race_increment(iterations: usize, threads: usize) -> u64 {
    COUNTER.store(0, Ordering::SeqCst);
    let mut handles = Vec::new();
    for _ in 0..threads {
        handles.push(thread::spawn(move || {
            for _ in 0..iterations {
                COUNTER.fetch_add(1, Ordering::SeqCst);  // FIX: атомарно
            }
        }));
    }
    for h in handles { let _ = h.join(); }
    COUNTER.load(Ordering::SeqCst)
}
```

**Подтверждение:** Тест `test_concurrency_data_race` возвращает ровно 400 000.