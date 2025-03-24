use salvo::{oapi::ToSchema, prelude::Json};
use serde::Serialize;

#[derive(Debug, Serialize, ToSchema)]
pub struct ResObj<T: ToSchema + 'static> {
    pub code: i32,
    pub data: Option<T>,
    pub msg: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Page<T: ToSchema + 'static> {
    pub rows: Vec<T>,
    pub total: u64,
}

impl<T: ToSchema> ResObj<T> {
    pub fn ok(data: Option<T>) -> Self {
        Self {
            code: 200,
            msg: "访问成功".to_string(),
            data,
        }
    }
    pub fn custom_code(code: i32, msg: String) -> Self {
        Self { code, msg, data: None }
    }

    pub fn err(err: String) -> Self {
        Self { code: 500, msg: err, data: None }
    }
}

#[allow(dead_code)]
pub fn res_ok<T: ToSchema>(data: Option<T>) -> ResObj<T> {
    ResObj::ok(data)
}

#[allow(dead_code)]
pub fn res_json_ok<T: ToSchema>(data: Option<T>) -> Json<ResObj<T>> {
    Json(ResObj::ok(data))
}

#[allow(dead_code)]
pub fn res_err<T: ToSchema>(msg: String) -> ResObj<T> {
    ResObj::err(msg)
}

#[allow(dead_code)]
pub fn res_json_err<T: ToSchema>(msg: String) -> Json<ResObj<T>> {
    Json(ResObj::err(msg))
}

#[allow(dead_code)]
pub fn res_custom<T: ToSchema>(code: i32, msg: String) -> ResObj<T> {
    ResObj::custom_code(code, msg)
}

#[allow(dead_code)]
pub fn res_json_custom<T: ToSchema>(code: i32, msg: String) -> Json<ResObj<T>> {
    Json(ResObj::custom_code(code, msg))
}

#[allow(dead_code)]
pub type Res<T> = Result<Json<ResObj<T>>, Json<ResObj<()>>>;

#[allow(dead_code)]
pub fn match_ok_common_result_no_error<T: ToSchema>(res: Result<T, ()>) -> Res<T> {
    match res {
        Ok(v) => Ok(res_json_ok(Some(v))),
        Err(_) => Err(res_json_custom(400, "服务器发生错误".to_string())),
    }
}
