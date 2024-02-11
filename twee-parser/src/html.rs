use crate::*;

pub use xmltree::{Element, XMLNode, ParseError};


fn search_storydata(e: &Element) -> Option<Element> {
    if e.name == "tw-storydata" {
        return Some(e.clone());
    }
    for c in &e.children {
        if let Some(e) = c.as_element() {
            if e.name == "tw-storydata" {
                return Some(e.clone());
            } else {
                if let Some(e) = search_storydata(&e) {
                    return Some(e);
                }
            }
        }
    }
    return None;
}

/// Parses a Twine archive, a list of &lt;tw-storydata&gt; tags, into a list of [Story]s.
pub fn parse_archive(source: &str) -> Result<Vec<(Story, Vec<Warning>)>, Error> {
    let e = Element::parse_all(source.as_bytes()).map_err(|e| Error::HTMLParseError(e))?;
    return e.into_iter().map(|e| e.as_element().ok_or(Error::HTMLStoryDataNotFound).and_then(|e| parse_element(e))).collect();
}

/// Parses a published Twine HTML file into a [Story], looking for a &lt;tw-storydata&gt; tag.
pub fn parse_html(source: &str) -> Result<(Story, Vec<Warning>), Error> {
    let e = Element::parse(source.as_bytes()).map_err(|e| Error::HTMLParseError(e))?;
    let storydata = search_storydata(&e).ok_or(Error::HTMLStoryDataNotFound)?;
    return parse_element(&storydata);
}

fn parse_element(storydata: &Element) -> Result<(Story, Vec<Warning>), Error> {
    let mut warnings = vec![];
    let mut passages: Vec<Passage> = vec![];
    let mut tag_colors = Map::new();
    let mut elements = storydata.children.iter().filter_map(|c| {
        c.as_element()
    }).collect::<Vec<&Element>>();
    elements.sort_by(|a, b| {
        let a = a.attributes.get("pid").and_then(|p| u32::from_str_radix(p, 10).ok()).unwrap_or(u32::MAX);
        let b = b.attributes.get("pid").and_then(|p| u32::from_str_radix(p, 10).ok()).unwrap_or(u32::MAX);
        a.cmp(&b)
    });
    for n in elements {
        match n.name.as_str() {
            "tw-passagedata" => {
                let mut meta = Map::new();
                for a in &n.attributes {
                    meta.insert(a.0.clone(), Value::String(a.1.clone()));
                }
                meta.remove("pid");
                if let Some(name) = meta.remove("name") {
                    let tags = meta.remove("tags").and_then(|tags| {
                        Some(tags.as_str().unwrap().split_whitespace().map(|s| s.to_string()).collect())
                    }).unwrap_or(vec![]);
                    let p = Passage {
                        name: name.as_str().unwrap().to_string(),
                        tags,
                        meta,
                        content: n.get_text().unwrap().clone().to_string(),
                    };
                    passages.push(p);
                }
            },
            "style" => {
                if let Some(p) = passages.iter_mut().find(|p| p.name == "StoryStylesheet") {
                    if let Some(t) = n.get_text() {
                        p.content += "\n";
                        p.content += &t;
                    }
                } else {
                    if let Some(t) = n.get_text() {
                        let p = Passage {
                            name: "StoryStylesheet".to_string(),
                            tags: vec!["stylesheet".to_string()],
                            meta: Map::new(),
                            content: t.clone().to_string(),
                        };
                        passages.push(p);
                    }
                }
            },
            "script" => {
                if let Some(p) = passages.iter_mut().find(|p| p.name == "StoryScript") {
                    if let Some(t) = n.get_text() {
                        p.content += "\n";
                        p.content += &t;
                    }
                } else {
                    if let Some(t) = n.get_text() {
                        let p = Passage {
                            name: "StoryScript".to_string(),
                            tags: vec!["script".to_string()],
                            meta: Map::new(),
                            content: t.clone().to_string(),
                        };
                        passages.push(p);
                    }
                }
            },
            "tw-tag" => {
                if let (Some(name), Some(value)) = (n.attributes.get("name"), n.attributes.get("color")) {
                    tag_colors.insert(name.clone(), Value::String(value.clone()));
                }
            }
            _ => {}
        }
    }
    
    
    let mut meta = Map::new();
    for a in &storydata.attributes {
        meta.insert(a.0.clone(), Value::String(a.1.clone()));
    }
    let mut title = "".to_string();
    meta.remove("hidden");
    if let Some(t) = meta.remove("name") {
        title = t.as_str().unwrap().to_string();
    } else {
        warnings.push(Warning::StoryTitleMissing);
    }
    if let Some(s) = meta.remove("startnode") {
        if let Some(start) = s.as_str() {
            let start = start.to_string();
            if let Some(start) = storydata.children.iter().find(|c| c.as_element().and_then(|e| Some(e.attributes.get("pid") == Some(&start))).is_some()) {
                if let Some(name) = start.as_element().and_then(|e| e.attributes.get("name")) {
                    meta.insert("start".to_string(), Value::String(name.clone()));
                }
            }
        }
    }
    meta.insert("tag-colors".to_string(), Value::Object(tag_colors));
    
    return Ok((Story {
        title,
        passages,
        meta,
    }, warnings));
}

/// Serializes a [Story] into a &lt;tw-storydata&gt; tag.
pub fn serialize_html(story: &Story) -> Element {
    let mut storydata = Element::new("tw-storydata");
    storydata.attributes.insert("name".to_string(), story.title.clone());
    
    let stylesheet = "stylesheet".to_string();
    let script = "script".to_string();
    let mut pid = 1;
    for p in &story.passages {
        let mut e;
        if p.tags.contains(&stylesheet) {
            if let Some(e) = storydata.children.iter_mut().find(|e| e.as_element().and_then(|e| Some(e.name == "style")) == Some(true)) {
                let e = e.as_mut_element().unwrap();
                e.children.push(XMLNode::Text("\n".to_string()));
                e.children.push(XMLNode::Text(p.content.clone()));
                continue;
            }
            e = Element::new("style");
            e.attributes.insert("role".to_string(), "stylesheet".to_string());
            e.attributes.insert("id".to_string(), "twine-user-stylesheet".to_string());
            e.attributes.insert("type".to_string(), "text/twine-css".to_string());
            e.children.push(XMLNode::Text(p.content.clone()));
        } else {
            if p.tags.contains(&script) {
                if let Some(e) = storydata.children.iter_mut().find(|e| e.as_element().and_then(|e| Some(e.name == "script")) == Some(true)) {
                    let e = e.as_mut_element().unwrap();
                    e.children.push(XMLNode::Text("\n".to_string()));
                    e.children.push(XMLNode::Text(p.content.clone()));
                    continue;
                }
                e = Element::new("script");
                e.attributes.insert("role".to_string(), "script".to_string());
                e.attributes.insert("id".to_string(), "twine-user-script".to_string());
                e.attributes.insert("type".to_string(), "text/twine-javascript".to_string());
                e.children.push(XMLNode::Text(p.content.clone()));
            } else {
                e = Element::new("tw-passagedata");
                e.attributes.insert("pid".to_string(), pid.to_string());
                pid += 1;
                e.attributes.insert("name".to_string(), p.name.clone());
                e.attributes.insert("tags".to_string(), p.tags.join(" "));
                for m in &p.meta {
                    if let Some(v) = m.1.as_str() {
                        e.attributes.insert(m.0.clone(), v.to_string());
                    }
                }
                let content = p.content.clone();
                e.children.push(XMLNode::Text(content));
            }
        }
        storydata.children.push(XMLNode::Element(e));
    }
    
    
    for m in &story.meta {
        match m.0.as_str() {
            "start" => {
                if let Some(s) = m.1.as_str() {
                    let s = s.to_string();
                    if let Some(start) = storydata.children.iter().find(|c| c.as_element().and_then(|e| Some(e.attributes.get("name") == Some(&s))) == Some(true)) {
                        storydata.attributes.insert("startnode".to_string(), start.as_element().unwrap().attributes.get("pid").unwrap().clone());
                    }
                }
            },
            "tag-colors" => {
                if let Some(tags) = m.1.as_object() {
                    for t in tags {
                        if let Some(v) = t.1.as_str() {
                            let mut e = Element::new("tw-tag");
                            e.attributes.insert("name".to_string(), t.0.clone());
                            e.attributes.insert("color".to_string(), v.to_string());
                            storydata.children.insert(0, XMLNode::Element(e));
                        }
                    }
                }
            },
            _ => {
                if let Some(v) = &m.1.as_str() {
                    storydata.attributes.insert(m.0.clone(), v.to_string());
                }
            }
        }
    }
    return storydata;
}

