use crate::consumer::Consumer;

pub struct Producer<T: Clone> {
    //Pointer to dynamically sized (due to just being an interface) object which implements Interface Consumer
    consumers: Vec<Box<dyn Consumer<T>>>,
}

impl <T:Clone> Producer<T> {
    pub fn new() -> Self {
        return Self {
            consumers: vec![],
        }
    }
    pub fn produce(&self, product: T) {
        for consumer in self.consumers.iter() {
            consumer.consume(product.clone());
        }
    }
    pub fn add_consumer(&mut self, consumer: Box<dyn Consumer<T>>) {
        self.consumers.push(consumer);
    }
}

