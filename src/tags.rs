// Tag constants galore

pub const META: &str = "#META#";
pub const META_END: &str = "#META#Header#End#";
pub const PAGE: &str = "PageV";
pub const RWY: &str = "# $RWY$";
pub const LINE: &str = "~~";

pub const HEMI: &str = "%~%";
pub const MILESTONE: &str = "Milestone300";
pub const MATN: &str = "@MATN@";
pub const HUKM: &str = "@HUKM@";
pub const ROUTE_FROM: &str = "#$#FROM";
pub const ROUTE_TOWA: &str = "#$#TOWA";
pub const ROUTE_DIST: &str = "#$#DIST";

pub const PHRASE_LV_TAGS: [&str; 7] = [
    HEMI, MILESTONE, MATN, HUKM, ROUTE_FROM, ROUTE_TOWA, ROUTE_DIST,
];

pub const YEAR_BIRTH: &str = "@YB";
pub const YEAR_DEATH: &str = "@YD";
pub const YEAR_OTHER: &str = "@YY";
pub const YEAR_AGE: &str = "@YA";

pub const SRC: &str = "@SRC";
pub const SOC_FULL: &str = "@SOC";
pub const SOC: &str = "@S";
pub const TOP_FULL: &str = "@TOP";
pub const TOP: &str = "@T";
pub const PER_FULL: &str = "@PER";
pub const PER: &str = "@P";

pub const EDITORIAL: &str = "### |EDITOR|";

pub const HEADER1: &str = "### |";
pub const HEADER2: &str = "### ||";
pub const HEADER3: &str = "### |||";
pub const HEADER4: &str = "### ||||";
pub const HEADER5: &str = "### |||||";
pub const HEADERS: [&str; 5] = [HEADER5, HEADER4, HEADER3, HEADER2, HEADER1];

pub const DIC: &str = "### $DIC_";

pub const DIC_NIS: &str = "### $DIC_NIS$";
pub const DIC_TOP: &str = "### $DIC_TOP$";
pub const DIC_LEX: &str = "### $DIC_LEX$";
pub const DIC_BIB: &str = "### $DIC_BIB$";
pub const DICTIONARIES: [&str; 4] = [DIC_NIS, DIC_TOP, DIC_LEX, DIC_BIB];

pub const DOX: &str = "### $DOX_";

pub const DOX_POS: &str = "### $DOX_POS$";
pub const DOX_SEC: &str = "### $DOX_SEC$";
pub const DOXOGRAPHICAL: [&str; 2] = [DOX_POS, DOX_SEC];

pub const BIO: &str = "### $BIO_";
pub const EVENT: &str = "### @";

pub const LIST_NAMES: &str = "### $$$$";
pub const LIST_NAMES_FULL: &str = "### $BIO_NLI$";
pub const BIO_MAN: &str = "### $";
pub const BIO_MAN_FULL: &str = "### $BIO_MAN$";
pub const BIO_WOM: &str = "### $$";
pub const BIO_WOM_FULL: &str = "### $BIO_WOM$";
pub const BIO_REF: &str = "### $$$";
pub const BIO_REF_FULL: &str = "### $BIO_REF$";
pub const EVENT_FULL: &str = "### $CHR_EVE$";
pub const LIST_EVENTS: &str = "### @ RAW";
pub const LIST_EVENTS_FULL: &str = "### $CHR_RAW$";

pub const BIOS_EVENTS: [&str; 12] = [
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
