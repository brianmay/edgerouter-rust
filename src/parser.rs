use std::unreachable;

use crate::types::{File, ObjectValue};

use super::types::Value;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "json.pest"]
struct JSONParser;

use pest::error::Error;

#[allow(clippy::result_large_err)]
pub fn parse_file(file: &str) -> Result<File, Error<Rule>> {
    let json = JSONParser::parse(Rule::json, file)?.next().unwrap();

    use pest::iterators::Pair;

    fn parse_value(pair: Pair<Rule>) -> Value {
        match pair.as_rule() {
            Rule::pairs => Value::Object(
                pair.into_inner()
                    .map(|pair| {
                        let mut inner_rules = pair.into_inner();
                        let maybe_first = inner_rules.next();
                        let maybe_second = inner_rules.next();
                        let maybe_third = inner_rules.next();

                        match (maybe_first, maybe_second, maybe_third) {
                            (Some(first), Some(second), Some(third)) => {
                                let object = first.as_str();
                                let name = second.as_str();
                                let value = parse_value(third);

                                ObjectValue::ObjectKeyValue(object, name, value)
                            }
                            (Some(first), Some(second), None) => {
                                let name = first.as_str();
                                let value = parse_value(second);
                                ObjectValue::KeyValue(name, value)
                            }
                            (Some(first), None, None) => {
                                let name = first.as_str();
                                ObjectValue::Key(name)
                            }
                            (_, _, _) => unreachable!(),
                        }
                    })
                    .collect(),
            ),
            Rule::string => Value::String(pair.into_inner().next().unwrap().as_str()),
            Rule::unquoted_string => Value::UnquotedString(pair.as_str()),
            Rule::keyword => Value::String(pair.as_str()),
            Rule::boolean => Value::Boolean(pair.as_str().parse().unwrap()),
            Rule::top => {
                // For parsing the top-level object only
                let mut inner_rules = pair.into_inner();
                let first = inner_rules.next().unwrap();
                let second = inner_rules.next().unwrap();

                let name = first.as_str();
                let value = parse_value(second);

                Value::Object(vec![ObjectValue::KeyValue(name, value)])
            }
            Rule::null => Value::Null,
            Rule::json
            | Rule::EOI
            | Rule::pair
            | Rule::value
            | Rule::inner
            | Rule::char
            | Rule::trailing_lines
            | Rule::inner_trailing_lines
            | Rule::object
            | Rule::WHITESPACE => {
                unreachable!()
            }
        }
    }

    // println!("json: {:?}\n", json);
    // panic!("ssss");
    let mut inner = json.into_inner();
    let top = inner.next().unwrap();
    let values = parse_value(top);

    let trailing_lines = (1..=3).map(|_| inner.next().unwrap().as_str()).collect();

    Ok(File {
        values,
        trailing_lines,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser() {
        let json = r#"
        person {
            name "John Doe"
            age 43
            metrics {
            height 1.72
            weight 73.5
            }
        }
        /* 1 */
        /* 2 */
        /* 3 */
        "#;

        let parsed = parse_file(json).unwrap();
        let expected_person: Value = Value::Object(vec![
            ObjectValue::KeyValue("name", Value::String("John Doe")),
            ObjectValue::KeyValue("age", Value::UnquotedString("43")),
            ObjectValue::KeyValue(
                "metrics",
                Value::Object(vec![
                    ObjectValue::KeyValue("height", Value::UnquotedString("1.72")),
                    ObjectValue::KeyValue("weight", Value::UnquotedString("73.5")),
                ]),
            ),
        ]);

        assert_eq!(
            parsed.values,
            Value::Object(vec![ObjectValue::KeyValue("person", expected_person)])
        );

        assert_eq!(parsed.trailing_lines, vec!["/* 1 */", "/* 2 */", "/* 3 */"]);
    }

    #[test]
    fn test_nested_object_parser() {
        let json = r#"
        person {
            name "John Doe"
            age 43
            metrics ideal {
                height 1.72
                weight 73.5
            }
            metrics actual {
                height 1.72
                weight 173.5
            }
        }
        /* 1 */
        /* 2 */
        /* 3 */
        "#;

        let parsed = parse_file(json).unwrap();
        let expected_person: Value = Value::Object(vec![
            ObjectValue::KeyValue("name", Value::String("John Doe")),
            ObjectValue::KeyValue("age", Value::UnquotedString("43")),
            ObjectValue::ObjectKeyValue(
                "metrics",
                "ideal",
                Value::Object(vec![
                    ObjectValue::KeyValue("height", Value::UnquotedString("1.72")),
                    ObjectValue::KeyValue("weight", Value::UnquotedString("73.5")),
                ]),
            ),
            ObjectValue::ObjectKeyValue(
                "metrics",
                "actual",
                Value::Object(vec![
                    ObjectValue::KeyValue("height", Value::UnquotedString("1.72")),
                    ObjectValue::KeyValue("weight", Value::UnquotedString("173.5")),
                ]),
            ),
        ]);

        assert_eq!(
            parsed.values,
            Value::Object(vec![ObjectValue::KeyValue("person", expected_person)])
        );

        assert_eq!(parsed.trailing_lines, vec!["/* 1 */", "/* 2 */", "/* 3 */"]);
    }

    #[test]
    fn test_keyword() {
        let json = r#"
        person {
            ethernet eth0 {
                address 192.168.0.1/24
                address 2001:44B8:4112:8A00::1/64
                disable
                duplex auto
            }
        }
        /* 1 */
        /* 2 */
        /* 3 */
        "#;

        let parsed = parse_file(json).unwrap();
        let expected_ethernet: Value = Value::Object(vec![
            ObjectValue::KeyValue("address", Value::UnquotedString("192.168.0.1/24")),
            ObjectValue::KeyValue(
                "address",
                Value::UnquotedString("2001:44B8:4112:8A00::1/64"),
            ),
            ObjectValue::Key("disable"),
            ObjectValue::KeyValue("duplex", Value::UnquotedString("auto")),
        ]);

        let expected_person = Value::Object(vec![ObjectValue::ObjectKeyValue(
            "ethernet",
            "eth0",
            expected_ethernet,
        )]);

        assert_eq!(
            parsed.values,
            Value::Object(vec![ObjectValue::KeyValue("person", expected_person)])
        );

        assert_eq!(parsed.trailing_lines, vec!["/* 1 */", "/* 2 */", "/* 3 */"]);
    }

    #[test]
    fn test_empty() {
        let json = r#"
        person {
            ethernet eth0 {
            }
            cpu {
            }
        }
        /* 1 */
        /* 2 */
        /* 3 */
        "#;

        let parsed = parse_file(json).unwrap();
        let expected_ethernet: Value = Value::Object(vec![]);
        let expected_cpu: Value = Value::Object(vec![]);

        let expected_person = Value::Object(vec![
            ObjectValue::ObjectKeyValue("ethernet", "eth0", expected_ethernet),
            ObjectValue::KeyValue("cpu", expected_cpu),
        ]);

        assert_eq!(
            parsed.values,
            Value::Object(vec![ObjectValue::KeyValue("person", expected_person)])
        );

        assert_eq!(parsed.trailing_lines, vec!["/* 1 */", "/* 2 */", "/* 3 */"]);
    }
}
