#[derive(Clone, Debug)]
pub enum AudioEvent {
    Finished(i32),
    BadHandleError(i32),
}
