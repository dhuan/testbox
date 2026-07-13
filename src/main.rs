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
    let matches = Command::new("testbox")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Test runner with Lua scripts.")
        .arg(Arg::new("files").action(ArgAction::Append))
        .arg(Arg::new("test-filter").short('t'))
        .arg(Arg::new("test-filter-exact").short('T'))
        .arg(Arg::new("prelude").short('p'))
        .arg(
            Arg::new("fail-fast")
                .long("fail-fast")
                .short('x')
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    let files = matches
        .get_many::<String>("files")
        .unwrap_or_default()
        .map(|v| v.as_str())
        .collect::<Vec<_>>();

    let test_filter = if let Some(value) = matches.get_one::<String>("test-filter") {
        Some(FilterMode::Regex(value.to_owned()))
    } else if let Some(value) = matches.get_one::<String>("test-filter-exact") {
        Some(FilterMode::Exact(value.to_owned()))
    } else {
        None
    };

    if files.len() == 0 {
        eprintln!("No files were given.");

        std::process::exit(1);
    }

    let lua = Lua::new();

    let prelude = if let Some(prelude_file) = matches.get_one::<String>("prelude") {
        Some(std::fs::read_to_string(prelude_file).expect(&format!(
            r#"Failed to load prelude file "{}""#,
            prelude_file
        )))
    } else {
        None
    };

    if let Some(prelude_file) = matches.get_one::<String>("prelude") {
        let prelude_file_content = std::fs::read_to_string(prelude_file).expect(&format!(
            r#"Failed to load prelude file "{}""#,
            prelude_file
        ));

        if let Err(err) = lua.load(prelude_file_content).exec() {
            eprintln!("{}", err.to_string());

            std::process::exit(1);
        }
    }

    let ctx = Rc::new(RefCell::new(LibContext {
        process_list: VecDeque::new(),
        test_filter,
        fail_fast: matches.get_flag("fail-fast"),
        stop_requested: false,
    }));

    add_func(&lua, "expect_equal", expect_equal(ctx.clone()));
    add_func(&lua, "expect_match", expect_match(ctx.clone()));
    add_func(&lua, "exec_bg", exec_bg(ctx.clone()));
    add_func(&lua, "exec", exec(ctx.clone()));
    add_func(&lua, "fetch", fetch(ctx.clone()));
    add_func(&lua, "test", test(ctx.clone()));
    add_func(&lua, "json_encode", json_encode(ctx.clone()));
    add_func(&lua, "json_decode", json_decode(ctx.clone()));
    add_func(&lua, "random_chars", random_chars(ctx.clone()));
    add_func(&lua, "merge", merge_table(ctx.clone()));
    add_func(&lua, "copy", copy_table(ctx.clone()));

    let mut failed = false;
    for test_file in files {
        let script = if test_file == "-" {
            stdin().expect("Failed to read stdin.")
        } else {
            std::fs::read_to_string(test_file).unwrap()
        };

        if let Err(err) = lua
            .load(format!(
                "{}\n{}",
                prelude.clone().unwrap_or_default(),
                script
            ))
            .exec()
        {
            if !ctx.borrow().stop_requested {
                eprintln!("{}", err.to_string());
                failed = true;
            }

            break;
        }

        if ctx.borrow().stop_requested {
            break;
        }
    }

    crate::common::kill_processes(&mut ctx.clone().borrow_mut().process_list);

    if failed {
        std::process::exit(1);
    }
}
