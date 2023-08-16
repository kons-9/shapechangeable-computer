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
