use chakracore as js;

mod console;
use console::create_console_logging;

fn main() {
    let runtime = js::Runtime::new().unwrap();
    let context = js::Context::new(&runtime).unwrap();
    let guard = context.make_current().unwrap();
    let go = guard.global();

    create_console_logging(&guard);

    let timeout = js::value::Function::with_name(&guard, "setTimeout", Box::new(|guard, args| {
        let time = args.arguments[0].to_integer(guard);
        if args.arguments.len() >1 {
            println!("{}", args.arguments[1].to_string(guard));
        }
        std::thread::sleep(std::time::Duration::from_millis(time as u64));
        println!("wow");
        Result::Ok(js::value::Number::new(guard, 10).into())
    }));
    //go.define_property(&guard, "setTimeout", timeout);
    go.set(&guard, js::Property::new(&guard,"setTimeout"), timeout);

    let func = js::script::parse(&guard, "\
    \
    class Prostokat {
        constructor(wysokosc, szerokosc) {
            this.wysokosc = wysokosc;
            this.szerokosc = szerokosc;
            console.log(\"constructed\");
        }\
    }
    //var cp2 = new Prostokat(1,2);
    //var cp = \"hoho\"; console.log(\"heheheszki\"); setTimeout(2000, ()=>{})\
    ").unwrap();

    func.call(&guard, &[]).unwrap();

    println!("props:");
    let props = go.get_own_property_names(&guard);
    for prop in props.iter(&guard) {
        println!("{}", prop.to_string(&guard));
    }
    //func.call(&guard, &[]).unwrap();
    //let obj2 = js::value::Object::new(&guard);
    //let p = func.construct(&guard,&obj2, &[]).unwrap();

    //let result = js::script::eval(&guard, "setTimeout(2000, cp, () => {})").unwrap();
    //assert_eq!(result.to_integer(&guard), 10);
}
