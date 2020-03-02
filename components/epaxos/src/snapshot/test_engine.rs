use super::errors::*;
use super::mem_engine::*;
use super::traits::*;
use crate::qpaxos::*;

#[test]
fn test_engine_mem_set_instance() {
    let mut eng = MemEngine::new().unwrap();
    test_set_instance(&mut eng);
}

fn test_set_instance(
    eng: &mut dyn InstanceEngine<ColumnId = ReplicaID, Obj = Instance, ObjId = InstanceID>,
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
    let iid2 = inst.instance_id.unwrap();
    eng.set_instance(&inst).unwrap();
    assert_eq!(iid, eng.get_ref("exec", leader_id).unwrap());

    // exec-ref is not updated

    inst.executed = false;
    inst.instance_id = Some((leader_id, 0).into());
    let iid3 = inst.instance_id.unwrap();
    eng.set_instance(&inst).unwrap();
    assert_eq!(iid, eng.get_ref("exec", leader_id).unwrap());
}

#[test]
fn test_engine_mem_get_instance() {
    let mut eng = MemEngine::new().unwrap();
    test_get_instance(&mut eng);
}

fn test_get_instance(
    eng: &mut dyn InstanceEngine<ColumnId = ReplicaID, Obj = Instance, ObjId = InstanceID>,
) {
    let leader_id = 2;
    let mut inst = new_foo_inst(leader_id);
    let iid = inst.instance_id.unwrap();

    let noninst = eng.get_instance(iid).unwrap();
    assert_eq!(None, noninst);

    eng.set_instance(&inst).unwrap();
    let got = eng.get_instance(iid).unwrap();
    assert_eq!(Some(inst), got);
}

#[test]
fn test_engine_mem_next_instance_id() {
    let mut eng = MemEngine::new().unwrap();
    test_next_instance_id(&mut eng);
}

fn test_next_instance_id(
    eng: &mut dyn InstanceEngine<ColumnId = ReplicaID, Obj = Instance, ObjId = InstanceID>,
) {
    let leader_id = 2;
    let max = (leader_id, 3).into();

    let init = eng.next_instance_id(leader_id).unwrap();
    assert_eq!(InstanceID::from((leader_id, 0)), init);

    let got = eng.next_instance_id(leader_id).unwrap();
    assert_eq!(InstanceID::from((leader_id, 1)), got);

    eng.set_ref("max", leader_id, max).unwrap();

    let got = eng.next_instance_id(leader_id).unwrap();
    assert_eq!(InstanceID::from((leader_id, 4)), got);
}

fn new_foo_inst(leader_id: i64) -> Instance {
    let iid1 = InstanceID::from((1, 10));
    let iid2 = InstanceID::from((2, 20));
    let iid3 = InstanceID::from((3, 30));
    let initial_deps = vec![iid1, iid2, iid3];

    let cmd1 = ("NoOp", "k1", "v1").into();
    let cmd2 = ("Get", "k2", "v2").into();
    let cmds = vec![cmd1, cmd2];
    let ballot = (0, 0, leader_id).into();
    let ballot2 = (1, 2, leader_id).into();

    let mut inst = Instance::of(&cmds[..], &ballot, &initial_deps[..]);
    // TODO move these to Instance::new_instance
    inst.instance_id = Some((leader_id, 1).into());
    inst.deps = [iid2].to_vec();
    inst.final_deps = [iid3].to_vec();
    inst.last_ballot = Some(ballot2);

    inst
}
