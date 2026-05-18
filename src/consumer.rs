pub trait Consumer<T> {
    fn consume(&mut self, to_consume: T);
}
