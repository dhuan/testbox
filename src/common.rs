use mlua::prelude::*;
use std::collections::VecDeque;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command};

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

pub fn spawn_background_process(command: &str) -> std::io::Result<Child> {
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .process_group(0)
        .spawn()
}

pub fn kill_processes(list: &mut VecDeque<Child>) {
    while let Some(mut child) = list.pop_back() {
        let process_group_id = child.id() as i32;
        eprintln!("Terminating process group {}...", process_group_id);

        if let Err(err) = kill_process_group(process_group_id) {
            if err.raw_os_error() != Some(3) {
                panic!("Failed to kill process group {}: {err}", process_group_id);
            }
        }

        child.wait().expect("Failed to reap background process.");
    }
}

fn kill_process_group(process_group_id: i32) -> std::io::Result<()> {
    let result = unsafe { kill(-process_group_id, 9) };

    if result == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

unsafe extern "C" {
    fn kill(pid: i32, sig: i32) -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn kill_processes_terminates_shell_children_in_the_same_group() {
        let child = spawn_background_process("sleep 1000 >&2").unwrap();
        let process_group_id = child.id() as i32;
        let mut list = VecDeque::from([child]);

        kill_processes(&mut list);
        std::thread::sleep(Duration::from_millis(50));

        let err = kill_process_group(process_group_id).unwrap_err();
        assert_eq!(err.raw_os_error(), Some(3));
    }
}
