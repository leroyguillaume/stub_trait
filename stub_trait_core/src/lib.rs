//! Core structs used by stub_trait library.

use std::sync::Mutex;

/// Kind of stub.
pub enum StubFnKind<F> {
    /// All calls of the function are stubbed in a same way.
    AllCalls(F),

    /// Each call of the function is stubbed differently.
    CallByCall(Vec<F>),
}

/// Stub of function.
pub struct StubFn<F> {
    /// Number of calls.
    pub count: Mutex<usize>,

    /// Kind of the stub.
    pub kind: StubFnKind<F>,
}
