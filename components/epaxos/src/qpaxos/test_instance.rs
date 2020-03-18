use crate::qpaxos::*;

#[test]
fn test_instance_conflict() {
    let nx = Command::from(("NoOp", "x", "1"));
    let gx = Command::from(("Get", "x", "1"));
    let sx = Command::from(("Set", "x", "1"));

    let ny = Command::from(("NoOp", "y", "1"));
    let gy = Command::from(("Get", "y", "1"));
    let sy = Command::from(("Set", "y", "1"));

    let nxny = Instance::of(&[nx.clone(), ny.clone()], (0, 0, 0).into(), &[]);
    let gxny = Instance::of(&[gx.clone(), ny.clone()], (0, 0, 0).into(), &[]);
    let sxny = Instance::of(&[sx.clone(), ny.clone()], (0, 0, 0).into(), &[]);
    let sxsy = Instance::of(&[sx.clone(), sy.clone()], (0, 0, 0).into(), &[]);
    let gxsy = Instance::of(&[gx.clone(), sy.clone()], (0, 0, 0).into(), &[]);

    assert!(!nxny.conflict(&nxny));
    assert!(!nxny.conflict(&gxny));
    assert!(!nxny.conflict(&sxny));
    assert!(!nxny.conflict(&sxsy));
    assert!(!nxny.conflict(&gxsy));

    assert!(!gxny.conflict(&gxny));
    assert!(gxny.conflict(&sxny));
    assert!(gxny.conflict(&sxsy));
    assert!(!gxny.conflict(&gxsy));

    assert!(sxny.conflict(&sxny));
    assert!(sxny.conflict(&sxsy));
    assert!(sxny.conflict(&gxsy));

    assert!(sxsy.conflict(&sxsy));
    assert!(sxsy.conflict(&gxsy));

    assert!(gxsy.conflict(&gxsy));
}

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
#[should_panic(expect = "NotFound instance_id with replica_id=2")]
fn test_instance_id_vec_index_panic() {
    let ids = InstanceIdVec {
        ids: vec![(1, 2).into(), (3, 4).into()],
    };

    let _ = ids[2];
}

#[test]
fn test_instance_id_vec_get() {
    let ids = InstanceIdVec {
        ids: vec![(1, 2).into(), (3, 4).into()],
    };

    assert_eq!(ids.ids[0], ids.get(1).unwrap());
    assert_eq!(ids.ids[1], ids.get(3).unwrap());
    assert_eq!(None, ids.get(2));
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
