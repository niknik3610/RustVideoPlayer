pub trait Consumer<T> {
    fn consume(&self, to_consume: T);
}
