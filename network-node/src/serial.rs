use anyhow::Result;

pub trait SerialTrait {
    fn send(&mut self, data: &[u8; 8]) -> Result<()>;
    fn receive(&mut self) -> Result<Option<[u8; 8]>>;
    fn flush_read(&mut self) -> Result<()>;
    fn flush_write(&mut self) -> Result<()>;
    fn flush_all(&mut self) -> Result<()> {
        self.flush_read()?;
        self.flush_write()?;
        Ok(())
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    pub struct TestSerial {
        pub data: Vec<[u8; 8]>,
    }

    impl TestSerial {
        pub fn new() -> Self {
            Self { data: Vec::new() }
        }
    }

    impl SerialTrait for TestSerial {
        fn send(&mut self, data: &[u8; 8]) -> Result<()> {
            self.data.push(*data);
            Ok(())
        }
        fn receive(&mut self) -> Result<Option<[u8; 8]>> {
            Ok(self.data.pop())
        }
        fn flush_read(&mut self) -> Result<()> {
            self.data.clear();
            Ok(())
        }
        fn flush_write(&mut self) -> Result<()> {
            Ok(())
        }
    }
}
