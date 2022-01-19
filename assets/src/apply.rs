use crate::{ContractId, MethodName};

struct Applicant {
    contract: ContractId,
    method_name: MethodName
}

struct Application {
    apply_id: String,
    applicant: Applicant,
    msg: String
}