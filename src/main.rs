extern crate rls_analysis as analysis;

fn main() {
    let mut home = std::env::home_dir().unwrap();
    //TODO: dynamically load toolchain/target
    //TODO: dynamically load analysis directory
    home.push(".rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/analysis");

    println!("loading analysis data...");

    let host = analysis::AnalysisHost::new(analysis::Target::Release);
    host.reload(&home, &home, true).unwrap();

    println!("done!");

    while let Ok(input) = read_line() {
        let input = input.trim();
        if input.is_empty() {
            break;
        }

        let search = host.search_for_id(&input).unwrap();

        if search.is_empty() {
            println!("No results for \"{}\"", input);
            continue;
        }

        for result in search {
            if let Ok(def) = host.get_def(result) {
                // println!("{:#?}", def);

                if !def.docs.trim().is_empty() {
                    print!("{}: ", def.qualname);
                }

                let mut dox = String::new();
                for ln in def.docs.lines() {
                    if ln.trim().is_empty() {
                        break;
                    }
                    dox.push_str(ln);
                }

                if !dox.is_empty() {
                    println!("{}", dox.trim());
                }

                if let Ok(url) = host.doc_url(&def.span) {
                    println!("  {}", url);
                }
            } else {
                println!("unknown error searching for \"{}\"", input);
            }
        }
    }

    println!("");
}

fn read_line() -> std::io::Result<String> {
    use std::io::Write;

    println!("");
    print!("query (send empty to quit): ");
    std::io::stdout().flush().unwrap();

    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;

    Ok(line)
}
