use regex::RegexBuilder;

use crate::*;

#[derive(PartialEq, Eq)]
enum PassageState {
    Title,
    Tags,
    Between,
}

/// Parses Twee3 into a [Story].
pub fn parse_twee3(source: &str) -> Result<(Story, Vec<Warning>), Error> {
    let re = RegexBuilder::new("^::").multi_line(true).build().unwrap();
    let mut warnings = vec![];
    let mut passages: Vec<Passage> = Vec::new();
    let mut start = 0;
    let mut name = Vec::<char>::new();
    let mut tags = Vec::<String>::new();
    let mut meta: &str = "{}";
    let mut title = String::new();
    let mut story_meta = None;
    while let Some(a) = re.find_at(source, start) {
        if start != 0 {
            let name: String = name.iter().collect();
            let name = name.trim().to_string();
            if name.len() == 0 {
                warnings.push(Warning::PassageNameMissing);
            } else {
                let mut content = source[start..(a.start())].to_string();
                if content.starts_with("\\::") {
                    content.remove(0);
                }
                match name.as_str() {
                    "StoryTitle" => {
                        if title.len() != 0 {
                            warnings.push(Warning::PassageDuplicated("StoryTitle".to_string()));
                        }
                        title = content.trim().to_string();
                    },
                    "StoryData" => {
                        if story_meta.is_some() {
                            warnings.push(Warning::PassageDuplicated("StoryData".to_string()));
                        }
                        story_meta = if let Ok(v) = serde_json::from_str(meta) {
                            let v: Value = v;
                            match v {
                                Value::Object(o) => {
                                    Some(o)
                                },
                                _ => {
                                    warnings.push(Warning::StoryMetadataMalformed);
                                    Some(Map::new())
                                }
                            }
                        } else {
                            warnings.push(Warning::StoryMetadataMalformed);
                            Some(Map::new())
                        };
                    },
                    _ => {
                        let mut dup = false;
                        for p in &passages {
                            if p.name == name {
                                warnings.push(Warning::PassageDuplicated(p.name.clone()));
                                dup = true;
                                break;
                            }
                        }
                        if ! dup {
                            let meta = if let Ok(v) = serde_json::from_str(meta) {
                                let v: Value = v;
                                match v {
                                    Value::Object(o) => {
                                        o
                                    },
                                    _ => {
                                        warnings.push(Warning::PassageMetadataMalformed(name.clone()));
                                        Map::new()
                                    }
                                }
                            } else {
                                warnings.push(Warning::PassageMetadataMalformed(name.clone()));
                                Map::new()
                            };
                            passages.push(Passage { name, tags: tags.clone(), meta, content: content.trim_end().to_string()});
                        }
                    }
                }
            }
        }
        start = a.end();
        name.clear();
        tags.clear();
        meta = "{}";
        let mut tag = Vec::<char>::new();
        let mut state = PassageState::Title;
        let mut escape = false;
        for (i, c) in source[start..].char_indices() {
            if ['\r', '\n'].contains(&c) {
                break;
            }
            match state {
                PassageState::Title => {
                    if escape {
                        escape = false;
                        name.push(c);
                        continue;
                    }
                    if c == '[' {
                        state = PassageState::Tags;
                        continue;
                    }
                    if c == '{' {
                        meta = &source[if let Some(newline) = source[i..].find("\n") {
                            i..(i + newline)
                        } else {
                            i..source.len()
                        }];
                        break;
                    }
                    if c == '\\' {
                        escape = true;
                        continue;
                    }
                    name.push(c);
                },
                PassageState::Tags => {
                    if escape {
                        escape = false;
                        tag.push(c);
                        continue;
                    }
                    if c == '\\' {
                        escape = true;
                        continue;
                    }
                    if c == ']' {
                        if ! tag.is_empty() {
                            tags.push(tag.iter().collect());
                        }
                        state = PassageState::Between;
                        continue;
                    }
                    if c.is_whitespace() && ! tag.is_empty() {
                        tags.push(tag.iter().collect());
                        tag = vec![];
                    } else {
                        tag.push(c);
                    }
                },
                PassageState::Between => {
                    if c == '{' {
                        meta = &source[if let Some(newline) = source[i..].find("\n") {
                            i..(i + newline)
                        } else {
                            i..source.len()
                        }];
                        break;
                    }
                }
            }
        }
        if state == PassageState::Tags {
            warnings.push(Warning::PassageTagsMalformed(name.iter().collect()));
        }
        if ! tag.is_empty() {
            tags.push(tag.iter().collect());
        }
        if meta.trim().len() == 0 {
            meta = "{}";
        }
    }
    return Ok((Story {
        title,
        passages,
        meta: story_meta.unwrap_or(Map::new()),
    }, warnings));
}


/// Serializes a [Story] into Twee3.
pub fn serialize_twee3(story: &Story) -> String {
    let escape = |t: &String| {
        t.replace("\\", "\\\\")
        .replace("[", "\\[")
        .replace("]", "\\]")
        .replace("{", "\\{")
        .replace("}", "\\}")
    };
    
    let mut res: Vec<char> = Vec::new();
    res.extend(":: StoryTitle\n".chars());
    res.extend(escape(&story.title).chars());
    
    res.extend("\n\n:: StoryData\n".chars());
    res.extend(serde_json::to_string_pretty(&story.meta).unwrap().chars());
    res.extend("\n\n".chars());
    
    for p in &story.passages {
        res.extend("\n:: ".chars());
        res.extend(escape(&p.name).chars());
        if ! p.tags.is_empty() {
            res.extend(" [".chars());
            res.extend(p.tags.iter().map(escape).collect::<Vec<String>>().join(" ").chars());
            res.push(']');
        }
        if ! p.meta.is_empty() {
            res.extend(" {".chars());
            res.extend(serde_json::to_string(&p.meta).unwrap().chars());
            res.push('}');
        }
        res.push('\n');
        if p.content.starts_with("::") {
            res.push('\\');
        }
        res.extend(p.content.chars());
        res.push('\n');
    }
    return res.into_iter().collect();
}


