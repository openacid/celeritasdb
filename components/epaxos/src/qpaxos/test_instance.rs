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
