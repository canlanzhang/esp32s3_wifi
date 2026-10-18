use std::sync::mpsc::{self, Sender};

#[derive(Debug, Clone, Copy)]
pub struct IrCommand {
    pub address: u8,
    pub command: u8,
}

pub type IrSender = Sender<IrCommand>;

pub fn channel() -> (IrSender, mpsc::Receiver<IrCommand>) {
    mpsc::channel()
}