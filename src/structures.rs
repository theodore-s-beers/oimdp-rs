use enum_as_inner::EnumAsInner;

// This needs ongoing review; I obviously couldn't replicate Python objects one-to-one

// Document

#[derive(Debug)]
pub struct Document {
    pub magic_value: String,
    pub simple_metadata: Vec<String>,
    pub content: Vec<Content>,
}

// Content

#[derive(Clone, Debug, EnumAsInner)]
pub enum Content {
    PageNumber(PageNumber),
    // Switched to use one para. variant, with field to indicate type
    Paragraph { orig: String, para_type: ParaType },
    Line(Line),
    MorphologicalPattern { orig: String, category: String },
    Editorial,
    SectionHeader { value: String, level: u32 },
    DictionaryUnit { orig: String, dic_type: DicType },
    DoxographicalItem { orig: String, dox_type: DoxType },
    BioOrEvent { orig: String, be_type: BeType },
    // Admin. regions not yet fully implemented in Python library
    AdministrativeRegion { orig: String },
}

#[derive(Clone, Debug)]
pub struct PageNumber {
    pub vol: String,
    pub page: String,
}

#[derive(Clone, Debug, EnumAsInner)]
pub enum BeType {
    Man,
    Wom,
    Ref,
    Names,
    Event,
    Events,
}

#[derive(Clone, Debug, EnumAsInner)]
pub enum DicType {
    Nis,
    Top,
    Lex,
    Bib,
}

#[derive(Clone, Debug, EnumAsInner)]
pub enum DoxType {
    Pos,
    Sec,
}

#[derive(Clone, Debug, EnumAsInner)]
pub enum ParaType {
    Normal,
    Riwayat,
}

// Line
// Here I switched to use one struct, with a field to indicate the type

#[derive(Clone, Debug)]
pub struct Line {
    pub text_only: Option<String>,
    pub parts: Vec<LinePart>,
    pub line_type: LineType,
}

#[derive(Clone, Debug, EnumAsInner)]
pub enum LineType {
    Normal,
    RouteOrDistance,
    Verse,
}

// Line parts
// PageNumber is a struct defined under Content; it can belong to either enum

#[derive(Clone, Debug, EnumAsInner)]
pub enum LinePart {
    Isnad,
    PageNumber(PageNumber),
    OpenTagUser {
        user: String,
        t_type: String,
        t_subtype: String,
        t_subsubtype: String,
    },
    OpenTagAuto {
        resp: String,
        t_type: String,
        category: String,
        review: String,
    },
    Hemistich {
        orig: String,
    },
    Milestone,
    Matn,
    Hukm,
    RouteFrom,
    RouteTowa,
    RouteDist,
    Date {
        value: String,
        date_type: DateType,
    },
    Age {
        value: String,
    },
    NamedEntity {
        prefix: u32,
        extent: u32,
        ne_type: EntityType,
    },
    TextPart {
        text: String,
    },
    NamedEntityText {
        text: String,
        ne_type: EntityType,
    },
}

#[derive(Clone, Debug, EnumAsInner)]
pub enum DateType {
    Birth,
    Death,
    Other,
}

#[derive(Clone, Debug, EnumAsInner)]
pub enum EntityType {
    Top,
    Per,
    Soc,
    Src,
}
