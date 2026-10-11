/// Эффективное удаление дубликатов за O(n log n)
pub fn slow_dedup(values: &[u64]) -> Vec<u64> {
    let mut out = values.to_vec();
    out.sort_unstable(); // Сортируем весь массив ОДИН раз
    out.dedup();         // Удаляем соседние дубликаты за один проход
    out
}

pub fn slow_fib(n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    let mut a = 0u64;
    let mut b = 1u64;
    for _ in 1..n {
        let c = a + b;
        a = b;
        b = c;
    }
    b
}
