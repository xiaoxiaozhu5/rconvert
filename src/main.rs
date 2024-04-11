use std::path::{Path, PathBuf};

use solp::parse_file;
use solp::Consume;
use solp::api::Solution;

use quick_xml::events::{Event};
use quick_xml::reader::Reader;

use structopt::StructOpt;

struct MyConsumer;

impl Consume for MyConsumer {
    fn ok(&mut self, solution: &Solution<'_>) {
        // println!("Successful parsing: {:?}", solution);
        //let serialized = serde_json::to_string_pretty(&solution).unwrap();
        //println!("{}", serialized);
        let full_path = Path::new(solution.path);
        let path = full_path.parent().unwrap();
        for project in solution.projects.iter() {
            //let project_path = path.join(project.path_or_uri);
            let mut project_path = format!("{}\\{}", path.display(), project.path_or_uri);
            if cfg!(not(target_os = "windows")) {
                let tmp_path = project_path.replace("\\", "/");
                project_path = tmp_path;
            }
            println!("name:{} url:{}", project.name, project_path);
            let mut reader = Reader::from_file(project_path).unwrap();

            let mut buffer = Vec::new();
            loop {
                match reader.read_event_into(&mut buffer) {
                    Err(error) => break println!("{}", error),
                    Ok(Event::Eof) => break println!("Completed."),
                    Ok(Event::Start(node)) => {
                        println!("{:?}", node.name());
                        for attr in node.attributes() {
                            match attr {
                                Ok(att) => {
                                    println!("key:{:?} value:{:?}", String::from_utf8_lossy(att.key.0), String::from_utf8_lossy(&att.value));
                                }
                                Err(e) => { println!("error: {:?}", e)}
                            }
                        }
                    }
                    Ok(Event::End(_)) => {
                    }
                    Ok(Event::Text(_)) => {
                    }
                    Ok(Event::Empty(node)) => {
                        println!("{:?}", node.name());
                        for attr in node.attributes() {
                            match attr {
                                Ok(att) => {
                                    println!("key:{:?} value:{:?}", String::from_utf8_lossy(att.key.0), String::from_utf8_lossy(&att.value));
                                }
                                Err(e) => { println!("error: {:?}", e)}
                            }
                        }
                    }
                    Ok(_) => { }
                }

                buffer.clear();
            }
        }
    }

    fn err(&self, path: &str) {
        println!("Error parsing file: {}", path);
    }
}

#[derive(StructOpt)]
struct Cli {
    #[structopt(short, long, parse(from_os_str))]
    sln: PathBuf,
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,
}

fn main() {
    let args = Cli::from_args();
    let path = args.sln.into_os_string().into_string().unwrap();
    let mut con = MyConsumer;
    let _result = parse_file(&path, &mut con);
}