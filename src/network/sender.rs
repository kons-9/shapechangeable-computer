use crate::serial::Serial;
use anyhow::Result;

pub trait Sender {
    fn send(&self, serial: &Serial) -> Result<()>;
    fn receive(serial: &Serial) -> Result<Self>
    where
        Self: Sized;
    fn send_broadcast(&self, serial: &Serial) -> Result<()>;
}

// pub struct Sender<'d> {
//     serial: Serial<'d>,
//     send_flit_buffer: Vec<Flit>,
//     receive_flit_buffer: Vec<Flit>,
// }
//
// impl<'d> Sender<'d> {
//     pub fn new(serial: Serial<'d>) -> Self {
//         Sender {
//             serial,
//             send_flit_buffer: Vec::new(),
//             receive_flit_buffer: Vec::new(),
//         }
//     }
//
//     pub fn send(&mut self, flit: Flit) -> Result<()> {
//         flit.send(&mut self.serial)?;
//         Ok(())
//     }
//
//     pub fn receive(&mut self) -> Result<Flit> {
//         let flit = Flit::receive(&mut self.serial)?;
//         Ok(flit)
//     }
// }
