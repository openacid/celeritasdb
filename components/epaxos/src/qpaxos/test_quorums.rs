use super::quorums::*;

#[test]
fn test_quorums() {
    let cases: Vec<(i32, i32, i32)> = vec![
        (0, 1, 1),
        (1, 1, 1),
        (2, 2, 2),
        (3, 2, 2),
        (4, 3, 3),
        (5, 3, 3),
        (6, 4, 4),
        (7, 4, 5),
        (8, 5, 5),
        (9, 5, 6),
    ];

    for (n_replicas, q, fastq) in cases {
        assert_eq!(q, quorum(n_replicas), "quorum n={}", n_replicas);
        assert_eq!(
            fastq,
            fast_quorum(n_replicas),
            "fast-quorum n={}",
            n_replicas
        );
    }
}
