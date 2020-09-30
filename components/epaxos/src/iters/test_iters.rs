use crate::qpaxos::Dep;
use crate::qpaxos::{Command, OpCode};
use crate::*;
use std::sync::Arc;
use storage::MemEngine;

#[test]
fn test_base_iter() {
    let sto: Storage = Arc::new(MemEngine::new().unwrap());

    let mut ks = vec![];
    let mut vs = vec![];
    for i in 0..100 {
        let k = format!("K{:>03}", i).as_bytes().to_vec();
        let v = format!("V{:?}", i).as_bytes().to_vec();
        sto.set(DBColumnFamily::KV, &k, &v).unwrap();
        ks.push(k);
        vs.push(v);
    }

    let cur = "K000".as_bytes().to_vec();
    let it = sto.get_iter(cur, false, true, DBColumnFamily::KV);
    let r: Vec<_> = it.collect();
    assert_eq!(0, r.len());

    let cur = "K099".as_bytes().to_vec();
    let it = sto.get_iter(cur, false, false, DBColumnFamily::KV);
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
        let it = sto.get_iter(cur.clone(), *include, *reverse, DBColumnFamily::KV);
        for (i, kv) in it.enumerate() {
            assert_eq!(ks[idx(i)], kv.0);
            assert_eq!(vs[idx(i)], kv.1);
        }
    }
}

#[test]
fn test_instance_iter() {
    let mut ints = Vec::<Instance>::new();
    let sto: Storage = Arc::new(MemEngine::new().unwrap());

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

            let _ = sto.set_instance(&inst).unwrap();

            let act = sto.get_instance(iid).unwrap().unwrap();
            assert_eq!(act.cmds, cmds);

            ints.push(inst);
        }
    }

    let cases = vec![
        (InstanceId::from((0, 0)), true, &ints[..10], &ints[0..1]),
        (InstanceId::from((0, 0)), false, &ints[1..10], &[]),
        (InstanceId::from((2, 0)), true, &ints[20..30], &ints[20..21]),
        (InstanceId::from((2, 9)), true, &ints[29..30], &ints[20..30]),
        (InstanceId::from((4, 0)), true, &[], &[]),
    ];

    for (start_iid, include, exp_insts, rev_exp_insts) in cases {
        let mut n = 0;

        let iter = sto.get_instance_iter(start_iid, include, false);

        for act_inst in iter {
            assert_eq!(act_inst.cmds, exp_insts[n].cmds);
            assert_eq!(act_inst.ballot, exp_insts[n].ballot);

            assert_eq!(act_inst.instance_id, exp_insts[n].instance_id);

            n = n + 1;
        }

        assert_eq!(exp_insts.len(), n);

        n = 0;
        let iter = sto.get_instance_iter(start_iid, include, true);

        let mut exp = vec![];
        exp.extend(rev_exp_insts.iter().rev());
        for act_inst in iter {
            assert_eq!(act_inst.cmds, exp[n].cmds);
            assert_eq!(act_inst.ballot, exp[n].ballot);

            assert_eq!(act_inst.instance_id, exp[n].instance_id);

            n = n + 1;
        }

        assert_eq!(exp.len(), n);
    }
}
