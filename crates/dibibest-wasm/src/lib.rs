//! WASM bridge for the web canvas. No wasm-bindgen: a three-function C ABI
//! (`alloc`, `dealloc`, `run`) keeps the build to plain `cargo` and the JS side
//! to ~30 lines. All logic lives in [`handle`], which is plain Rust and tested natively.
//!
//! Request (JSON): `{"op":"erd"|"convert","src":..,"from":..,"to":..}`.
//! Response (JSON): `{"ok":true,"output":..,"fidelity":{..}}` or `{"ok":false,"error":..}`.

use dibibest_core::erdjson::{apply_layout, LayoutStrategy};
use dibibest_core::{default_registry, FormatId};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
struct Req {
    op: String,
    src: String,
    from: String,
    #[serde(default)]
    to: Option<String>,
}

/// `erd`: source → ERD-JSON with grid layout applied (`output` is the document).
/// `convert`: source → `to` text. Both carry the real core `Fidelity`.
pub fn handle(req: &str) -> String {
    let out = serde_json::from_str::<Req>(req)
        .map_err(|e| format!("bad request: {e}"))
        .and_then(run_req);
    match out {
        Ok(v) => v,
        Err(e) => json!({"ok": false, "error": e}),
    }
    .to_string()
}

fn run_req(r: Req) -> Result<Value, String> {
    let reg = default_registry();
    let from = FormatId::new(r.from);
    let to = match r.op.as_str() {
        "erd" => FormatId::new("erdjson"),
        "convert" => FormatId::new(r.to.ok_or("convert needs `to`")?),
        op => return Err(format!("unknown op: {op}")),
    };
    let conv = reg
        .convert_explicit(&r.src, &from, &to)
        .map_err(|e| e.to_string())?;
    let output = if r.op == "erd" {
        let mut doc: Value = serde_json::from_str(&conv.output).map_err(|e| e.to_string())?;
        apply_layout(&mut doc, LayoutStrategy::Grid).map_err(|e| e.to_string())?;
        doc.to_string()
    } else {
        conv.output
    };
    Ok(json!({"ok": true, "output": output, "fidelity": conv.fidelity}))
}

#[no_mangle]
pub extern "C" fn alloc(len: usize) -> *mut u8 {
    leak(vec![0; len])
}

/// Boxed slices have capacity == len by construction, so `dealloc` can rebuild them exactly.
fn leak(v: Vec<u8>) -> *mut u8 {
    Box::into_raw(v.into_boxed_slice()) as *mut u8
}

/// # Safety
/// `ptr`/`len` must come from [`alloc`] (or a [`run`] result) with the same `len`.
#[no_mangle]
pub unsafe extern "C" fn dealloc(ptr: *mut u8, len: usize) {
    drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(ptr, len)));
}

/// Returns a buffer `[len: u32 LE][json bytes]`; free it with `dealloc(ptr, 4 + len)`.
///
/// # Safety
/// `ptr`/`len` must describe a live buffer of UTF-8 bytes written by the caller.
#[no_mangle]
pub unsafe extern "C" fn run(ptr: *const u8, len: usize) -> *mut u8 {
    let req = String::from_utf8_lossy(std::slice::from_raw_parts(ptr, len));
    let body = handle(&req).into_bytes();
    let mut buf = Vec::with_capacity(4 + body.len());
    buf.extend_from_slice(&(body.len() as u32).to_le_bytes());
    buf.extend_from_slice(&body);
    leak(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(v: Value) -> Value {
        serde_json::from_str(&handle(&v.to_string())).unwrap()
    }

    #[test]
    fn erd_carries_layout_and_fidelity() {
        let r = call(json!({"op":"erd","from":"dbml","src":"Table a { id integer [pk] }"}));
        assert_eq!(r["ok"], true);
        let doc: Value = serde_json::from_str(r["output"].as_str().unwrap()).unwrap();
        assert_eq!(doc["meta"]["layout"], "grid");
        assert_eq!(r["fidelity"]["tables"]["kept"], 1);
        assert_eq!(r["fidelity"]["idempotent"], true);
    }

    #[test]
    fn convert_and_errors() {
        let r =
            call(json!({"op":"convert","from":"dbml","to":"sql","src":"Table a { id integer }"}));
        assert!(r["output"].as_str().unwrap().contains("CREATE TABLE"));
        assert_eq!(
            call(json!({"op":"convert","from":"dbml","src":""}))["ok"],
            false
        );
        assert_eq!(call(json!({"op":"x","from":"dbml","src":""}))["ok"], false);
        assert!(handle("nope").contains("bad request"));
    }
}
