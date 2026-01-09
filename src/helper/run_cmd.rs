use std::ffi::OsStr;
use std::process::{ Command, Output };

// show output for failed Command so it's easier to debug

pub fn run_cmd<I, S>(program: &str, args: I) -> std::io::Result<Output>
    where I: IntoIterator<Item = S>, S: AsRef<OsStr>
{
    let args_vec: Vec<String> = args
        .into_iter()
        .map(|a| a.as_ref().to_string_lossy().to_string())
        .collect();

    let output = Command::new(program).args(&args_vec).output()?;

    if !output.status.success() || !output.stderr.is_empty() {
        eprintln!("command: {} {}", program, args_vec.join(" "));

        if !output.stdout.is_empty() {
            eprintln!("stdout:\n{}", String::from_utf8_lossy(&output.stdout));
        }
        if !output.stderr.is_empty() {
            eprintln!("stderr:\n{}", String::from_utf8_lossy(&output.stderr));
        }
    }

    Ok(output)
}
