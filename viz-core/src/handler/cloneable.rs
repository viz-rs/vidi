use super::Handler;

pub(crate) type BoxCloneable<Input, Output> = Box<dyn Cloneable<Input, Output = Output> + Send>;

pub(crate) trait Cloneable<Input>: Handler<Input> {
    fn clone_box(&self) -> BoxCloneable<Input, Self::Output>;
}

impl<Input, T> Cloneable<Input> for T
where
    T: Handler<Input> + Send + Clone + 'static,
{
    fn clone_box(&self) -> BoxCloneable<Input, Self::Output> {
        Box::new(self.clone())
    }
}
