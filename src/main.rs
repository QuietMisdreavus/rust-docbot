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

        let elems = input.split("::").collect::<Vec<_>>();
        if elems.is_empty() {
            break;
        }

        let name = elems.last().unwrap();

        let search = host.search_for_id(name).unwrap();

        if search.is_empty() {
            println!("No results for \"{}\"", input);
            continue;
        }

        let def = if elems.len() == 1 {
            host.get_def(*search.first().unwrap()).unwrap()
        } else {
            let mut def = None;

            for result in &search {
                let def_guess = host.get_def(*result).unwrap();

                if let Some(p_id) = def_guess.parent {
                    let parent = if let Ok(p) = host.get_def(p_id) {
                        p
                    } else {
                        continue;
                    };

                    if Some(&*parent.name) == elems.iter().cloned().rev().skip(1).next() {
                        def = Some(def_guess);
                    }
                } else {
                    if Some(&*def_guess.qualname) == elems.iter().cloned().rev().skip(1).next() {
                        def = Some(def_guess);
                    }
                }
            }

            if let Some(def) = def {
                def
            } else {
                println!("No results for \"{}\"", input);
                continue;
            }
        };

        print_def(&def, &host);
    }

    println!("");
}

fn print_def(def: &analysis::Def, host: &analysis::AnalysisHost) {
    print!("{:?} {}: ", def.kind, def.qualname);

    let mut dox = String::new();
    for ln in def.docs.lines() {
        if ln.trim().is_empty() {
            break;
        }
        dox.push(' ');
        dox.push_str(ln.trim());
    }

    if !dox.is_empty() {
        println!("{}", dox.trim());
    }

    if let Ok(url) = host.doc_url(&def.span) {
        println!("  {}", url);
    }
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
