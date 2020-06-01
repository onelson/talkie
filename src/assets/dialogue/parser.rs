use super::Dialogue;
use amethyst::error::Error;
use pest::Parser;

#[derive(pest_derive::Parser)]
#[grammar = "assets/dialogue/dialogue.pest"]
struct DialogueParser;

pub fn parse(input: &str) -> Result<Dialogue, Error> {
    let mut ret = Dialogue::default();
    let mut root = DialogueParser::parse(Rule::root, input)?;

    for token in root.next().unwrap().into_inner() {
        match token.as_rule() {
            Rule::passage => {
                let passage = reformat_passage(token.as_str());
                debug_assert!(!passage.is_empty());
                ret.passages.push(passage)
            }
            Rule::EOI => (),
            _ => {
                unreachable!();
            }
        }
    }
    debug_assert!(!ret.passages.is_empty());
    Ok(ret)
}

/// Passages are blocks of text separated by 2 or more newlines, but the
/// newlines inside the passage text itself should be elided.
/// This is to say, we want to remove the newlines and ensure there is one (one)
/// space between the thing prior to the newline, and the next non-whitespace
/// character.
fn reformat_passage(passage: &str) -> String {
    passage
        .lines()
        .filter_map(|s| {
            let s = s.trim();

            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::reformat_passage;

    #[test]
    fn test_reformat_passage() {
        assert_eq!(
            "A B C",
            &reformat_passage(
                r#"  A
        B
        
            C   
        "#
            )
        );
    }
}
