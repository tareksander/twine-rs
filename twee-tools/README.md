# Twee-Tools

[![Crates.io Version](https://img.shields.io/crates/v/twee-tools)](https://crates.io/crates/twee-tools)
![Crates.io License](https://img.shields.io/crates/l/twee-tools)


A compiler for [Twine](https://twinery.org/) stories to HTML or HTML to Twee3. The installed command is `twee`.

Tools:

- `unpack`: Unpacks an archive into Twee files.
- `decompile`: Unpacks a Twine HTML file into a Twee file.
- `init`: Writes the default `config.toml` in the current directly, if there isn't already one, and sets up an example .twee, .js and .css file.
- `build`: Builds the story in the current directory using the `config.toml`. See the default config.toml for configuration options.
- `watch`: Builds the story and rebuilds on any change. You can use a web server with auto-refresh such as the [Live Server](https://marketplace.visualstudio.com/items?itemName=ritwickdey.LiveServer) Visual Studio Code extension to view the story easily during development.

To find out about a command's exact usage, use the -h or --help options.  
`build` and `watch` also accept a -d or --debug option, which turns on the story format's debug mode.

### Features

- Support for including files in passages, e.g. including an css file as the stylesheet and a js file as the story Javascript.
That means you get syntax highlighting and completion in your editor, which doesn't work inside the twee files.
- All of Twine's default story formats are bundled.
    - Currently, the Paperthin proofing format isn't supported
- By default all twee files in the directory are put together into the final story, so you can separate passages into multiple files for organization.


### Twee Format Extensions

To be fully backwards-compatible with the Twee format, all extensions are located in the metadata of the story or passages or in passages. The metadata is formatted in [JSON](https://www.w3schools.com/js/js_json_syntax.asp).  

#### Passage Metadata Commands/Actions:
- `"include"`: Discards the content of the passage in the Twee file and replaces it with the contents of the specified file, i.e `:: StoryInit {"include": "file.twee"}`. Arrays of paths are also supported. The paths support globbing.
- `"include-before"`: Same as `"include"`, but doesn't discard the contents and instead prepend the contents of the file to the passage.
- `"include-after"`: Same as `"include-before"`, but appends instead.
- `"prepend"`: Like `"include-before"`, but prepends the value instead of interpreting the value as a file path.
- "`append"`: Like `"prepend"`, but appends instead.

Beware: The extra commands are processed in-order, that is if you use `"include-before"` and then `"include"`, the result will just be `"include"`.

#### Command Passages

A passage with the tag `twee-cmd` will be interpreted as a series of commands to build the passage contents. This will happen before metadata actions like "prepend" are evaluated. Therefore this passage is also formatted as a JSON array.

- Strings will be included as-is.
- Commands are objects with specific properties:
    - `{ "include": "file" }`: functions the same as the "include" metadata action, but only inserts the content instead of replacing it.


#### TweeTools Passage
This passage can be used for additional includes. It is formatted as a JSON object like the StoryData passage.

- `"include"`: Includes a list of Twee files in the story. Paths from commands in the files will be interpreted relative to the file's directory. The files can even have their own TweeTools passages with includes. The compiler ensures that each twee file is only included once.



### Globbing

With globbing you can specify many files at one, by leaving wildcards in the path that then get resolved by searching for paths that match the pattern.  
E.g. `*.js` would match all files that end with `.js` in the current directory.  
  
Supported Wildcards:
- `*`: Matches all characters except the path separator, i.e. \ on Windows and / on most other OSs.
- `**`: Matches all subdirectories and the current directory. can only be used between path separators, i.e. `/**/`, `**/*.js`.
- `?`: Matches a single character except the path separator.
- `[]`: Matches any single character inside the brackets. Also supports ranges, i.e. `[0-9]` for all digits. Can contain `*` and `?` and will match the literal characters, no substitution. A `-` at the start or end matches a `-` literally.
- `[!]`: Matches anything *besides* the characters between the brackets(without the `!`). Also supports ranges.
- `[]]`: Matches the character `]`.


Note:
- The globbing for `twee` is case-sensitive, i.e. "Ab" will not match the filename "aB".
- Files and folders starting with `.` won't be matched if you don't include a `.`, e.g. `*/.*.js.` This is to avoid issues wih git and other editors maybe using hidden directories to store files with common format's you'd want to include.
- You can use \ or / as the path separator, it gets converted to the one your OS uses.

### Limitations

- Currently, the StoryData and StoryTitle passages have to be in the main Twee file.
- Currently, additional user-defined story formats aren't supported.

### License
This library is licensed under the MPL2.0.
