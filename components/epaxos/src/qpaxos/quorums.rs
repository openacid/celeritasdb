pub fn quorum(n: i32) -> i32 {
    n / 2 + 1
}

pub fn fast_quorum(n: i32) -> i32 {
    let q = n / 2 + 1;
    let f = (n - 1) / 2;
    let fq = f + q / 2;
    // Except f + q/2, fast_quorum must satisfy another condition:
    // two fast_quorum must have intersection.
    if fq < q {
        q
    } else {
        fq
    }
}
