
use std::{fs::File, io::{stderr, Read, Write}, path::PathBuf, sync::OnceLock, time::Duration};

use anyhow::Ok;
use clap::{Parser, Subcommand, ValueEnum};
use notify::{Event, Watcher};
use rand::{RngCore, SeedableRng};
use twee_parser::{parse_archive, parse_html, parse_twee3, serde_json::Value, serialize_html, serialize_twee3, xmltree::EmitterConfig, Story};

const DEFAULT_CONFIG: &str = include_str!("../config.toml.default");
const DEFAULT_TWEE: &str = include_str!("../story.twee.default");
const DEFAULT_JS: &str = include_str!("../story.js.default");
const DEFAULT_CSS: &str = include_str!("../story.css.default");

static FORMAT_HARLOWE: OnceLock<String> = OnceLock::new();
static FORMAT_CHAPBOOK: OnceLock<String> = OnceLock::new();
static FORMAT_SNOWMAN: OnceLock<String> = OnceLock::new();
static FORMAT_SUGARCUBE: OnceLock<String> = OnceLock::new();

mod build;
use build::*;



/// A compiler for Twine Stories
/// 
/// For a complete reference, visit https://github.com/tareksander/twine-rs/blob/main/twee-tools/README.md
#[derive(Debug, Parser)]
#[command(version)]
struct Cli {
    
    #[command(subcommand)]
    command: Command
}


#[derive(Debug, Clone, Copy, ValueEnum)]
enum StoryFormat {
    Harlowe,
    Chapbook,
    Snowman,
    Sugarcube,
}


impl StoryFormat {
    fn format_name(&self) -> String {
        match self {
            StoryFormat::Harlowe => "Harlowe",
            StoryFormat::Chapbook => "Chapbook",
            StoryFormat::Snowman => "Snowman",
            StoryFormat::Sugarcube => "SugarCube",
        }.to_owned()
    }
    
    fn from_name(name: &str) -> anyhow::Result<Self> {
        Ok(match name {
            "Harlowe" => Self::Harlowe,
            "Chapbook" => Self::Chapbook,
            "Snowman" => Self::Snowman,
            "SugarCube" => Self::Sugarcube,
            _ => {
                return Err(Error::UnknownStoryFormat(name.to_string()).into());
            }
        })
    }
    
    fn format_version(&self) -> String {
        match self {
            StoryFormat::Harlowe => "3.3.8",
            StoryFormat::Chapbook => "1.2.3",
            StoryFormat::Snowman => "2.0.2",
            StoryFormat::Sugarcube => "2.36.1",
        }.to_string()
    }
    
    fn format_contents(&self) -> String {
        match self {
            StoryFormat::Harlowe => FORMAT_HARLOWE.get().unwrap().clone(),
            StoryFormat::Chapbook => FORMAT_CHAPBOOK.get().unwrap().clone(),
            StoryFormat::Snowman => FORMAT_SNOWMAN.get().unwrap().clone(),
            StoryFormat::Sugarcube => FORMAT_SUGARCUBE.get().unwrap().clone(),
        }
    }
    
}



#[derive(Debug, Subcommand)]
enum Command {
    /// Unpack a Twine HTML archive into .twee files
    Unpack {
        /// The file to unpack
        file: PathBuf,
        /// The directory to create the .twee files in
        #[arg(default_value = ".")]
        dir: String,
    },
    /// Decompiles a Twine HTML story into a .twee file
    Decompile {
        /// The file to decompile
        file: PathBuf,
        /// The file to write. Defaults to <story title>.twee
        out: Option<PathBuf>,
    },
    /// Initializes a new Twine project
    Init {
        /// The title of the story
        title: String,
        
        /// The story format to use
        format: StoryFormat,
        
        /// The directory to create the project in
        #[arg(default_value = ".")]
        dir: PathBuf,
    },
    
    /// Builds the Story in the current directory.
    Build {
        /// Enables the debug mode of the story format.
        #[arg(short, long)]
        debug: bool,
        
        /// Writes the HTML to standard output instead of the file in config.toml
        #[arg(short, long)]
        stdout: bool,
    },
    
    /// Builds the Story in the current directory on any changes.
    Watch {
        /// Enables the debug mode of the story format.
        #[arg(short, long)]
        debug: bool,
    },
}



type Result = anyhow::Result<(), anyhow::Error>;





fn unpack(file: PathBuf, dir: PathBuf) -> Result {
    if ! dir.exists() {
        return Err(Error::DirNotFound(dir.to_string_lossy().to_string()).into());
    }
    let mut file = if let std::result::Result::Ok(f) = File::open(&file) {
        f
    } else {
        return Err(Error::FileNotFound(file.to_string_lossy().to_string()).into());
    };
    let mut i = 0;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let archive = parse_archive(&content)?;
    for (story, warnings) in archive {
        for w in warnings {
            print_warning(w);
        }
        let title = if ! story.title.is_empty() {
                story.title.clone()
            } else {
                i += 1;
                String::from("story-") + &i.to_string()
            };
        let mut file = File::create(dir.join(title + ".twee"))?;
        file.write_all(serialize_twee3(&story).as_bytes())?
    }
    Ok(())
}

fn decompile(file: PathBuf, out: Option<PathBuf>) -> Result {
    let mut f = if let std::result::Result::Ok(f) = File::open(&file) {
        f
    } else {
        return Err(Error::FileNotFound(file.to_string_lossy().to_string()).into());
    };
    let mut content = String::new();
    f.read_to_string(&mut content)?;
    let (story, warnings) = parse_html(&content)?;
    for w in warnings {
        print_warning(w);
    }
    let title = if ! story.title.is_empty() {
        story.title.clone()
    } else {
        String::from("story")
    };
    let mut file = if let Some(out) = out {
        File::create(out)?
    } else {
        File::create(file.parent().unwrap().join(title + ".twee"))?
    };
    file.write_all(serialize_twee3(&story).as_bytes())?;
    Ok(())
}

fn gen_ifid() -> String {
    let mut r = rand::rngs::StdRng::from_entropy();
    let mut s = String::new();
    fn hex_bytes(b: &[u8]) -> String {
        b.iter().map(|b| format!("{:X}", b)).fold("".to_string(), |a, b| a + &b)
    }
    let mut uuid: [u8; 16] = [0; 16];
    r.fill_bytes(&mut uuid[..]);
    s += &hex_bytes(&uuid[0..4]);
    s += "-";
    s += &hex_bytes(&uuid[4..6]);
    s += "-";
    s += &hex_bytes(&uuid[6..8]);
    s += "-";
    s += &hex_bytes(&uuid[8..10]);
    s += "-";
    s += &hex_bytes(&uuid[10..16]);
    s
}

fn init(dir: PathBuf, format: StoryFormat, title: String) -> Result {
    if ! dir.exists() {
        return Err(Error::DirNotFound(dir.to_string_lossy().to_string()).into());
    }
    if dir.join("config.toml").exists() {
        writeln!(stderr(), "Project already initialized")?;
        return Ok(());
    }
    let mut story = parse_twee3(DEFAULT_TWEE).unwrap().0;
    story.title = title;
    story.meta.insert("ifid".to_string(), gen_ifid().into());
    story.meta.insert("format".to_string(), format.format_name().into());
    story.meta.insert("format-version".to_string(), format.format_version().into());
    
    fn write_file(path: PathBuf, contents: &str) -> Result {
        let mut f = File::create(path)?;
        f.write_all(contents.as_bytes())?;
        Ok(())
    }
    
    write_file(dir.join("story.css"), DEFAULT_CSS)?;
    write_file(dir.join("story.js"), DEFAULT_JS)?;
    write_file(dir.join("story.twee"), &serialize_twee3(&story))?;
    write_file(dir.join("config.toml"), DEFAULT_CONFIG)?;
    Ok(())
}




fn build(debug: bool) -> anyhow::Result<PathBuf> {
    if ! PathBuf::from("config.toml").exists() {
        return Err(Error::FileNotFound("config.toml".to_string()).into());
    }
    let config: Config = toml::from_str(&read_file("config.toml")?)?;
    let story = build_story(&config, debug)?;
    let format = {
        if let Some(Value::String(s)) = story.meta.get("format") {
            StoryFormat::from_name(s)?
        } else {
            writeln!(stderr(), "No story format")?;
            return Err(Error::UnknownStoryFormat("".to_string()).into());
        }
    };
    let out = if let Some(out) = config.output {
        PathBuf::from(out)
    } else {
        PathBuf::from(".").join(story.title.clone() + ".html")
    };
    let html = build_html(format, &story)?;
    File::create(out.clone())?.write_all(html.as_bytes())?;
    Ok(out)
}

fn build_html(format: StoryFormat, story: &Story) -> anyhow::Result<String> {
    let mut html: Vec<u8> = Vec::new();
    serialize_html(&story).write_with_config(&mut html, EmitterConfig {
        normalize_empty_elements: false,
        write_document_declaration: false,
        ..Default::default()})?;
    Ok(format.format_contents().replace("{{STORY_NAME}}", &story.title).replace("{{STORY_DATA}}", &String::from_utf8(html).unwrap()))
}

fn watch(debug: bool) -> Result {
    let mut out = build(debug)?.canonicalize()?;
    let mut w = notify::recommended_watcher(move |e: std::result::Result<Event, notify::Error>| {
        let event = e.unwrap();
        if event.paths.iter().any(|p| {
            if let std::result::Result::Ok(p) = p.canonicalize() {
                p == out
            } else {
                false
            }
        }) {
            return;
        }
        match event.kind {
            notify::EventKind::Modify(_m) => {
                out = build(debug).unwrap().canonicalize().unwrap();
            },
            notify::EventKind::Remove(_r) => {
                out = build(debug).unwrap().canonicalize().unwrap();
            },
            _ => {}
        }
        
    })?;
    w.configure(notify::Config::default().with_poll_interval(Duration::from_secs(1)))?;
    w.watch(&PathBuf::from("."), notify::RecursiveMode::Recursive)?;
    loop {}
}

fn main() -> Result {
    FORMAT_HARLOWE.set(serde_json::from_str::<serde_json::Value>(include_str!("../formats/harlowe-3.3.8.json")).unwrap().as_object().unwrap().get("source").unwrap().as_str().unwrap().to_string()).unwrap();
    FORMAT_CHAPBOOK.set(serde_json::from_str::<serde_json::Value>(include_str!("../formats/chapbook-1.2.3.json")).unwrap().as_object().unwrap().get("source").unwrap().as_str().unwrap().to_string()).unwrap();
    FORMAT_SNOWMAN.set(serde_json::from_str::<serde_json::Value>(include_str!("../formats/snowman-2.0.2.json")).unwrap().as_object().unwrap().get("source").unwrap().as_str().unwrap().to_string()).unwrap();
    FORMAT_SUGARCUBE.set(serde_json::from_str::<serde_json::Value>(include_str!("../formats/sugarcube-2.36.1.json")).unwrap().as_object().unwrap().get("source").unwrap().as_str().unwrap().to_string()).unwrap();
    
    let cli = Cli::parse();
    match cli.command {
        Command::Unpack { file, dir } => unpack(file, PathBuf::from(dir))?,
        Command::Decompile { file, out } => decompile(file, out)?,
        Command::Init { dir , format, title} => init(dir, format, title)?,
        Command::Build{debug, stdout} => {
            if stdout {
                if ! PathBuf::from("config.toml").exists() {
                    return Err(Error::FileNotFound("config.toml".to_string()).into());
                }
                let config: Config = toml::from_str(&read_file("config.toml")?)?;
                let story = build_story(&config, debug)?;
                let format = {
                    if let Some(Value::String(s)) = story.meta.get("format") {
                        StoryFormat::from_name(s)?
                    } else {
                        writeln!(stderr(), "No story format")?;
                        return Err(Error::UnknownStoryFormat("".to_string()).into());
                    }
                };
                std::io::stdout().write_all(build_html(format, &story)?.as_bytes())?;
            } else {
                build(debug)?;
            }
        },
        Command::Watch{debug} => watch(debug)?,
    }
    Ok(())
}
