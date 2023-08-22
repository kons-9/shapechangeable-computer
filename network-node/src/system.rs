use crate::utils::type_alias::Id;
pub trait SystemInfo {
    fn get_system_info(&self) -> Id;
}

#[cfg(test)]
pub mod test {
    use super::*;

    pub struct TestSystemInfo {
        id: Id,
    }

    impl TestSystemInfo {
        pub fn new(id: Id) -> Self {
            Self { id }
        }
    }
    impl super::SystemInfo for TestSystemInfo {
        fn get_system_info(&self) -> Id {
            self.id
        }
    }
}
