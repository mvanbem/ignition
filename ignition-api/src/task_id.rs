use crate::sys;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct TaskId(u32);

impl TaskId {
    pub const INIT: TaskId = TaskId(!0);

    pub fn new(task_id: u32) -> Self {
        Self(task_id)
    }
}

impl From<sys::TaskId> for TaskId {
    fn from(task_id: sys::TaskId) -> Self {
        Self(task_id.0)
    }
}

impl Into<sys::TaskId> for TaskId {
    fn into(self) -> sys::TaskId {
        sys::TaskId(self.0)
    }
}
