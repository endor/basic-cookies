#[macro_use]
pub(crate) extern crate lalrpop_util;

pub mod from_user_agent;
mod linked_list;

pub(crate) use self::linked_list::LinkedList;
