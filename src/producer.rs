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
    pub fn produce(&mut self, product: T) {
        // if self.consumers.len() == 1 {
        //     self.consumers[0].consume(product);
        // }
        for consumer in self.consumers.iter_mut() {
            consumer.consume(product.clone());
        }
    }
    pub fn add_consumer(&mut self, consumer: Box<dyn Consumer<T>>) {
        self.consumers.push(consumer);
    }
}

