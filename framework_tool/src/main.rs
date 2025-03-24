use framework_lib::commandline;

/// Get commandline arguments
fn get_args() -> Vec<String> {
    std::env::args().collect()
}

fn main() -> Result<(), &'static str> {
    //let args = get_args();
    let args: Vec<String> = [
        "framework_tool",
        "--rgbkbd",
        "0",
        "0xFFFFFF",
        "0xFFFFFF",
        "0xFFFFFF",
        "0xFFFFFF",
        "0xFFFFFF",
        "0xFFFFFF",
        "0xFFFFFF",
        "0xFFFFFF",
    ].iter().map(|x| x.to_string()).collect();
    let args = commandline::parse(&args);
    if (commandline::run_with_args(&args, false)) != 0 {
        return Err("Fail");
    }
    Ok(())
}
