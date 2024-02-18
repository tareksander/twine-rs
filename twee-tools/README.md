# twee-tools

A compiler for [Twine](https://twinery.org/) stories to HTML, HTML to Twee3. The installed command is `twee`.

Tools:

- `unpack`: Unpacks an archive into Twee files.
- `decompile`: Unpacks a Twine HTML file into a Twee file.
- `init`: Writes the default `config.toml` in the current directly, if there isn't already one, and sets up an example .twee, .js and .css file.
- `build`: Builds the story in the current directory using the `config.toml`. See the default config.toml for configuration options.
- `watch`: Builds the story and rebuilds on any change. You can use a web server with auto-refresh such as the [Live Server](https://marketplace.visualstudio.com/items?itemName=ritwickdey.LiveServer) Visual Studio Code extension to view the story easily during development.


### Features

- Support for including files in passages, e.g. including an css file as the stylesheet and a js file as the story Javascript.
That means you get syntax highlighting and completion in your editor, which doesn't work inside the twee files.
- All of Twine's default story formats are bundled.
- By default all twee files in the directory are put together into the final story, so you can separate passages into multiple files for organization.


### Twee Format Extensions

To be fully backwards-compatible with the Twee format, all extensions are located in the metadata of the story or passages.  
Currently, all file paths are relative to the config.toml.

Passage metadata:
- "include": Discards the content of the passage in the Twee file and replaces it with the contents of the specified file, i.e `:: StoryInit {"include": "file.twee"}`.
- "include-before": Same as include, but doesn't discard the contents and instead prepend the contents of the file to the passage.
- "include-after": Same as include-before, but appends instead.




### License
This library is licensed under the MPL2.0.
