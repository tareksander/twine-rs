use std::{fs::File, io::{stderr, Read, Write}, path::{Path, PathBuf}};

use glob::MatchOptions;
use serde::Deserialize;
use serde_json::{Map, Value};
use thiserror::Error;
use twee_parser::{parse_archive, parse_twee3, Passage, Story, Warning};





#[derive(Deserialize)]
pub struct Config {
    pub output: Option<String>,
    pub style: Vec<String>,
    pub script: Vec<String>,
    pub main: String,
    pub prebuild: Vec<String>
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Could not open file: {0}")]
    FileNotFound(String),
    #[error("Could not open directory: {0}")]
    DirNotFound(String),
    #[error("Unknown story format: {0}")]
    UnknownStoryFormat(String),
    #[error("Prebuild command exited with error")]
    PrebuildError
}

pub(crate) fn read_file<P>(p: P) -> anyhow::Result<String>  where P: AsRef<Path> {
    let mut f = File::open(p)?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    Ok(s)  
}


pub(crate) fn print_warning(w: Warning) {
    writeln!(stderr(), "Warning: {}",
    match w {
        Warning::StoryMetadataMalformed => "Story metadata is not valid JSON and has been discarded.".to_owned(),
        Warning::StoryTitleMissing => "Story title is missing.".to_owned(),
        Warning::PassageMetadataMalformed(p) => format!("Passage \"{}\" metadata is not valid JSON and has been discarded.", p),
        Warning::PassageTagsMalformed(p) => format!("Passage \"{}\" tags are not valid and have been discarded.", p),
        Warning::PassageDuplicated(p) => format!("Passage \"{}\" is duplicated, using the last occurrence.", p),
        Warning::PassageNameMissing => "Passage name is missing, passage has been discarded.".to_owned(),
    }).unwrap();
}

fn glob(s: &str, parent: PathBuf) -> std::result::Result<Vec<PathBuf>, anyhow::Error> {
    let mut res = vec![];
    for r in glob::glob_with(s, MatchOptions {
        case_sensitive: true,
        require_literal_separator: true,
        require_literal_leading_dot: true,
    })? {
        if let Ok(r) = r {
            res.push(parent.join(r));
        }
    }
    Ok(res)
}

fn process_story_fragment(story: &mut Story, path: &Path, included: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    for p in &mut story.passages {
        if let Some(i) = p.tags.iter().position(|t| t == "twee-cmd") {
            p.tags.remove(i);
            if let Some(contents) = serde_json::from_str::<serde_json::Value>(&p.content)?.as_array() {
                p.content = String::new();
                for v in contents {
                    match v {
                        Value::String(s) => {
                            p.content += s;
                        },
                        Value::Object(m) => {
                            if let Some(s) = m.get("include").and_then(|i| i.as_str()) {
                                let files = glob(s, path.parent().unwrap().to_path_buf())?;
                                if files.len() == 0 {
                                    writeln!(stderr(), "Warning: No matching file found for pattern: {}", s)?;
                                }
                                for f in files {
                                    p.content += &read_file(&f)?;
                                }
                                continue;
                            }
                            writeln!(stderr(), "Warning: [twee-cmd] entry was not a recognized command and has been discarded")?;
                        }
                        _ => {
                            writeln!(stderr(), "Warning: [twee-cmd] entry was neither a string nor an object and has been discarded")?;
                        }
                    }
                }
            } else {
                writeln!(stderr(), "Warning: [twee-cmd] passage is not a JSON array and has been discarded")?;
            }
        }
        if let Some(Value::String(f)) = p.meta.get("include") {
            let files = glob(f, path.parent().unwrap().to_path_buf())?;
            if files.len() == 0 {
                writeln!(stderr(), "Warning: No matching file found for pattern: {}", f)?;
            }
            p.content = String::new();
            for f in files {
                p.content += &read_file(&f)?;
            }
            p.meta.remove("include");
        }
        if let Some(Value::Array(f)) = p.meta.get("include") {
            p.content = String::new();
            for f in f {
                if let Some(s) = f.as_str() {
                        let files = glob(s, path.parent().unwrap().to_path_buf())?;
                        if files.len() == 0 {
                            writeln!(stderr(), "Warning: No matching file found for pattern: {}", s)?;
                        }
                        for f in files {
                            p.content += &read_file(&f)?;
                        }
                } else {
                    writeln!(stderr(), "Warning: include entry wasn't a string and has been ignored: {}", serde_json::to_string(f)?)?;
                }
            }
            p.meta.remove("include");
        }
        if let Some(Value::String(f)) = p.meta.get("include-before") {
            p.content = read_file(f)? + &p.content;
            p.meta.remove("include-before");
        }
        if let Some(Value::String(f)) = p.meta.get("include-after") {
            p.content += &read_file(f)?;
            p.meta.remove("include-after");
        }
        if let Some(Value::String(f)) = p.meta.get("prepend") {
            p.content = f.clone() + &p.content;
            p.meta.remove("prepend");
        }
        if let Some(Value::String(f)) = p.meta.get("append") {
            p.content += &f;
            p.meta.remove("append");
        }
    }
    if let Some(p) = story.passages.iter().position(|p| p.name == "TweeTools") {
        let p = story.passages.remove(p);
        if let Some(contents) = serde_json::from_str::<serde_json::Value>(&p.content)?.as_object() {
            if let Some(includes) = contents.get("include").and_then(|i| i.as_array()) {
                for i in includes {
                    if let Some(s) = i.as_str() {
                        let files = glob(s, path.parent().unwrap().to_path_buf())?;
                        if files.len() == 0 {
                            writeln!(stderr(), "Warning: No matching file found for pattern: {}", s)?;
                        }
                        for twee in files {
                            if ! included.contains(&twee.canonicalize()?) {
                                let (mut part, warnings) = parse_twee3(&read_file(&twee)?)?;
                                for w in warnings {
                                    match &w {
                                        Warning::StoryMetadataMalformed => {},
                                        Warning::StoryTitleMissing => {},
                                        _ => print_warning(w)
                                    }
                                }
                                included.push(twee.canonicalize()?);
                                process_story_fragment(&mut part, &twee, included)?;
                            }
                        }
                    } else {
                        writeln!(stderr(), "Warning: include entry wasn't a string and has been ignored: {}", serde_json::to_string(i)?)?;
                    }
                }
            }
            if let Some(includes) = contents.get("include-archive").and_then(|i| i.as_array()) {
                for i in includes {
                    if let Some(f) = i.as_str() {
                        let f = PathBuf::from(f.to_string());
                        let stories = parse_archive(&read_file(&f)?)?;
                        for s in stories {
                            let (mut part, warnings) = s;
                            for w in warnings {
                                match &w {
                                    Warning::StoryMetadataMalformed => {},
                                    Warning::StoryTitleMissing => {},
                                    _ => print_warning(w)
                                }
                            }
                            included.push(f.canonicalize()?);
                            process_story_fragment(&mut part, &f, included)?;
                        }
                    } else {
                        writeln!(stderr(), "Warning: include entry wasn't a string and has been ignored: {}", serde_json::to_string(i)?)?;
                    }
                }
            }
        } else {
            writeln!(stderr(), "Warning: TweeTools passage is not a JSON object and has been discarded")?;
        }
    }
    Ok(())
}

pub fn build_story(config: &Config, debug: bool) -> Result<Story, anyhow::Error> {
    
    
    let twee = read_file(&config.main)?;
    let (mut story, warnings) = parse_twee3(&twee)?;
    if debug {
        story.meta.insert("options".to_string(), "debug".into());
    }
    for w in warnings {
        print_warning(w);
    }
    if story.title.is_empty() {
        story.title = "Story".to_string();
    }
    let mut included = vec![PathBuf::from(config.main.clone()).canonicalize()?];
    process_story_fragment(&mut story, Path::new(&config.main), &mut included)?;
    
    let mut i = 0;
    for f in &config.script {
        i += 1;
        story.passages.push(Passage {
            name: "script".to_string() + &i.to_string(),
            tags: vec!["script".to_string()],
            meta: Map::new(),
            content: read_file(f)?
        });
    }
    let mut i = 0;
    for f in &config.style {
        i += 1;
        story.passages.push(Passage {
            name: "stylesheet".to_string() + &i.to_string(),
            tags: vec!["stylesheet".to_string()],
            meta: Map::new(),
            content: read_file(f)?
        });
    }
    Ok(story)
}

