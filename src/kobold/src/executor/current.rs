use std::option::IntoIter as OptionIter;

use super::Task;

pub struct Current;

impl Current {
    #[must_use]
    pub fn dispatch(&self, mut task: Task) -> OptionIter<Task> {
        task.process();
        Some(task).into_iter()
    }
}
