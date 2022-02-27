use anyhow::{anyhow, Result};
use once_cell::sync::OnceCell;
use regex::{Captures, Regex};

mod structures;
pub use crate::structures::*;

mod tags;
use crate::tags::*;

// Regex macro from once_cell

macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: OnceCell<Regex> = OnceCell::new();
        RE.get_or_init(|| Regex::new($re).unwrap())
    }};
}

// Smaller helper functions

fn split_keep<'a>(re: &Regex, text: &'a str) -> Vec<&'a str> {
    let mut result = Vec::new();
    let mut last = 0;

    // Is it safe that this function works based on the byte length of strings?
    // I guess it's probably ok in this context

    for (index, separator) in text.match_indices(re) {
        // Ok, we matched on a separator
        // First add to the result the text leading up to the separator
        // i.e., the text since "last"
        if last != index {
            result.push(&text[last..index]);
        }

        // Also add to the result the separator itself
        // And update "last" to just after the separator (I think)
        result.push(separator);
        last = index + separator.len();
    }

    // If some text remains between the end of the last separator and the end of the string,
    // also add that to the result
    if last < text.len() {
        result.push(&text[last..]);
    }

    result
}

fn remove_phrase_lv_tags(line: String) -> String {
    let mut text_only = line;

    // First strip tags that don't involve regex
    for tag in PHRASE_LV_TAGS {
        text_only = text_only.replace(tag, "");
    }

    // Then use this nasty regex to strip the remainder

    let yikes = regex!(
        r"(?x)
        (
            @YA\d{1,4}     | # Year 1 (age)
            @YD\d{1,4}     | # Year 2 (death)
            @YB\d{1,4}     | # Year 3 (birth)
            @YY\d{1,4}     | # Year 4 (other)
            @TOP\d{1,2}    | # Topological full
            @T\d{1,2}      | # Topological
            @PER\d{1,2}    | # Person full
            @P\d{1,2}      | # Person
            @SRC\d{1,2}    | # Source (?)
            @SOC\d{1,2}    | # SOC full (?)
            @S\d{1,2}      | # SOC (?)
            @([^@]+?)@([^_@]+?)_([^_@]+?)(_([^_@]+?))?@               | # OPEN_TAG_CUSTOM_GRP
            @([A-Z]{3})@([A-Z]{3,})@([A-Za-z]+)@(-@([0tf][ftalmr])@)? | # OPEN_TAG_AUTO_GRP
            PageV(\d+)P(\d+) # Page pattern
        )"
    );

    // We can also trim outside whitespace before returning
    text_only = yikes.replace_all(&text_only, "").trim().into();

    // Replace any occurrence of multiple spaces with one space
    // This comes up because of tag removal and was annoying me
    let multiple_spaces = regex!(r"\s{2,}");
    text_only = multiple_spaces.replace_all(&text_only, " ").into();

    text_only
}

// Line parsing function

fn parse_line(tagged_line: &str, kind: Option<LineType>, first_token: bool) -> Option<Line> {
    // Remove initial line marker
    let line = tagged_line.trim_start_matches(LINE);

    // Remove phrase-level tags (whatever that means)
    let without_tags = remove_phrase_lv_tags(line.to_owned());

    // This was weird: the Python library returns None here if there's nothing left
    // after stripping tags from the line. But that caused a problem for lines where
    // there's a page number tag (which we do want to parse) and nothing else
    if without_tags.is_empty() && !line.contains(PAGE) {
        return None;
    }

    // Create vec for line parts
    let mut parts: Vec<LinePart> = Vec::new();

    // If any first_token was indicated, add an Isnad part to the line
    // This is weird right now because the only kind of first_token that has been implemented
    // is Isnad. So I made the function argument into a bool for simplicity.
    // But I still don't understand what this is supposed to accomplish...
    if first_token {
        parts.push(LinePart::Isnad);
    }

    // Regex sadness

    let ungodly = regex!(
        r"(?x)
        (
            PageV\d+P\d+ | # Page number
            @[A-Z]{3}@[A-Z]{3,}@[A-Za-z]+@(?:-@[0tf][ftalmr]@)? | # OPEN_TAG_AUTO
            @[^@]+?@[^_@]+?_[^_@]+?(?:_[^_@]+?)?@               | # OPEN_TAG_CUSTOM
            %~%          | # HEMI
            Milestone300 | # MILESTONE
            @MATN@       | # MATN
            @HUKM@       | # HUKM
            \#\$\#FROM   | # ROUTE_FROM
            \#\$\#TOWA   | # ROUTE_TOWA
            \#\$\#DIST   | # ROUTE_DIST
            @YA\d{1,4}   | # Year 1 (age)
            @YD\d{1,4}   | # Year 2 (death)
            @YB\d{1,4}   | # Year 3 (birth)
            @YY\d{1,4}   | # Year 4 (other)
            @TOP\d{1,2}  | # Topological full (?)
            @T\d{1,2}    | # Topological (?)
            @PER\d{1,2}  | # Person full (?)
            @P\d{1,2}    | # Person (?)
            @SRC\d{1,2}  | # Source (?)
            @SOC\d{1,2}  | # SOC full (?)
            @S\d{1,2}      # SOC (?)
        )"
    );

    // Split the line on tokens using the mega-regex, while keeping the tokens
    // This requires a custom function in Rust, unlike in Python
    let tokens = split_keep(ungodly, line);

    // More regex for the upcoming loop

    let open_tag_custom_pattern_grouped = regex!("^@([^@]+?)@([^_@]+?)_([^_@]+?)(_([^_@]+?))?@");
    let open_tag_auto_pattern_grouped =
        regex!("^@([A-Z]{3})@([A-Z]{3,})@([A-Za-z]+)@(-@([0tf][ftalmr])@)?");

    let page_pattern = regex!(r"PageV(\d+)P(\d+)");

    // When we come upon a tag for a "named entity," we can set this variable to indicate how
    // many of the *following* words (i.e., how much of the next token, I guess) to set aside
    // as the text of that entity
    // I can't make it work in Rust quite like it does in Python, though
    let mut include_words: u32 = 0;

    // Given the changes I've made, I also want to indicate entity type for NamedEntityText
    let mut entity_type: Option<EntityType> = None;

    // Iterate over line tokens
    // Basically, a token could be a tag, or any text falling between two tags
    // We've already split the line on tags as separators, while keeping those tags
    for token in tokens {
        // Again, let's start by trimming whitespace, and use this version henceforth
        // This is not done in the Python library, but I prefer it
        let token_trimmed = token.trim();

        // I guess this would mean a content-less line? How would that happen at this point?
        if token_trimmed.is_empty() {
            continue;
        }

        // Capture "open tag custom" or "open tag auto" (whatever that means)

        let mut opentag_captures: Option<Captures> = None;
        let mut opentagauto_captures: Option<Captures> = None;

        if token_trimmed.starts_with('@') {
            opentag_captures = open_tag_custom_pattern_grouped.captures(token_trimmed);
            opentagauto_captures = open_tag_auto_pattern_grouped.captures(token_trimmed);
        }

        // Here begin the if/else branches that take up the rest of the function

        // Page number
        if token_trimmed.contains(PAGE) {
            let page_captures = page_pattern.captures(token_trimmed);

            if let Some(page_matches) = page_captures {
                let vol = page_matches[1].into();
                let page = page_matches[2].into();

                parts.push(LinePart::PageNumber(PageNumber { vol, page }));
            } else {
                // An exception is raised here in the Python library; I haven't done anything
            }
        // "Open tag custom" (?)
        } else if let Some(opentag_matches) = opentag_captures {
            let user = opentag_matches[1].into();
            let t_type = opentag_matches[2].into();
            let t_subtype = opentag_matches[3].into();
            let t_subsubtype = opentag_matches[5].into();

            parts.push(LinePart::OpenTagUser {
                user,
                t_type,
                t_subtype,
                t_subsubtype,
            });
        // "Open tag auto" (?)
        } else if let Some(opentagauto_matches) = opentagauto_captures {
            let resp = opentagauto_matches[1].into();
            let t_type = opentagauto_matches[2].into();
            let category = opentagauto_matches[3].into();
            let review = opentagauto_matches[5].into();

            parts.push(LinePart::OpenTagAuto {
                resp,
                t_type,
                category,
                review,
            });
        // Hemistich
        } else if token_trimmed.contains(HEMI) {
            parts.push(LinePart::Hemistich {
                orig: token_trimmed.into(),
            });
        // "Milestone" (used to break up texts into manageable units)
        } else if token_trimmed.contains(MILESTONE) {
            parts.push(LinePart::Milestone);
        // Matn (?)
        } else if token_trimmed.contains(MATN) {
            parts.push(LinePart::Matn);
        // Ḥukm (?)
        } else if token_trimmed.contains(HUKM) {
            parts.push(LinePart::Hukm);
        // Route from
        } else if token_trimmed.contains(ROUTE_FROM) {
            parts.push(LinePart::RouteFrom);
        // Route toward
        } else if token_trimmed.contains(ROUTE_TOWA) {
            parts.push(LinePart::RouteTowa);
        // Route distance (?)
        } else if token_trimmed.contains(ROUTE_DIST) {
            parts.push(LinePart::RouteDist);
        // Year of birth
        } else if token_trimmed.contains(YEAR_BIRTH) {
            let value = token_trimmed.trim_start_matches(YEAR_BIRTH).into();
            let date_type = DateType::Birth;

            parts.push(LinePart::Date { value, date_type });
        // Year of death
        } else if token_trimmed.contains(YEAR_DEATH) {
            let value = token_trimmed.trim_start_matches(YEAR_DEATH).into();
            let date_type = DateType::Death;

            parts.push(LinePart::Date { value, date_type });
        // Other year
        } else if token_trimmed.contains(YEAR_OTHER) {
            let value = token_trimmed.trim_start_matches(YEAR_OTHER).into();
            let date_type = DateType::Other;

            parts.push(LinePart::Date { value, date_type });
        // Age
        } else if token_trimmed.contains(YEAR_AGE) {
            let value = token_trimmed.trim_start_matches(YEAR_AGE).into();
            parts.push(LinePart::Age { value });
        // Source
        } else if token_trimmed.contains(SRC) {
            // This should yield a string representation of a two-digit number
            let val = token_trimmed.trim_start_matches(SRC);
            let mut iter = val.chars();

            // Now make an int out of each char. This is so f'ing janky
            let prefix_char = iter.next().unwrap();
            let prefix: u32 = prefix_char.to_digit(10).unwrap();

            let extent_char = iter.next().unwrap();
            let extent: u32 = extent_char.to_digit(10).unwrap();

            // I guess this is the number of words to put into this iteration's
            // text field in the next iteration? Yikes
            // Literally can't figure out how to do that in Rust
            // I'm taking a different approach: the subsequent LinePart will be
            // NamedEntityText
            include_words = extent;
            entity_type = Some(EntityType::Src);

            parts.push(LinePart::NamedEntity {
                prefix,
                extent,
                ne_type: EntityType::Src,
            });
        // Not sure what SOC means
        } else if token_trimmed.starts_with(SOC_FULL) {
            let val = token_trimmed.trim_start_matches(SOC_FULL);
            let mut iter = val.chars();

            let prefix_char = iter.next().unwrap();
            let prefix: u32 = prefix_char.to_digit(10).unwrap();

            let extent_char = iter.next().unwrap();
            let extent: u32 = extent_char.to_digit(10).unwrap();

            include_words = extent;
            entity_type = Some(EntityType::Soc);

            parts.push(LinePart::NamedEntity {
                prefix,
                extent,
                ne_type: EntityType::Soc,
            });
        // Again SOC...
        } else if token_trimmed.starts_with(SOC) {
            let val = token_trimmed.trim_start_matches(SOC);
            let mut iter = val.chars();

            let prefix_char = iter.next().unwrap();
            let prefix: u32 = prefix_char.to_digit(10).unwrap();

            let extent_char = iter.next().unwrap();
            let extent: u32 = extent_char.to_digit(10).unwrap();

            include_words = extent;
            entity_type = Some(EntityType::Soc);

            parts.push(LinePart::NamedEntity {
                prefix,
                extent,
                ne_type: EntityType::Soc,
            });
        // Topological entity (I think)
        } else if token_trimmed.starts_with(TOP_FULL) {
            let val = token_trimmed.trim_start_matches(TOP_FULL);
            let mut iter = val.chars();

            let prefix_char = iter.next().unwrap();
            let prefix: u32 = prefix_char.to_digit(10).unwrap();

            let extent_char = iter.next().unwrap();
            let extent: u32 = extent_char.to_digit(10).unwrap();

            include_words = extent;
            entity_type = Some(EntityType::Top);

            parts.push(LinePart::NamedEntity {
                prefix,
                extent,
                ne_type: EntityType::Top,
            });
        // Again topological entity
        } else if token_trimmed.starts_with(TOP) {
            let val = token_trimmed.trim_start_matches(TOP);
            let mut iter = val.chars();

            let prefix_char = iter.next().unwrap();
            let prefix: u32 = prefix_char.to_digit(10).unwrap();

            let extent_char = iter.next().unwrap();
            let extent: u32 = extent_char.to_digit(10).unwrap();

            include_words = extent;
            entity_type = Some(EntityType::Top);

            parts.push(LinePart::NamedEntity {
                prefix,
                extent,
                ne_type: EntityType::Top,
            });
        // Person (?)
        } else if token_trimmed.starts_with(PER_FULL) {
            let val = token_trimmed.trim_start_matches(PER_FULL);
            let mut iter = val.chars();

            let prefix_char = iter.next().unwrap();
            let prefix: u32 = prefix_char.to_digit(10).unwrap();

            let extent_char = iter.next().unwrap();
            let extent: u32 = extent_char.to_digit(10).unwrap();

            include_words = extent;
            entity_type = Some(EntityType::Per);

            parts.push(LinePart::NamedEntity {
                prefix,
                extent,
                ne_type: EntityType::Per,
            });
        // Again person
        } else if token_trimmed.starts_with(PER) {
            let val = token_trimmed.trim_start_matches(PER);
            let mut iter = val.chars();

            let prefix_char = iter.next().unwrap();
            let prefix: u32 = prefix_char.to_digit(10).unwrap();

            let extent_char = iter.next().unwrap();
            let extent: u32 = extent_char.to_digit(10).unwrap();

            include_words = extent;
            entity_type = Some(EntityType::Per);

            parts.push(LinePart::NamedEntity {
                prefix,
                extent,
                ne_type: EntityType::Per,
            });
        } else if include_words > 0 {
            // This block becomes active if we assigned a new value to include_words
            // That would mean that there is some NamedEntity that has been added
            // The idea, again, is that we take a number of words *after* the tag introducing
            // the NamedEntity, and, in the following iteration, add those words as the
            // text field of the NamedEntity. I don't see how this can be done in Rust with
            // static typing, the borrow checker, etc.
            // So I gave up and changed how this works. We instead add a NamedEntityText object
            // that should occur just after the NamedEntity object. And we can still capture the
            // correct number of words.
            let mut entity = String::new();
            let mut rest = String::new();

            let words: Vec<&str> = token_trimmed.split(' ').collect();

            for (pos, word) in words.iter().rev().enumerate() {
                if pos < (include_words as usize) {
                    entity.push_str(word);
                    entity.push(' ');
                } else {
                    rest.push_str(word);
                    rest.push(' ');
                }
            }

            if !entity.is_empty() {
                parts.push(LinePart::NamedEntityText {
                    text: entity.trim().into(),
                    ne_type: entity_type.unwrap(),
                });
            }

            if !rest.is_empty() {
                parts.push(LinePart::TextPart {
                    text: rest.trim().into(),
                });
            }

            // Reset include_words to 0, entity_type to None
            include_words = 0;
            entity_type = None;
        } else {
            // If we made it to this point and no tag or anything else matched,
            // we can just add it to the line as textual content
            parts.push(LinePart::TextPart {
                text: token.trim().into(),
            });
        }
    }

    // Set up return value

    // If a line type was passed in to the function, use it
    // Otherwise assume normal "line"
    let line_type = if let Some(specified) = kind {
        specified
    } else {
        LineType::Normal
    };

    // Determine text_only field
    let text_only = if !without_tags.is_empty() {
        Some(without_tags)
    } else {
        None
    };

    // I've tried to match the Python library here, in particular using the
    // "line" variable for the orig field
    let line_struct = Line {
        text_only,
        parts,
        line_type,
    };

    Some(line_struct)
}

// Main parser function

pub fn parser(input: String) -> Result<Document> {
    // This is our return value, gods willing
    let mut doc = Document {
        magic_value: String::new(),
        simple_metadata: Vec::new(),
        content: Vec::new(),
    };

    // Regexes. It would probably be ok to skip the once_cell approach here, but whatever
    let page_pattern = regex!(r"PageV(\d+)P(\d+)");
    let morpho_pattern = regex!("#~:([^:]+?):");
    let para_pattern = regex!("^#($|[^#])");
    let bio_pattern = regex!("### $[^#]");
    let region_pattern =
        regex!(r"(#$#PROV|#$#REG\d) .*? #$#TYPE .*? (#$#REG\d|#$#STTL) ([\w# ]+) $");

    // Main loop
    for (i, line) in input.lines().enumerate() {
        // Start by trimming whitespace. This version is all we'll use henceforth
        let line_trimmed = line.trim();

        // Check for magic value
        if i == 0 && line_trimmed.starts_with("######OpenITI#") {
            doc.magic_value = line_trimmed.into();

            // Need to specify continue here; but everything that follows is if/else
            continue;
        } else if i == 0 {
            // If it's the first line and doesn't start with the magic value, abort
            return Err(anyhow!(
                "This does not appear to be an OpenITI mARkdown document"
            ));
        }

        // Non-machine-readable metadata
        if line_trimmed.starts_with(META) {
            // I guess the metadata ending tag gets dropped in parsing
            if line_trimmed == META_END {
                continue;
            }

            // Much trimming!
            let value = line_trimmed.trim_start_matches(META).trim().into();
            doc.simple_metadata.push(value);
        // Page number (not sure why this would happen)
        } else if line_trimmed.starts_with(PAGE) {
            // Try to capture volume and page numbers
            if let Some(cap) = page_pattern.captures(line_trimmed) {
                let vol = cap[1].into();
                let page = cap[2].into();

                doc.content
                    .push(Content::PageNumber(PageNumber { vol, page }));
            } else {
                // An exception is raised here in the Python library; I haven't done anything
            }
        // Riwāya
        } else if line_trimmed.starts_with(RWY) {
            // First add the whole line
            doc.content.push(Content::Paragraph {
                orig: line_trimmed.into(),
                para_type: ParaType::Riwayat,
            });

            // Then parse everything after the riwāya tag
            let double_trimmed = line_trimmed.trim_start_matches(RWY);
            let first_line = parse_line(double_trimmed, None, true);

            if let Some(first_line_content) = first_line {
                doc.content.push(Content::Line(first_line_content));
            }
        // Route from
        } else if line_trimmed.starts_with(ROUTE_FROM) {
            let kind = LineType::RouteOrDistance;
            let parsed_line = parse_line(line_trimmed, Some(kind), false);

            if let Some(parsed_line_content) = parsed_line {
                doc.content.push(Content::Line(parsed_line_content));
            }
        // Morphological pattern
        } else if let Some(cap) = morpho_pattern.captures(line_trimmed) {
            let category = cap[1].into();

            doc.content.push(Content::MorphologicalPattern {
                orig: line_trimmed.into(),
                category,
            });
        // Paragraph
        } else if para_pattern.is_match(line_trimmed) {
            // This line will be parsed without the initial paragraph marker
            let no_marker = &line_trimmed[1..];

            // If line contains hemistich marker (which can occur in the middle)...
            if line_trimmed.contains(HEMI) {
                let kind = LineType::Verse;
                let verse_parsed = parse_line(no_marker, Some(kind), false);

                if let Some(verse_content) = verse_parsed {
                    doc.content.push(Content::Line(verse_content));
                }
            } else {
                doc.content.push(Content::Paragraph {
                    orig: line_trimmed.into(),
                    para_type: ParaType::Normal,
                });

                let first_line = parse_line(no_marker, None, false);
                if let Some(first_line_content) = first_line {
                    doc.content.push(Content::Line(first_line_content));
                }
            }
        // Line
        } else if line_trimmed.starts_with(LINE) {
            let parsed_line = parse_line(line_trimmed, None, false);

            if let Some(parsed_line_content) = parsed_line {
                doc.content.push(Content::Line(parsed_line_content));
            }
        // Editorial (whatever that means)
        } else if line_trimmed.starts_with(EDITORIAL) {
            doc.content.push(Content::Editorial);
        // Heading
        } else if line_trimmed.starts_with(HEADER1) {
            // I think "value" means the actual heading content, minus the tag
            let mut value = line_trimmed.to_owned();

            for tag in HEADERS {
                value = value.replace(tag, "");
            }

            value = remove_phrase_lv_tags(value);

            // The following comment is copied from the Python library
            // TODO: capture tags as PhraseParts

            // Now we determine the heading level
            let mut level: u32 = 1;

            if line_trimmed.contains(HEADER5) {
                level = 5;
            } else if line_trimmed.contains(HEADER4) {
                level = 4;
            } else if line_trimmed.contains(HEADER3) {
                level = 3;
            } else if line_trimmed.contains(HEADER2) {
                level = 2;
            }

            doc.content.push(Content::SectionHeader { value, level });
        // Dictionary content (?)
        } else if line_trimmed.starts_with(DIC) {
            // Strip tags
            let mut no_tag = line_trimmed.to_owned();
            for tag in DICTIONARIES {
                no_tag = no_tag.replace(tag, "");
            }

            // Parse stripped line
            let first_line = parse_line(&no_tag, None, false);

            // Determine dictionary content type
            let dic_type = if line_trimmed.contains(DIC_LEX) {
                DicType::Lex
            } else if line_trimmed.contains(DIC_NIS) {
                DicType::Nis
            } else if line_trimmed.contains(DIC_TOP) {
                DicType::Top
            } else {
                DicType::Bib
            };

            // Add dictionary unit
            doc.content.push(Content::DictionaryUnit {
                orig: line_trimmed.into(),
                dic_type,
            });

            // If there was other line content, add that
            if let Some(first_line_content) = first_line {
                doc.content.push(Content::Line(first_line_content));
            }
        // Doxographical content (?)
        } else if line_trimmed.starts_with(DOX) {
            // Strip tags
            let mut no_tag = line_trimmed.to_owned();
            for tag in DOXOGRAPHICAL {
                no_tag = no_tag.replace(tag, "");
            }

            // Parse stripped line
            let first_line = parse_line(&no_tag, None, false);

            // Determine doxographical content type
            let dox_type = if line_trimmed.contains(DOX_SEC) {
                DoxType::Sec
            } else {
                DoxType::Pos
            };

            // Add doxographical item
            doc.content.push(Content::DoxographicalItem {
                orig: line_trimmed.into(),
                dox_type,
            });

            // If there was other line content, add that
            if let Some(first_line_content) = first_line {
                doc.content.push(Content::Line(first_line_content));
            }
        // Biographical item
        } else if bio_pattern.is_match(line_trimmed)
            || line_trimmed.starts_with(BIO)
            || line_trimmed.starts_with(EVENT)
        {
            // Strip tags
            let mut no_tag = line_trimmed.to_owned();
            for tag in BIOS_EVENTS {
                no_tag = no_tag.replace(tag, "");
            }

            // Parse stripped line
            let first_line = parse_line(&no_tag, None, false);

            // Determine type of biographical item
            let be_type =
                if line_trimmed.contains(LIST_NAMES_FULL) || line_trimmed.contains(LIST_NAMES) {
                    BeType::Names
                } else if line_trimmed.contains(BIO_REF_FULL) || line_trimmed.contains(BIO_REF) {
                    BeType::Ref
                } else if line_trimmed.contains(BIO_WOM_FULL) || line_trimmed.contains(BIO_WOM) {
                    BeType::Wom
                } else if line_trimmed.contains(LIST_EVENTS) {
                    BeType::Events
                } else if line_trimmed.contains(EVENT) {
                    BeType::Event
                } else {
                    BeType::Man
                };

            // Add biographical item
            doc.content.push(Content::BioOrEvent {
                orig: line_trimmed.into(),
                be_type,
            });

            // If there was other line content, add that
            if let Some(first_line_content) = first_line {
                doc.content.push(Content::Line(first_line_content));
            }
        // Region
        } else if region_pattern.is_match(line_trimmed) {
            doc.content.push(Content::AdministrativeRegion {
                orig: line_trimmed.into(),
            });
        } else {
            // Can just no-op this (which I'm sure the compiler does anyway)
            // continue;
        }
    }

    Ok(doc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;
    use std::fs;

    static PARSED: Lazy<Document> = Lazy::new(|| {
        let full_text = fs::read_to_string("test.md").unwrap();
        let text_parsed = parser(full_text).unwrap();

        text_parsed
    });

    #[test]
    fn heading_five() {
        let content = &PARSED.content;

        // Level 5 heading (orig, text, level)
        assert_eq!(
            content[54].as_section_header().unwrap(),
            (&"(نهج ابن هشام في هذا الكتاب) :".to_string(), &5u32)
        );
    }

    #[test]
    fn heading_one() {
        let content = &PARSED.content;

        // Level 1 heading (orig, text, level)
        assert_eq!(
            content[50].as_section_header().unwrap(),
            (
                &"ذكر سرد النسب الزكي من محمد صلى الله عليه وآله وسلم، إلى آدم عليه السلام"
                    .to_string(),
                &1u32
            )
        );
    }

    #[test]
    fn isnad_matn() {
        let content = &PARSED.content;

        if let Content::Line(Line {
            text_only: _,
            parts,
            line_type,
        }) = &content[47]
        {
            assert!(line_type.is_normal());
            assert!(parts[0].is_isnad());

            assert_eq!(
                parts[1].as_text_part().unwrap(),
                "this section contains isnād"
            );
        } else {
            panic!("Not a Line");
        }
    }

    #[test]
    fn metadata() {
        let text_parsed = &PARSED;
        let simple_metadata = &text_parsed.simple_metadata;

        assert_eq!(simple_metadata.len(), 33);
        assert_eq!(simple_metadata[1], "000.SortField	:: Shamela_0023833");
        assert_eq!(
            simple_metadata[simple_metadata.len() - 1],
            "999.MiscINFO	:: NODATA"
        );
    }

    #[test]
    fn named_entity_soc() {
        let content = &PARSED.content;

        if let Content::Line(Line {
            text_only: _,
            parts,
            line_type,
        }) = &content[63]
        {
            assert!(line_type.is_normal());

            if let LinePart::NamedEntity {
                prefix,
                extent,
                ne_type,
            } = &parts[1]
            {
                assert_eq!(*prefix, 0);
                assert_eq!(*extent, 2);
                assert!(ne_type.is_soc());
            } else {
                panic!("Not a NamedEntity");
            }

            if let LinePart::NamedEntityText { text, ne_type } = &parts[2] {
                assert_eq!(text, "معمر شيخ:");
                assert!(ne_type.is_soc());
            } else {
                panic!("Not NamedEntityText");
            }

            assert_eq!(parts[3].as_text_part().unwrap(), r#"واسط.. 1"018: نزيل:"#);
        } else {
            panic!("Not a Line");
        }
    }

    #[test]
    fn open_tag_auto() {
        let content = &PARSED.content;

        if let Content::Line(Line {
            text_only: _,
            parts,
            line_type,
        }) = &content[73]
        {
            assert!(line_type.is_normal());

            assert_eq!(
                parts[1].as_open_tag_auto().unwrap(),
                (
                    &"RES".to_string(),
                    &"TYPE".to_string(),
                    &"Category".to_string(),
                    &"fr".to_string()
                )
            );
        } else {
            panic!("Not a Line");
        }
    }

    #[test]
    fn open_tag_user() {
        let content = &PARSED.content;

        if let Content::Line(Line {
            text_only: _,
            parts,
            line_type,
        }) = &content[71]
        {
            assert!(line_type.is_normal());

            assert_eq!(
                parts[1].as_open_tag_user().unwrap(),
                (
                    &"USER".to_string(),
                    &"CAT".to_string(),
                    &"SUBCAT".to_string(),
                    &"SUBSUBCAT".to_string()
                )
            );
        } else {
            panic!("Not a Line");
        }
    }

    #[test]
    fn poetry() {
        let content = &PARSED.content;

        if let Content::Line(Line {
            text_only: _,
            parts,
            line_type,
        }) = &content[55]
        {
            assert!(line_type.is_verse());

            assert_eq!(
                parts[0].as_text_part().unwrap(),
                "وجمع العرب تحت لواء الرسول محمد عليه الصلاة"
            );

            assert_eq!(parts[1].as_hemistich().unwrap(), "%~%");

            assert_eq!(
                parts[2].as_text_part().unwrap(),
                "والسلام، وما يضاف إلى ذلك من"
            );
        } else {
            panic!("Not a Line");
        }
    }

    #[test]
    fn riwayat() {
        let content = &PARSED.content;

        if let Content::Paragraph { orig: _, para_type } = &content[46] {
            assert!(para_type.is_riwayat());
        } else {
            panic!("Not a Paragraph");
        }
    }

    #[test]
    fn route_or_distance() {
        let content = &PARSED.content;

        if let Content::Line(Line {
            text_only: _,
            parts,
            line_type,
        }) = &content[49]
        {
            assert!(line_type.is_route_or_distance());
            assert!(parts[0].is_route_from());
            assert!(parts[2].is_route_towa());

            assert_eq!(parts[5].as_text_part().unwrap(), "distance_as_recorded");
        } else {
            panic!("Not a Line");
        }
    }
}
