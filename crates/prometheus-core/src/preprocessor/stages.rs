//! Compile-time preprocessor stage switches.
//!
//! Capitals + static markup stay on (main baseline). Word/phrase dict and
//! template stages are trial flags — flip only for measured SCORE trials.

/// Fold word-initial capitals to lowercase + separator markers.
pub const PRE_STAGE_CAPITALS: bool = true;

/// Static XML/wiki markup dictionary (`0x80..` tokens).
pub const PRE_STAGE_MARKUP: bool = true;

/// Natural-language word/phrase dictionary (XML-free). Off until a SCORE win.
pub const PRE_STAGE_WORDS: bool = true;

/// Selected word-dict size (`4096` / `16384` / `49152`). `0` = unused/empty.
pub const WORD_DICT_N: usize = 4096;

/// Mined template dictionary (companion trial).
pub const PRE_STAGE_TEMPLATES: bool = true;

/// Cap on embedded template entries (`0` = all in blob).
pub const TEMPLATE_DICT_N: usize = 64;

/// `true` = params follow each template token; `false` = trailing side-stream.
pub const PARAM_INLINE: bool = false;
