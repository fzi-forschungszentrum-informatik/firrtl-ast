//! Example reading FIRRTL code from files and printing each module's dependencies

fn main() {
    use std::io::Read;

    for path in std::env::args_os().skip(1) {
        let mut buf = Default::default();
        std::fs::File::open(path)
            .expect("Could not open file")
            .read_to_string(&mut buf)
            .expect("Failed to read from stdin");

        firrtl_ast::circuit::consumer(buf.as_ref())
            .expect("Failed to parse circuit")
            .try_for_each(|m| m.map(|m| {
                print!("{} {}:", m.kind().keyword(), m.name());
                m.referenced_modules().for_each(|m| print!(" {}", m.name()));
                println!("")
            })).expect("Failed to parse some module");
    }
}
