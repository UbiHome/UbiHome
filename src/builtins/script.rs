//! A shared JavaScript engine for the builtin `template` entities: it backs
//! a template switch/number's `lambda` (computes the reported state) and the
//! `lambda` [action](crate::builtins::run_actions) (runs arbitrary JS as part
//! of a trigger's `then` list).
//!
//! This is deliberately separate from the standalone `ubihome-lambda`
//! platform crate (`components/lambda`), which only computes read-only
//! sensors and has no access to entities or globals. This engine instead
//! runs centrally in the main binary (like the rest of `crate::builtins`) so
//! its lambdas can reference entities/globals across every platform via
//! `id()`/`set_global()`, mirroring ESPHome's C++ lambdas:
//!
//! - `id(some_switch).turn_on()` / `.turn_off()` sends a switch command.
//! - `id(some_button).press()` presses a button.
//! - `id(some_global)` reads a [`globals`](crate::builtins::globals) variable
//!   (ESPHome instead lets you assign directly, e.g. `id(x) = 5;`; this JS
//!   engine can't offer that, so writing uses `set_global(name, value)`).
//! - `x` is the value passed to the trigger, for triggers that carry one
//!   (currently only a template number's `set_action`); `None` everywhere
//!   else.
//! - `log(message)` writes to the application log, same as the standalone
//!   `lambda` sensor platform.
//!
//! Every lambda runs on one dedicated OS thread (Boa's `Context` is not
//! `Send`), sharing a single JS context/global scope - like ESPHome globals,
//! a top-level `var`/`globalThis` assignment in one lambda is visible to
//! every other lambda. Each evaluation's own body still runs in its own
//! function scope (see [`wrap_lambda`]), so `let`/`const` declared inside one
//! lambda never collides with another's.

use boa_engine::object::ObjectInitializer;
use boa_engine::{
    js_string, Context, Finalize, JsObject, JsResult, JsValue, NativeFunction, Source, Trace,
};
use log::error;
use std::sync::mpsc;
use tokio::sync::broadcast::Sender;
use tokio::sync::oneshot;
use ubihome_core::global_value::GlobalValue;
use ubihome_core::PublishedMessage;

use crate::builtins::globals::{GlobalType, Globals};

/// Cap on JS loop iterations per evaluation, so a runaway `while` loop (like
/// the one in a template number's `set_action` example) fails instead of
/// hanging the engine thread.
const DEFAULT_MAX_LOOP_ITERATIONS: u64 = 10_000_000;

/// The result of evaluating a `lambda`. `None` mirrors returning nothing (or
/// `null`) from an ESPHome lambda: "publish no update" for an entity state
/// lambda, and simply "no return value" for a `lambda` action.
#[derive(Clone, Debug, PartialEq)]
pub enum ScriptValue {
    Number(f32),
    Bool(bool),
    Text(String),
    None,
}

/// Data captured by every native function the engine registers. Cheap to
/// clone (both fields are reference-counted handles), and holds no `Gc`
/// pointers, so it's safe to hand to Boa as opaque, non-traced captures (see
/// the `Trace`/`Finalize` impls below).
#[derive(Clone)]
struct EngineContext {
    globals: Globals,
    tx: Sender<PublishedMessage>,
}

impl Finalize for EngineContext {}
// SAFETY: `EngineContext` only holds a `Globals` (Arc-based) and a broadcast
// `Sender`, neither of which contain `Gc`-managed pointers, so there is
// nothing for the collector to trace.
unsafe impl Trace for EngineContext {
    boa_engine::gc::empty_trace!();
}

enum Request {
    Eval {
        source: String,
        x: Option<f64>,
        respond: oneshot::Sender<Result<ScriptValue, String>>,
    },
}

/// A cheap, cloneable handle to the dedicated script engine thread.
#[derive(Clone)]
pub struct ScriptEngine {
    requests: mpsc::Sender<Request>,
}

impl ScriptEngine {
    /// Spawns the dedicated OS thread that owns the JS engine, and returns a
    /// handle every consumer (template switches/numbers, `run_actions`) can
    /// clone and send requests to.
    pub fn spawn(globals: Globals, tx: Sender<PublishedMessage>) -> Self {
        let (requests, receiver) = mpsc::channel();
        std::thread::spawn(move || engine_thread(globals, tx, receiver));
        ScriptEngine { requests }
    }

    /// Evaluate a `lambda` (an entity's state lambda, or a `lambda` action's
    /// source). `x` is bound as the JS `x` global for triggers that carry a
    /// commanded value (currently a template number's `set_action`); pass
    /// `None` everywhere else.
    pub async fn eval(&self, source: &str, x: Option<f64>) -> Result<ScriptValue, String> {
        let (respond, response) = oneshot::channel();
        self.requests
            .send(Request::Eval {
                source: source.to_string(),
                x,
                respond,
            })
            .map_err(|_| "script engine is no longer running".to_string())?;
        response
            .await
            .map_err(|_| "script engine dropped the request".to_string())?
    }
}

/// Compile (but do not run) a lambda to surface syntax errors at config
/// validation time. Evaluating a function *expression* parses the whole body
/// without executing it, so referencing `id`/`x`/`set_global` (which are
/// only defined at evaluation time) is not itself an error here.
pub fn check_syntax(source: &str) -> Result<(), String> {
    let mut context = Context::default();
    context
        .eval(Source::from_bytes(&wrap_lambda(&preprocess(source))))
        .map(|_| ())
        .map_err(|e| format!("Invalid lambda: {}", e))
}

/// `garde` custom validator wrapping [`check_syntax`] for `Option<String>`
/// entity `lambda` fields (see the template number/switch config).
pub fn validate_lambda(source: &Option<String>, _: &()) -> garde::Result {
    let Some(source) = source else {
        return Ok(());
    };
    check_syntax(source).map_err(garde::Error::new)
}

fn engine_thread(
    globals: Globals,
    tx: Sender<PublishedMessage>,
    requests: mpsc::Receiver<Request>,
) {
    let mut context = Context::default();
    context
        .runtime_limits_mut()
        .set_loop_iteration_limit(DEFAULT_MAX_LOOP_ITERATIONS);

    let engine_context = EngineContext { globals, tx };
    if let Err(e) = setup_context(&mut context, &engine_context) {
        error!("Failed to set up script engine context: {}", e);
        return;
    }

    while let Ok(request) = requests.recv() {
        match request {
            Request::Eval { source, x, respond } => {
                let result = eval_source(&mut context, &source, x);
                let _ = respond.send(result);
            }
        }
    }
}

/// Register the `id`/`set_global`/`log` host functions.
fn setup_context(context: &mut Context, engine_context: &EngineContext) -> Result<(), String> {
    context
        .register_global_builtin_callable(
            js_string!("id"),
            1,
            NativeFunction::from_copy_closure_with_captures(id_native, engine_context.clone()),
        )
        .map_err(|e| e.to_string())?;
    context
        .register_global_builtin_callable(
            js_string!("set_global"),
            2,
            NativeFunction::from_copy_closure_with_captures(
                set_global_native,
                engine_context.clone(),
            ),
        )
        .map_err(|e| e.to_string())?;
    context
        .register_global_builtin_callable(
            js_string!("log"),
            1,
            NativeFunction::from_copy_closure(log_native),
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn log_native(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    if let Some(arg) = args.first() {
        let message = arg.to_string(context)?.to_std_string_escaped();
        log::info!("[lambda] {}", message);
    }
    Ok(JsValue::undefined())
}

/// `id(name)`: a known global's current value, or a command handle
/// (`turn_on()`/`turn_off()`/`press()`) for anything else. See the module
/// docs for why globals and entities aren't disambiguated any other way.
fn id_native(
    _this: &JsValue,
    args: &[JsValue],
    captures: &EngineContext,
    context: &mut Context,
) -> JsResult<JsValue> {
    let Some(key) = args.first() else {
        return Ok(JsValue::undefined());
    };
    let key = key.to_string(context)?.to_std_string_escaped();

    if let Some(value) = captures.globals.get(&key) {
        return Ok(global_value_to_js(&value));
    }

    Ok(entity_handle(&key, captures, context).into())
}

/// Builds the `{ turn_on, turn_off, press }` object `id()` returns for a
/// name that isn't a known global. Calling a method that doesn't apply to
/// the referenced entity (e.g. `.press()` on a switch) is a harmless no-op:
/// it just publishes a command nobody listens for.
fn entity_handle(key: &str, captures: &EngineContext, context: &mut Context) -> JsObject {
    ObjectInitializer::new(context)
        .function(
            NativeFunction::from_copy_closure_with_captures(
                turn_on_native,
                (captures.clone(), key.to_string()),
            ),
            js_string!("turn_on"),
            0,
        )
        .function(
            NativeFunction::from_copy_closure_with_captures(
                turn_off_native,
                (captures.clone(), key.to_string()),
            ),
            js_string!("turn_off"),
            0,
        )
        .function(
            NativeFunction::from_copy_closure_with_captures(
                press_native,
                (captures.clone(), key.to_string()),
            ),
            js_string!("press"),
            0,
        )
        .build()
}

fn turn_on_native(
    _this: &JsValue,
    _args: &[JsValue],
    (engine_context, key): &(EngineContext, String),
    _context: &mut Context,
) -> JsResult<JsValue> {
    let _ = engine_context
        .tx
        .send(PublishedMessage::SwitchStateCommand {
            key: key.clone(),
            state: true,
        });
    Ok(JsValue::undefined())
}

fn turn_off_native(
    _this: &JsValue,
    _args: &[JsValue],
    (engine_context, key): &(EngineContext, String),
    _context: &mut Context,
) -> JsResult<JsValue> {
    let _ = engine_context
        .tx
        .send(PublishedMessage::SwitchStateCommand {
            key: key.clone(),
            state: false,
        });
    Ok(JsValue::undefined())
}

fn press_native(
    _this: &JsValue,
    _args: &[JsValue],
    (engine_context, key): &(EngineContext, String),
    _context: &mut Context,
) -> JsResult<JsValue> {
    let _ = engine_context
        .tx
        .send(PublishedMessage::ButtonPressed { key: key.clone() });
    Ok(JsValue::undefined())
}

/// `set_global(name, value)`: writes a [`globals`](crate::builtins::globals)
/// variable. The declared `type:` of the global decides whether a JS number
/// becomes an `Int` or a `Float` (JS has only one number type).
fn set_global_native(
    _this: &JsValue,
    args: &[JsValue],
    captures: &EngineContext,
    context: &mut Context,
) -> JsResult<JsValue> {
    let Some(name) = args.first() else {
        return Ok(JsValue::undefined());
    };
    let name = name.to_string(context)?.to_std_string_escaped();
    let value = args.get(1).cloned().unwrap_or(JsValue::undefined());

    let global_value = if let Some(b) = value.as_boolean() {
        GlobalValue::Bool(b)
    } else if let Some(number) = value.as_number() {
        match captures.globals.type_of(&name) {
            Some(GlobalType::Int) => GlobalValue::Int(number.round() as i64),
            _ => GlobalValue::Float(number),
        }
    } else {
        GlobalValue::String(value.to_string(context)?.to_std_string_escaped())
    };

    captures.globals.set(&name, global_value);
    Ok(JsValue::undefined())
}

fn global_value_to_js(value: &GlobalValue) -> JsValue {
    match value {
        GlobalValue::Bool(value) => JsValue::from(*value),
        GlobalValue::Int(value) => JsValue::from(*value as f64),
        GlobalValue::Float(value) => JsValue::from(*value),
        GlobalValue::String(value) => JsValue::from(js_string!(value.clone())),
    }
}

/// Wraps the lambda body in a function expression. Evaluating the expression
/// compiles the body (surfacing syntax errors) without executing it;
/// appending `()` executes it and yields the `return` value.
fn wrap_lambda(source: &str) -> String {
    format!("(function() {{ {}\n }})", source)
}

/// Sets the JS `x` global, then compiles and runs `source`, converting its
/// return value (if any) into a [`ScriptValue`].
fn eval_source(context: &mut Context, source: &str, x: Option<f64>) -> Result<ScriptValue, String> {
    let x_code = match x {
        Some(value) => format!("globalThis.x = {};", value),
        None => "globalThis.x = undefined;".to_string(),
    };
    context
        .eval(Source::from_bytes(&x_code))
        .map_err(|e| e.to_string())?;

    let code = format!("{}()", wrap_lambda(&preprocess(source)));
    let value = context
        .eval(Source::from_bytes(&code))
        .map_err(|e| e.to_string())?;
    convert_return_value(value)
}

fn convert_return_value(value: JsValue) -> Result<ScriptValue, String> {
    if value.is_null_or_undefined() {
        return Ok(ScriptValue::None);
    }
    if let Some(value) = value.as_boolean() {
        return Ok(ScriptValue::Bool(value));
    }
    if let Some(value) = value.as_number() {
        return Ok(ScriptValue::Number(value as f32));
    }
    if let Some(value) = value.as_string() {
        return Ok(ScriptValue::Text(value.to_std_string_escaped()));
    }
    Err("lambda must return a number, a boolean, a string, or nothing".to_string())
}

/// Rewrites ESPHome-style bare `id(name)` references (`id(global_volume)`,
/// `id(volume_up_button)`) into `id("name")` calls the `id` native function
/// (which only ever receives a string) can actually handle. `name` is never a
/// real JS variable, so without this substitution it would fail to resolve
/// at evaluation time. An already-quoted argument (`id("x")`, `id('x')`) is
/// left untouched.
fn preprocess(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len());
    let mut i = 0;
    while i < bytes.len() {
        let preceded_by_ident = i > 0 && is_ident_char(bytes[i - 1]);
        let followed_by_ident = bytes.get(i + 2).is_some_and(|b| is_ident_char(*b));
        if !preceded_by_ident && !followed_by_ident && source[i..].starts_with("id") {
            if let Some((ident, after)) = match_id_call(source, i + 2) {
                out.push_str("id(\"");
                out.push_str(ident);
                out.push_str("\")");
                i = after;
                continue;
            }
        }
        let ch = source[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// Given the byte offset just after `id`, matches `(<spaces><ident><spaces>)`
/// and returns the identifier together with the offset just past the
/// closing `)`. Returns `None` (leaving the input untouched) for anything
/// else, including an already-quoted argument.
fn match_id_call(source: &str, after_id: usize) -> Option<(&str, usize)> {
    let bytes = source.as_bytes();
    let mut j = after_id;
    while bytes.get(j).is_some_and(|b| b.is_ascii_whitespace()) {
        j += 1;
    }
    if bytes.get(j) != Some(&b'(') {
        return None;
    }
    j += 1;
    while bytes.get(j).is_some_and(|b| b.is_ascii_whitespace()) {
        j += 1;
    }
    let ident_start = j;
    while bytes.get(j).is_some_and(|b| is_ident_char(*b)) {
        j += 1;
    }
    let ident_end = j;
    while bytes.get(j).is_some_and(|b| b.is_ascii_whitespace()) {
        j += 1;
    }
    if ident_end == ident_start || bytes.get(j) != Some(&b')') {
        return None;
    }
    Some((&source[ident_start..ident_end], j + 1))
}

fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocess_rewrites_bare_identifiers() {
        assert_eq!(preprocess("id(global_volume)"), r#"id("global_volume")"#);
        assert_eq!(
            preprocess("id(volume_up_button).press()"),
            r#"id("volume_up_button").press()"#
        );
        assert_eq!(preprocess("id( spaced_out  )"), r#"id("spaced_out")"#);
    }

    #[test]
    fn test_preprocess_leaves_quoted_calls_untouched() {
        assert_eq!(
            preprocess(r#"id("already_quoted")"#),
            r#"id("already_quoted")"#
        );
        assert_eq!(preprocess("id('single')"), "id('single')");
    }

    #[test]
    fn test_preprocess_ignores_lookalike_identifiers() {
        assert_eq!(preprocess("valid_id(x)"), "valid_id(x)");
        assert_eq!(preprocess("myid(x)"), "myid(x)");
        assert_eq!(preprocess("id2(x)"), "id2(x)");
    }

    #[test]
    fn test_check_syntax_rejects_invalid_js() {
        assert!(check_syntax("return 1 +;").is_err());
    }

    #[test]
    fn test_check_syntax_accepts_id_and_x_usage() {
        assert!(check_syntax("return id(global_volume) - x;").is_ok());
    }

    fn test_globals() -> Globals {
        Globals::new(&[crate::builtins::globals::GlobalConfig {
            id: "global_volume".to_string(),
            value_type: GlobalType::Float,
            initial_value: Some(GlobalValue::Float(10.0)),
        }])
    }

    #[tokio::test]
    async fn test_eval_reads_global_and_x() {
        let (tx, _rx) = tokio::sync::broadcast::channel(16);
        let engine = ScriptEngine::spawn(test_globals(), tx);
        let result = engine
            .eval("return id(global_volume) - x;", Some(3.0))
            .await;
        assert_eq!(result.unwrap(), ScriptValue::Number(7.0));
    }

    #[tokio::test]
    async fn test_eval_runaway_loop_is_interrupted() {
        let (tx, _rx) = tokio::sync::broadcast::channel(16);
        let engine = ScriptEngine::spawn(test_globals(), tx);
        let result = engine.eval("while (true) {}", None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_eval_dispatches_button_press() {
        let (tx, mut rx) = tokio::sync::broadcast::channel(16);
        let engine = ScriptEngine::spawn(test_globals(), tx);
        engine
            .eval("id(volume_up_button).press();", None)
            .await
            .unwrap();
        let message = rx.recv().await.unwrap();
        assert!(matches!(
            message,
            PublishedMessage::ButtonPressed { key } if key == "volume_up_button"
        ));
    }
}
