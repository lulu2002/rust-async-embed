use core::pin::Pin;

pub trait Task {
    async fn run(&mut self) -> Pin<impl Future<Output = ()>>;
}
