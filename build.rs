fn main() {
    println!("Running custom build script...");

    let output = std::process::Command::new("sh")
        .arg("./src/engine/shaders/compile.sh")
        .output()
        .expect("Failed to execute command");

    println!("{:?}", output);

    if !output.status.success() {
        panic!("Build script failed");
    }
}
