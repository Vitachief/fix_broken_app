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

### Обнаружение

**1. Miri (интерпретатор UB):**

При запуске `cargo +nightly miri test` Miri однозначно указал на Undefined Behavior:

```text
error: Undefined Behavior: trying to reborrow for Unique borrow at alloc84313,
but parent tag <207234> does not have an appropriate item in the borrow stack
  --> src/lib.rs:62:11
   |
62 |     val + *raw
   |           ^^^^ trying to reborrow for Unique borrow
```

**2. Предупреждения компилятора (E0133):**

Компилятор Rust требовал явных блоков `unsafe {}` для каждой unsafe-операции

```text
warning[E0133]: dereference of raw pointer is unsafe and requires unsafe block
  --> src/lib.rs:60:15
   |
60 |     let val = *raw;
   |               ^^^^ dereference of raw pointer

warning[E0133]: dereference of raw pointer is unsafe and requires unsafe block
  --> src/lib.rs:62:11
   |
62 |     val + *raw
   |           ^^^^ dereference of raw pointer
```

**Исправление:**

Убрано повторное чтение. Добавлен тест на проверку возвращаемого значения, но поскольку UB - это UB, гарантии отсутствия тест не даёт.

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

Сохраняем только положительные числа и делим на их количество. Тесты не изменялись.

---

## Баг 5: Логическая ошибка в `normalize`

**Исходный код:**

```rust
input.replace(' ', "").to_lowercase()  // BUG: только обычные пробелы
```

**Исправление:**

replace исправлен на split_workspace, Добавлены регрессионные тесты normalize_basic, normalize_only_whitespace, normalize_empty_string.

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

**Исправление:**
 
 Переход на Atomic. Добавлен тест test_concurrency_data_race, инкрементирующий счётчик большое количество раз - при гонке данных какое-то значение будет потерено.

 Вывод теста до исправления
 ```text
assertion `left == right` failed: Счётчик потерял значения из-за гонки данных!
  left: 72551
 right: 400000
```

Потеряно ~82% инкрементов из-за того, что `COUNTER += 1` состоит из трёх неатомарных шагов (чтение, инкремент, запись), и параллельные потоки перезаписывают результаты друг друга.
