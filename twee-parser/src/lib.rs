//! # twee_parser
//! 
//! The [Story] and [Passage] structs describe a Twine story.  
//! They can be constructed by the user, or parsed using the parse_* functions.  
//! A [Story] can then be modified and serialized again using the serialize_* functions.  


pub use serde_json;
use serde_json::{Value, Map};

/// An in-memory representation of a Twine story.
#[derive(Debug, Clone)]
pub struct Story {
    /// The name of the story.
    pub title: String,
    /// The list of [Passage]s.
    pub passages: Vec<Passage>,
    /// The metadata.
    /// Please refer to the [specification](https://github.com/iftechfoundation/twine-specs/blob/master/twine-2-htmloutput-spec.md#story-data)
    /// for standard fields.  
    /// To be serializable to HTML, the values have to be strings, except tags, which are supported specifically.
    pub meta: Map<String, Value>,
}

/// Representation of a passage in a [Story].
#[derive(Debug, Clone)]
pub struct Passage {
    /// The name of the passage.
    pub name: String,
    /// The passage tags. Cannot contain spaces.
    pub tags: Vec<String>,
    /// The passage metadata.
    pub meta: Map<String, Value>,
    /// The text content of the passage.
    pub content: String,
}

/// Possible parsing errors.
#[derive(Error, Debug)]
pub enum Error {
    /// The xmltree library couldn't parse the data, or it doesn't have the right format.
    #[error("Could not parse HTML: {0}")]
    #[cfg(feature = "html")]
    HTMLParseError(ParseError),
    /// No &lt;tw-storydata&gt; tag was found.
    #[error("No tw-storydata tag found in HTML")]
    #[cfg(feature = "html")]
    HTMLStoryDataNotFound,
}

/// Possible warnings during parsing.  
/// Per specification, the parser is quite generous and generates many things as warnings instead of errors.
#[derive(Debug, Clone)]
pub enum Warning {
    /// The story metadata wasn't a valid JSON object.
    StoryMetadataMalformed,
    /// The story's title is missing.
    StoryTitleMissing,
    /// The passage metadata wasn't a valid inline JSON object.  
    /// The argument is the passage name.
    PassageMetadataMalformed(String),
    /// The passage tags weren't closed.  
    /// The argument is the passage name.
    PassageTagsMalformed(String),
    /// 2 passages with the same name were found.
    /// The argument is the passage name.
    PassageDuplicated(String),
    /// A passage is missing it's name.
    PassageNameMissing,
}

mod twee3;
use thiserror::Error;
pub use twee3::*;

#[cfg(feature = "html")]
mod html;
#[cfg(feature = "html")]
pub use html::*;
#[cfg(feature = "html")]


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn parse_twee() {
        let story = parse_twee3(include_str!("../test-data/Test Story.twee")).unwrap();
        assert!(story.1.len() == 0, "{:?}", story.1);
    }
}
