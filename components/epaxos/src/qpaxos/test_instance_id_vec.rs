use crate::qpaxos::InstanceId;
use crate::qpaxos::InstanceIdVec;
pub use std::cmp::Ordering;

#[test]
fn test_instance_id_vec_deref() {
    let ids = InstanceIdVec {
        ids: vec![(1, 2).into(), (3, 4).into()],
    };

    let mut it = ids.iter();
    assert_eq!(&ids.ids[0], it.next().unwrap());
    assert_eq!(&ids.ids[1], it.next().unwrap());
    assert_eq!(None, it.next());

    let mut ids = InstanceIdVec {
        ids: vec![(1, 2).into(), (3, 4).into()],
    };

    let mut it = ids.iter_mut();
    assert_eq!(&InstanceId::from((1, 2)), it.next().unwrap());
    assert_eq!(&InstanceId::from((3, 4)), it.next().unwrap());
    assert_eq!(None, it.next());
}

#[test]
fn test_instance_id_vec_index() {
    let ids = InstanceIdVec {
        ids: vec![(1, 2).into(), (3, 4).into()],
    };

    assert_eq!(ids.ids[0], ids[1]);
    assert_eq!(ids.ids[1], ids[3]);
}

#[test]
#[should_panic(expected = "NotFound instance_id with replica_id=2")]
fn test_instance_id_vec_index_panic() {
    let ids = InstanceIdVec {
        ids: vec![(1, 2).into(), (3, 4).into()],
    };

    let _ = ids[2];
}

#[test]
fn test_instance_id_vec_cmd_inst() {
    let id12 = InstanceId::from((1, 2));
    let id34 = InstanceId::from((3, 4));

    let ids = InstanceIdVec {
        ids: vec![id12, id34],
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
fn test_instance_id_vec_get() {
    let ids = InstanceIdVec {
        ids: vec![(1, 2).into(), (3, 4).into()],
    };

    assert_eq!(ids.ids[0], ids.get(1).unwrap());
    assert_eq!(ids.ids[1], ids.get(3).unwrap());
    assert_eq!(None, ids.get(2));

    let refids = &ids;
    assert_eq!(ids.ids[0], refids.get(1).unwrap());
    assert_eq!(ids.ids[1], refids.get(3).unwrap());
    assert_eq!(None, ids.get(2));

    let sm = Some(ids.clone());
    let refids = sm.as_ref().unwrap();

    assert_eq!(ids.ids[0], refids.get(1i64).unwrap());
    assert_eq!(ids.ids[1], refids.get(3).unwrap());
    assert_eq!(None, refids.get(2));
}

#[test]
fn test_instance_id_vec_set() {
    let id01 = InstanceId::from((0, 1));
    let id12 = InstanceId::from((1, 2));
    let id13 = InstanceId::from((1, 3));
    let id34 = InstanceId::from((3, 4));
    let id56 = InstanceId::from((5, 6));

    let mut ids = InstanceIdVec {
        ids: vec![id12, id34],
    };

    let r = ids.set((1, 3).into());
    assert_eq!((0, Some(id12)), r);
    assert_eq!(id13, ids.get(1).unwrap());

    // set a same instanceId twice
    let r = ids.set((1, 3).into());
    assert_eq!((0, Some(id13)), r);
    assert_eq!(id13, ids.get(1).unwrap());

    // appended
    let r = ids.set((0, 1).into());
    assert_eq!((2, None), r);
    assert_eq!(id01, ids.get(0).unwrap());

    // appended
    let r = ids.set((5, 6).into());
    assert_eq!((3, None), r);
    assert_eq!(id56, ids.get(5).unwrap());
}

#[test]
fn test_instance_id_vec_with_dup() {
    let ids = InstanceIdVec {
        ids: vec![(1, 2).into(), (3, 4).into(), (1, 100).into()],
    };

    assert_eq!(ids.ids[0], ids.get(1).unwrap());
    assert_eq!(ids.ids[1], ids.get(3).unwrap());
    assert_eq!(None, ids.get(2));

    assert_eq!(ids.ids[0], ids[1]);
    assert_eq!(ids.ids[1], ids[3]);
}

#[test]
fn test_instance_id_vec_from() {
    let iid = InstanceId::from((1, 2));

    let sl: &[_] = &[iid];
    let ids: InstanceIdVec = sl.into();
    assert_eq!(iid, ids[1]);

    let ids: InstanceIdVec = vec![iid].into();
    assert_eq!(iid, ids[1]);

    let sl: &[_] = &[(1, 2), (3, 4)];
    let ids: InstanceIdVec = sl.into();
    assert_eq!(iid, ids[1]);

    let sl: &[(i32, i64)] = &[(1, 2), (3, 4)];
    let ids: InstanceIdVec = sl.into();
    assert_eq!(iid, ids[1]);
}

#[test]
fn test_instance_id_vec_from_array() {
    let iid = InstanceId::from((1, 2));

    let arr: [i32; 0] = [];
    let ids: InstanceIdVec = arr.into();
    assert_eq!(0, ids.len());

    let arr = [(1, 2)];
    let ids: InstanceIdVec = arr.into();
    assert_eq!(iid, ids[1]);

    let arr = [(1, 2), (3, 4)];
    let ids: InstanceIdVec = arr.into();
    assert_eq!(iid, ids[1]);

    let arr = [(1, 2), (3, 4), (5, 6)];
    let ids: InstanceIdVec = arr.into();
    assert_eq!(iid, ids[1]);

    let arr = [(1, 2), (3, 4), (5, 6), (7, 8)];
    let ids: InstanceIdVec = arr.into();
    assert_eq!(iid, ids[1]);

    let arr = [(1, 2), (3, 4), (5, 6), (7, 8), (9, 10)];
    let ids: InstanceIdVec = arr.into();
    assert_eq!(iid, ids[1]);

    let arr = [(1, 2), (3, 4), (5, 6), (7, 8), (9, 10), (11, 12)];
    let ids: InstanceIdVec = arr.into();
    assert_eq!(iid, ids[1]);

    let arr = [(1, 2), (3, 4), (5, 6), (7, 8), (9, 10), (11, 12), (13, 14)];
    let ids: InstanceIdVec = arr.into();
    assert_eq!(iid, ids[1]);

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
    let ids: InstanceIdVec = arr.into();
    assert_eq!(iid, ids[1]);
}

#[test]
fn test_instance_id_vec_from_array_ref() {
    let iid = InstanceId::from((1, 2));

    let arr: &[i32; 0] = &[];
    let ids: InstanceIdVec = arr.into();
    assert_eq!(0, ids.len());

    let arr = &[(1, 2)];
    let ids: InstanceIdVec = arr.into();
    assert_eq!(iid, ids[1]);

    let arr = &[(1, 2), (3, 4)];
    let ids: InstanceIdVec = arr.into();
    assert_eq!(iid, ids[1]);

    let arr = &[(1, 2), (3, 4), (5, 6)];
    let ids: InstanceIdVec = arr.into();
    assert_eq!(iid, ids[1]);

    let arr = &[(1, 2), (3, 4), (5, 6), (7, 8)];
    let ids: InstanceIdVec = arr.into();
    assert_eq!(iid, ids[1]);

    let arr = &[(1, 2), (3, 4), (5, 6), (7, 8), (9, 10)];
    let ids: InstanceIdVec = arr.into();
    assert_eq!(iid, ids[1]);

    let arr = &[(1, 2), (3, 4), (5, 6), (7, 8), (9, 10), (11, 12)];
    let ids: InstanceIdVec = arr.into();
    assert_eq!(iid, ids[1]);

    let arr = &[(1, 2), (3, 4), (5, 6), (7, 8), (9, 10), (11, 12), (13, 14)];
    let ids: InstanceIdVec = arr.into();
    assert_eq!(iid, ids[1]);

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
    let ids: InstanceIdVec = arr.into();
    assert_eq!(iid, ids[1]);
}
