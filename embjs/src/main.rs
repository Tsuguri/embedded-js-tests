use chakracore as js;

mod console;

use console::create_console_logging;
use std::path::{Path, PathBuf};
use std::mem::ManuallyDrop;

struct ScriptFactory {
}

impl ScriptFactory {
    fn from_code(guard: &js::ContextGuard, name: String, path: &Path, code: &str) -> Result<js::value::Function, js::Error> {
        let def = js::script::parse(guard, code)?;
        let factory = def.construct(&guard, guard.global(), &[])?;
        let factory = match factory.into_function() {
            Some(elem) => elem,
            None => return Result::Err(js::Error::ScriptCompilation("Not a function".to_string()))
        };
        Result::Ok(factory)
    }
    fn from_path(guard: &js::ContextGuard, path: &Path) -> Result<js::value::Function, js::Error> {
        let name = path.file_stem().unwrap().to_str().unwrap().to_owned();

        let code = std::fs::read_to_string(path).unwrap();
        Self::from_code(guard, name, path, &code)
    }
}

struct JsEngine {
    runtime: js::Runtime,
    context: ManuallyDrop<js::Context>,
}

// very sad, as context has to be destroyed before runtime is even touched.
impl std::ops::Drop for JsEngine {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.context);
        }
    }
}

impl JsEngine {
    pub fn guard<'a>(&'a self) -> js::ContextGuard<'a> {
        self.context.make_current().unwrap()
    }
    pub fn new() -> Result<JsEngine, js::Error> {
        let runtime = js::Runtime::new()?;
        let context = js::Context::new(&runtime)?;
        Result::Ok(JsEngine {
            runtime,
            context: ManuallyDrop::new(context),
        })
    }

    fn load_at_path(guard: &js::ContextGuard, parent: &js::value::Object, directory: &Path) -> Result<(), &'static str> {
        println!("loading scripts from: {:?}", directory);

        let paths = std::fs::read_dir(directory).map_err(|err| "counldn't read directory")?;
        for path in paths {
            let path = path.map_err(|err| "error reading script directory")?;

            if path.path().is_dir() {
                let p = path.path();
                let p2 = p.file_stem().unwrap();
                let namespace = match p2.to_str() {
                    Option::None =>
                        return Result::Err("invalid character in namespace string"),
                    Option::Some(name) => name,

                };
                println!("creating namespace: {:?}", namespace);
                let obj = js::value::Object::new(guard);
                Self::load_at_path(guard, &obj, &path.path())?;
                parent.set(guard, js::Property::new(guard, namespace), obj);
            } else {
                let p = path.path();
                let p2 = p.file_stem().unwrap().to_str().unwrap();
                let factory = ScriptFactory::from_path(guard, &p).unwrap();
                parent.set(guard, js::Property::new(guard, p2), factory);

            }
        }

        Result::Ok(())
    }

    pub fn load_all_scripts(&mut self, directory: &Path) -> Result<(), &'static str> {
        let guard = self.context.make_current().map_err(|err| "couldn't make context current")?;
        let go = guard.global();


        Self::load_at_path(&guard, &go, directory)
    }
}

fn main() {
    let mut engine = JsEngine::new().unwrap();

    engine.load_all_scripts("./test_scripts/".as_ref()).unwrap();
    let engine = engine;
    let guard = engine.guard();
    let go = guard.global();

    create_console_logging(&guard);

    let timeout = js::value::Function::with_name(&guard, "setTimeout", Box::new(|guard, args| {
        let time = args.arguments[0].to_integer(guard);
        if args.arguments.len() > 1 {
            println!("{}", args.arguments[1].to_string(guard));
        }
        std::thread::sleep(std::time::Duration::from_millis(time as u64));
        println!("wow");
        Result::Ok(js::value::Number::new(guard, 10).into())
    }));
    go.set(&guard, js::Property::new(&guard, "setTimeout"), timeout);


    let factory = ScriptFactory::from_code(&guard, "Prostokat".to_string(), "".as_ref(), "\
    \
    class Prostokat {
        constructor() {
            this.wysokosc = 10;
            this.szerokosc = 20;
            this.whatever = new namespace1.file2()
            console.log(\"constructed\");
        }\
    }").unwrap();

    let p = factory.construct(&guard, go, &[]).unwrap();


//    for prop in p.get_own_property_names(&guard).iter(&guard) {
//        println!("{}: {}", prop.to_string(&guard), p.get(&guard, js::Property::new(&guard, &prop.to_string(&guard))).to_string(&guard));
//    }
}
