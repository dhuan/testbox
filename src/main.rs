use mlua::prelude::*;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

mod common;
mod script_lib;

use common::*;
use script_lib::*;

fn main() {
    let test_files = std::env::args().collect::<Vec<String>>().to_vec()[1..].to_vec();

    if test_files.len() == 0 {
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
    add_func(&lua, "to_json", to_json(ctx.clone()));

    for test_file in test_files {
        lua.load(std::fs::read_to_string(test_file).unwrap())
            .exec()
            .unwrap();
    }

    crate::common::kill_processes(&mut ctx.clone().borrow_mut().process_list);
}
