use core::convert::{TryFrom, TryInto};
use core::num::TryFromIntError;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct TaskId(u32);

impl TaskId {
    pub const INIT: TaskId = TaskId(!0);

    pub fn new(task_id: u32) -> Self {
        Self(task_id)
    }

    pub fn as_u32(self) -> u32 {
        self.0
    }
}

impl TryFrom<usize> for TaskId {
    type Error = TryFromIntError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Ok(Self(value.try_into()?))
    }
}

impl From<TaskId> for usize {
    fn from(task_id: TaskId) -> Self {
        task_id.0 as usize
    }
}
