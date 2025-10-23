//! Simple .obj and .mtl file parser library

use thiserror::Error;

/// An object in an obj file
#[derive(Debug, Clone, Default)]
pub struct Object {
    pub name: Option<String>,
}

/// A parsed obj file
#[derive(Debug, Clone)]
pub struct ParsedObj {
    /// The objects in the obj file
    pub objects: Vec<Object>,
}

/// An error while pasring an object file
#[derive(Debug, Error)]
pub enum ParseObjErr {
    /// Encountered an I/O error
    #[error("Encountered I/O error: {}", .0)]
    IO(#[from] std::io::Error),

    /// Unknown statement encountered
    #[error("Unknown statement encountered: {}", .0)]
    UnknownStatement(String),
}

#[derive(Debug, Default)]
struct ParserState {
    cur_obj: Option<Object>,
}

/// Parses the data from the provided reader
pub fn parse<R: std::io::BufRead>(reader: &mut R) -> Result<ParsedObj, ParseObjErr> {
    let mut line_num: usize = 0;
    let mut obj = ParsedObj { objects: vec![] };

    let mut line = String::new();

    let mut state = ParserState::default();

    loop {
        line.clear();
        line_num += 1;

        let len = reader.read_line(&mut line)?;

        if len == 0 {
            break;
        }

        parse_line(&mut state, &line, &mut obj)?;

        dbg!(line_num);
        dbg!(&state);
        dbg!(&obj);
    }

    Ok(obj)
}

fn parse_line(state: &mut ParserState, line: &str, obj: &mut ParsedObj) -> Result<(), ParseObjErr> {
    let line = line.trim();

    if line.starts_with('#') {
        // Ignore comment lines
        return Ok(());
    }

    let words = line.split_whitespace().collect::<Vec<_>>();

    let header = words[0];
    let rest = &words[1..];

    if header == "mtllib" {
        log::warn!("Ignoring not-yet-implemented mtllib line: {line}");
        return Ok(());
    }

    if header == "o" {
        if let Some(was_building) = state.cur_obj.take() {
            obj.objects.push(was_building);
        }

        let name = if rest.len() != 0 {
            Some(rest.join(" "))
        } else {
            None
        };

        state.cur_obj = Some(Object { name });

        return Ok(());
    }

    if header == "v" {
        todo!()
    }

    Err(ParseObjErr::UnknownStatement(line.to_string()))
}

#[cfg(test)]
mod test {
    use std::io::BufReader;

    use crate::parse;

    #[test]
    fn test_salping() {
        let file =
            std::fs::File::open("C:\\Users\\Woute\\Desktop\\objs\\Salpingectomy 36B.obj").unwrap();

        let mut reader = BufReader::new(file);

        let result = parse(&mut reader).unwrap();

        dbg!(result);
    }
}
