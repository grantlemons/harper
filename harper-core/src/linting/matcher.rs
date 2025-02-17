use crate::linting::{Lint, LintKind, Linter, Suggestion};
use crate::{CharString, Document, Punctuation, Span, Token, TokenKind, WordMetadata};

#[derive(Debug, PartialEq, PartialOrd, Clone)]
struct PatternToken {
    /// The general variant of the token.
    /// The inner data of the [`TokenKind`] should be replaced with the default
    /// value.
    kind: TokenKind,
    content: Option<CharString>,
}

impl PatternToken {
    fn from_token(token: Token, document: &Document) -> Self {
        if token.kind.is_word() {
            Self {
                kind: token.kind.with_default_data(),
                content: Some(document.get_span_content(token.span).into()),
            }
        } else {
            Self {
                kind: token.kind,
                content: None,
            }
        }
    }
}

macro_rules! vecword {
    ($lit:literal) => {
        $lit.chars().collect()
    };
}

macro_rules! pt {
    ($str:literal) => {
        PatternToken {
            kind: TokenKind::Word(WordMetadata::default()),
            content: Some($str.chars().collect()),
        }
    };
    (Word) => {
        PatternToken {
            kind: TokenKind::Word,
            content: None,
        }
    };
    (Period) => {
        PatternToken {
            kind: TokenKind::Punctuation(Punctuation::Period),
            content: None,
        }
    };
    (Hyphen) => {
        PatternToken {
            kind: TokenKind::Punctuation(Punctuation::Hyphen),
            content: None,
        }
    };
    (Space) => {
        PatternToken {
            kind: TokenKind::Space(1),
            content: None,
        }
    };
    ( $($($str:literal),* => $repl:literal),*) => {
        vec![
            $(
                {
                    let mut rule = Rule {
                        pattern: vec![$(
                            pt!($str),
                            pt!(Space),
                        )*],
                        replace_with: $repl.chars().collect()
                    };

                    if rule.pattern.len() > 0{
                        rule.pattern.pop();
                    }

                    rule
                },
            )*
        ]
    };
}

struct Rule {
    pattern: Vec<PatternToken>,
    replace_with: Vec<char>,
}

/// A linter that uses a variety of curated pattern matches to find and fix
/// common grammatical issues.
pub struct Matcher {
    triggers: Vec<Rule>,
}

impl Matcher {
    pub fn new() -> Self {
        // This match list needs to be automatically expanded instead of explicitly
        // defined like it is now.
        let mut triggers = Vec::new();

        // stylistic improvements
        triggers.extend(pt! {
            "all", "of", "the" => "all the",
            "and","also" => "and"
        });

        // phrase typos, each word passes spellcheck but one word is wrong
        triggers.extend(pt! {
            "an","in" => "and in",
            "bee","there" => "been there",
            "can","be","seem" => "can be seen",
            "eight","grade" => "eighth grade",
            "gong","to" => "going to",
            "I","a","m" => "I am",
            "It","cam" => "It can",
            "kid","regards" => "kind regards",
            "mu","house" => "my house",
            "no","to" => "not to",
            "No","to" => "not to",
            "the", "this" => "that this",
            "The","re" => "There",
            "though", "process" => "thought process"
        });

        // phrase capitalization
        triggers.extend(pt! {
            "black","sea" => "Black Sea",
            "geiger","counter" => "Geiger counter",
            "my","french" => "my French"
        });

        // hyphenate phrasal adjectives
        triggers.extend(pt! {
            "case", "sensitive" => "case-sensitive",
            "ngram" => "n-gram",
            "off","the","cuff" => "off-the-cuff",
            "Tree", "sitter" => "Tree-sitter",
            "wellbeing" => "well-being"
        });

        // expand abbreviations
        triggers.extend(pt! {
            "dep" => "dependency",
            "deps" => "dependencies",
            "hr" => "hour",
            "hrs" => "hours",
            "min" => "minimum",
            "min" => "minute",
            "mins" => "minutes",
            "ms" => "milliseconds",
            "sec" => "second",
            "secs" => "seconds",
            "stdin" => "standard input",
            "stdout" => "standard output",
            "w/" => "with",
            "w/o" => "without"
        });

        // replace euphemisms
        triggers.extend(pt! {
            "fatal","outcome" => "death"
        });

        // spellos
        triggers.extend(pt! {
            "grammer" => "grammar"
        });

        // expand compound words
        triggers.extend(pt! {
            "hashmap" => "hash map",
            "hashtable" => "hash table",
            "wordlist" => "word list"
        });

        // mixing up than/then in context
        triggers.extend(pt! {
            "more","then" => "more than",
            "then","her" => "than her",
            "then","hers" => "than hers",
            "then","him" => "than him",
            "then","his" => "than his",
            "then","last","week" => "than last week"
        });

        // not a perfect fit for any of the other categories
        triggers.extend(pt! {
            "performing","this" => "perform this",
            "simply","grammatical" => "simple grammatical",
            "the","challenged" => "that challenged",
            "to", "towards" => "towards",
            "To-Do" => "To-do",
            "todo" => "to-do"
        });

        // wrong set phrases and collocations
        triggers.extend(pt! {
            "could", "of" => "could have",
            "could", "of" => "could've",
            "couldn't", "of" => "couldn't have",
            "had", "of" => "had have",
            "had", "of" => "had've",
            "hadn't", "of" => "hadn't have",
            "should", "of" => "should have",
            "should", "of" => "should've",
            "shouldn't", "of" => "shouldn't have",
            "would", "of" => "would have",
            "would", "of" => "would've",
            "wouldn't", "of" => "wouldn't have",
            "discuss", "about" => "discuss",
            "discussed", "about" => "discussed",
            "discusses", "about" => "discusses",
            "discussing", "about" => "discussing",
            "same", "than" => "same as",
            "Same", "than" => "same as",
            "sooner","than","later" => "sooner rather than later",
            "sooner","than","later" => "sooner or later"
        });

        // belonging to multiple of the other categories
        triggers.extend(pt! {
            "same", "then" => "same as",
            "Same", "then" => "same as"
        });

        // near homophones
        triggers.extend(pt! {
            "want","be" => "won't be"
        });

        // normalization
        triggers.extend(pt! {
            "world","war","2" => "World War II",
            "world","War","ii" => "World War II",
            "World","war","ii" => "World War II",
            "World","War","iI" => "World War II",
            "World","War","Ii" => "World War II"
        });

        // countries and capitals with special casing or punctuation
        triggers.extend(pt! {
           "andorra","la","vella" => "Andorra la Vella",
           "Andorra","La","vella" => "Andorra la Vella",
           "Andorra","La","Vella" => "Andorra la Vella",
           "antigua","and","barbuda" => "Antigua and Barbuda",
           "Antigua","and","barbuda" => "Antigua and Barbuda",
           "Antigua","And","Barbuda" => "Antigua and Barbuda",
           "bosnia and herzegovina" => "Bosnia and Herzegovina",
           "Bosnia","and","herzegovina" => "Bosnia and Herzegovina",
           "Bosnia","And","herzegovina" => "Bosnia and Herzegovina",
           "democratic","republic","of","the","congo" => "Democratic Republic of the Congo",
           "Democratic","republic","of","the","congo" => "Democratic Republic of the Congo",
           "Democratic","Republic","Of","The","Congo" => "Democratic Republic of the Congo",
           "guinea","bissau" => "Guinea-Bissau",
           "Guinea","bissau" => "Guinea-Bissau",
           "Guinea","Bissau" => "Guinea-Bissau",
           "isle","of","man" => "Isle of Man",
           "Isle","of","man" => "Isle of Man",
           "Isle","Of","Man" => "Isle of Man",
           "ndjamena" => "N'Djamena",
           "Ndjamena" => "N'Djamena",
           "n'djamena" => "N'Djamena",
           "N'djamena" => "N'Djamena",
           "port","au","prince" => "Port-au-Prince",
           "Port","au","prince" => "Port-au-Prince",
           "Port","Au","Prince" => "Port-au-Prince",
           // port-au-prince won't work here because the left side has hyphens
           // Port-au-prince ditto
           // Port-Au-Prince ditto
           "porto","novovo" => "Porto-Novo",
           "Porto","novovo" => "Porto-Novo",
           "saint","kitts","and","nevis" => "Saint Kitts and Nevis",
           "Saint","kitts","and","nevis" => "Saint Kitts and Nevis",
           "Saint","Kitts","And","Nevis" => "Saint Kitts and Nevis",
           "saint","pierre","and","miqueleon" => "Saint Pierre and Miquelon",
           "Saint","pierre","and","miquelon" => "Saint Pierre and Miquelon",
           "Saint","Pierre","And","Miquelon" => "Saint Pierre and Miquelon",
           "saint","vincent","and","the","grenadines" => "Saint Vincent and the Grenadines",
           "Saint","vincent","and","the","grenadines" => "Saint Vincent and the Grenadines",
           "Saint","Vincent","And","The","Grenadines" => "Saint Vincent and the Grenadines",
           "st","georges" => "St. George's",
           // "st.","georges" => "St. George's",
           "st","george's" => "St. George's",
           // "st.","george's" => "St. George's",
           "St","georges" => "St. George's",
           // "St.","georges" => "St. George's",
           "St","george's" => "St. George's",
           // "St.","george's" => "St. George's",
           "St","Georges" => "St. George's",
           // "St.","Georges" => "St. George's",
           "St","George's" => "St. George's",
           "trinidad","and","tobago" => "Trinidad and Tobago",
           "Trinidad","and","tobago" => "Trinidad and Tobago",
           "Trinidad","And","Tobago" => "Trinidad and Tobago"
        });

        // countries and capitals with accents and diacritics
        triggers.extend(pt! {
            "asuncion" => "Asunción",
            "asunción" => "Asunción",
            "Asuncion" => "Asunción",
            "chisinau" => "Chișinău",
            "chișinău" => "Chișinău",
            "Chisinau" => "Chișinău",
            "bogota" => "Bogotá",
            "bogotá" => "Bogotá",
            "Bogota" => "Bogotá",
            "curacao" => "Curaçao",
            "curaçao" => "Curaçao",
            "curacao" => "Curaçao",
            "lome" => "Lomé",
            "lomé" => "Lomé",
            "Lome" => "Lomé",
            "male" => "Malé",
            "malé" => "Malé",
            "Male" => "Malé",
            "nukualofa" => "Nukuʻalofa",
            "Nukualofa" => "Nukuʻalofa",
            "nuku'alofa" => "Nukuʻalofa",
            "Nuku'alofa" => "Nukuʻalofa",
            "reykjavik" => "Reykjavík",
            "reykjavík" => "Reykjavík",
            "Reykjavik" => "Reykjavík",
            "san","jose" => "San José",
            "san","josé" => "San José",
            "San","jose" => "San José",
            "sao","tome" => "São Tomé",
            "são","tomé" => "São Tomé",
            "Sao","Tome" => "São Tomé",
            "sao","tome","and","principe" => "São Tomé and Príncipe",
            "são","tomé","and","príncipe" => "São Tomé and Príncipe",
            "Sao","Tome","and","Principe" => "São Tomé and Príncipe",
            "Sao","Tome","And","Principe" => "São Tomé and Príncipe",
            "torshavn" => "Tórshavn",
            "tórshavn" => "Tórshavn",
            "Torshavn" => "Tórshavn",
            "turkiye" => "Türkiye",
            "türkiye" => "Türkiye",
            "Turkiye" => "Türkiye",
            "yaounde" => "Yaoundé",
            "yaoundé" => "Yaoundé",
            "Yaounde" => "Yaoundé"
        });

        triggers.push(Rule {
            pattern: vec![pt!("L"), pt!(Period), pt!("L"), pt!(Period), pt!("M")],
            replace_with: vecword!("large language model"),
        });

        triggers.push(Rule {
            pattern: vec![
                pt!("L"),
                pt!(Period),
                pt!("L"),
                pt!(Period),
                pt!("M"),
                pt!(Period),
            ],
            replace_with: vecword!("large language model"),
        });

        Self { triggers }
    }
}

impl Default for Matcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Linter for Matcher {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        let mut match_tokens = Vec::new();

        for (index, _) in document.tokens().enumerate() {
            for trigger in &self.triggers {
                match_tokens.clear();

                for (p_index, pattern) in trigger.pattern.iter().enumerate() {
                    let Some(token) = document.get_token(index + p_index) else {
                        break;
                    };

                    let t_pattern = PatternToken::from_token(token, document);

                    if t_pattern != *pattern {
                        break;
                    }

                    match_tokens.push(token);
                }

                if match_tokens.len() == trigger.pattern.len() && !match_tokens.is_empty() {
                    let span = Span::new(
                        match_tokens.first().unwrap().span.start,
                        match_tokens.last().unwrap().span.end,
                    );

                    lints.push(Lint {
                        span,
                        lint_kind: LintKind::Miscellaneous,
                        suggestions: vec![Suggestion::ReplaceWith(trigger.replace_with.to_owned())],
                        message: format!(
                            "Did you mean “{}”?",
                            trigger.replace_with.iter().collect::<String>()
                        ),
                        priority: 15,
                    })
                }
            }
        }

        lints
    }

    fn description(&self) -> &'static str {
        "A collection of curated rules. A catch-all that will be removed in the future."
    }
}
