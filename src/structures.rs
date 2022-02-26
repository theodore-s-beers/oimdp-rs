// This still needs review; I obviously couldn't replicate the Python objects one-to-one

// Document

pub struct Document {
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
    Editorial,
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
pub struct SectionHeader {
    pub orig: String,
    pub value: String,
    pub level: u32,
}

#[derive(Clone, Debug)]
pub struct DictionaryUnit {
    pub orig: String,
    pub dic_type: DicType,
}

#[derive(Clone, Debug)]
pub enum DicType {
    Nis,
    Top,
    Lex,
    Bib,
}

#[derive(Clone, Debug)]
pub struct DoxographicalItem {
    pub orig: String,
    pub dox_type: DoxType,
}

#[derive(Clone, Debug)]
pub enum DoxType {
    Pos,
    Sec,
}

#[derive(Clone, Debug)]
pub struct BioOrEvent {
    pub orig: String,
    pub be_type: BeType,
}

#[derive(Clone, Debug)]
pub enum BeType {
    Man,
    Wom,
    Ref,
    Names,
    Event,
    Events,
}

// Administrative regions are not yet fully implemented in the Python library
#[derive(Clone, Debug)]
pub struct AdministrativeRegion {
    pub orig: String,
}

// Paragraph
// Here I switched to use one struct, with a field to indicate the type

#[derive(Clone, Debug)]
pub struct Paragraph {
    pub orig: String,
    pub para_type: ParaType,
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub enum LineType {
    Normal,
    RouteOrDistance,
    Verse,
}

// Line parts
// PageNumber is defined under Content, but it belongs to both enums

#[derive(Clone, Debug)]
pub enum LinePart {
    Isnad,
    PageNumber(PageNumber),
    OpenTagUser(OpenTagUser),
    OpenTagAuto(OpenTagAuto),
    Hemistich(Hemistich),
    Milestone,
    Matn,
    Hukm,
    RouteFrom,
    RouteTowa,
    RouteDist,
    Date(Date),
    Age(Age),
    NamedEntity(NamedEntity),
    TextPart(TextPart),
    NamedEntityText(NamedEntityText),
}

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
pub struct Date {
    pub orig: String,
    pub value: String,
    pub date_type: DateType,
}

#[derive(Clone, Debug)]
pub enum DateType {
    Birth,
    Death,
    Other,
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
    pub ne_type: EntityType,
}

#[derive(Clone, Debug)]
pub struct NamedEntityText {
    pub text: String,
    pub ne_type: EntityType,
}

#[derive(Clone, Debug)]
pub enum EntityType {
    Top,
    Per,
    Soc,
    Src,
}

#[derive(Clone, Debug)]
pub struct TextPart {
    // In the Python library, there are fields for both "orig" and "text"
    // But they're identical!
    pub text: String,
}
