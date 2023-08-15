use crate::utils::type_alias::Id;
pub trait SystemInfo {
    fn get_system_info(&self) -> Id;
}
