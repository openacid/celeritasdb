use super::traits::*;
use crate::qpaxos::*;
use crate::snapshot::errors::*;

pub fn test_set_instance(
    eng: &mut dyn InstanceEngine<ColumnId = ReplicaID, Obj = Instance, ObjId = InstanceId>,
) {
    let leader_id = 2;
    let mut inst = new_foo_inst(leader_id);
    let iid = inst.instance_id.unwrap();
    eng.set_instance(&inst).unwrap();

    assert_eq!(Err(Error::NotFound), eng.get_ref("exec", leader_id));

    // exec-ref is updated if executed

    inst.executed = true;
    eng.set_instance(&inst).unwrap();
    assert_eq!(iid, eng.get_ref("exec", leader_id).unwrap());

    // exec-ref is not updated

    inst.executed = false;
    inst.instance_id = Some((leader_id, 10).into());
    eng.set_instance(&inst).unwrap();
    assert_eq!(iid, eng.get_ref("exec", leader_id).unwrap());

    // exec-ref is not updated

    inst.executed = false;
    inst.instance_id = Some((leader_id, 0).into());
    eng.set_instance(&inst).unwrap();
    assert_eq!(iid, eng.get_ref("exec", leader_id).unwrap());
}

pub fn test_get_instance(
    eng: &mut dyn InstanceEngine<ColumnId = ReplicaID, Obj = Instance, ObjId = InstanceId>,
) {
    let leader_id = 2;
    let inst = new_foo_inst(leader_id);
    let iid = inst.instance_id.unwrap();

    let noninst = eng.get_instance(iid).unwrap();
    assert_eq!(None, noninst);

    eng.set_instance(&inst).unwrap();
    let got = eng.get_instance(iid).unwrap();
    assert_eq!(Some(inst), got);
}

pub fn test_next_instance_id(
    eng: &mut dyn InstanceEngine<ColumnId = ReplicaID, Obj = Instance, ObjId = InstanceId>,
) {
    let leader_id = 2;
    let max = (leader_id, 3).into();

    let init = eng.next_instance_id(leader_id).unwrap();
    assert_eq!(InstanceId::from((leader_id, 0)), init);

    let got = eng.next_instance_id(leader_id).unwrap();
    assert_eq!(InstanceId::from((leader_id, 1)), got);

    eng.set_ref("max", leader_id, max).unwrap();

    let got = eng.next_instance_id(leader_id).unwrap();
    assert_eq!(InstanceId::from((leader_id, 4)), got);
}

pub fn test_get_instance_iter(
    eng: &mut dyn InstanceEngine<ColumnId = ReplicaID, Obj = Instance, ObjId = InstanceId>,
) {
    let mut ints = Vec::<Instance>::new();

    for rid in 1..4 {
        for idx in 0..10 {
            let iid = InstanceId::from((rid, idx));

            let cmds = vec![Command::of(
                OpCode::NoOp,
                format!("k1{:}", rid * idx).as_bytes(),
                format!("v1{:}", rid * idx).as_bytes(),
            )];

            let ballot = (rid as i32, idx as i32, 0).into();

            let deps = vec![InstanceId::from((rid + 1, idx + 1))];

            let mut inst = Instance::of(&cmds[..], ballot, &deps[..]);
            inst.instance_id = Some(iid);

            eng.set_instance(&inst).unwrap();

            let act = eng.get_obj(iid).unwrap().unwrap();

            assert_eq!(act.cmds, cmds);
            assert_eq!(act.ballot, Some(ballot));

            for (idx, inst_id) in act.initial_deps.unwrap().iter().enumerate() {
                assert_eq!(*inst_id, deps[idx]);
            }

            ints.push(inst);
        }
    }

    let cases = vec![
        ((1, 0).into(), true, &ints[..10], &ints[..1]),
        ((1, 1).into(), true, &ints[1..10], &ints[..2]),
        ((1, 9).into(), true, &ints[9..10], &ints[..10]),
        ((1, 10).into(), true, &[], &ints[..10]),
        ((3, 0).into(), true, &ints[20..30], &ints[20..21]),
        ((0, 0).into(), true, &[], &[]), // before any present instance.
        ((6, 0).into(), true, &[], &[]), // after all present instance.
        ((1, 0).into(), false, &ints[1..10], &[]),
        ((1, 1).into(), false, &ints[2..10], &ints[..1]),
        ((1, 9).into(), false, &[], &ints[..9]),
        ((3, 0).into(), false, &ints[21..30], &[]),
    ];

    for (iid, include, exp_insts, rev_exp_insts) in cases {
        let mut n = 0;
        for act_inst in eng.get_instance_iter(iid, include, false) {
            assert_eq!(act_inst, exp_insts[n]);
            n = n + 1;
        }

        assert_eq!(exp_insts.len(), n);

        n = 0;
        let mut exp = vec![];
        exp.extend(rev_exp_insts.iter().rev());
        for act_inst in eng.get_instance_iter(iid, include, true) {
            assert_eq!(act_inst, *exp[n]);
            n = n + 1;
        }
        assert_eq!(exp.len(), n);
    }
}

fn new_foo_inst(leader_id: i64) -> Instance {
    let iid1 = InstanceId::from((1, 10));
    let iid2 = InstanceId::from((2, 20));
    let iid3 = InstanceId::from((3, 30));
    let initial_deps = vec![iid1, iid2, iid3];

    let cmd1 = ("NoOp", "k1", "v1").into();
    let cmd2 = ("Get", "k2", "v2").into();
    let cmds = vec![cmd1, cmd2];
    let ballot = (0, 0, leader_id).into();
    let ballot2 = (1, 2, leader_id).into();

    let mut inst = Instance::of(&cmds[..], ballot, &initial_deps[..]);
    // TODO move these to Instance::new_instance
    inst.instance_id = Some((leader_id, 1).into());
    inst.deps = Some([iid2].to_vec().into());
    inst.final_deps = Some([iid3].to_vec().into());
    inst.last_ballot = Some(ballot2);

    inst
}

pub fn test_base_trait(eng: &mut dyn Base) {
    let none = eng.next_kv(&"init".as_bytes().to_vec(), true);
    assert_eq!(none, None);

    let prefix = "k".as_bytes().to_vec();
    let kvs = vec![
        ("k0".as_bytes().to_vec(), "v0".as_bytes().to_vec()),
        ("k1".as_bytes().to_vec(), "v1".as_bytes().to_vec()),
        ("k2".as_bytes().to_vec(), "v2".as_bytes().to_vec()),
        ("k3".as_bytes().to_vec(), "v3".as_bytes().to_vec()),
    ];

    eng.set_kv(kvs[0].0.clone(), kvs[0].1.clone()).unwrap();
    let v0 = eng.get_kv(&kvs[0].0).unwrap();
    assert_eq!(v0, kvs[0].1.clone());

    let next0 = eng.next_kv(&prefix, true);
    assert_eq!(next0, Some(kvs[0].clone()));

    for (k, v) in kvs.clone() {
        eng.set_kv(k, v).unwrap();
    }

    let next0 = eng.next_kv(&kvs[0].0, true);
    assert_eq!(next0, Some(kvs[0].clone()));

    let next1 = eng.next_kv(&kvs[0].0, false);
    assert_eq!(next1, Some(kvs[1].clone()));

    let next_last = eng.next_kv(&kvs[3].0.clone(), false);
    assert_eq!(next_last, None);

    let iter = eng.get_iter(kvs[0].0.clone(), true, false);
    for (idx, item) in iter.enumerate() {
        assert_eq!(kvs[idx], item)
    }

    let iter = eng.get_iter(kvs[3].0.clone(), true, true);
    for (idx, item) in iter.enumerate() {
        assert_eq!(kvs[kvs.len() - idx - 1], item)
    }
}

pub fn test_columned_trait(
    eng: &mut dyn ColumnedEngine<ColumnId = ReplicaID, Obj = Instance, ObjId = InstanceId>,
) {
    let cases = vec![(1i64, 2), (2i64, 3)];

    for (rid, idx) in cases {
        let iid = InstanceId::from((rid, idx));

        eng.set_ref("max", rid, iid).unwrap();
        let act = eng.get_ref("max", rid).unwrap();

        assert_eq!(act, iid);

        eng.set_ref("exec", rid, iid).unwrap();
        let act = eng.get_ref("exec", rid).unwrap();

        assert_eq!(act, iid);
    }
}
