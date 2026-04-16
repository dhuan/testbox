use mlua::prelude::*;

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
