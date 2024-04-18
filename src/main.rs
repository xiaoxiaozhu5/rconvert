use std::fs;
use std::path::{Path, PathBuf};
use std::io::BufWriter;
use std::io::Write;

use solp::parse_file;
use solp::Consume;
use solp::api::Solution;

use quick_xml::events::{Event};
use quick_xml::reader::Reader;
use quick_xml::se::to_string;

use structopt::StructOpt;

mod field;
use crate::field::{File, MagicFolder, Project, Workspace};

struct Content {
    path: String,
    sources: Vec<String>,
    headers: Vec<String>,
}

struct MyConsumer {
    projects: Vec<Content>,
}

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
            //println!("name:{} url:{}", project.name, project_path);
            let mut reader = Reader::from_file(&project_path).unwrap();

            let mut ct = Content { path: project_path.clone(), sources: Vec::new(), headers: Vec::new() };
            let mut buffer = Vec::new();
            loop {
                match reader.read_event_into(&mut buffer) {
                    Err(error) => break println!("{}", error),
                    Ok(Event::Eof) => break println!("parse {} completed.", project_path),
                    Ok(Event::Start(node)) => {
                        //println!("{:?}", node.name());
                        for attr in node.attributes() {
                            match attr {
                                Ok(att) => {
                                    //println!("key:{:?} value:{:?}", String::from_utf8_lossy(att.key.0), String::from_utf8_lossy(&att.value));
                                    if node.name().as_ref() == b"ClInclude" {
                                        if String::from_utf8_lossy(att.key.0) == "Include" {
                                            ct.headers.push(String::from_utf8_lossy(&att.value).to_string());
                                        }
                                    } else if node.name().as_ref() == b"ClCompile" {
                                        if String::from_utf8_lossy(att.key.0) == "Include" {
                                            ct.sources.push(String::from_utf8_lossy(&att.value).to_string());
                                        }
                                    }
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
                        //println!("{:?}", node.name());
                        for attr in node.attributes() {
                            match attr {
                                Ok(att) => {
                                    //println!("key:{:?} value:{:?}", String::from_utf8_lossy(att.key.0), String::from_utf8_lossy(&att.value));
                                    if node.name().as_ref() == b"ClInclude" {
                                        if String::from_utf8_lossy(att.key.0) == "Include" {
                                            ct.headers.push(String::from_utf8_lossy(&att.value).to_string());
                                        }
                                    } else if node.name().as_ref() == b"ClCompile" {
                                        if String::from_utf8_lossy(att.key.0) == "Include" {
                                            ct.sources.push(String::from_utf8_lossy(&att.value).to_string());
                                        }
                                    }
                                }
                                Err(e) => { println!("error: {:?}", e)}
                            }
                        }
                    }
                    Ok(_) => { }
                }

                buffer.clear();
            }
            self.projects.push(ct);
        }
    }

    fn err(&self, path: &str) {
        println!("Error parsing file: {}", path);
    }
}

fn generate_workspace_file(allprojects : MyConsumer, dest: String, clean: bool) {
    let mut projects: Vec<Project> = Vec::new();
    for project in allprojects.projects.iter() {
        let sln_path = PathBuf::from(&project.path);
        let sln_parent_path = sln_path.parent().unwrap();
        let sln_name = sln_path.file_stem().unwrap();
        let ext = String::from(".pnproj");
        let name = sln_name.to_os_string().into_string().unwrap();
        let output_full_name = format!("{}{}", name, ext);
        let output_full_path = sln_parent_path.join(output_full_name);
        projects.push( Project { Path: Some(output_full_path.as_os_str().to_string_lossy().to_string()), name: None, File: None, MagicFolder: None } );
    }
    let workspace = Workspace::new(projects, None);
    let xml = to_string(&workspace).unwrap();
    // println!("{}", xml);

    let sln_path = PathBuf::from(&dest);
    let sln_parent_path = sln_path.parent().unwrap();
    let sln_name = sln_path.file_stem().unwrap();
    let ext = String::from(".pnws");
    let name = sln_name.to_os_string().into_string().unwrap();
    let output_full_name = format!("{}{}", name, ext);
    let output_full_path = sln_parent_path.join(output_full_name);
    if clean {
        fs::remove_file(&output_full_path).unwrap_or_else(|why| {
            println!("{} {:?}", output_full_path.display(), why.kind())
        });
    } else {
        let mut writer = BufWriter::new(std::fs::File::create(output_full_path).unwrap());
        writer.write_all(&xml.as_bytes()).unwrap();
        writer.flush().unwrap();
    }
}

#[derive(StructOpt)]
struct Cli {
    #[structopt(short, long, parse(from_os_str))]
    sln: PathBuf,
    #[structopt(short = "c", long = "clean", help = "remove generated pnws/pnproj")]
    clean: bool,
}

fn main() {
    let args = Cli::from_args();
    let path = args.sln.into_os_string().into_string().unwrap();
    let clean = args.clean;
    let mut con = MyConsumer { projects: Vec::new() };
    let _result = parse_file(&path, &mut con);
    for prj in &con.projects {
        //let name = prj.path;
        let name = PathBuf::from(&prj.path).file_stem().unwrap().to_string_lossy().to_string();
        let mut magic_folder: Vec<MagicFolder> = Vec::new();
        let mut source_files: Vec<File> = Vec::new();
        let mut header_files: Vec<File> = Vec::new();
        for hdr in &prj.headers {
            let path = Path::new(hdr);
            match path.parent() {
                Some(_p) => {
                    header_files.push( File { Path: path.file_name().unwrap().to_string_lossy().to_string()} );
                },
                None => {
                    header_files.push( File { Path: hdr.to_string()} );
                }
            }
        }
        for src in &prj.sources {
            let path = Path::new(src);
            match path.parent() {
                Some(_p) => {
                    source_files.push( File { Path: path.file_name().unwrap().to_string_lossy().to_string()} );
                },
                None => {
                    source_files.push( File { Path: src.to_string()} );
                }
            }
        }
        let source_folder = MagicFolder { exclude: String::from("CVS;.svn;.git;.vs"), filter: String::from("*.c;*.cpp;*.cc;*.cxx"), name: String::from("Source Files"), path: String::from(""), File: Some(source_files) };
        let header_folder = MagicFolder { exclude: String::from("CVS;.svn;.git;.vs"), filter: String::from("*.h;*.hpp;*.hxx"), name: String::from("Header Files"), path: String::from(""), File: Some(header_files) };
        magic_folder.push(header_folder);
        magic_folder.push(source_folder);
        let project = Project { name: Some(name), MagicFolder: Some(magic_folder), Path: None, File: None };
        let xml = to_string(&project).unwrap();
        let output_path = PathBuf::from(&prj.path);
        let output_name = output_path.file_stem().unwrap();
        let output_path = output_path.parent();
        match output_path {
            Some(p) => {
                let ext = String::from(".pnproj");
                let name = output_name.to_os_string().into_string().unwrap();
                let output_full_name = format!("{}{}", name, ext);
                let op = p.join(output_full_name);
                if clean {
                    fs::remove_file(&op).unwrap_or_else(|why| {
                        println!("{} {:?}", op.display(), why.kind())
                    });
                } else {
                    let mut writer = BufWriter::new(std::fs::File::create(op).unwrap());
                    writer.write_all(&xml.as_bytes()).unwrap();
                    writer.flush().unwrap();
                }
            },
            None => {}
        }
    }
    generate_workspace_file(con, path, clean);
}
