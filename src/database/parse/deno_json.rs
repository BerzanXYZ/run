use crate::{
    database::{db::Database, errors::ParseError},
    script::Script,
};

const DENO_JSON: &'static str = "deno.json";

impl<'a> Database<'a> {
    pub fn from_deno_json(deno_json: &'a str) -> Result<Database<'a>, ParseError> {
        let mut db = Database::new();
        let chars = deno_json.chars().collect::<Vec<char>>();

        let mut bufs: Vec<(&'a str, usize)> = vec![];

        let mut line_no = 1;

        let mut is_inside_quotes = false;
        let mut is_inside_scripts = false;

        let mut start_index = 0;
        let mut all_len = 0;

        for i in 0..chars.len() {
            let ch = chars[i];
            let prev_ch = if i > 0 { chars.get(i - 1) } else { None };

            match ch {
                '"' => {
                    if prev_ch != Some(&'\\') {
                        is_inside_quotes = !is_inside_quotes;
                        if is_inside_quotes {
                            start_index = all_len + 1;
                        } else {
                            let buf = &deno_json[start_index..all_len];

                            if is_inside_scripts {
                                bufs.push((buf, line_no));
                            }

                            if buf == "tasks" {
                                is_inside_scripts = true;
                            }
                            start_index = 0;
                        }
                    }
                }

                '}' => {
                    if !is_inside_quotes && is_inside_scripts {
                        break;
                    }
                }

                '\n' => {
                    line_no += 1;
                }

                _ => (),
            }

            all_len += ch.len_utf8();
        }

        for i in 0..bufs.len() {
            if i % 2 == 0 {
                let (mut name, name_line_no) = bufs[i];

                name = name.trim();

                let command = match bufs.get(i + 1) {
                    Some((command, _)) => command.trim(),
                    None => return Err(ParseError::NoCommand(name_line_no, DENO_JSON)),
                };

                match db.add(name, Script::new(command)) {
                    Ok(()) => (),
                    Err(err) => return Err(err.into_parse_error(name_line_no, DENO_JSON)),
                };
            }
        }

        Ok(db)
    }
}

#[test]
fn r() {
    let package_json = r#"{
        "tasks": {
          "start": "deno run -A --watch=static/,routes/ dev.ts"
        },
        "importMap": "./import_map.json",
        "compilerOptions": {
          "jsx": "react-jsx",
          "jsxImportSource": "preact"
        }
      }"#;

    let db = Database::from_package_json(package_json).ok();

    let mut expected_db = Database::new();

    expected_db.add("compile", Script::new("tsc"));
    expected_db.add("bundle", Script::new("rollup -c"));
}
