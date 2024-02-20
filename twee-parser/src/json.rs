use serde_json::{Map, Value};

use crate::{Error, Story, Warning};



pub fn parse_json(source: &str) -> Result<(Story, Vec<Warning>), Error> {
    let mut warnings = vec![];
    let mut v = serde_json::from_str::<Map<String, Value>>(source).map_err(|e| Error::JSONParseError(e))?;
    let title = v.remove("name");
    if title.is_none() {
        warnings.push(Warning::StoryTitleMissing);
    }
    let start = v.remove("start");
    let style = v.remove("style");
    let srcipt = v.remove("script");
    
    Ok((Story {
        title: if let Some(title) = title {
            serde_json::from_value::<String>(title.clone()).map_err(|e| Error::JSONParseError(e))?
        } else {
            "".to_owned()
        },
        passages: todo!(),
        meta: v
    }, warnings))
}







