use std::marker::PhantomData;
use crate::consumer::Consumer;

pub struct Producer<T, C: Consumer<T>> {
    consumers: Vec<C>,
    _marker: PhantomData<T>,
}

impl <T, C: Consumer<T>> Producer<T, C> {
    fn produce(&self, product: T) {
        for consumer in self.consumers.iter() {
            consumer.consume(&product);
        }
    }
}

