
use std::env;
use std::fs;
use std::path::Path;

use cargo_toml::Manifest;
// use saphyr::Yaml;


// const RESERVED_KEYWORDS: [&str; 5] = ["ubihome", "button", "sensor", "binary_sensor", "text_sensor"];

fn main() {
    // println!("cargo:rerun-if-changed=NULL");

    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=Cargo.toml");
    // let yaml_path =  Path::join(Path::new(&env::var_os("CARGO_MANIFEST_DIR").unwrap()), "config.yaml");
        // if let Ok(content) = fs::read_to_string(yaml_path) {
            // let config = Yaml::load_from_str(&content).unwrap();
        // let yaml = &config[0]; // select the first YAML document

        // let modules = &yaml.as_hash().map(|h| h.raw_entry().).iter().skip_while(|y| RESERVED_KEYWORDS.iter().any(y)).unwrap();
        // println!("cargo::error=HELLO: {:?}", &yaml.as_hash().map(|h| h.keys()).unwrap()); 
        // assert_eq!(yaml[0].as_a().unwrap(), 1); // access elements by index
    // } 


    let toml_path =  Path::join(Path::new(&env::var_os("CARGO_MANIFEST_DIR").unwrap()), "Cargo.toml");
    let cargo_toml = Manifest::from_path(toml_path).unwrap();
    let import_packages = cargo_toml.dependencies.iter().filter(|k| k.0.starts_with("ubihome-")).map(|(k, _)| k).collect::<Vec<_>>(); 

    let usings = import_packages.clone().iter()
        .map(|p| p.replace("-", "_"))
        .map(|p| format!(r#"use {}::start;"#, p))
        .collect::<Vec<_>>().join("\n");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("start.rs");
    fs::write(
        &dest_path,
        usings, 
    ).unwrap();

    let dest_path = Path::new(&out_dir).join("config.rs");
    fs::write(
        &dest_path,
        format!("#[derive(Clone, Deserialize, Debug)]
pub struct Config {{
    pub ubihome: UbiHome,
    pub logger: Option<Logger>,

    pub button: Option<Vec<ButtonConfig>>,
    pub binary_sensor: Option<Vec<BinarySensor>>,

{}
    // pub mqtt: Option<MqttConfig>,
    // pub shell: Option<ShellConfig>,
    // pub web_server: Option<WebServerConfig>,
    // pub gpio: Option<GpioConfig>,
}}", &import_packages.iter().map(|p| format!("    pub {}: Option<{}Config>", p.replace("ubihome-", ""), package_name_to_camel_case(p.replace("ubihome-", "")))).collect::<Vec<_>>().join(",\n"))).unwrap();

}

fn package_name_to_camel_case(package_name: String) -> String {
    package_name
        .split('-')
        .map(|s| s.chars().next().unwrap().to_uppercase().collect::<String>() + &s[1..])
        .collect::<Vec<_>>()
        .join("")
}