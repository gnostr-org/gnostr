# HTML as Rust types

This library lets you represent HTML using Rust types, so that you can
construct HTML, manipulate it, and generate HTML code for browsers.

This crate aims to follow the WhatWG specification at
<https://html.spec.whatwg.org/>.

## Example

~~~rust
use html_page::{Document, Element, Tag};
let title = Element::new(Tag::Title).with_text("my page");
let doc = Document::default().with_head_element(&title);
assert_eq!(format!("{}", doc), "<!DOCTYPE html>\n<HTML>\n\
<HEAD><TITLE>my page</TITLE></HEAD>\n<BODY/>\n</HTML>\n");
~~~


## License

This library is released using the MIT licence. The copy of the
licence text is from <https://mit-license.org/> originally.

Copyright 2023  Lars Wirzenius

Permission is hereby granted, free of charge, to any person obtaining
a copy of this software and associated documentation files (the
“Software”), to deal in the Software without restriction, including
without limitation the rights to use, copy, modify, merge, publish,
distribute, sublicense, and/or sell copies of the Software, and to
permit persons to whom the Software is furnished to do so, subject to
the following conditions:

The above copyright notice and this permission notice shall be
included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

Fork this project to create your own MIT license that you can always
link to.
