use crate::js;

use js::ContextGuard;
use js::value::Value;
use js::value::function::CallbackInfo;

fn console_log(guard: &ContextGuard, args: CallbackInfo) -> Result<Value, Value> {
    for arg in args.arguments {
        println!("{}", arg.to_string(guard));
    }
    Result::Ok(js::value::null(guard))
}

pub fn create_console_logging(guard: &ContextGuard){
    let go = guard.global();

    let obj = js::value::Object::new(guard);
    let fun = js::value::Function::new(guard, Box::new(|a,b| console_log(a,b)));
    obj.set(guard, js::Property::new(guard, "log"), fun);
    go.set(guard, js::Property::new(guard, "console"), obj);
}
