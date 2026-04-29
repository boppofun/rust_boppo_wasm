#[derive(Clone, Debug)]
pub enum AudioEvent {
    Opened { req_id: i32, handle: i32 },
    Finished(i32),
    BadHandleError(i32),
}
