//! Compile-time preprocessor stage switches.
//!
//! Markup + word dict on (main baseline). Capitals OFF after P2-PRE KEEP
//! (enwik8 SCORE 20753959). Template stage remains a trial flag.

/// Fold word-initial capitals to lowercase + separator markers.
/// Disabled: measured SCORE win with capitals off (P2-PRE KEEP).
pub const PRE_STAGE_CAPITALS: bool = false;

/// Static XML/wiki markup dictionary (`0x80..` tokens).
pub const PRE_STAGE_MARKUP: bool = true;

/// Natural-language word/phrase dictionary (XML-free). Off until a SCORE win.
pub const PRE_STAGE_WORDS: bool = true;

/// Selected word-dict size (`4096` / `16384` / `49152`). `0` = unused/empty.
pub const WORD_DICT_N: usize = 4096;

/// Mined template dictionary (companion trial; leave off here).
#[allow(dead_code)]
pub const PRE_STAGE_TEMPLATES: bool = false;
