use anyhow::{anyhow, Result};
use regex::Regex;

mod structures;
use crate::structures::*;

mod tags;
use crate::tags::*;

// Regex macro from once_cell

macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

// Smaller helper functions

fn split_keep<'a>(re: &Regex, text: &'a str) -> Vec<&'a str> {
    let mut result = Vec::new();
    let mut last = 0;

    for (index, matched) in text.match_indices(re) {
        if last != index {
            result.push(&text[last..index]);
        }

        result.push(matched);
        last = index + matched.len();
    }

    if last < text.len() {
        result.push(&text[last..]);
    }

    result
}

fn remove_phrase_lv_tags(line: String) -> String {
    let mut text_only = line;

    for tag in PHRASE_LV_TAGS {
        text_only = text_only.replace(tag, "");
    }

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

    text_only = yikes.replace_all(&text_only, "").to_string();

    text_only
}

// Line parsing function (needs review)

fn parse_line(tagged_line: &str, kind: Option<String>, first_token: bool) -> Option<Line> {
    // Remove initial line marker
    let line = tagged_line.trim_start_matches(LINE);

    // Remove phrase-level tags (whatever that means)
    let text_only = remove_phrase_lv_tags(line.to_string());

    // Return early if there's nothing left at this point
    if text_only.is_empty() {
        return None;
    }

    // Create vec for line parts
    let mut parts: Vec<LinePart> = Vec::new();

    // If any first_token was indicated, add an Isnad part to the line
    // This is weird right now because the only kind of first_token that has been implemented
    // is Isnad. So I made the function argument into a bool for simplicity.
    // But I still don't understand what this is supposed to accomplish
    if first_token {
        parts.push(LinePart::Isnad(Isnad {}))
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

    let open_tag_custom_pattern_grouped = regex!("^@([^@]+?)@([^_@]+?)_([^_@]+?)(_([^_@]+?))?@");
    let open_tag_auto_pattern_grouped =
        regex!("^@([A-Z]{3})@([A-Z]{3,})@([A-Za-z]+)@(-@([0tf][ftalmr])@)?");

    let page_pattern = regex!(r"PageV(\d+)P(\d+)");

    /* These were temporary tests to make sure certain regexes worked properly
    let custom_pattern_test = open_tag_custom_pattern_grouped
        .captures("@USER@CAT_SUBCAT_SUBSUBCAT@")
        .unwrap();

    assert_eq!(&custom_pattern_test[1], "USER");
    assert_eq!(&custom_pattern_test[2], "CAT");
    assert_eq!(&custom_pattern_test[3], "SUBCAT");
    assert_eq!(&custom_pattern_test[5], "SUBSUBCAT");

    let auto_pattern_test = open_tag_auto_pattern_grouped
        .captures("@RES@TYPE@Category@-@fr@")
        .unwrap();

    assert_eq!(&auto_pattern_test[1], "RES");
    assert_eq!(&auto_pattern_test[2], "TYPE");
    assert_eq!(&auto_pattern_test[3], "Category");
    assert_eq!(&auto_pattern_test[5], "fr");
    */

    let mut include_words: u32 = 0;

    for token in tokens {
        let token_trimmed = token.trim();

        if token_trimmed.is_empty() {
            continue;
        }

        let mut opentag_captures: Option<regex::Captures> = None;
        let mut opentagauto_captures: Option<regex::Captures> = None;

        if token_trimmed.starts_with('@') {
            opentag_captures = open_tag_custom_pattern_grouped.captures(token_trimmed);
            opentagauto_captures = open_tag_auto_pattern_grouped.captures(token_trimmed);
        }

        if token_trimmed.contains(PAGE) {
            let page_captures = page_pattern.captures(token_trimmed);
            if let Some(page_matches) = page_captures {
                let vol = page_matches[1].to_string();
                let page = page_matches[2].to_string();

                parts.push(LinePart::PageNumber(PageNumber { vol, page }));
            } else {
                // They raise an exception here...
            }
        } else if let Some(opentag_matches) = opentag_captures {
            let user = opentag_matches[1].to_string();
            let t_type = opentag_matches[2].to_string();
            let t_subtype = opentag_matches[3].to_string();
            let t_subsubtype = opentag_matches[5].to_string();

            parts.push(LinePart::OpenTagUser(OpenTagUser {
                orig: token_trimmed.to_string(),
                user,
                t_type,
                t_subtype,
                t_subsubtype,
            }))
        } else if let Some(opentagauto_matches) = opentagauto_captures {
            let resp = opentagauto_matches[1].to_string();
            let t_type = opentagauto_matches[2].to_string();
            let category = opentagauto_matches[3].to_string();
            let review = opentagauto_matches[5].to_string();

            parts.push(LinePart::OpenTagAuto(OpenTagAuto {
                orig: token_trimmed.to_string(),
                resp,
                t_type,
                category,
                review,
            }))
        } else if token_trimmed.contains(HEMI) {
            parts.push(LinePart::Hemistich(Hemistich {
                orig: token_trimmed.to_string(),
            }));
        } else if token_trimmed.contains(MILESTONE) {
            parts.push(LinePart::Milestone(Milestone {}));
        } else if token_trimmed.contains(MATN) {
            parts.push(LinePart::Matn(Matn {}));
        } else if token_trimmed.contains(HUKM) {
            parts.push(LinePart::Hukm(Hukm {}));
        } else if token_trimmed.contains(ROUTE_FROM) {
            parts.push(LinePart::RouteFrom(RouteFrom {}));
        } else if token_trimmed.contains(ROUTE_TOWA) {
            parts.push(LinePart::RouteTowa(RouteTowa {}));
        } else if token_trimmed.contains(ROUTE_DIST) {
            parts.push(LinePart::RouteDist(RouteDist {}));
        } else if token_trimmed.contains(YEAR_BIRTH) {
            let orig = token_trimmed.to_string();
            let value = token_trimmed.trim_start_matches(YEAR_BIRTH).to_string();
            let date_type = "birth".to_string();

            parts.push(LinePart::Date(Date {
                orig,
                value,
                date_type,
            }));
        } else if token_trimmed.contains(YEAR_DEATH) {
            let orig = token_trimmed.to_string();
            let value = token_trimmed.trim_start_matches(YEAR_DEATH).to_string();
            let date_type = "death".to_string();

            parts.push(LinePart::Date(Date {
                orig,
                value,
                date_type,
            }));
        } else if token_trimmed.contains(YEAR_OTHER) {
            let orig = token_trimmed.to_string();
            let value = token_trimmed.trim_start_matches(YEAR_OTHER).to_string();
            let date_type = "other".to_string();

            parts.push(LinePart::Date(Date {
                orig,
                value,
                date_type,
            }));
        } else if token_trimmed.contains(YEAR_AGE) {
            let orig = token_trimmed.to_string();
            let value = token_trimmed.trim_start_matches(YEAR_AGE).to_string();

            parts.push(LinePart::Age(Age { orig, value }))
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

            parts.push(LinePart::NamedEntity(NamedEntity {
                orig: token_trimmed.to_string(),
                prefix,
                extent,
                ne_type: "src".to_string(),
            }));
        } else if token_trimmed.starts_with(SOC_FULL) {
            let val = token_trimmed.trim_start_matches(SOC_FULL);
            let mut iter = val.chars();

            let prefix_char = iter.next().unwrap();
            let prefix: u32 = prefix_char.to_digit(10).unwrap();

            let extent_char = iter.next().unwrap();
            let extent: u32 = extent_char.to_digit(10).unwrap();

            include_words = extent;

            parts.push(LinePart::NamedEntity(NamedEntity {
                orig: token_trimmed.to_string(),
                prefix,
                extent,
                ne_type: "soc".to_string(),
            }));
        } else if token_trimmed.starts_with(SOC) {
            let val = token_trimmed.trim_start_matches(SOC);
            let mut iter = val.chars();

            let prefix_char = iter.next().unwrap();
            let prefix: u32 = prefix_char.to_digit(10).unwrap();

            let extent_char = iter.next().unwrap();
            let extent: u32 = extent_char.to_digit(10).unwrap();

            include_words = extent;

            parts.push(LinePart::NamedEntity(NamedEntity {
                orig: token_trimmed.to_string(),
                prefix,
                extent,
                ne_type: "soc".to_string(),
            }));
        } else if token_trimmed.starts_with(TOP_FULL) {
            let val = token_trimmed.trim_start_matches(TOP_FULL);
            let mut iter = val.chars();

            let prefix_char = iter.next().unwrap();
            let prefix: u32 = prefix_char.to_digit(10).unwrap();

            let extent_char = iter.next().unwrap();
            let extent: u32 = extent_char.to_digit(10).unwrap();

            include_words = extent;

            parts.push(LinePart::NamedEntity(NamedEntity {
                orig: token_trimmed.to_string(),
                prefix,
                extent,
                ne_type: "top".to_string(),
            }));
        } else if token_trimmed.starts_with(TOP) {
            let val = token_trimmed.trim_start_matches(TOP);
            let mut iter = val.chars();

            let prefix_char = iter.next().unwrap();
            let prefix: u32 = prefix_char.to_digit(10).unwrap();

            let extent_char = iter.next().unwrap();
            let extent: u32 = extent_char.to_digit(10).unwrap();

            include_words = extent;

            parts.push(LinePart::NamedEntity(NamedEntity {
                orig: token_trimmed.to_string(),
                prefix,
                extent,
                ne_type: "top".to_string(),
            }));
        } else if token_trimmed.starts_with(PER_FULL) {
            let val = token_trimmed.trim_start_matches(PER_FULL);
            let mut iter = val.chars();

            let prefix_char = iter.next().unwrap();
            let prefix: u32 = prefix_char.to_digit(10).unwrap();

            let extent_char = iter.next().unwrap();
            let extent: u32 = extent_char.to_digit(10).unwrap();

            include_words = extent;

            parts.push(LinePart::NamedEntity(NamedEntity {
                orig: token_trimmed.to_string(),
                prefix,
                extent,
                ne_type: "per".to_string(),
            }));
        } else if token_trimmed.starts_with(PER) {
            let val = token_trimmed.trim_start_matches(PER);
            let mut iter = val.chars();

            let prefix_char = iter.next().unwrap();
            let prefix: u32 = prefix_char.to_digit(10).unwrap();

            let extent_char = iter.next().unwrap();
            let extent: u32 = extent_char.to_digit(10).unwrap();

            include_words = extent;

            parts.push(LinePart::NamedEntity(NamedEntity {
                orig: token_trimmed.to_string(),
                prefix,
                extent,
                ne_type: "per".to_string(),
            }));
        } else if include_words > 0 {
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
                parts.push(LinePart::NamedEntityText(NamedEntityText { text: entity }))
            }

            if !rest.is_empty() {
                parts.push(LinePart::TextPart(TextPart { text: rest }))
            }

            include_words = 0;
        } else {
            parts.push(LinePart::TextPart(TextPart {
                text: token_trimmed.to_string(),
            }))
        }
    }

    // Set up return value

    let line_type = if let Some(specified) = kind {
        specified
    } else {
        "line".to_string()
    };

    let line_struct = Line {
        orig: line.to_string(),
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
        orig_text: input.clone(),
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
            doc.magic_value = line_trimmed.to_string();

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
            let value = line_trimmed.trim_start_matches(META).trim().to_string();
            doc.simple_metadata.push(value);
        // Page number (not sure why this would happen)
        } else if line_trimmed.starts_with(PAGE) {
            // Try to capture volume and page numbers
            if let Some(cap) = page_pattern.captures(line_trimmed) {
                let vol = cap[1].to_string();
                let page = cap[2].to_string();

                doc.content
                    .push(Content::PageNumber(PageNumber { vol, page }));
            } else {
                // An exception is raised here in the Python library; I haven't done anything
            }
        // Riwāya
        } else if line_trimmed.starts_with(RWY) {
            // First add the whole line
            doc.content.push(Content::Paragraph(Paragraph {
                orig: line_trimmed.to_string(),
                para_type: "riwayat".to_string(),
            }));

            // Then parse everything after the riwāya tag
            let double_trimmed = line_trimmed.trim_start_matches(RWY);
            let first_line = parse_line(double_trimmed, None, false);

            if let Some(first_line_content) = first_line {
                doc.content.push(Content::Line(first_line_content));
            }
        // Route from
        } else if line_trimmed.starts_with(ROUTE_FROM) {
            let kind = "route_or_distance".to_string();
            let parsed_line = parse_line(line_trimmed, Some(kind), false);

            if let Some(parsed_line_content) = parsed_line {
                doc.content.push(Content::Line(parsed_line_content));
            }
        // Morphological pattern
        } else if let Some(cap) = morpho_pattern.captures(line_trimmed) {
            let category = cap[1].to_string();

            doc.content
                .push(Content::MorphologicalPattern(MorphologicalPattern {
                    orig: line_trimmed.to_string(),
                    category,
                }));
        // Paragraph
        } else if para_pattern.is_match(line_trimmed) {
            // This line will be parsed without the initial paragraph marker
            let no_marker = &line_trimmed[1..];

            // If line contains hemistich marker (which can occur in the middle)...
            if line_trimmed.contains(HEMI) {
                let kind = "verse".to_string();
                let verse_parsed = parse_line(no_marker, Some(kind), false);

                if let Some(verse_content) = verse_parsed {
                    doc.content.push(Content::Line(verse_content));
                }
            } else {
                doc.content.push(Content::Paragraph(Paragraph {
                    orig: line_trimmed.to_string(),
                    para_type: "para".to_string(),
                }));

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
            doc.content.push(Content::Editorial(Editorial {
                orig: line_trimmed.to_string(),
            }));
        // Heading
        } else if line_trimmed.starts_with(HEADER1) {
            // I think "value" means the actual heading content, minus the tag
            let mut value = line_trimmed.to_string();

            for tag in HEADERS {
                value = value.replace(tag, "").to_string();
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

            doc.content.push(Content::SectionHeader(SectionHeader {
                orig: line_trimmed.to_string(),
                value,
                level,
            }));
        // Dictionary content (?)
        } else if line_trimmed.starts_with(DIC) {
            // Strip tags
            let mut no_tag = line_trimmed.to_string();
            for tag in DICTIONARIES {
                no_tag = no_tag.replace(tag, "");
            }

            // Parse stripped line
            let first_line = parse_line(&no_tag, None, false);

            // Determine dictionary content type
            let mut dic_type = "bib";

            if line_trimmed.contains(DIC_LEX) {
                dic_type = "lex";
            } else if line_trimmed.contains(DIC_NIS) {
                dic_type = "nis";
            } else if line_trimmed.contains(DIC_TOP) {
                dic_type = "top";
            }

            // Add dictionary unit
            doc.content.push(Content::DictionaryUnit(DictionaryUnit {
                orig: line_trimmed.to_string(),
                dic_type: dic_type.to_string(),
            }));

            // If there was other line content, add that
            if let Some(first_line_content) = first_line {
                doc.content.push(Content::Line(first_line_content));
            }
        // Doxographical content (?)
        } else if line_trimmed.starts_with(DOX) {
            // Strip tags
            let mut no_tag = line_trimmed.to_string();
            for tag in DOXOGRAPHICAL {
                no_tag = no_tag.replace(tag, "");
            }

            // Parse stripped line
            let first_line = parse_line(&no_tag, None, false);

            // Determine doxographical content type
            let mut dox_type = "pos";
            if line_trimmed.contains(DOX_SEC) {
                dox_type = "sec";
            }

            // Add doxographical item
            doc.content
                .push(Content::DoxographicalItem(DoxographicalItem {
                    orig: line_trimmed.to_string(),
                    dox_type: dox_type.to_string(),
                }));

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
            let mut no_tag = line_trimmed.to_string();
            for tag in BIOS_EVENTS {
                no_tag = no_tag.replace(tag, "");
            }

            // Parse stripped line
            let first_line = parse_line(&no_tag, None, false);

            // Determine type of biographical item
            let mut be_type = "man";

            if line_trimmed.contains(LIST_NAMES_FULL) || line_trimmed.contains(LIST_NAMES) {
                be_type = "names";
            } else if line_trimmed.contains(BIO_REF_FULL) || line_trimmed.contains(BIO_REF) {
                be_type = "ref";
            } else if line_trimmed.contains(BIO_WOM_FULL) || line_trimmed.contains(BIO_WOM) {
                be_type = "wom";
            } else if line_trimmed.contains(LIST_EVENTS) {
                be_type = "events";
            } else if line_trimmed.contains(EVENT) {
                be_type = "event";
            }

            // Add biographical item
            doc.content.push(Content::BioOrEvent(BioOrEvent {
                orig: line_trimmed.to_string(),
                be_type: be_type.to_string(),
            }));

            // If there was other line content, add that
            if let Some(first_line_content) = first_line {
                doc.content.push(Content::Line(first_line_content));
            }
        // Region
        } else if region_pattern.is_match(line_trimmed) {
            doc.content
                .push(Content::AdministrativeRegion(AdministrativeRegion {
                    orig: line_trimmed.to_string(),
                }));
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
    use std::fs;

    #[test]
    fn metadata() {
        let full_text = fs::read_to_string("test.md").unwrap();
        let mut partial_text = String::new();

        // Save time by taking the first 100 lines of the test file
        for (i, line) in full_text.lines().enumerate() {
            if i > 99 {
                break;
            }
            partial_text += line;
            partial_text += "\n";
        }

        let text_parsed = parser(partial_text).unwrap();
        let simple_metadata = text_parsed.simple_metadata;

        assert_eq!(simple_metadata.len(), 33);
        assert_eq!(simple_metadata[1], "000.SortField	:: Shamela_0023833");
        assert_eq!(
            simple_metadata[simple_metadata.len() - 1],
            "999.MiscINFO	:: NODATA"
        );
    }

    #[test]
    fn line_parts() {
        let line =
            r###"~~ الصلاة والسلام، وما يضاف إلى ذلك @SOC02 نزيل: 1"018: واسط.. شيخ: معمر"###;

        let line_parsed = parse_line(line, None, false).unwrap();
        let parts = line_parsed.parts;

        // Testing specific fields like this will not be easy in Rust
        if let LinePart::TextPart(TextPart { text }) = &parts[3] {
            assert_eq!(text, r###"واسط.. 1"018: نزيل: "###);
        } else {
            panic!("Not the type that we were expecting");
        }
    }
}
