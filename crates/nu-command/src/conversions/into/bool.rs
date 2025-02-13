use nu_engine::CallExt;
use nu_protocol::{
    ast::{Call, CellPath},
    engine::{Command, EngineState, Stack},
    Category, Example, PipelineData, ShellError, Signature, Span, SyntaxShape, Value,
};

#[derive(Clone)]
pub struct SubCommand;

impl Command for SubCommand {
    fn name(&self) -> &str {
        "into bool"
    }

    fn signature(&self) -> Signature {
        Signature::build("into bool")
            .rest(
                "rest",
                SyntaxShape::CellPath,
                "column paths to convert to boolean (for table input)",
            )
            .category(Category::Conversions)
    }

    fn usage(&self) -> &str {
        "Convert value to boolean"
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<nu_protocol::PipelineData, nu_protocol::ShellError> {
        into_bool(engine_state, stack, call, input)
    }

    fn examples(&self) -> Vec<Example> {
        let span = Span::test_data();
        vec![
            Example {
                description: "Convert value to boolean in table",
                example: "echo [[value]; ['false'] ['1'] [0] [1.0] [$true]] | into bool value",
                result: Some(Value::List {
                    vals: vec![
                        Value::Record {
                            cols: vec!["value".to_string()],
                            vals: vec![Value::boolean(false, span)],
                            span,
                        },
                        Value::Record {
                            cols: vec!["value".to_string()],
                            vals: vec![Value::boolean(true, span)],
                            span,
                        },
                        Value::Record {
                            cols: vec!["value".to_string()],
                            vals: vec![Value::boolean(false, span)],
                            span,
                        },
                        Value::Record {
                            cols: vec!["value".to_string()],
                            vals: vec![Value::boolean(true, span)],
                            span,
                        },
                        Value::Record {
                            cols: vec!["value".to_string()],
                            vals: vec![Value::boolean(true, span)],
                            span,
                        },
                    ],
                    span,
                }),
            },
            Example {
                description: "Convert bool to boolean",
                example: "$true | into bool",
                result: Some(Value::boolean(true, span)),
            },
            Example {
                description: "convert integer to boolean",
                example: "1 | into bool",
                result: Some(Value::boolean(true, span)),
            },
            Example {
                description: "convert decimal string to boolean",
                example: "'0.0' | into bool",
                result: Some(Value::boolean(false, span)),
            },
            Example {
                description: "convert string to boolean",
                example: "'true' | into bool",
                result: Some(Value::boolean(true, span)),
            },
        ]
    }
}

fn into_bool(
    engine_state: &EngineState,
    stack: &mut Stack,
    call: &Call,
    input: PipelineData,
) -> Result<PipelineData, ShellError> {
    let head = call.head;
    let column_paths: Vec<CellPath> = call.rest(engine_state, stack, 0)?;

    input.map(
        move |v| {
            if column_paths.is_empty() {
                action(&v, head)
            } else {
                let mut ret = v;
                for path in &column_paths {
                    let r =
                        ret.update_cell_path(&path.members, Box::new(move |old| action(old, head)));
                    if let Err(error) = r {
                        return Value::Error { error };
                    }
                }

                ret
            }
        },
        engine_state.ctrlc.clone(),
    )
}

fn string_to_boolean(s: &str, span: Span) -> Result<bool, ShellError> {
    match s.trim().to_lowercase().as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        o => {
            let val = o.parse::<f64>();
            match val {
                Ok(f) => Ok(f.abs() >= f64::EPSILON),
                Err(_) => Err(ShellError::CantConvert(
                    "boolean".to_string(),
                    "string".to_string(),
                    span,
                )),
            }
        }
    }
}

fn action(input: &Value, span: Span) -> Value {
    match input {
        Value::Bool { .. } => input.clone(),
        Value::Int { val, .. } => Value::Bool {
            val: *val != 0,
            span,
        },
        Value::Float { val, .. } => Value::Bool {
            val: val.abs() >= f64::EPSILON,
            span,
        },
        Value::String { val, .. } => match string_to_boolean(val, span) {
            Ok(val) => Value::Bool { val, span },
            Err(error) => Value::Error { error },
        },
        _ => Value::Error {
            error: ShellError::UnsupportedInput(
                "'into bool' does not support this input".into(),
                span,
            ),
        },
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_examples() {
        use crate::test_examples;

        test_examples(SubCommand {})
    }
}
