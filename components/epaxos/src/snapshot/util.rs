use super::Error;
use crate::instance::InstanceID;

/// format instance_id to String, maybe implement with protobuf?
pub fn inst_id_to_string(inst_id: &InstanceID) -> String {
    return format!("{}-{}", inst_id.replica_id, inst_id.num);
}

/// format instance_id to String, maybe implement with protobuf?
pub fn inst_id_from_string(str_inst_id: &str) -> Result<InstanceID, Error> {
    let strs = str_inst_id.split("-").collect::<Vec<&str>>();

    if strs.len() < 2 {
        return Err(Error::ParseToInstanceIDError{msg: format!("source string can not to instance, got {}", str_inst_id)});
    }

    let replica_id = strs[0].parse::<i64>()?;
    let num = strs[1].parse::<i64>()?;

    let inst = InstanceID{replica_id: replica_id, num: num};

    return Ok(inst);
}
