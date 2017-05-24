extern crate rls_analysis as analysis;
extern crate irc;

use std::collections::HashMap;

use irc::client::prelude::*;

fn main() {
    let mut home = std::env::home_dir().unwrap();
    //TODO: dynamically load toolchain/target
    //TODO: dynamically load analysis directory
    home.push(".rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/analysis");

    println!("loading analysis data...");

    let host = analysis::AnalysisHost::new(analysis::Target::Release);
    host.reload(&home, &home, true).unwrap();

    println!("done!");

    println!("loading prelude...");

    let prelude = prelude(&host);

    println!("done!");

    println!("connecting...");

    let irc_conf = Config::load("config.json").unwrap();
    let srv = IrcServer::from_config(irc_conf).unwrap();
    srv.identify().unwrap();

    println!("ready!");

    let my_nick = srv.config().nickname.as_ref().unwrap().as_str();

    for msg in srv.iter() {
        let msg = msg.unwrap();

        match msg.command {
            Command::JOIN(ref channel, _, _) => {
                if let &Some(ref prefix) = &msg.prefix {
                    if prefix.starts_with(my_nick) {
                        println!("Joined to {}", channel);
                    }
                }
            }
            Command::PRIVMSG(ref target, ref text) => {
                let text = text.trim();

                if let Some(nick) = msg.source_nickname() {
                    let (target, cmd): (&str, Option<&str>) = if target == my_nick {
                        if !text.trim().is_empty() {
                            (nick, Some(text))
                        } else {
                            (nick, None)
                        }
                    } else {
                        let cmd = if text.starts_with(my_nick) {
                            let text = &text[my_nick.len()..];
                            if text.starts_with(&[',', ':'][..]) && !text[1..].trim().is_empty() {
                                Some(text[1..].trim())
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        (target.as_str(), cmd)
                    };

                    if let Some(cmd) = cmd {
                        let result;

                        if let Some(def) = prelude.get(cmd.trim()) {
                            result = Some(def.clone());
                        } else {
                            result = find_def(cmd, &host);
                        }

                        if let Some(def) = result {
                            let text = format_def(&def, &host).unwrap();

                            srv.send_privmsg(target, &text).unwrap();
                        } else {
                            srv.send_privmsg(target, &format!("No results for \"{}\".", cmd.trim())).unwrap();
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

// some of the items in the prelude aren't actually the first things returned when just their name
// is searched, so let's make a quick lookup table for it
//
// this listing is manually copied from the prelude as of 1.17.0
fn prelude(host: &analysis::AnalysisHost) -> HashMap<&'static str, analysis::Def> {
    let prelude = ["marker::Copy", "marker::Sized", "marker::Send", "marker::Sync", "ops::Drop",
                   "ops::Fn", "ops::FnMut", "ops::FnOnce", "mem::drop", "boxed::Box",
                   "borrow::ToOwned", "clone::Clone", "cmp::PartialEq", "cmp::PartialOrd",
                   "cmp::Eq", "cmp::Ord", "convert::AsRef", "convert::AsMut", "convert::Into",
                   "convert::From", "default::Default", "iter::Iterator", "iter::Extend",
                   "iter::IntoIterator", "iter::DoubleEndedIterator", "iter::ExactSizeIterator",
                   "option::Option", "Option::Some", "Option::None", "result::Result",
                   "Result::Ok", "Result::Err", "slice::SliceConcatExt", "string::String",
                   "string::ToString", "vec::Vec"];

    let mut map = HashMap::new();

    for name in prelude.iter().cloned() {
        let def = find_def(name, host).expect("missing prelude item");

        let suffix = name.split("::").last().unwrap();

        map.insert(suffix, def);
    }

    map
}

fn find_def(input: &str, host: &analysis::AnalysisHost) -> Option<analysis::Def> {
    let input = input.trim();

    let elems = input.split("::").collect::<Vec<_>>();
    if elems.is_empty() {
        return None;
    }

    let name = elems.last().unwrap();

    let search = host.search_for_id(name).unwrap();

    if search.is_empty() {
        return None;
    }

    let mut def = None;

    if elems.len() == 1 {
        for result in &search {
            if let Ok(res) = host.get_def(*result) {
                def = Some(res);
                break;
            }
        }
    } else {
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
                    break;
                }
            } else {
                if elems.iter().cloned().rev().skip(1).next()
                        .map(|p| def_guess.qualname.contains(p)).unwrap_or(false) {
                    def = Some(def_guess);
                    break;
                }
            }
        }
    }

    def
}

fn format_def(def: &analysis::Def, host: &analysis::AnalysisHost) -> Result<String, std::fmt::Error> {
    use std::fmt::Write;

    let mut output = String::new();

    write!(output, "{:?} {}: ", def.kind, def.qualname)?;

    let mut dox = String::new();
    for ln in def.docs.lines() {
        if ln.trim().is_empty() && !dox.trim().is_empty() {
            break;
        }
        dox.push(' ');
        dox.push_str(ln.trim());
    }

    if !dox.is_empty() {
        write!(output, "{}", dox.trim())?;
    } else {
        write!(output, "(no docs available)")?;
    }

    if let Ok(url) = host.doc_url(&def.span) {
        write!(output, " - {}", url)?;
    }

    Ok(output)
}
