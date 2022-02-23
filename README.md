# oimdp-rs

Rust reimplementation of [oimdp](https://github.com/OpenITI/oimdp). Still a work in progress---the functions are all there, but tests need to be written.

I wanted to figure out if OpenITI mARkdown is worth investing in, and I learned a lot about the format by translating the parser from Python. So far, I'm not convinced. This feels brittle and un-ergonomic. The use of regular expressions was astounding to me.

What I like about normal Markdown (Pandoc being my preferred variant) is that it's in a "Goldilocks zone" of complexity. It's human-readable; it mostly stays out of my way; I can use it to generate rich text or HTML at a reasonable level; it's plain text and plays nicely with version control; it can be parsed, auto-formatted, linted, etc. It just feels right to me. If it were less---i.e., if it were plain-plain text---I wouldn't want to use it to write documents of any length or complexity. If, on the other hand, Markdown tried to do much more than it does... then I don't think the tradeoffs would make sense anymore. We reach a point at which it's simply better to separate the writing environment from the underlying document format---with the latter becoming at least semi-structured data. Of course, that brings huge advantages. It can also be frustrating, as we all know from using Word or managing blogs in WordPress.

The question I keep coming back to is about tradeoffs. There's this push and pull between writing experience and document format. It's really hard to find a happy place where the two can meet. Almost no system gets this right---except for Markdown, for those of us who can get used to it.

But what OpenITI mARkdown tries to accomplish is many steps too far. Parsing this is madness, let alone the idea that we should be directly manipulating it. Humans who want to view or edit OpenITI documents should instead be working with them in some kind of web app environment, and the actual data should be in JSON. That's just the reality. But maybe I could still be convinced otherwise...
