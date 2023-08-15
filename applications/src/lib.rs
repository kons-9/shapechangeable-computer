use anyhow::Result;
pub trait App {
    // interface of network
    type Output;
    fn get_messages(&mut self) -> Vec<u8>;
    fn process_messages(&mut self, messages: Vec<u8>) -> Result<Self::Output>;
}

pub mod image {
    use anyhow::Result;
    use embedded_graphics::geometry::Point;

    pub struct GetImageApp {}
    impl GetImageApp {
        pub fn new() -> Self {
            Self {}
        }
    }
    impl super::App for GetImageApp {
        type Output = (Vec<u8>, u32, Option<Point>);
        fn get_messages(&mut self) -> Vec<u8> {
            unimplemented!()
        }
        fn process_messages(&mut self, messages: Vec<u8>) -> Result<Self::Output> {
            unimplemented!()
        }
    }
}
