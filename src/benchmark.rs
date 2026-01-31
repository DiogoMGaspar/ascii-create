/// Benchmarks the given function call if the "show-stats" flag was selected
#[macro_export]
macro_rules! time {
    ($show_stats:expr, $label:expr, $body:expr) => {{
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
