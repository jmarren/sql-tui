use ratatui::text::Span;
use tree_sitter_highlight::{Highlight, HighlightConfiguration, HighlightEvent, Highlighter};
use crate::lib::styles::TextColor;


pub struct HighlightParser<'a>{
    pub highlighter: tree_sitter_highlight::Highlighter,
    pub sql_config: HighlightConfiguration,
    pub spans: Vec<Span<'a>>,
}

impl <'a> HighlightParser<'a> {
    pub fn new() -> HighlightParser<'a>  {
      let mut parser = tree_sitter::Parser::new();
      let language = tree_sitter_sequel::LANGUAGE;
      parser
          .set_language(&language.into())
          .expect("Error loading Sql parser");
     
     let hightlighter = Highlighter::new();
     let sql_language = tree_sitter_sequel::LANGUAGE.into();

     let mut sql_config = HighlightConfiguration::new(
                sql_language,
                "sql",
                tree_sitter_sequel::HIGHLIGHTS_QUERY,
                "",
                "",
         ).unwrap();


      sql_config.configure(&HIGHLIGHT_NAMES);
        
        HighlightParser { 
            highlighter: hightlighter,
            sql_config: sql_config,
            spans: Vec::<Span>::new(),
        }
    }

    pub fn highlight(&mut self, text: String) {
            // clear current spans
            self.spans.clear();

            // get highlights
            let highlights = self.highlighter.highlight(
                &self.sql_config,
                text.as_bytes(),
                None,
                |_| None
            ).unwrap();
    
           // declar spans and curr_color
           let mut spans = Vec::<Span>::new();
           let mut curr_color = TextColor::BurntOrange;

           // iterate through hightlights
           // on highlight start => set the curr_color
           // on source => apply the curr_color to get the styled span 
           //              and push it into spans
           for event in highlights {
               match event.unwrap() {
                   HighlightEvent::Source {start, end} => {
                       let target_str = text[start .. end].to_string().clone();
                       let span = curr_color.highlight(target_str.clone()).clone();
                       spans.push(span);
                   },
                   HighlightEvent::HighlightStart(s) => {
                       match s {
                           Highlight(33) => {
                               curr_color = TextColor::BurntOrange;
                           },
                           Highlight(14) => {
                               curr_color = TextColor::Cyan;
                           },
                           Highlight(38) => {
                               curr_color = TextColor::Magenta;
                           },
                           Highlight(46) => {
                               curr_color = TextColor::Blue1;
                           },
                           Highlight(48) => {
                               curr_color = TextColor::Todo;
                           },
                           Highlight(40) => {
                               curr_color = TextColor::Todo2;
                           },
                           _ => {
                                println!("missing color for s = {:?}", s);
                                curr_color = TextColor::Gray;
                           }
                       }
                   },
                   HighlightEvent::HighlightEnd => {},
               }
            }
           // set spans to the new styled spans
           self.spans = spans;
    }
}



static HIGHLIGHT_NAMES: [&str; 52] = [
                        "attribute",
                        "boolean",
                        "carriage-return",
                        "comment",
                        "comment.documentation",
                        "constant",
                        "constant.builtin",
                        "constructor",
                        "constructor.builtin",
                        "embedded",
                        "error",
                        "escape",
                        "function",
                        "function.builtin",
                        "keyword",
                        "markup",
                        "markup.bold",
                        "markup.heading",
                        "markup.italic",
                        "markup.link",
                        "markup.link.url",
                        "markup.list",
                        "markup.list.checked",
                        "markup.list.numbered",
                        "markup.list.unchecked",
                        "markup.list.unnumbered",
                        "markup.quote",
                        "markup.raw",
                        "markup.raw.block",
                        "markup.raw.inline",
                        "markup.strikethrough",
                        "module",
                        "number",
                        "operator",
                        "property",
                        "property.builtin",
                        "punctuation",
                        "punctuation.bracket",
                        "punctuation.delimiter",
                        "punctuation.special",
                        "string",
                        "string.escape",
                        "string.regexp",
                        "string.special",
                        "string.special.symbol",
                        "tag",
                        "type",
                        "type.builtin",
                        "variable",
                        "variable.builtin",
                        "variable.member",
                        "variable.parameter",
    ];
