//! The code here is mainly stolen from [egui]
//!
//! [egui]: https://github.com/emilk/egui/blob/339b28b4708450181ff93e1024aec328bac986cb/egui_demo_lib/src/syntax_highlighting.rs
use eframe::{egui, epaint::text::LayoutJob};
use syntect::{
    highlighting::ThemeSet,
    parsing::{SyntaxDefinition, SyntaxSet},
};

pub struct Highlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

impl Default for Highlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl Highlighter {
    fn new() -> Self {
        let mut ss = SyntaxSet::load_defaults_newlines().into_builder();
        ss.add(
            SyntaxDefinition::load_from_str(include_str!("../TOML.sublime-syntax"), true, None)
                .unwrap(),
        );

        Self {
            syntax_set: ss.build(),
            theme_set: ThemeSet::load_defaults(),
        }
    }

    pub fn highlight(&self, code: &str, lang: &str) -> LayoutJob {
        use eframe::{
            egui::{Color32, FontId, TextFormat},
            epaint::text::LayoutSection,
        };
        use syntect::{easy::HighlightLines, util::LinesWithEndings};
        let syntax = self.syntax_set.find_syntax_by_extension(lang).unwrap();
        let mut h = HighlightLines::new(syntax, &self.theme_set.themes["base16-ocean.dark"]);

        let mut job = LayoutJob {
            text: code.into(),
            ..LayoutJob::default()
        };

        for line in LinesWithEndings::from(code) {
            for (style, line) in h.highlight(line, &self.syntax_set) {
                let fg = style.foreground;

                job.sections.push(LayoutSection {
                    leading_space: 0.0,
                    byte_range: as_byte_range(code, line),
                    format: TextFormat {
                        font_id: FontId::monospace(14.0),
                        color: Color32::from_rgb(fg.r, fg.g, fg.b),
                        ..TextFormat::default()
                    },
                });
            }
        }

        job
    }
}

fn as_byte_range(whole: &str, range: &str) -> std::ops::Range<usize> {
    let whole_start = whole.as_ptr() as usize;
    let range_start = range.as_ptr() as usize;
    assert!(whole_start <= range_start);
    assert!(range_start + range.len() <= whole_start + whole.len());
    let offset = range_start - whole_start;
    offset..(offset + range.len())
}

pub fn highlight(ctx: &egui::Context, code: &str, language: &str) -> LayoutJob {
    impl egui::util::cache::ComputerMut<(&str, &str), LayoutJob> for Highlighter {
        fn compute(&mut self, (code, lang): (&str, &str)) -> LayoutJob {
            self.highlight(code, lang)
        }
    }

    type HighlightCache<'a> = egui::util::cache::FrameCache<LayoutJob, Highlighter>;

    let mut memory = ctx.memory();
    let highlight_cache = memory.caches.cache::<HighlightCache<'_>>();
    highlight_cache.get((code, language))
}
