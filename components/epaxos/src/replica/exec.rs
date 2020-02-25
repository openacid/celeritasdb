use crate::instance::Instance;
use crate::instance::InstanceID;
use crate::replica::Replica;

#[cfg(test)]
#[path = "./tests/exec_tests.rs"]
mod tests;

impl<Engine> Replica<Engine> {
    // R1          R2
    // -------------
    // |           |
    // d(NotFound) |
    // | ↖ ........b(Committed)
    // a(Executed) |
    //
    // instances((a, d]) not exist in this replica, recover instance(a+1) first
    fn find_missing_insts(
        &self,
        min_insts: &Vec<Instance>,
        exec_up_to: &Vec<InstanceID>,
    ) -> Option<Vec<InstanceID>> {
        let mut rst = Vec::new();

        for inst in min_insts {
            for dep_inst_id in inst.final_deps.iter() {
                if let Some(_) = min_insts.iter().find(|x| match &x.instance_id {
                    Some(iid) => return iid.replica_id == dep_inst_id.replica_id,
                    None => return false,
                }) {
                    continue;
                }

                if let Some(iid) = exec_up_to
                    .iter()
                    .find(|&x| x.replica_id == dep_inst_id.replica_id)
                {
                    if dep_inst_id.idx > iid.idx {
                        rst.push(InstanceID::of(iid.replica_id, iid.idx + 1));
                    }
                }
            }
        }

        if rst.len() > 0 {
            return Some(rst);
        }

        None
    }
}
