use anyhow::{bail, ensure, Context, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

pub fn parse_mapping(chars: &str) -> Result<Vec<KeyEvent>> {
    let tokens = to_tokens(chars)?;
    tokens.into_iter().map(Token::try_into_key_event).collect()
}

const fn is_char_valid(ch: char) -> bool {
    // reserve '^' for future
    ch != '^' && ch.is_ascii_graphic()
}

fn to_tokens(chars: &str) -> Result<Vec<Token<'_>>> {
    let mut tokens = vec![];
    let mut diamond_start = None;

    let e = |ch, pos| anyhow::bail!("Unexpected char {ch} on pos {pos}");

    for (i, ch) in chars.chars().enumerate() {
        if ch.is_ascii_whitespace() {
            break;
        }

        assert!(is_char_valid(ch), "{ch:?} is not a valid char");

        match diamond_start {
            None if ch == '<' => diamond_start = Some(i),
            None if ch == '>' => e(ch, i)?,
            None => tokens.push(Token::Char(ch)),
            Some(_) if ch == '<' => e(ch, i)?,
            Some(start) if ch == '>' => {
                tokens.push(Token::Diamond(&chars[start + 1..i]));
                diamond_start = None;
            }
            Some(_) => {}
        }
    }

    Ok(tokens)
}

#[derive(Debug, PartialEq, Eq)]
enum Token<'a> {
    /// surrounded by '<>'
    Diamond(&'a str),
    Char(char),
}

impl<'a> Token<'a> {
    pub fn try_into_key_event(self) -> Result<KeyEvent> {
        match self {
            Token::Char(c) => {
                let modifier = if c.is_ascii_uppercase() {
                    KeyModifiers::SHIFT
                } else {
                    KeyModifiers::NONE
                };

                Ok(KeyEvent::new(KeyCode::Char(c), modifier))
            }
            Token::Diamond(string) => {
                let mut chars = string.chars();
                let first_char = chars.next().context("no start")?;

                if first_char.to_ascii_uppercase() == 'F' {
                    let num = string[1..].parse()?;
                    return Ok(KeyEvent::new(KeyCode::F(num), KeyModifiers::empty()));
                }

                if let Some(event) = to_known_special_keyevent(string) {
                    return Ok(event);
                }

                let modifier = match first_char.to_ascii_uppercase() {
                    'C' => KeyModifiers::CONTROL,
                    'M' => KeyModifiers::ALT,
                    _ => bail!("Unknown modifier: {first_char}"),
                };

                let next = chars.next();
                ensure!(next == Some('-'), "Expected -, got {next:?}");

                let code = chars
                    .next()
                    .context("Unexpected end of input between diamond brackets")?;

                ensure!(is_char_valid(code));
                ensure!(chars.next().is_none(), "trailing input");

                Ok(KeyEvent::new(KeyCode::Char(code), modifier))
            }
        }
    }
}

fn to_known_special_keyevent(string: &str) -> Option<KeyEvent> {
    let mut modifiers = KeyModifiers::empty();
    let code = match &*string.to_uppercase() {
        "ESC" => KeyCode::Esc,
        "BS" => KeyCode::Backspace,
        "DEL" => KeyCode::Backspace,
        "CR" => KeyCode::Enter,
        "TAB" => KeyCode::Tab,
        "S-TAB" => {
            modifiers = KeyModifiers::SHIFT;
            KeyCode::BackTab
        }
        "LEFT" => KeyCode::Left,
        "DOWN" => KeyCode::Down,
        "UP" => KeyCode::Up,
        "RIGHT" => KeyCode::Right,
        _ => return None,
    };

    Some(KeyEvent {
        code,
        modifiers,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })
}

#[cfg(test)]
mod test {
    use super::*;

    fn assert_tokens(mapping: &'static str, expected_tokens: &[Token<'static>]) {
        let tokens = to_tokens(mapping).unwrap();
        assert_eq!(tokens, expected_tokens);
    }

    #[test]
    fn test_to_tokens() {
        assert_tokens(
            "<C-b><C-a>x",
            &[
                Token::Diamond("C-b"),
                Token::Diamond("C-a"),
                Token::Char('x'),
            ],
        );
        assert_tokens(
            "xxd",
            &[Token::Char('x'), Token::Char('x'), Token::Char('d')],
        );
        assert_tokens(
            "<S-a>xd",
            &[Token::Diamond("S-a"), Token::Char('x'), Token::Char('d')],
        );
    }
}
