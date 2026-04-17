use clap::{Arg, ArgAction, Command};
use mlua::prelude::*;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

mod common;
mod script_lib;

use common::*;
use script_lib::*;

fn main() {
    let matches = Command::new("MyApp")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Test runner with Lua scripts.")
        .arg(Arg::new("files").action(ArgAction::Append))
        .get_matches();

    let files = matches
        .get_many::<String>("files")
        .unwrap_or_default()
        .map(|v| v.as_str())
        .collect::<Vec<_>>();

    if files.len() == 0 {
        eprintln!("No files were given.");

        std::process::exit(1);
    }

    let lua = Lua::new();

    let ctx = Rc::new(RefCell::new(LibContext {
        process_list: VecDeque::new(),
    }));

    add_func(&lua, "expect_equal", expect_equal(ctx.clone()));
    add_func(&lua, "exec_bg", exec_bg(ctx.clone()));
    add_func(&lua, "fetch", fetch(ctx.clone()));
    add_func(&lua, "test", test(ctx.clone()));
    add_func(&lua, "json_encode", json_encode(ctx.clone()));

    for test_file in files {
        lua.load(std::fs::read_to_string(test_file).unwrap())
            .exec()
            .unwrap();
    }

    crate::common::kill_processes(&mut ctx.clone().borrow_mut().process_list);
}
