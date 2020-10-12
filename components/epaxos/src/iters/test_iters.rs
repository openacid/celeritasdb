use crate::qpaxos::Dep;
use crate::qpaxos::{Command, OpCode};
use crate::*;
use std::sync::Arc;
use storage::Storage;
use storage::{MemEngine, RawKV};

fn new_two_sto() -> (Storage, Storage) {
    let eng = Arc::new(MemEngine::new().unwrap());
    let s1: Storage = Storage::new("1/", eng.clone());
    let s2: Storage = Storage::new("2/", eng);
    (s1, s2)
}

#[test]
fn test_base_iter() {
    let sto: Storage = Storage::new("foo", Arc::new(MemEngine::new().unwrap()));

    let mut ks = vec![];
    let mut vs = vec![];
    for i in 0..100 {
        let k = format!("K{:>03}", i).as_bytes().to_vec();
        let v = format!("V{:?}", i).as_bytes().to_vec();
        sto.set_raw(DBColumnFamily::Record, &k, &v).unwrap();
        ks.push(k);
        vs.push(v);
    }

    let cur = "K000".as_bytes().to_vec();
    let it = sto.get_iter(cur, false, true, DBColumnFamily::Record);
    let r: Vec<_> = it.collect();
    assert_eq!(0, r.len());

    let cur = "K099".as_bytes().to_vec();
    let it = sto.get_iter(cur, false, false, DBColumnFamily::Record);
    let r: Vec<_> = it.collect();
    assert_eq!(0, r.len());

    let cases: Vec<(_, _, _, Box<dyn Fn(usize) -> usize>)> = vec![
        ("K000", true, false, Box::new(|x| x)),
        ("K000", false, false, Box::new(|x| x + 1)),
        ("K099", true, true, Box::new(|x| 100 - x - 1)),
        ("K099", false, true, Box::new(|x| 100 - x - 1 - 1)),
        ("K050", true, false, Box::new(|x| x + 50)),
        ("K050", false, false, Box::new(|x| x + 50 + 1)),
        ("K050", true, true, Box::new(|x| 100 - x - 50)),
        ("K050", false, true, Box::new(|x| 100 - x - 50 - 1)),
    ];
    for (cur, include, reverse, idx) in cases.iter() {
        let cur = cur.as_bytes().to_vec();
        let it = sto.get_iter(cur.clone(), *include, *reverse, DBColumnFamily::Record);
        for (i, kv) in it.enumerate() {
            assert_eq!(ks[idx(i)], kv.0);
            assert_eq!(vs[idx(i)], kv.1);
        }
    }
}

#[test]
fn test_instance_iter() {
    let mut ints = Vec::<Instance>::new();
    let (sto, s2) = new_two_sto();

    for rid in 0..3 {
        for idx in 0..10 {
            let iid = InstanceId::from((rid, idx));

            let cmds = vec![Command::from((
                OpCode::NoOp,
                format!("k1{:}", rid * idx).as_bytes(),
                format!("v1{:}", rid * idx).as_bytes(),
            ))];

            let ballot = (idx as i32, 0).into();
            let depvec = vec![Dep::from((rid + 1, idx + 1))];
            let mut inst = Instance::of(&cmds[..], ballot, &depvec[..]);
            inst.instance_id = Some(iid);

            let _ = sto.set_instance(&inst.instance_id.unwrap(), &inst).unwrap();

            let act = sto.get_instance(&iid).unwrap().unwrap();
            assert_eq!(act.cmds, cmds);

            ints.push(inst);
        }
    }

    let cases: Vec<(InstanceId, bool, &[Instance], &[Instance])> = vec![
        (InstanceId::from((0, 0)), true, &ints[..10], &ints[0..1]),
        (InstanceId::from((0, 0)), false, &ints[1..10], &[]),
        (InstanceId::from((2, 0)), true, &ints[20..30], &ints[20..21]),
        (InstanceId::from((2, 9)), true, &ints[29..30], &ints[20..30]),
        (InstanceId::from((4, 0)), true, &[], &[]),
    ];

    for (start_iid, include, exp_insts, rev_exp_insts) in cases {
        let iter = sto.get_instance_iter(start_iid, include, false);
        let mut exp = vec![];
        exp.extend(exp_insts.iter());
        assert_inst_iter(iter, &exp);

        let iter = sto.get_instance_iter(start_iid, include, true);
        let mut exp = vec![];
        exp.extend(rev_exp_insts.iter().rev());
        assert_inst_iter(iter, &exp);

        // storage is separated by namespace

        let exp = vec![];

        let iter = s2.get_instance_iter(start_iid, include, false);
        assert_inst_iter(iter, &exp);

        let iter = s2.get_instance_iter(start_iid, include, true);
        assert_inst_iter(iter, &exp);
    }
}

fn assert_inst_iter(it: InstanceIter, want: &[&Instance]) {
    let mut n = 0;

    for act_inst in it {
        assert_eq!(act_inst.cmds, want[n].cmds);
        assert_eq!(act_inst.ballot, want[n].ballot);

        assert_eq!(act_inst.instance_id, want[n].instance_id);

        n = n + 1;
    }

    assert_eq!(want.len(), n);
}
