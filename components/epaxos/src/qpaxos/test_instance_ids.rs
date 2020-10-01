use crate::qpaxos::InstanceId;
use crate::qpaxos::InstanceIds;
pub use std::cmp::Ordering;

#[test]
fn test_instance_ids_deref() {
    let ids = InstanceIds {
        ids: hashmap! {
            1 => 2,
            3 => 4,
        },
    };

    assert_eq!(ids[&1], 2);
    assert_eq!(ids[&3], 4);

    let mut ids = InstanceIds { ids: hashmap! {} };

    ids.insert(1, 2);
    assert_eq!(ids[&1], 2);
}

#[test]
#[should_panic]
fn test_instance_ids_index_panic() {
    let ids = InstanceIds {
        ids: hashmap! {
            1 => 2,
            3 => 4,
        },
    };

    let _ = ids[&2];
}

#[test]
fn test_instance_ids_cmp_inst() {
    let ids = InstanceIds {
        ids: hashmap! {
            1 => 2,
            3 => 4,
        },
    };

    assert_eq!(Some(Ordering::Less), ids.partial_cmp(&(1, 3).into()));
    assert_eq!(Some(Ordering::Equal), ids.partial_cmp(&(1, 2).into()));
    assert_eq!(Some(Ordering::Greater), ids.partial_cmp(&(1, 1).into()));
    assert_eq!(Some(Ordering::Less), ids.partial_cmp(&(3, 5).into()));
    assert_eq!(Some(Ordering::Equal), ids.partial_cmp(&(3, 4).into()));
    assert_eq!(Some(Ordering::Greater), ids.partial_cmp(&(3, 3).into()));
    assert_eq!(Some(Ordering::Less), ids.partial_cmp(&(2, 1).into()));

    assert!(ids < (1, 3).into());
    assert!(ids > (1, 1).into());
    assert!(ids == InstanceId::from((1, 2)));

    // Absent replica-id always results in Less
    assert!(ids < (2, 1).into());
    assert!(ids <= (2, 1).into());

    assert!(!(ids == InstanceId::from((2, 2))));
    assert!(ids != InstanceId::from((2, 2)));
}

#[test]
fn test_instance_ids_from() {
    let iid = InstanceId::from((1, 2));

    let sl: &[_] = &[iid];
    let ids: InstanceIds = sl.into();
    assert_eq!(2, ids[&1]);

    let ids: InstanceIds = vec![iid].into();
    assert_eq!(2, ids[&1]);

    let sl: &[_] = &[(1, 2), (3, 4)];
    let ids: InstanceIds = sl.into();
    assert_eq!(2, ids[&1]);

    let sl: &[(i32, i64)] = &[(1, 2), (3, 4)];
    let ids: InstanceIds = sl.into();
    assert_eq!(2, ids[&1]);
}

#[test]
fn test_instance_ids_from_array() {
    let arr: [i32; 0] = [];
    let ids: InstanceIds = arr.into();
    assert_eq!(0, ids.len());

    let arr = [(1, 2)];
    let ids: InstanceIds = arr.into();
    assert_eq!(2, ids[&1]);

    let arr = [(1, 2), (3, 4)];
    let ids: InstanceIds = arr.into();
    assert_eq!(2, ids[&1]);

    let arr = [(1, 2), (3, 4), (5, 6)];
    let ids: InstanceIds = arr.into();
    assert_eq!(2, ids[&1]);

    let arr = [(1, 2), (3, 4), (5, 6), (7, 8)];
    let ids: InstanceIds = arr.into();
    assert_eq!(2, ids[&1]);

    let arr = [(1, 2), (3, 4), (5, 6), (7, 8), (9, 10)];
    let ids: InstanceIds = arr.into();
    assert_eq!(2, ids[&1]);

    let arr = [(1, 2), (3, 4), (5, 6), (7, 8), (9, 10), (11, 12)];
    let ids: InstanceIds = arr.into();
    assert_eq!(2, ids[&1]);

    let arr = [(1, 2), (3, 4), (5, 6), (7, 8), (9, 10), (11, 12), (13, 14)];
    let ids: InstanceIds = arr.into();
    assert_eq!(2, ids[&1]);

    let arr = [
        (1, 2),
        (3, 4),
        (5, 6),
        (7, 8),
        (9, 10),
        (11, 12),
        (13, 14),
        (15, 16),
    ];
    let ids: InstanceIds = arr.into();
    assert_eq!(2, ids[&1]);
}

#[test]
fn test_instance_ids_from_array_ref() {
    let arr: &[i32; 0] = &[];
    let ids: InstanceIds = arr.into();
    assert_eq!(0, ids.len());

    let arr = &[(1, 2)];
    let ids: InstanceIds = arr.into();
    assert_eq!(2, ids[&1]);

    let arr = &[(1, 2), (3, 4)];
    let ids: InstanceIds = arr.into();
    assert_eq!(2, ids[&1]);

    let arr = &[(1, 2), (3, 4), (5, 6)];
    let ids: InstanceIds = arr.into();
    assert_eq!(2, ids[&1]);

    let arr = &[(1, 2), (3, 4), (5, 6), (7, 8)];
    let ids: InstanceIds = arr.into();
    assert_eq!(2, ids[&1]);

    let arr = &[(1, 2), (3, 4), (5, 6), (7, 8), (9, 10)];
    let ids: InstanceIds = arr.into();
    assert_eq!(2, ids[&1]);

    let arr = &[(1, 2), (3, 4), (5, 6), (7, 8), (9, 10), (11, 12)];
    let ids: InstanceIds = arr.into();
    assert_eq!(2, ids[&1]);

    let arr = &[(1, 2), (3, 4), (5, 6), (7, 8), (9, 10), (11, 12), (13, 14)];
    let ids: InstanceIds = arr.into();
    assert_eq!(2, ids[&1]);

    let arr = &[
        (1, 2),
        (3, 4),
        (5, 6),
        (7, 8),
        (9, 10),
        (11, 12),
        (13, 14),
        (15, 16),
    ];
    let ids: InstanceIds = arr.into();
    assert_eq!(2, ids[&1]);
}
