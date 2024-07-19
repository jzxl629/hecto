use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Default)]
pub struct Line {
    fragments: Vec<TextFragment>,
}

#[derive(Clone, Copy)]
enum GraphemeWidth {
    Half,
    Full,
}

impl GraphemeWidth {
    const fn saturating_add(self, other: usize) -> usize {
        match self {
            Self::Half => other.saturating_add(1),
            Self::Full => other.saturating_add(2),
        }
    }
}

struct TextFragment {
    grapheme: String,
    rendered_width: GraphemeWidth,
    replacement: Option<char>,
}

impl Line {
    pub fn from(line_str: &str) -> Self {
        let fragments = Self::str_to_fragments(line_str);
        Self { fragments }
    }

    fn str_to_fragments(line_str: &str) -> Vec<TextFragment> {
        let fragments = line_str
            .graphemes(true)
            .map(|grapheme| {
                let (replacement, rendered_width) = Self::replacement_character(grapheme)
                    .map_or_else(
                        || {
                            let unicode_width = grapheme.width();
                            match unicode_width {
                                0 | 1 => (None, GraphemeWidth::Half),
                                _ => (None, GraphemeWidth::Full),
                            }
                        },
                        |c| (Some(c), GraphemeWidth::Half),
                    );
                TextFragment {
                    grapheme: grapheme.to_string(),
                    rendered_width,
                    replacement,
                }
            })
            .collect();
        fragments
    }

    pub fn insert(&mut self, c: char, grapheme_index: usize) {
        let mut result = String::new();
        for (index, fragment) in self.fragments.iter().enumerate() {
            if index == grapheme_index {
                result.push(c);
            }
            result.push_str(&fragment.grapheme);
        }
        if grapheme_index >= self.fragments.len() {
            result.push(c);
        }
        self.fragments = Self::str_to_fragments(&result);
    }

    pub fn delete(&mut self, grapheme_index: usize) {
        let mut result = String::new();
        for (index, fragment) in self.fragments.iter().enumerate() {
            if index != grapheme_index {
                result.push_str(&fragment.grapheme);
            }
        }
        self.fragments = Self::str_to_fragments(&result);
    }

    fn replacement_character(string: &str) -> Option<char> {
        let width = string.width();
        match string {
            " " => None,
            "\t" => Some(' '),
            _ if width > 0 && string.trim().is_empty() => Some('␣'),
            _ if width == 0 => {
                let mut chars = string.chars();
                if let Some(c) = chars.next() {
                    if c.is_control() && chars.next().is_none() {
                        return Some('▯');
                    }
                }
                Some('·')
            }
            _ => None,
        }
    }

    pub fn graphemes_len(&self) -> usize {
        self.fragments.len()
    }

    pub fn get_visible_graphemes(&self, range: Range<usize>) -> String {
        let start = range.start;
        let end = range.end;
        if start >= end {
            return String::new();
        }
        let mut result = String::new();
        let mut current_pos = 0;
        for fragment in &self.fragments {
            let fragment_end = fragment.rendered_width.saturating_add(current_pos);
            if current_pos >= end {
                break;
            }
            if fragment_end > start {
                if fragment_end > end || current_pos < start {
                    result.push('⋯');
                } else if let Some(char) = fragment.replacement {
                    result.push(char);
                } else {
                    result.push_str(&fragment.grapheme);
                }
            }
            current_pos = fragment_end;
        }
        return result;
    }

    pub fn get_previous_width(&self, grapheme_index: usize) -> usize {
        self.fragments
            .iter()
            .take(grapheme_index)
            .map(|fragment| match fragment.rendered_width {
                GraphemeWidth::Half => 1,
                GraphemeWidth::Full => 2,
            })
            .sum()
    }

    pub fn append(&mut self, other: &Self) {
        let mut merged_line = String::new();
        for (_, fragment) in self.fragments.iter().enumerate() {
            merged_line.push_str(&fragment.grapheme);
        }
        for (_, fragment) in other.fragments.iter().enumerate() {
            merged_line.push_str(&fragment.grapheme);
        }
        self.fragments = Self::str_to_fragments(&merged_line);
    }

    pub fn split(&mut self, split_index: usize) -> Self {
        if split_index > self.graphemes_len() {
            return Self::default();
        }
        let split_fragments = self.fragments.split_off(split_index);
        Self {
            fragments: split_fragments,
        }
    }

    pub fn line_to_string(&self) -> String {
        let mut result = String::new();
        for (_, fragment) in self.fragments.iter().enumerate() {
            result.push_str(&fragment.grapheme);
        }
        result
    }
}
