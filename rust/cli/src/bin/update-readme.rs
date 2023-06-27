//! Updates README.md with the output of running the main executable with `-h` and `--help`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{exit, Command};

const BEGIN_FULL_MARKER: &str = "<!-- BEGIN AUTOGENERATED HELP -->";
const END_FULL_MARKER: &str = "<!-- END AUTOGENERATED HELP -->";
const BEGIN_SHORT_MARKER: &str = "<!-- BEGIN AUTOGENERATED HELP-SHORT -->";
const END_SHORT_MARKER: &str = "<!-- END AUTOGENERATED HELP-SHORT -->";

fn main() {
    let readme_path = get_readme_path();
    println!("Updating: {:?}", readme_path);

    let readme = fs::read_to_string(&readme_path).expect("Could not read README.md");

    let readme = updated_readme_content(
        readme,
        BEGIN_FULL_MARKER,
        get_full_help_text(),
        END_FULL_MARKER,
    );
    let readme = updated_readme_content(
        readme,
        BEGIN_SHORT_MARKER,
        get_short_help_text(),
        END_SHORT_MARKER,
    );

    write_new_readme(readme_path, readme);
}

fn get_readme_path() -> std::path::PathBuf {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

    Path::new(&manifest_dir).join("README.md")
}

fn updated_readme_content(
    readme: String,
    begin_marker: &str,
    help_text: String,
    end_marker: &str,
) -> String {
    let begin_index =
        readme.find(begin_marker).expect("Begin marker not found") + begin_marker.len();
    let end_index = readme.find(end_marker).expect("End marker not found");

    create_readme_text_with_inlined_help(readme, begin_index, help_text, end_index)
}

fn write_new_readme(readme_path: std::path::PathBuf, new_readme: String) {
    let mut file = fs::File::create(readme_path).expect("Could not open README.md");
    file.write_all(new_readme.as_bytes())
        .expect("Could not write to README.md");
    file.flush().expect("Could not flush output to README.md");
}

fn create_readme_text_with_inlined_help(
    readme: String,
    begin_index: usize,
    help_text: String,
    end_index: usize,
) -> String {
    let mut new_readme = String::new();

    new_readme.push_str(&readme[..begin_index]);
    new_readme
        .push_str("\n<!-- This content is autogenerated and should not be manually changed. -->");
    new_readme.push_str("\n<!-- To update this section, run `make`. -->");
    new_readme.push_str("\n```\n");
    new_readme.push_str(&help_text);
    new_readme.push_str("```\n");
    new_readme.push_str(&readme[end_index..]);

    new_readme
}

fn get_short_help_text() -> String {
    get_hl_output("-h")
}

fn get_full_help_text() -> String {
    get_hl_output("--help")
}

fn get_hl_output(option: &str) -> String {
    let output = Command::new("cargo")
        .arg("run")
        .arg("-q")
        .arg("--")
        .arg(option)
        .output()
        .expect("Cargo run failed");

    if !output.status.success() {
        eprintln!("Cargo run executed with failure");
        exit(1);
    }

    String::from_utf8(output.stdout).expect("Failed to convert output to string")
}