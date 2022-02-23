use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use regex::Regex;

// Document

pub struct Document {
    pub orig_text: String,
    pub magic_value: String,
    pub simple_metadata: Vec<String>,
    pub content: Vec<Content>,
}

// Content

#[derive(Clone, Debug)]
pub enum Content {
    PageNumber(PageNumber),
    Paragraph(Paragraph),
    Line(Line),
    MorphologicalPattern(MorphologicalPattern),
    Editorial(Editorial),
    SectionHeader(SectionHeader),
    DictionaryUnit(DictionaryUnit),
    DoxographicalItem(DoxographicalItem),
    BioOrEvent(BioOrEvent),
    AdministrativeRegion(AdministrativeRegion),
}

#[derive(Clone, Debug)]
pub struct PageNumber {
    pub vol: String,
    pub page: String,
}

#[derive(Clone, Debug)]
pub struct MorphologicalPattern {
    pub orig: String,
    pub category: String,
}

#[derive(Clone, Debug)]
pub struct Editorial {
    pub orig: String,
}

#[derive(Clone, Debug)]
pub struct SectionHeader {
    pub orig: String,
    pub value: String,
    pub level: u32,
}

#[derive(Clone, Debug)]
pub struct DictionaryUnit {
    pub orig: String,
    pub dic_type: String,
}

#[derive(Clone, Debug)]
pub struct DoxographicalItem {
    pub orig: String,
    pub dox_type: String,
}

#[derive(Clone, Debug)]
pub struct BioOrEvent {
    pub orig: String,
    pub be_type: String,
}

#[derive(Clone, Debug)]
pub struct AdministrativeRegion {
    pub orig: String,
}

// Paragraph

#[derive(Clone, Debug)]
pub struct Paragraph {
    pub orig: String,
    pub para_type: String,
}

// Line

#[derive(Clone, Debug)]
pub struct Line {
    pub orig: String,
    pub text_only: String,
    pub parts: Vec<LinePart>,
    pub line_type: String,
}

// Line parts
// PageNumber is grouped under Content, but it can belong to either enum

#[derive(Clone, Debug)]
pub enum LinePart {
    Isnad(Isnad),
    PageNumber(PageNumber),
    OpenTagUser(OpenTagUser),
    OpenTagAuto(OpenTagAuto),
    Hemistich(Hemistich),
    Milestone(Milestone),
    Matn(Matn),
    Hukm(Hukm),
    RouteFrom(RouteFrom),
    RouteTowa(RouteTowa),
    RouteDist(RouteDist),
    Date(Date),
    Age(Age),
    NamedEntity(NamedEntity),
    TextPart(TextPart),
    NamedEntityText(NamedEntityText),
}

#[derive(Clone, Debug)]
pub struct Isnad {}

#[derive(Clone, Debug)]
pub struct OpenTagUser {
    pub orig: String,
    pub user: String,
    pub t_type: String,
    pub t_subtype: String,
    pub t_subsubtype: String,
}

#[derive(Clone, Debug)]
pub struct OpenTagAuto {
    pub orig: String,
    pub resp: String,
    pub t_type: String,
    pub category: String,
    pub review: String,
}

#[derive(Clone, Debug)]
pub struct Hemistich {
    pub orig: String,
}

#[derive(Clone, Debug)]
pub struct Milestone {}

#[derive(Clone, Debug)]
pub struct Matn {}

#[derive(Clone, Debug)]
pub struct Hukm {}

#[derive(Clone, Debug)]
pub struct RouteFrom {}

#[derive(Clone, Debug)]
pub struct RouteTowa {}

#[derive(Clone, Debug)]
pub struct RouteDist {}

#[derive(Clone, Debug)]
pub struct Date {
    pub orig: String,
    pub value: String,
    pub date_type: String,
}

#[derive(Clone, Debug)]
pub struct Age {
    pub orig: String,
    pub value: String,
}

#[derive(Clone, Debug)]
pub struct NamedEntity {
    pub orig: String,
    pub prefix: u32,
    pub extent: u32,
    pub ne_type: String,
}

#[derive(Clone, Debug)]
pub struct NamedEntityText {
    pub text: String,
}

#[derive(Clone, Debug)]
pub struct TextPart {
    pub text: String,
}

// Tag constants

const META: &str = "#META#";
const META_END: &str = "#META#Header#End#";
const PAGE: &str = "PageV";
const RWY: &str = "# $RWY$";
const LINE: &str = "~~";

const HEMI: &str = "%~%";
const MILESTONE: &str = "Milestone300";
const MATN: &str = "@MATN@";
const HUKM: &str = "@HUKM@";
const ROUTE_FROM: &str = "#$#FROM";
const ROUTE_TOWA: &str = "#$#TOWA";
const ROUTE_DIST: &str = "#$#DIST";

const PHRASE_LV_TAGS: [&str; 7] = [
    HEMI, MILESTONE, MATN, HUKM, ROUTE_FROM, ROUTE_TOWA, ROUTE_DIST,
];

const YEAR_BIRTH: &str = "@YB";
const YEAR_DEATH: &str = "@YD";
const YEAR_OTHER: &str = "@YY";
const YEAR_AGE: &str = "@YA";

const SRC: &str = "@SRC";
const SOC_FULL: &str = "@SOC";
const SOC: &str = "@S";
const TOP_FULL: &str = "@TOP";
const TOP: &str = "@T";
const PER_FULL: &str = "@PER";
const PER: &str = "@P";

const EDITORIAL: &str = "### |EDITOR|";

const HEADER1: &str = "### |";
const HEADER2: &str = "### ||";
const HEADER3: &str = "### |||";
const HEADER4: &str = "### ||||";
const HEADER5: &str = "### |||||";
const HEADERS: [&str; 5] = [HEADER5, HEADER4, HEADER3, HEADER2, HEADER1];

const DIC: &str = "### $DIC_";

const DIC_NIS: &str = "### $DIC_NIS$";
const DIC_TOP: &str = "### $DIC_TOP$";
const DIC_LEX: &str = "### $DIC_LEX$";
const DIC_BIB: &str = "### $DIC_BIB$";
const DICTIONARIES: [&str; 4] = [DIC_NIS, DIC_TOP, DIC_LEX, DIC_BIB];

const DOX: &str = "### $DOX_";

const DOX_POS: &str = "### $DOX_POS$";
const DOX_SEC: &str = "### $DOX_SEC$";
const DOXOGRAPHICAL: [&str; 2] = [DOX_POS, DOX_SEC];

const BIO: &str = "### $BIO_";
const EVENT: &str = "### @";

const LIST_NAMES: &str = "### $$$$";
const LIST_NAMES_FULL: &str = "### $BIO_NLI$";
const BIO_MAN: &str = "### $";
const BIO_MAN_FULL: &str = "### $BIO_MAN$";
const BIO_WOM: &str = "### $$";
const BIO_WOM_FULL: &str = "### $BIO_WOM$";
const BIO_REF: &str = "### $$$";
const BIO_REF_FULL: &str = "### $BIO_REF$";
const EVENT_FULL: &str = "### $CHR_EVE$";
const LIST_EVENTS: &str = "### @ RAW";
const LIST_EVENTS_FULL: &str = "### $CHR_RAW$";

const BIOS_EVENTS: [&str; 12] = [
    LIST_NAMES_FULL,
    LIST_NAMES,
    BIO_WOM_FULL,
    BIO_MAN_FULL,
    BIO_REF_FULL,
    LIST_EVENTS,
    EVENT,
    BIO_REF,
    BIO_WOM,
    BIO_MAN,
    EVENT_FULL,
    LIST_EVENTS_FULL,
];

// Functions

fn split_keep<'a>(r: &Regex, text: &'a str) -> Vec<&'a str> {
    let mut result = Vec::new();
    let mut last = 0;
    for (index, matched) in text.match_indices(r) {
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

    lazy_static! {
        static ref YIKES: Regex = Regex::new(
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
            )",
        )
        .unwrap();
    }

    text_only = YIKES.replace_all(&text_only, "").to_string();

    text_only
}

fn parse_line(tagged_line: &str, kind: Option<String>, first_token: bool) -> Option<Line> {
    // Remove initial line marker
    let line = tagged_line.trim_start_matches(LINE);

    let text_only = remove_phrase_lv_tags(line.to_string());

    if text_only.is_empty() {
        return None;
    }

    let mut parts: Vec<LinePart> = Vec::new();

    if first_token {
        parts.push(LinePart::Isnad(Isnad {}))
    }

    // Aaaaaaaaaaaa

    lazy_static! {
        static ref UNGODLY: Regex = Regex::new(
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
            )",
        )
        .unwrap();
    }

    lazy_static! {
        static ref OPEN_TAG_CUSTOM_PATTERN_GROUPED: Regex =
            Regex::new(r"^@([^@]+?)@([^_@]+?)_([^_@]+?)(_([^_@]+?))?@").unwrap();
    }

    lazy_static! {
        static ref OPEN_TAG_AUTO_PATTERN_GROUPED: Regex =
            Regex::new(r"^@([A-Z]{3})@([A-Z]{3,})@([A-Za-z]+)@(-@([0tf][ftalmr])@)?").unwrap();
    }

    lazy_static! {
        static ref PAGE_PATTERN: Regex = Regex::new(r"PageV(\d+)P(\d+)").unwrap();
    }

    let tokens = split_keep(&UNGODLY, line);

    let custom_pattern_test = OPEN_TAG_CUSTOM_PATTERN_GROUPED
        .captures("@USER@CAT_SUBCAT_SUBSUBCAT@")
        .unwrap();

    assert_eq!(&custom_pattern_test[1], "USER");
    assert_eq!(&custom_pattern_test[2], "CAT");
    assert_eq!(&custom_pattern_test[3], "SUBCAT");
    assert_eq!(&custom_pattern_test[5], "SUBSUBCAT");

    let auto_pattern_test = OPEN_TAG_AUTO_PATTERN_GROUPED
        .captures("@RES@TYPE@Category@-@fr@")
        .unwrap();

    assert_eq!(&auto_pattern_test[1], "RES");
    assert_eq!(&auto_pattern_test[2], "TYPE");
    assert_eq!(&auto_pattern_test[3], "Category");
    assert_eq!(&auto_pattern_test[5], "fr");

    let mut include_words: u32 = 0;

    for token in tokens {
        let token_trimmed = token.trim();

        if token_trimmed.is_empty() {
            continue;
        }

        let mut opentag_captures: Option<regex::Captures> = None;
        let mut opentagauto_captures: Option<regex::Captures> = None;

        if token_trimmed.starts_with('@') {
            opentag_captures = OPEN_TAG_CUSTOM_PATTERN_GROUPED.captures(token_trimmed);
            opentagauto_captures = OPEN_TAG_AUTO_PATTERN_GROUPED.captures(token_trimmed);
        }

        if token_trimmed.contains(PAGE) {
            let page_captures = PAGE_PATTERN.captures(token_trimmed);
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

pub fn parser(input: String) -> Result<Document> {
    let mut doc = Document {
        orig_text: input.clone(),
        magic_value: String::new(),
        simple_metadata: Vec::new(),
        content: Vec::new(),
    };

    lazy_static! {
        static ref PAGE_PATTERN: Regex = Regex::new(r"PageV(\d+)P(\d+)").unwrap();
    }

    lazy_static! {
        static ref MORPHO_PATTERN: Regex = Regex::new(r"#~:([^:]+?):").unwrap();
    }

    lazy_static! {
        static ref PARA_PATTERN: Regex = Regex::new(r"^#($|[^#])").unwrap();
    }

    lazy_static! {
        static ref BIO_PATTERN: Regex = Regex::new(r"### $[^#]").unwrap();
    }

    lazy_static! {
        static ref REGION_PATTERN: Regex =
            Regex::new(r"(#$#PROV|#$#REG\d) .*? #$#TYPE .*? (#$#REG\d|#$#STTL) ([\w# ]+) $")
                .unwrap();
    }

    for (i, line) in input.lines().enumerate() {
        let line_trimmed = line.trim();

        // Magic value
        if i == 0 && line_trimmed.starts_with("######OpenITI#") {
            doc.magic_value = line_trimmed.to_string();
            continue;
        } else if i == 0 {
            return Err(anyhow!(
                "This does not appear to be an OpenITI mARkdown document"
            ));
        }

        // Non-machine-readable metadata
        if line_trimmed.starts_with(META) {
            if line_trimmed == META_END {
                continue;
            }
            let value = line.trim_start_matches(META).trim();
            doc.simple_metadata.push(value.to_string());
        } else if line_trimmed.starts_with(PAGE) {
            if let Some(cap) = PAGE_PATTERN.captures(line_trimmed) {
                let vol = cap[1].to_string();
                let page = cap[2].to_string();

                doc.content
                    .push(Content::PageNumber(PageNumber { vol, page }));
            } else {
                // They raise an exception here...
            }
        } else if line_trimmed.starts_with(RWY) {
            doc.content.push(Content::Paragraph(Paragraph {
                orig: line_trimmed.to_string(),
                para_type: "riwayat".to_string(),
            }));
            let first_line = parse_line(line_trimmed.trim_start_matches(RWY), None, false);
            if let Some(first_line_content) = first_line {
                doc.content.push(Content::Line(first_line_content));
            }
        } else if line_trimmed.starts_with(ROUTE_FROM) {
            let parsed_line =
                parse_line(line_trimmed, Some("route_or_distance".to_string()), false);
            if let Some(parsed_line_content) = parsed_line {
                doc.content.push(Content::Line(parsed_line_content));
            }
        } else if let Some(cap) = MORPHO_PATTERN.captures(line_trimmed) {
            let category = cap[1].to_string();
            doc.content
                .push(Content::MorphologicalPattern(MorphologicalPattern {
                    orig: line_trimmed.to_string(),
                    category,
                }));
        } else if PARA_PATTERN.is_match(line_trimmed) {
            if line_trimmed.contains(HEMI) {
                let verse_parsed = parse_line(&line_trimmed[1..], Some("verse".to_string()), false);
                if let Some(verse_content) = verse_parsed {
                    doc.content.push(Content::Line(verse_content));
                }
            } else {
                doc.content.push(Content::Paragraph(Paragraph {
                    orig: line_trimmed.to_string(),
                    para_type: "para".to_string(),
                }));

                let first_line = parse_line(&line_trimmed[1..], None, false);
                if let Some(first_line_content) = first_line {
                    doc.content.push(Content::Line(first_line_content));
                }
            }
        } else if line_trimmed.starts_with(LINE) {
            let parsed_line = parse_line(line_trimmed, None, false);
            if let Some(parsed_line_content) = parsed_line {
                doc.content.push(Content::Line(parsed_line_content));
            }
        } else if line_trimmed.starts_with(EDITORIAL) {
            doc.content.push(Content::Editorial(Editorial {
                orig: line_trimmed.to_string(),
            }));
        } else if line_trimmed.starts_with(HEADER1) {
            let mut value = line_trimmed.to_string();
            for tag in HEADERS {
                value = value.replace(tag, "").to_string();
            }
            value = remove_phrase_lv_tags(value);
            // TODO: capture tags as PhraseParts
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
        } else if line_trimmed.starts_with(DIC) {
            let mut no_tag = line_trimmed.to_string();
            for tag in DICTIONARIES {
                no_tag = no_tag.replace(tag, "");
            }
            let first_line = parse_line(&no_tag, None, false);
            let mut dic_type = "bib";
            if line_trimmed.contains(DIC_LEX) {
                dic_type = "lex";
            } else if line_trimmed.contains(DIC_NIS) {
                dic_type = "nis";
            } else if line_trimmed.contains(DIC_TOP) {
                dic_type = "top";
            }
            doc.content.push(Content::DictionaryUnit(DictionaryUnit {
                orig: line_trimmed.to_string(),
                dic_type: dic_type.to_string(),
            }));
            if let Some(first_line_content) = first_line {
                doc.content.push(Content::Line(first_line_content));
            }
        } else if line_trimmed.starts_with(DOX) {
            let mut no_tag = line_trimmed.to_string();
            for tag in DOXOGRAPHICAL {
                no_tag = no_tag.replace(tag, "");
            }
            let first_line = parse_line(&no_tag, None, false);
            let mut dox_type = "pos";
            if line_trimmed.contains(DOX_SEC) {
                dox_type = "sec";
            }
            doc.content
                .push(Content::DoxographicalItem(DoxographicalItem {
                    orig: line_trimmed.to_string(),
                    dox_type: dox_type.to_string(),
                }));
            if let Some(first_line_content) = first_line {
                doc.content.push(Content::Line(first_line_content));
            }
        } else if BIO_PATTERN.is_match(line_trimmed)
            || line_trimmed.starts_with(BIO)
            || line_trimmed.starts_with(EVENT)
        {
            let mut no_tag = line_trimmed.to_string();
            for tag in BIOS_EVENTS {
                no_tag = no_tag.replace(tag, "");
            }
            let first_line = parse_line(&no_tag, None, false);
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
            doc.content.push(Content::BioOrEvent(BioOrEvent {
                orig: line_trimmed.to_string(),
                be_type: be_type.to_string(),
            }));
            if let Some(first_line_content) = first_line {
                doc.content.push(Content::Line(first_line_content));
            }
        } else if REGION_PATTERN.is_match(line_trimmed) {
            doc.content
                .push(Content::AdministrativeRegion(AdministrativeRegion {
                    orig: line_trimmed.to_string(),
                }));
        } else {
            continue;
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
        let text_parsed = parser(full_text).unwrap();
        let simple_metadata = text_parsed.simple_metadata;

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

        if let LinePart::TextPart(TextPart { text }) = &parts[3] {
            assert_eq!(text, r###"واسط.. 1"018: نزيل: "###);
        } else {
            panic!("Not the type that we were expecting");
        }
    }
}
