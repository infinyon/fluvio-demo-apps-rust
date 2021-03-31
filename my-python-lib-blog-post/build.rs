use flapigen::{LanguageConfig, PythonConfig};
use std::{env, path::Path};

fn main() {
    let in_src = Path::new("src").join("glue.rs.in");
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_src = Path::new(&out_dir).join("glue.rs");

    let python_cfg = PythonConfig::new("my_python_lib".to_owned());
    let flap_gen =
        flapigen::Generator::new(LanguageConfig::PythonConfig(python_cfg)).rustfmt_bindings(true);
    flap_gen.expand("python bindings", &in_src, &out_src);
    println!("cargo:rerun-if-changed={}", in_src.display());
}
