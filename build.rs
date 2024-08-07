fn main() {
    println!("Running custom build script...");

    let output = std::process::Command::new("bash")
        .arg("./src/engine/shaders/compile.sh")
        .arg("./src/engine/shaders")
        .output()
        .expect("Failed to execute command");

    println!("{:?}", output);

    if !output.status.success() {
        panic!("Build script failed");
    }
}
