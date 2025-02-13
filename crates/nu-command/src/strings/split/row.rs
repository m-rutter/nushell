use nu_engine::CallExt;
use nu_protocol::{
    ast::Call,
    engine::{Command, EngineState, Stack},
    Category, Example, PipelineData, ShellError, Signature, Span, Spanned, SyntaxShape, Value,
};

#[derive(Clone)]
pub struct SubCommand;

impl Command for SubCommand {
    fn name(&self) -> &str {
        "split row"
    }

    fn signature(&self) -> Signature {
        Signature::build("split row")
            .required(
                "separator",
                SyntaxShape::String,
                "the character that denotes what separates rows",
            )
            .category(Category::Strings)
    }

    fn usage(&self) -> &str {
        "splits contents over multiple rows via the separator."
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<nu_protocol::PipelineData, nu_protocol::ShellError> {
        split_row(engine_state, stack, call, input)
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "Split a string into rows of char",
                example: "echo 'abc' | split row ''",
                result: Some(Value::List {
                    vals: vec![
                        Value::test_string("a"),
                        Value::test_string("b"),
                        Value::test_string("c"),
                    ],
                    span: Span::test_data(),
                }),
            },
            Example {
                description: "Split a string into rows by the specified separator",
                example: "echo 'a--b--c' | split row '--'",
                result: Some(Value::List {
                    vals: vec![
                        Value::test_string("a"),
                        Value::test_string("b"),
                        Value::test_string("c"),
                    ],
                    span: Span::test_data(),
                }),
            },
        ]
    }
}

fn split_row(
    engine_state: &EngineState,
    stack: &mut Stack,
    call: &Call,
    input: PipelineData,
) -> Result<nu_protocol::PipelineData, nu_protocol::ShellError> {
    let name_span = call.head;
    let separator: Spanned<String> = call.req(engine_state, stack, 0)?;

    input.flat_map(
        move |x| split_row_helper(&x, &separator, name_span),
        engine_state.ctrlc.clone(),
    )
}

fn split_row_helper(v: &Value, separator: &Spanned<String>, name: Span) -> Vec<Value> {
    match v.span() {
        Ok(v_span) => {
            if let Ok(s) = v.as_string() {
                let splitter = separator.item.replace("\\n", "\n");
                s.split(&splitter)
                    .filter_map(|s| {
                        if s.trim() != "" {
                            Some(Value::string(s, v_span))
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                vec![Value::Error {
                    error: ShellError::PipelineMismatch("string".into(), name, v_span),
                }]
            }
        }
        Err(error) => vec![Value::Error { error }],
    }
}

// #[cfg(test)]
// mod tests {
//     use super::ShellError;
//     use super::SubCommand;

//     #[test]
//     fn examples_work_as_expected() -> Result<(), ShellError> {
//         use crate::examples::test as test_examples;

//         test_examples(SubCommand {})
//     }
// }
