use super::Dialogue;
use super::PassageGroup;
use amethyst::error::Error;
use pest::{iterators::Pair, Parser};

#[derive(pest_derive::Parser)]
#[grammar = "assets/dialogue/dialogue.pest"]
struct DialogueParser;

pub fn parse(input: &str) -> Result<Dialogue, Error> {
    let mut ret = Dialogue::default();
    let mut root = DialogueParser::parse(Rule::root, input)?;

    for token in root.next().unwrap().into_inner() {
        match token.as_rule() {
            Rule::passage_group => {
                let group = build_passage_group(token);
                ret.passage_groups.push(group);
            }
            Rule::EOI => (),
            unexpected => {
                eprintln!("Found unexpected rule: {:?}", unexpected);
                unreachable!();
            }
        }
    }

    debug_assert!(!ret.passage_groups.is_empty());
    Ok(ret)
}

/// Traverse a sub-tree of the parsed AST to produce a `PassageGroup`.
fn build_passage_group(token: Pair<Rule>) -> PassageGroup {
    debug_assert_eq!(Rule::passage_group, token.as_rule());
    let mut group = PassageGroup::default();
    for token in token.into_inner() {
        match token.as_rule() {
            Rule::speaker => {
                let name = token.into_inner().next().unwrap();
                debug_assert_eq!(Rule::name, name.as_rule());
                group.speaker.push_str(name.as_str());
            }
            Rule::passages => {
                for token in token.into_inner() {
                    let passage = reformat_passage(token.as_str());
                    debug_assert!(!passage.is_empty());
                    group.passages.push(passage);
                }
            }
            unexpected => {
                eprintln!("Found unexpected rule: {:?}", unexpected);
                unreachable!();
            }
        }
    }

    debug_assert!(!group.passages.is_empty());
    group
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
