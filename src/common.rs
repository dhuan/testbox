use mlua::prelude::*;
use std::collections::VecDeque;
use std::process::Child;

pub fn add_func<F, A, R>(lua: &Lua, func_name: &str, func: F)
where
    F: FnMut(&Lua, A) -> LuaResult<R> + mlua::MaybeSend + 'static,
    A: FromLuaMulti,
    R: IntoLuaMulti,
{
    lua.globals()
        .set(func_name, lua.create_function_mut(func).unwrap())
        .unwrap();
}

pub fn kill_processes(list: &mut VecDeque<Child>) {
    while let Some(mut process) = list.pop_back() {
        eprintln!("Terminating {}...", process.id());
        process.kill().expect("Failed to kill process.");
    }
}
