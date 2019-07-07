use chakracore as js;

mod console;

use console::create_console_logging;
use std::path::{Path, PathBuf};
use std::mem::ManuallyDrop;

use js::ContextGuard;

struct ScriptFactory {}


struct Script {
    object: js::value::Object,
    update: Option<js::value::Function>,
}

impl Script {
    pub fn new(guard: &ContextGuard, object: js::value::Object) -> Self {
        let update = object.get(guard, js::Property::new(guard, "update")).into_function();
        Self {
            object,
            update,
        }
    }
}

impl ScriptFactory {
    fn from_code(guard: &ContextGuard, name: String, path: &Path, code: &str) -> Result<js::value::Function, js::Error> {
        let def = js::script::parse(guard, code)?;
        let factory = def.construct(&guard, guard.global(), &[])?;
        let factory = match factory.into_function() {
            Some(elem) => elem,
            None => return Result::Err(js::Error::ScriptCompilation("Not a function".to_string()))
        };
        Result::Ok(factory)
    }
    fn from_path(guard: &ContextGuard, path: &Path) -> Result<js::value::Function, js::Error> {
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
    pub fn guard<'a>(&'a self) -> ContextGuard<'a> {
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

    fn load_at_path(guard: &ContextGuard, parent: &js::value::Object, directory: &Path) -> Result<(), &'static str> {
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


    pub fn create(&self, name: &str) -> Result<Script, js::Error> {
        let command = format!("new {}()", name);
        let guard = self.guard();

        let obj = js::script::eval(&guard, &command)?;
        let obj = obj.into_object().unwrap();

        Result::Ok(Script::new(&guard, obj))
    }

    pub fn run_with<F: FnOnce(&ContextGuard)>(&self, callback: F) {
        let p = self.guard();
        callback(&p);
    }
}

struct GameObject {

}

struct Scene {
    objects: Vec<GameObject>,
}

impl Scene {
    pub fn new() -> Box<Scene> {
        let sc = Scene {objects: vec![]};
        Box::new(sc)
    }
}

struct Vector {
    data: [f32; 3],
}
fn create_js_game_object(guard: &ContextGuard) {
    let object_prototype = js::value::Object::new(guard);
}

fn create_scene(scene: &mut Scene, guard: &ContextGuard) {
    let go = guard.global();
    let sc = unsafe {js::value::External::from_ptr(guard, scene);};
    go.set(guard, js::Property::new(guard, "_&sc"), sc);
}

fn create_js_vector(guard: &ContextGuard) {
    let object_prototype = js::value::Object::new(guard);

    {
        let object_function1 = js::value::Function::new(guard, Box::new(|g, args| unsafe {
            let val = args.this.into_external().unwrap().value::<Vector>();

            println!("printing from vector function");


            Result::Ok(js::value::null(g))
        }));
        object_prototype.set(guard, js::Property::new(guard, "log"), object_function1);
    }
    {
        let prop = js::value::Object::new(guard);
        prop.set(guard, js::Property::new(guard, "get"), js::value::Function::new(guard, Box::new(|g, args| unsafe {
            let val = args.this.into_external().unwrap().value::<Vector>();
            println!("get!");
            Result::Ok(js::value::Number::new(g, 12).into())
        })));
        prop.set(guard, js::Property::new(guard, "set"), js::value::Function::new(guard, Box::new(|g, args| unsafe {
            let val = args.this.into_external().unwrap().value::<Vector>();
            println!("set!");
            Result::Ok(js::value::null(g))
        })));

        object_prototype.define_property(guard, js::Property::new(guard, "some_prop"), prop);
    }
    let fnc = js::value::Function::new(guard, Box::new(move |g, args| {
        let obj = js::value::External::new(g, Box::new(Vector { data: [0.0f32, 0.0, 0.0] }));
        obj.set_prototype(g, object_prototype.clone());
        Result::Ok(obj.into())
    }));

    let global = guard.global();

    global.set(guard, js::Property::new(guard, "Vector"), fnc);
}

fn main() {
    let mut engine = JsEngine::new().unwrap();

    engine.load_all_scripts("./test_scripts/".as_ref()).unwrap();
    let engine = engine;
    let guard = engine.guard();

    create_js_vector(&guard);
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
            this.whatever = new namespace1.file2();
            this.whatever2 = new file1();
            console.log(\"constructed\");
        }\
    }").unwrap();

    let p = factory.construct(&guard, go, &[]).unwrap().into_object().unwrap();

    let g = engine.create("namespace2.file3").unwrap();


    let objs: Vec<_> = (0..10).map(|x| {
        engine.create("namespace2.file3").unwrap()
    }).collect();

    engine.run_with(|x| {
        for prop in p.get_own_property_names(x).iter(x) {
            println!("{}: {}", prop.to_string(x), p.get(x, js::Property::new(x, &prop.to_string(x))).to_string(x));
        }

        for obj in objs {
            match &obj.update {
                None => (),
                Some(fun) => { fun.call(x, &[]).unwrap(); }
            }
        }


        println!("{:?}, {:?}", g.object.to_string(x), g.update.map(|y| y.to_string(x)));
    });

    engine.run_with(|x| {
        js::script::eval(x, "\
        class Ppp extends Vector {
            constructor() {
                super()
            }
        }
        let p = new Ppp();
        p.log();
        let f = p.some_prop;
        p.some_prop=12
        ").unwrap();
    })
}
