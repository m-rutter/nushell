use itertools::Either;
use nu_engine::CallExt;
use nu_protocol::ast::{Call, RangeInclusion};
use nu_protocol::engine::{Command, EngineState, Stack};
use nu_protocol::{
    Category, Example, FromValue, IntoInterruptiblePipelineData, PipelineData, PipelineIterator,
    Range, ShellError, Signature, Span, Spanned, SyntaxShape, Value,
};

#[derive(Clone)]
pub struct DropNth;

impl Command for DropNth {
    fn name(&self) -> &str {
        "drop nth"
    }

    fn signature(&self) -> Signature {
        Signature::build("drop nth")
            .required(
                "row number or row range",
                // FIXME: we can make this accept either Int or Range when we can compose SyntaxShapes
                SyntaxShape::Any,
                "the number of the row to drop or a range to drop consecutive rows",
            )
            .rest("rest", SyntaxShape::Any, "the number of the row to drop")
            .category(Category::Filters)
    }

    fn usage(&self) -> &str {
        "Drop the selected rows."
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                example: "[sam,sarah,2,3,4,5] | drop nth 0 1 2",
                description: "Drop the first, second, and third row",
                result: Some(Value::List {
                    vals: vec![Value::test_int(3), Value::test_int(4), Value::test_int(5)],
                    span: Span::test_data(),
                }),
            },
            Example {
                example: "[0,1,2,3,4,5] | drop nth 0 1 2",
                description: "Drop the first, second, and third row",
                result: Some(Value::List {
                    vals: vec![Value::test_int(3), Value::test_int(4), Value::test_int(5)],
                    span: Span::test_data(),
                }),
            },
            Example {
                example: "[0,1,2,3,4,5] | drop nth 0 2 4",
                description: "Drop rows 0 2 4",
                result: Some(Value::List {
                    vals: vec![Value::test_int(1), Value::test_int(3), Value::test_int(5)],
                    span: Span::test_data(),
                }),
            },
            Example {
                example: "[0,1,2,3,4,5] | drop nth 2 0 4",
                description: "Drop rows 2 0 4",
                result: Some(Value::List {
                    vals: vec![Value::test_int(1), Value::test_int(3), Value::test_int(5)],
                    span: Span::test_data(),
                }),
            },
            Example {
                description: "Drop range rows from second to fourth",
                example: "echo [first second third fourth fifth] | drop nth (1..3)",
                result: Some(Value::List {
                    vals: vec![Value::test_string("first"), Value::test_string("fifth")],
                    span: Span::test_data(),
                }),
            },
        ]
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        // let mut rows: Vec<usize> = call.rest(engine_state, stack, 0)?;
        // rows.sort_unstable();
        // let pipeline_iter: PipelineIterator = input.into_iter();

        let number_or_range = extract_int_or_range(engine_state, stack, call)?;
        let rows = match number_or_range {
            Either::Left(row_number) => {
                let and_rows: Vec<Spanned<i64>> = call.rest(engine_state, stack, 1)?;

                let mut rows: Vec<_> = and_rows.into_iter().map(|x| x.item as usize).collect();
                rows.push(row_number as usize);
                rows.sort_unstable();
                rows
            }
            Either::Right(row_range) => {
                let from = row_range.from.as_integer()? as usize;
                let to = row_range.to.as_integer()? as usize;

                if matches!(row_range.inclusion, RangeInclusion::Inclusive) {
                    (from..=to).collect()
                } else {
                    (from..to).collect()
                }
            }
        };

        Ok(DropNthIterator {
            input: input.into_iter(),
            rows,
            current: 0,
        }
        .into_pipeline_data(engine_state.ctrlc.clone()))
    }
}

fn extract_int_or_range(
    engine_state: &EngineState,
    stack: &mut Stack,
    call: &Call,
) -> Result<Either<i64, Range>, ShellError> {
    let value = call.req::<Value>(engine_state, stack, 0)?;

    let int_opt = value.as_integer().map(Either::Left).ok();
    let range_opt: Result<nu_protocol::Range, ShellError> = FromValue::from_value(&value);

    let range_opt = range_opt.map(Either::Right).ok();

    int_opt.or(range_opt).ok_or_else(|| {
        ShellError::TypeMismatch(
            "int or range".into(),
            value.span().unwrap_or_else(|_| Span::new(0, 0)),
        )
    })
}

struct DropNthIterator {
    input: PipelineIterator,
    rows: Vec<usize>,
    current: usize,
}

impl Iterator for DropNthIterator {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(row) = self.rows.get(0) {
                if self.current == *row {
                    self.rows.remove(0);
                    self.current += 1;
                    let _ = self.input.next();
                    continue;
                } else {
                    self.current += 1;
                    return self.input.next();
                }
            } else {
                return self.input.next();
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_examples() {
        use crate::test_examples;

        test_examples(DropNth {})
    }
}
