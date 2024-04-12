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

use serde::{Serialize, Deserialize};

struct content {
   path: String,
   sources: Vec<String>,
   headers: Vec<String>,
}

struct MyConsumer {
    projects: Vec<content>,
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
            println!("name:{} url:{}", project.name, project_path);
            let mut reader = Reader::from_file(&project_path).unwrap();

            let mut ct = content { path: project_path, sources: Vec::new(), headers: Vec::new() };
            let mut buffer = Vec::new();
            loop {
                match reader.read_event_into(&mut buffer) {
                    Err(error) => break println!("{}", error),
                    Ok(Event::Eof) => break println!("Completed."),
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

#[derive(Serialize, Deserialize)]
struct File {
    #[serde(rename = "@path")]
    pub Path: String,
}

#[derive(Serialize, Deserialize)]
struct MagicFolder {
    #[serde(rename = "@excludeFolders")]
    pub exclude: String,
    #[serde(rename = "@filter")]
    pub filter: String,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@path")]
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub File: Option<Vec<File>>,
}

#[derive(Serialize, Deserialize)]
struct Project {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@path")]
    pub Path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub File: Option<Vec<File>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub MagicFolder: Option<Vec<MagicFolder>>,
}

#[derive(Serialize, Deserialize)]
struct Workspace {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub File: Option<Vec<File>>,
    pub Project: Vec<Project>
}

impl Workspace {
    pub fn new(projects: Vec<Project>, files: Option<Vec<File>>) -> Self {
        Workspace {
            File: files,
            Project: projects,
        }
    }
}

fn generate_workspace_file(solution: &Solution<'_>) {
    let full_path = Path::new(solution.path);
    let path = full_path.parent().unwrap();

    let mut projects: Vec<Project> = Vec::new();
    for project in solution.projects.iter() {
        let mut project_path = format!("{}\\{}", path.display(), project.path_or_uri);
        if cfg!(not(target_os = "windows")) {
            let tmp_path = project_path.replace("\\", "/");
            project_path = tmp_path;
        }
        //println!("name:{} url:{}", project.name, project_path);
        projects.push( Project { Path: Some(project_path), name: None, File: None, MagicFolder: None } );
    }
    let workspace = Workspace::new(projects, None);
    let xml = to_string(&workspace).unwrap();
    // println!("{}", xml);

    let mut writer = BufWriter::new(std::fs::File::create("test.pnws").unwrap());
    writer.write_all(&xml.as_bytes()).unwrap();
    writer.flush().unwrap();
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
    let output = args.output.into_os_string().into_string().unwrap();
    let mut con = MyConsumer { projects: Vec::new() };
    let _result = parse_file(&path, &mut con);
    for prj in con.projects {
        let name = prj.path;
        let mut magic_folder: Vec<MagicFolder> = Vec::new();
        let mut source_files: Vec<File> = Vec::new();
        let mut header_files: Vec<File> = Vec::new();
        for hdr in prj.headers {
            header_files.push( File { Path: hdr} );
        }
        for src in prj.sources {
            source_files.push( File { Path: src} );
        }
        let mut source_folder = MagicFolder { exclude: String::from("CVS;.svn"), filter: String::from("*"), name: String::from("Source Files"), path: String::from(""), File: Some(source_files) };
        let mut header_folder = MagicFolder { exclude: String::from("CVS;.svn"), filter: String::from("*"), name: String::from("Header Files"), path: String::from(""), File: Some(header_files) };
        magic_folder.push(header_folder);
        magic_folder.push(source_folder);
        let project = Project { name: Some(name), MagicFolder: Some(magic_folder), Path: None, File: None };
        let xml = to_string(&project).unwrap();
        let mut writer = BufWriter::new(std::fs::File::create("test.pnproj").unwrap());
        writer.write_all(&xml.as_bytes()).unwrap();
        writer.flush().unwrap();
    }
}
