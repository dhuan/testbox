use mlua::prelude::{LuaSerdeExt, *};
use rand::distr::{Alphanumeric, SampleString};
use reqwest::header::HeaderMap;
use serde_json::Value;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::os::raw::c_void;
use std::rc::Rc;
use std::{io::prelude::*, sync::mpsc, time::Duration};

struct FetchOptions {
    url: String,
    method: reqwest::Method,
    body: Option<String>,
    headers: Option<reqwest::header::HeaderMap>,
}

pub enum FilterMode {
    Regex(String),
    Exact(String),
}

pub struct LibContext {
    pub process_list: VecDeque<std::process::Child>,
    pub test_filter: Option<FilterMode>,
    pub fail_fast: bool,
    pub stop_requested: bool,
    pub test_file_name: String,
    pub test_file_header_printed: bool,
    pub any_test_file_header_printed: bool,
}

impl LibContext {
    pub fn set_test_file_name(&mut self, test_file_name: String) {
        self.test_file_name = test_file_name;
        self.test_file_header_printed = false;
    }

    fn print_test_file_header(&mut self) {
        if self.test_file_header_printed {
            return;
        }

        if self.any_test_file_header_printed {
            println!();
        }

        println!("📁 {}", self.test_file_name);
        self.test_file_header_printed = true;
        self.any_test_file_header_printed = true;
    }
}

struct ExecBgOptions {
    wait: Option<LuaFunction>,
}

enum WaitProcessError {
    IoError(std::io::Error),
    LuaError(LuaError),
    TimedOut { stdout: Vec<u8>, stderr: Vec<u8> },
}

enum ProcessOutput {
    Stdout(Vec<u8>),
    Stderr(Vec<u8>),
}

const WAIT_TIMEOUT: Duration = Duration::from_secs(5);
const READ_POLL_INTERVAL: Duration = Duration::from_millis(100);

fn wait_process_until<F>(
    child: &mut std::process::Child,
    f: F,
) -> Result<(Option<String>, Option<String>), WaitProcessError>
where
    F: Fn(&[u8], &[u8]) -> LuaResult<bool>,
{
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    if stdout.is_none() && stderr.is_none() {
        return Ok((None, None));
    }

    let (sender, receiver) = mpsc::channel();

    if let Some(mut stdout) = stdout {
        let sender = sender.clone();
        std::thread::spawn(move || {
            let mut buffer = [0u8; 1024];
            let mut send_output = true;

            loop {
                match stdout.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        if send_output
                            && sender
                                .send(Ok(ProcessOutput::Stdout(buffer[..n].to_vec())))
                                .is_err()
                        {
                            send_output = false;
                        }
                    }
                    Err(err) => {
                        if send_output {
                            let _ = sender.send(Err(err));
                        }
                        break;
                    }
                }
            }
        });
    }

    if let Some(mut stderr) = stderr {
        let sender = sender.clone();
        std::thread::spawn(move || {
            let mut buffer = [0u8; 1024];
            let mut send_output = true;

            loop {
                match stderr.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        if send_output
                            && sender
                                .send(Ok(ProcessOutput::Stderr(buffer[..n].to_vec())))
                                .is_err()
                        {
                            send_output = false;
                        }
                    }
                    Err(err) => {
                        if send_output {
                            let _ = sender.send(Err(err));
                        }
                        break;
                    }
                }
            }
        });
    }
    drop(sender);

    let mut stdout_result: Vec<u8> = vec![];
    let mut stderr_result: Vec<u8> = vec![];
    let started_at = std::time::Instant::now();

    loop {
        if started_at.elapsed() >= WAIT_TIMEOUT {
            return Err(WaitProcessError::TimedOut {
                stdout: stdout_result,
                stderr: stderr_result,
            });
        };

        match receiver.recv_timeout(READ_POLL_INTERVAL) {
            Ok(Ok(chunk)) => {
                match chunk {
                    ProcessOutput::Stdout(chunk) => stdout_result.extend(&chunk),
                    ProcessOutput::Stderr(chunk) => stderr_result.extend(&chunk),
                }

                if f(&stdout_result, &stderr_result).map_err(WaitProcessError::LuaError)? {
                    return Ok(to_exec_bg_result(stdout_result, stderr_result));
                }
            }
            Ok(Err(err)) => return Err(WaitProcessError::IoError(err)),
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if f(&stdout_result, &stderr_result).map_err(WaitProcessError::LuaError)? {
                    return Ok(to_exec_bg_result(stdout_result, stderr_result));
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err(WaitProcessError::TimedOut {
                    stdout: stdout_result,
                    stderr: stderr_result,
                });
            }
        }
    }
}

fn to_exec_bg_result(stdout: Vec<u8>, stderr: Vec<u8>) -> (Option<String>, Option<String>) {
    (
        Some(String::from_utf8(stdout).unwrap_or_default()),
        Some(String::from_utf8(stderr).unwrap_or_default()),
    )
}

pub fn exec_bg(
    ctx: Rc<RefCell<LibContext>>,
) -> impl Fn(&Lua, (String, Option<LuaTable>)) -> LuaResult<LuaTable> {
    move |lua, (command, options)| {
        eprintln!("Executing command: {}", command);

        let options = options
            .map(|options| ExecBgOptions {
                wait: options.get("wait").ok(),
            })
            .unwrap_or(ExecBgOptions { wait: None });

        let mut process = crate::common::spawn_background_process(&command).unwrap();

        let result: Result<(Option<String>, Option<String>), LuaError> = if let Some(wait) =
            options.wait
        {
            match wait_process_until(&mut process, |stdout, stderr| {
                let stdout_str = String::from_utf8_lossy(stdout).to_string();
                let stderr_str = String::from_utf8_lossy(stderr).to_string();

                wait.call::<bool>((stdout_str, stderr_str))
            }) {
                Err(WaitProcessError::IoError(err)) => Err(LuaError::RuntimeError(err.to_string())),
                Err(WaitProcessError::LuaError(err)) => Err(err),
                Err(WaitProcessError::TimedOut { stdout, stderr }) => {
                    Err(LuaError::RuntimeError(format!(
                        "Timed out: stdout: {}; stderr: {}",
                        String::from_utf8_lossy(&stdout),
                        String::from_utf8_lossy(&stderr)
                    )))
                }
                Ok((stdout, stderr)) => Ok((stdout, stderr)),
            }
        } else {
            Ok((None, None))
        };

        ctx.borrow_mut().process_list.push_back(process);

        let (stdout, stderr) = match result {
            Err(err) => return Err(err),
            Ok((stdout, stderr)) => (stdout.unwrap_or_default(), stderr.unwrap_or_default()),
        };

        let result_lua = lua.create_table().unwrap();
        result_lua.set("stdout", stdout).unwrap();
        result_lua.set("stderr", stderr).unwrap();

        LuaResult::Ok(result_lua)
    }
}

struct ExecResult {
    stdout: Option<String>,
    stderr: Option<String>,
    error: Option<String>,
    status: Option<i32>,
}

struct ExecOptions {
    stdin: Option<String>,
}

pub fn exec(
    _ctx: Rc<RefCell<LibContext>>,
) -> impl Fn(&Lua, (String, Option<LuaTable>)) -> LuaResult<LuaTable> {
    move |lua, (command, options)| {
        eprintln!("Executing command: {}", command);

        let options = options
            .map(|options| ExecOptions {
                stdin: options.get("stdin").ok(),
            })
            .unwrap_or(ExecOptions { stdin: None });

        let mut command_builder = std::process::Command::new("sh");
        command_builder.arg("-c").arg(command);

        if options.stdin.is_some() {
            command_builder.stdin(std::process::Stdio::piped());
        }

        let result = match command_builder.output_with_stdin(options.stdin) {
            Err(err) => ExecResult {
                stdout: Some("".to_string()),
                stderr: Some("".to_string()),
                error: Some(err.to_string()),
                status: None,
            },
            Ok(result) => ExecResult {
                stdout: Some(
                    String::from_utf8(result.stdout)
                        .unwrap_or_default()
                        .trim_end()
                        .to_string(),
                ),
                stderr: Some(
                    String::from_utf8(result.stderr)
                        .unwrap_or_default()
                        .trim_end()
                        .to_string(),
                ),
                error: None,
                status: result.status.code(),
            },
        };

        let result_lua = lua.create_table().unwrap();
        result_lua
            .set("stdout", result.stdout.unwrap_or_default())
            .unwrap();
        result_lua
            .set("stderr", result.stderr.unwrap_or_default())
            .unwrap();
        result_lua
            .set(
                "error",
                result.error.map(|err| lua.create_string(err).unwrap()),
            )
            .unwrap();
        result_lua.set("status", result.status).unwrap();

        LuaResult::Ok(result_lua)
    }
}

trait OutputWithStdin {
    fn output_with_stdin(&mut self, stdin: Option<String>)
    -> std::io::Result<std::process::Output>;
}

impl OutputWithStdin for std::process::Command {
    fn output_with_stdin(
        &mut self,
        stdin: Option<String>,
    ) -> std::io::Result<std::process::Output> {
        if let Some(stdin) = stdin {
            self.stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped());

            let mut child = self.spawn()?;
            let stdin_writer = child.stdin.take().map(|mut child_stdin| {
                std::thread::spawn(move || match child_stdin.write_all(stdin.as_bytes()) {
                    Ok(_) => Ok(()),
                    Err(err) if err.kind() == std::io::ErrorKind::BrokenPipe => Ok(()),
                    Err(err) => Err(err),
                })
            });

            let output = child.wait_with_output()?;

            if let Some(stdin_writer) = stdin_writer {
                match stdin_writer.join() {
                    Ok(Ok(())) => {}
                    Ok(Err(err)) => return Err(err),
                    Err(_) => return Err(std::io::Error::other("stdin writer thread panicked")),
                }
            }

            Ok(output)
        } else {
            self.output()
        }
    }
}

fn test_match(filter: &FilterMode, test_name: &str) -> bool {
    match filter {
        FilterMode::Exact(value) => value.to_lowercase() == test_name.to_lowercase(),
        FilterMode::Regex(value) => {
            if let Ok(r) = regex::Regex::new(&value) {
                r.is_match(test_name)
            } else {
                false
            }
        }
    }
}

pub fn test(ctx: Rc<RefCell<LibContext>>) -> impl Fn(&Lua, (String, LuaFunction)) -> LuaResult<()> {
    move |_lua, (test_name, func)| {
        if ctx.borrow().stop_requested {
            return Ok(());
        }

        if let Some(test_filter) = &ctx.clone().borrow().test_filter {
            if !test_match(&test_filter, &test_name) {
                return Ok(());
            }
        }

        ctx.borrow_mut().print_test_file_header();

        if let Err(err) = func.call::<()>(Some(123)) {
            println!(
                "❌ {}\n{}",
                test_name,
                match err.clone() {
                    LuaError::CallbackError {
                        cause,
                        traceback: _,
                    } =>
                        if let LuaError::RuntimeError(err) = cause.clone().as_ref() {
                            err.to_owned()
                        } else {
                            err.to_string()
                        },
                    _ => err.to_string(),
                },
            );

            let mut ctx = ctx.borrow_mut();
            if ctx.fail_fast {
                ctx.stop_requested = true;
                return Err(LuaError::RuntimeError("fail-fast requested".to_string()));
            }
        } else {
            println!("✅ {}", test_name);
        }

        crate::common::kill_processes(&mut ctx.clone().borrow_mut().process_list);

        Ok(())
    }
}

pub fn fetch(_ctx: Rc<RefCell<LibContext>>) -> impl Fn(&Lua, LuaValue) -> LuaResult<LuaTable> {
    move |lua, value| {
        let options = serde_json::to_value(value)
            .unwrap()
            .as_object()
            .ok_or(LuaError::RuntimeError("Invalid fetch options.".to_string()))?
            .clone();

        let fetch_options = FetchOptions {
            url: options
                .get("url")
                .ok_or(LuaError::RuntimeError("Missing URL".to_string()))?
                .as_str()
                .ok_or(LuaError::RuntimeError(
                    "Wrong format for URL field".to_string(),
                ))?
                .to_string(),
            method: reqwest::Method::try_from(
                options
                    .get("method")
                    .unwrap_or(&Value::String("get".to_string()))
                    .as_str()
                    .ok_or(LuaError::RuntimeError(
                        "Failed to decode method".to_string(),
                    ))?
                    .to_uppercase()
                    .as_str(),
            )
            .unwrap(),
            body: if let Some(body) = options.get("body") {
                Some(
                    body.as_str()
                        .ok_or(LuaError::RuntimeError("Body must be string".to_string()))?
                        .to_string(),
                )
            } else {
                None
            },
            headers: if let Some(headers) = options.get("headers") {
                if let Some(headers) = headers.as_object() {
                    let mut hm = HeaderMap::new();
                    for (key, value) in headers {
                        let value = value.as_str();
                        if value.is_none() {
                            continue;
                        }

                        hm.insert(
                            reqwest::header::HeaderName::from_bytes(key.as_bytes()).unwrap(),
                            reqwest::header::HeaderValue::from_str(value.unwrap()).unwrap(),
                        );
                    }

                    Some(hm)
                } else {
                    None
                }
            } else {
                None
            },
        };

        let mut request = reqwest::blocking::Request::new(
            fetch_options.method,
            url::Url::parse(&fetch_options.url).unwrap(),
        );

        if let Some(body) = fetch_options.body {
            let body_set = request.body_mut();
            *body_set = Some(reqwest::blocking::Body::from(body));
        }

        if let Some(headers) = fetch_options.headers {
            *request.headers_mut() = headers;
        }

        let response = reqwest::blocking::Client::new().execute(request).unwrap();

        let lua_response = lua.create_table()?;
        lua_response.set("status", response.status().as_u16())?;
        lua_response.set("headers", {
            let headers = lua.create_table().unwrap();
            for (key, value) in response.headers().iter() {
                headers
                    .set(key.as_str().to_string(), value.to_str().unwrap())
                    .unwrap();
            }

            headers
        })?;

        let response_body = response.text().unwrap_or_default();

        lua_response.set("body", response_body.clone())?;
        lua_response.set(
            "json",
            lua.to_value(match &serde_json::from_str::<Value>(&response_body).ok() {
                Some(json) => json,
                _ => &Value::Null,
            })
            .unwrap(),
        )?;

        Ok(lua_response)
    }
}

pub fn expect_equal(
    _ctx: Rc<RefCell<LibContext>>,
) -> impl Fn(&Lua, LuaMultiValue) -> LuaResult<()> {
    move |_lua, values| {
        if values.len() != 2 {
            return Err(LuaError::BindError);
        }

        let values = values
            .iter()
            .map(|value| serde_json::to_value(value).unwrap())
            .collect::<Vec<Value>>();

        let [a, b]: [Value; 2] = values.try_into().unwrap();

        if !a.eq(&b) {
            return Err(LuaError::RuntimeError(format!(
                "Not equal!\nLeft:  {}\nRight: {}",
                a, b,
            )));
        }

        Ok(())
    }
}

pub fn expect_match(
    _ctx: Rc<RefCell<LibContext>>,
) -> impl Fn(&Lua, LuaMultiValue) -> LuaResult<()> {
    move |_lua, values| {
        if values.len() != 2 {
            return Err(LuaError::BindError);
        }

        let values = values.into_iter().collect::<Vec<LuaValue>>();
        let [actual_lua, expected_lua]: [LuaValue; 2] = values.try_into().unwrap();

        if !matches!(actual_lua, LuaValue::Table(_)) || !matches!(expected_lua, LuaValue::Table(_))
        {
            return Err(LuaError::RuntimeError(
                "expect_match expects two tables".to_string(),
            ));
        }

        let actual = serde_json::to_value(actual_lua).unwrap();
        let expected = serde_json::to_value(expected_lua).unwrap();

        if let Some(message) = match_partial_value(&actual, &expected, "$") {
            return Err(LuaError::RuntimeError(format!(
                "Not matching!\n{}",
                message
            )));
        }

        Ok(())
    }
}

fn match_partial_value(actual: &Value, expected: &Value, path: &str) -> Option<String> {
    match expected {
        Value::Object(expected_object) => {
            let Value::Object(actual_object) = actual else {
                return Some(format!(
                    "At:    {}\nLeft:  {}\nRight: {}",
                    path, actual, expected
                ));
            };

            for (key, expected_value) in expected_object {
                let key_path = format!("{}.{}", path, key);
                let Some(actual_value) = actual_object.get(key) else {
                    return Some(format!(
                        "Missing key: {}\nRight:       {}",
                        key_path, expected_value
                    ));
                };

                if let Some(message) = match_partial_value(actual_value, expected_value, &key_path)
                {
                    return Some(message);
                }
            }

            None
        }
        _ if actual.eq(expected) => None,
        _ => Some(format!(
            "At:    {}\nLeft:  {}\nRight: {}",
            path, actual, expected
        )),
    }
}

pub fn json_encode(_ctx: Rc<RefCell<LibContext>>) -> impl Fn(&Lua, LuaTable) -> LuaResult<String> {
    move |lua, value| {
        let json_value: serde_json::Value = lua.from_value(LuaValue::Table(value))?;

        Ok(serde_json::to_string(&json_value).unwrap())
    }
}

pub fn json_decode(_ctx: Rc<RefCell<LibContext>>) -> impl Fn(&Lua, String) -> LuaResult<LuaValue> {
    move |lua, value| {
        let json_decoded: serde_json::Value = serde_json::from_str(&value).unwrap();

        Ok(lua.to_value(&json_decoded)?)
    }
}

pub fn random_chars(_ctx: Rc<RefCell<LibContext>>) -> impl Fn(&Lua, i32) -> LuaResult<String> {
    move |_lua, len| Ok(get_rand_chars(len as usize))
}

pub fn merge_table(
    _ctx: Rc<RefCell<LibContext>>,
) -> impl Fn(&Lua, LuaMultiValue) -> LuaResult<LuaTable> {
    move |lua, values| {
        let merged = lua.create_table()?;

        for value in values {
            let table = match value {
                LuaValue::Table(table) => table,
                value => {
                    return Err(LuaError::RuntimeError(format!(
                        "merge expects tables, got {}",
                        value.type_name()
                    )));
                }
            };

            for pair in table.pairs::<LuaValue, LuaValue>() {
                let (key, value) = pair?;
                merged.raw_set(key, value)?;
            }
        }

        Ok(merged)
    }
}

pub fn copy_table(_ctx: Rc<RefCell<LibContext>>) -> impl Fn(&Lua, LuaTable) -> LuaResult<LuaTable> {
    move |lua, value| {
        let mut visited = HashMap::new();
        copy_lua_table(lua, value, &mut visited)
    }
}

fn copy_lua_table(
    lua: &Lua,
    table: LuaTable,
    visited: &mut HashMap<*const c_void, LuaTable>,
) -> LuaResult<LuaTable> {
    let table_ptr = table.to_pointer();
    if let Some(copied) = visited.get(&table_ptr) {
        return Ok(copied.clone());
    }

    let copied = lua.create_table()?;
    visited.insert(table_ptr, copied.clone());

    for pair in table.pairs::<LuaValue, LuaValue>() {
        let (key, value) = pair?;
        copied.raw_set(
            copy_lua_value(lua, key, visited)?,
            copy_lua_value(lua, value, visited)?,
        )?;
    }

    if let Some(metatable) = table.metatable() {
        copied.set_metatable(Some(copy_lua_table(lua, metatable, visited)?))?;
    }

    Ok(copied)
}

fn copy_lua_value(
    lua: &Lua,
    value: LuaValue,
    visited: &mut HashMap<*const c_void, LuaTable>,
) -> LuaResult<LuaValue> {
    match value {
        LuaValue::Table(table) => Ok(LuaValue::Table(copy_lua_table(lua, table, visited)?)),
        value => Ok(value),
    }
}

pub fn get_rand_chars(len: usize) -> String {
    let mut rng = rand::rng();
    Alphanumeric.sample_string(&mut rng, len)
}
