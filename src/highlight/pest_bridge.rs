use pest::iterators::Pairs;
use pest::RuleType;
use ratatui::style::Style;

use super::StyleSegment;

pub fn pest_pairs_to_segments<R, F>(pairs: Pairs<R>, style_for: F) -> Vec<StyleSegment>
where
    R: RuleType,
    F: Fn(R) -> Option<Style>,
{
    let mut segments = Vec::new();
    collect(pairs, &style_for, &mut segments);
    segments
}

fn collect<R, F>(pairs: Pairs<R>, style_for: &F, segments: &mut Vec<StyleSegment>)
where
    R: RuleType,
    F: Fn(R) -> Option<Style>,
{
    for pair in pairs {
        if let Some(style) = style_for(pair.as_rule()) {
            let span = pair.as_span();
            segments.push(StyleSegment {
                start: span.start(),
                end: span.end(),
                style,
            });
        } else {
            collect(pair.into_inner(), style_for, segments);
        }
    }
}
