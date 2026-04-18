use mlua::prelude::{LuaSerdeExt, *};
use reqwest::header::HeaderMap;
use serde_json::Value;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

struct FetchOptions {
    url: String,
    method: reqwest::Method,
    body: Option<String>,
    headers: Option<reqwest::header::HeaderMap>,
}

pub struct LibContext {
    pub process_list: VecDeque<std::process::Child>,
}

pub fn exec_bg(ctx: Rc<RefCell<LibContext>>) -> impl Fn(&Lua, String) -> LuaResult<()> {
    move |_lua, command| {
        eprintln!("Executing command: {}", command);

        ctx.borrow_mut()
            .process_list
            .push_back(crate::common::spawn_background_process(&command).unwrap());

        LuaResult::Ok(())
    }
}

pub fn test(ctx: Rc<RefCell<LibContext>>) -> impl Fn(&Lua, (String, LuaFunction)) -> LuaResult<()> {
    move |_lua, (test_name, func)| {
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
        } else {
            println!("✅ {}", test_name);
        }

        crate::common::kill_processes(&mut ctx.clone().borrow_mut().process_list);

        Ok(())
    }
}

pub fn fetch(_ctx: Rc<RefCell<LibContext>>) -> impl Fn(&Lua, LuaValue) -> LuaResult<LuaTable> {
    move |lua, value| {
        eprintln!("Let's make a request...");

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
