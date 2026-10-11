use broken_app::{algo, leak_buffer, normalize, sum_even};

#[test]
fn sums_even_numbers() {
    let nums = [1, 2, 3, 4];
    // Ожидаем корректное суммирование: 2 + 4 = 6.
    assert_eq!(sum_even(&nums), 6);
}

#[test]
fn counts_non_zero_bytes() {
    let data = [0_u8, 1, 0, 2, 3];
    assert_eq!(leak_buffer(&data), 3);
}

#[test]
fn dedup_preserves_uniques() {
    let uniq = algo::slow_dedup(&[5, 5, 1, 2, 2, 3]);
    assert_eq!(uniq, vec![1, 2, 3, 5]); // порядок и состав важны
}

#[test]
fn fib_small_numbers() {
    assert_eq!(algo::slow_fib(10), 55);
}

#[test]
fn normalize_simple() {
    assert_eq!(normalize(" Hello World "), "helloworld");
}

#[test]
fn averages_only_positive() {
    let nums = [-5, 5, 15];
    // Ожидается (5 + 15) / 2 = 10, но текущая реализация делит на все элементы.
    assert!((broken_app::average_positive(&nums) - 10.0).abs() < f64::EPSILON);
}

#[test]
fn normalize_basic() {
    // Обычный случай: множественные пробелы, табы и переносы строк
    let input = "  Привет \t мир \n Rust  ";
    assert_eq!(broken_app::normalize(input), "приветмирrust");
}

#[test]
fn normalize_already_clean() {
    // Строка уже в правильном формате
    let input = "helloworld";
    assert_eq!(broken_app::normalize(input), "helloworld");
}

#[test]
fn normalize_only_whitespace() {
    // Строка, состоящая только из пробельных символов
    let input = "   \t\n\r  ";
    assert_eq!(broken_app::normalize(input), "");
}

#[test]
fn normalize_empty_string() {
    // Пустая строка
    let input = "";
    assert_eq!(broken_app::normalize(input), "");
}

#[test]
fn normalize_mixed_case_and_punctuation() {
    // Проверка регистра и сохранения знаков препинания
    let input = "RUST  is \t  AWESOME!!!";
    assert_eq!(broken_app::normalize(input), "rustisawesome!!!");
}

#[test]
fn test_concurrency_data_race() {
    // 100 000 итераций * 4 потока = ожидаем 400 000
    // Из-за data race реальное число будет меньше!
    let result = broken_app::concurrency::race_increment(100_000, 4);
    
    // Этот assert, многовероятно, упадёт
    assert_eq!(result, 400_000, "Счётчик потерял значения из-за гонки данных!");
}

#[test]
fn sums_even_numbers() {
    assert_eq!(sum_even(&[1, 2, 3, 4, 5, 6]), 12);
    assert_eq!(sum_even(&[]), 0);
    assert_eq!(sum_even(&[1, 3, 5]), 0);
}

/// Тест на корректность функции use_after_free.
/// 
/// Примечание: Use-After-Free является Undefined Behavior (UB). 
/// Этот тест проверяет логическую корректность возвращаемого значения.
/// Гарантию отсутствия UB даёт только запуск этого теста через Miri:
/// `cargo +nightly miri test use_after_free_is_safe`
#[test]
fn use_after_free_is_safe() {
    // Вызываем unsafe функцию. 
    // Если внутри есть UAF, Miri немедленно завершит тест с ошибкой.
    // В обычной сборке UB может привести к возврату мусора или панике.
    let result = unsafe { broken_app::use_after_free() };
    
    // Функция должна вернуть значение, которое было в Box (42),
    // а не мусор из освобождённой памяти.
    assert_eq!(result, 42, "Функция вернула некорректное значение, возможен Use-After-Free");
}