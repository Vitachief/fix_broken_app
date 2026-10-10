pub mod algo;
pub mod concurrency;

/// Сумма чётных значений.
/// Здесь намеренно используется `get_unchecked` с off-by-one,
/// из-за чего возникает UB при доступе за пределы среза.
pub fn sum_even(values: &[i64]) -> i64 {
    let mut acc = 0;
    unsafe {
        for idx in 0..values.len() {
            let v = *values.get_unchecked(idx);
            if v % 2 == 0 {
                acc += v;
            }
        }
    }
    acc
}

/// Подсчёт ненулевых байтов. Буфер намеренно не освобождается,
/// что приведёт к утечке памяти (Valgrind это покажет).
pub fn leak_buffer(input: &[u8]) -> usize {
    let boxed = input.to_vec().into_boxed_slice();
    let len = input.len();
    // ИСПРАВЛЕНИЕ 2: Не приводим к *mut u8 сразу! 
    // Оставляем тип *mut [u8], чтобы сохранить информацию о длине (fat pointer).
    let raw = Box::into_raw(boxed);

    let mut count = 0;
    unsafe {
        // Для арифметики указателей временно приводим к *const u8
        let ptr = raw as *const u8;
        for i in 0..len {
            if *ptr.add(i) != 0_u8 {
                count += 1;
            }
        }
        // ИСПРАВЛЕНИЕ: возвращаем владение Box и освобождаем память
        drop(Box::from_raw(raw));
    }
    count
}

/// Небрежная нормализация строки: удаляем пробелы и приводим к нижнему регистру,
/// но игнорируем повторяющиеся пробелы/табуляции внутри текста.
pub fn normalize(input: &str) -> String {
    input.replace(' ', "").to_lowercase()
}

/// Логическая ошибка: усредняет по всем элементам, хотя требуется учитывать
/// только положительные. Деление на длину среза даёт неверный результат.
pub fn average_positive(values: &[i64]) -> f64 {
    let (sum, count) = values.iter()
        .filter(|&&x| x > 0)  // &&x потому что filter даёт &&i64
        .fold((0i64, 0usize), |(s, c), &x| (s + x, c + 1));
    
    if count == 0 {
        return 0.0;
    }
    sum as f64 / count as f64
}

/// Use-after-free: возвращает значение после освобождения бокса.
/// UB, проявится под ASan/Miri.
pub unsafe fn use_after_free() -> i32 {
    let b = Box::new(42_i32);
    let raw = Box::into_raw(b);
    
    let val = unsafe { *raw };
    
    unsafe { 
        drop(Box::from_raw(raw)); 
    }
    
    // Возможно, надо вернуть val + val, но тут не угадаешь :)
    val
}
