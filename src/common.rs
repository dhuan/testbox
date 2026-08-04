use mlua::prelude::*;
use std::collections::VecDeque;
use std::io::Read;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command};

const ESRCH: i32 = 3;
const SIGKILL: i32 = 9;

pub fn add_func<F, A, R>(lua: &Lua, func_name: &str, func: F)
where
    F: Fn(&Lua, A) -> LuaResult<R> + mlua::MaybeSend + 'static,
    A: FromLuaMulti,
    R: IntoLuaMulti,
{
    lua.globals()
        .set(func_name, lua.create_function(func).unwrap())
        .unwrap();
}

pub fn stdin() -> Option<String> {
    let mut stdin_buffer = String::new();

    std::io::stdin().read_to_string(&mut stdin_buffer).ok()?;

    Some(stdin_buffer)
}

pub fn spawn_background_process(command: &str) -> std::io::Result<Child> {
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .process_group(0)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
}

pub fn kill_processes(list: &mut VecDeque<Child>, print: &dyn Fn(&str)) {
    kill_processes_from(list, 0, print);
}

pub fn kill_processes_from(list: &mut VecDeque<Child>, start: usize, print: &dyn Fn(&str)) {
    while list.len() > start {
        let Some(mut child) = list.pop_back() else {
            break;
        };
        let process_group_id = child.id() as i32;

        print(&format!(
            "Terminating process group {}...",
            process_group_id
        ));

        if child
            .try_wait()
            .expect("Failed to check background process status.")
            .is_some()
        {
            continue;
        }

        if let Err(err) = kill_process_group(process_group_id) {
            if err.raw_os_error() == Some(ESRCH) {
                child.wait().expect("Failed to reap background process.");
                continue;
            }

            if err.kind() == std::io::ErrorKind::PermissionDenied {
                eprintln!(
                    "Failed to kill process group {}: {err}. Trying the child process instead...",
                    process_group_id
                );

                if let Err(child_err) = child.kill() {
                    if child_err.kind() == std::io::ErrorKind::PermissionDenied {
                        eprintln!(
                            "Failed to kill child process {}: {child_err}. Skipping reap.",
                            child.id()
                        );
                        continue;
                    }

                    panic!("Failed to kill child process {}: {child_err}", child.id());
                }
            } else {
                panic!("Failed to kill process group {}: {err}", process_group_id);
            }
        }

        child.wait().expect("Failed to reap background process.");
    }
}

fn kill_process_group(process_group_id: i32) -> std::io::Result<()> {
    let result = unsafe { kill(-process_group_id, SIGKILL) };

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

    #[test]
    fn kill_processes_from_keeps_earlier_processes() {
        let first = spawn_background_process("sleep 1000 >&2").unwrap();
        let first_process_group_id = first.id() as i32;
        let second = spawn_background_process("sleep 1000 >&2").unwrap();
        let second_process_group_id = second.id() as i32;
        let mut list = VecDeque::from([first, second]);

        kill_processes_from(&mut list, 1);
        std::thread::sleep(Duration::from_millis(50));

        assert_eq!(list.len(), 1);
        let second_err = kill_process_group(second_process_group_id).unwrap_err();
        assert_eq!(second_err.raw_os_error(), Some(3));
        kill_process_group(first_process_group_id).unwrap();

        let mut first = list.pop_back().unwrap();
        first.wait().expect("Failed to reap background process.");
    }

    #[test]
    fn kill_processes_reaps_processes_that_exited_before_cleanup() {
        let child = spawn_background_process("true").unwrap();
        let mut list = VecDeque::from([child]);

        std::thread::sleep(Duration::from_millis(50));
        kill_processes(&mut list);

        assert_eq!(list.len(), 0);
    }
}
