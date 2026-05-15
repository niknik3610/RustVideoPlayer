use std::sync::mpsc::{Sender};

pub trait Producer<T> {
    fn produce(&self); 
    fn set_channel(&mut self, channel: Sender<T>);
}
