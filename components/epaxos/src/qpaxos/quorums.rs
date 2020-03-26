pub fn quorum(n: i32) -> i32 {
    n / 2 + 1
}

pub fn fast_quorum(n: i32) -> i32 {
    let q = n / 2 + 1;
    let f = (n - 1) / 2;
    f + q / 2
}
