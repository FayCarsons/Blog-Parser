# Blog Parser

This is an efficient, though currently very niche, markdown parser with some special features. It converts a markdown file into HTML, adds attributes defined in a local JSON file(of which there is an example in this repo), then encapsulates the HTML in a JSON object with all the relevant fields so I can easily serve and then parse my blog posts on he client side with minimal runtime cost. It also writes a Typescript file with a `Post` type declaration and an array of the title of each post, for easy fetching.

Its built on top of the `Markdown` rust crate and therefore supports all valid markdown syntax.

The Date must be specified in he first line of the file, in the format `Date: month day, year`. The next line should be a header. Anything can follow that though!