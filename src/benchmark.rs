/// Benchmarks the given function call if the "show-stats" flag was selected
#[macro_export]
macro_rules! time {
    ($body:expr, $label:expr, $show_stats:expr) => {{
        if $show_stats {
            let start = std::time::Instant::now();
            let result = $body;
            let duration = start.elapsed();
            println!("{} took {:?}", $label, duration);
            result
        } else {
            $body
        }
    }};
}
