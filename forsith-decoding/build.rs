use std::{env, fs, path::{Path, PathBuf}};

fn main() {
    let mut out = String::new();

    let test_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests");

    let pngsuite_dir = test_dir.clone().join("pngsuite");

    for file in fs::read_dir(pngsuite_dir.clone().join("png")).unwrap() {
        let path = file.unwrap().path();

        let mut solution_filename = PathBuf::from(path.clone().file_name().unwrap());
        solution_filename.set_extension("json");
        let solution_path = pngsuite_dir.join("json").join(solution_filename);

        if path.extension().is_some_and(|x| x == "png") {
            out.push_str(&format!(
                r#"
                #[test]
                fn test_decode_{}() {{
                    match test_image("{}", "{}") {{Ok(_) => (), Err(err) => panic!("{{err}}")}};
                }}
                "#,
                path.file_stem().unwrap().to_str().unwrap(),
                path.display(),
                solution_path.display()
            ));
        }
    }

    fs::write(test_dir.join("generated_png_tests.rs"), out).unwrap();
}
