use nu_engine::CallExt;
use nu_protocol::ast::Call;
use nu_protocol::engine::{Command, EngineState, Stack};
use nu_protocol::{
    Category, Example, IntoInterruptiblePipelineData, PipelineData, ShellError, Signature, Span,
    Spanned, SyntaxShape, Type, Value,
};
use std::collections::VecDeque;

#[derive(Clone)]
pub struct Take;

impl Command for Take {
    fn name(&self) -> &str {
        "take"
    }

    fn signature(&self) -> Signature {
        Signature::build("take")
            .input_output_types(vec![
                (Type::Table(vec![]), Type::Table(vec![])),
                (
                    Type::List(Box::new(Type::Any)),
                    Type::List(Box::new(Type::Any)),
                ),
                (Type::Binary, Type::Binary),
                (Type::Range, Type::List(Box::new(Type::Number))),
            ])
            .required(
                "n",
                SyntaxShape::Int,
                "the number of elements to return from the front (if positive) or the back (if negative) of the input",
            )
            .category(Category::Filters)
    }

    fn usage(&self) -> &str {
        "Take only the first (or optionally last) n elements of the input"
    }

    fn search_terms(&self) -> Vec<&str> {
        vec!["first", "slice", "head"]
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<nu_protocol::PipelineData, nu_protocol::ShellError> {
        let rows_desired: Spanned<i64> = call.req(engine_state, stack, 0)?;
        let take_last = rows_desired.item < 0;

        let rows_desired: usize = match rows_desired.item.abs().try_into() {
            Ok(r) => r,
            Err(_) => {
                return Err(ShellError::OperatorOverflow(
                    "converting rows parameter overflowed".into(),
                    rows_desired.span,
                ))
            }
        };

        let ctrlc = engine_state.ctrlc.clone();
        let metadata = input.metadata();

        let input_span = input.span();
        let input_not_supported_error = || -> ShellError {
            // can't always get a span for input, so try our best and fall back on the span for the `take` call if needed
            if let Some(span) = input_span {
                ShellError::UnsupportedInput("take does not support this input type".into(), span)
            } else {
                ShellError::UnsupportedInput(
                    "take was given an unsupported input type".into(),
                    call.span(),
                )
            }
        };

        match input {
            PipelineData::Value(val, _) => match val {
                Value::List { vals, .. } => {
                    if take_last {
                        // only keep last `rows_desired` rows in memory
                        let mut buf = VecDeque::<_>::new();
                        for row in vals.into_iter() {
                            if buf.len() == rows_desired {
                                buf.pop_front();
                            }

                            buf.push_back(row);
                        }

                        Ok(buf.into_pipeline_data(ctrlc).set_metadata(metadata))
                    } else {
                        Ok(vals
                            .into_iter()
                            .take(rows_desired)
                            .into_pipeline_data(ctrlc)
                            .set_metadata(metadata))
                    }
                }
                Value::Binary { val, span } => {
                    let slice: Vec<u8> = if take_last {
                        val.into_iter().rev().take(rows_desired).rev().collect()
                    } else {
                        val.into_iter().take(rows_desired).collect()
                    };
                    Ok(PipelineData::Value(
                        Value::Binary { val: slice, span },
                        metadata,
                    ))
                }
                Value::Range { val, .. } => {
                    if take_last {
                        // only keep last `rows_desired` rows in memory
                        let mut buf = VecDeque::<_>::new();
                        for row in val.into_range_iter(ctrlc.clone())? {
                            if buf.len() == rows_desired {
                                buf.pop_front();
                            }

                            buf.push_back(row);
                        }

                        Ok(buf.into_pipeline_data(ctrlc).set_metadata(metadata))
                    } else {
                        Ok(val
                            .into_range_iter(ctrlc.clone())?
                            .take(rows_desired)
                            .into_pipeline_data(ctrlc)
                            .set_metadata(metadata))
                    }
                }
                _ => Err(input_not_supported_error()),
            },
            PipelineData::ListStream(ls, metadata) => {
                if take_last {
                    // only keep last `rows_desired` rows in memory
                    let mut buf = VecDeque::<_>::new();
                    for row in ls {
                        if buf.len() == rows_desired {
                            buf.pop_front();
                        }

                        buf.push_back(row);
                    }

                    Ok(buf.into_pipeline_data(ctrlc).set_metadata(metadata))
                } else {
                    Ok(ls
                        .take(rows_desired)
                        .into_pipeline_data(ctrlc)
                        .set_metadata(metadata))
                }
            }
            _ => Err(input_not_supported_error()),
        }
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "Return the first 2 items of a list/table",
                example: "[1 2 3] | take 2",
                result: Some(Value::List {
                    vals: vec![Value::test_int(1), Value::test_int(2)],
                    span: Span::test_data(),
                }),
            },
            Example {
                description: "Return the last 2 items of a list/table",
                example: "[1 2 3] | take -2",
                result: Some(Value::List {
                    vals: vec![Value::test_int(2), Value::test_int(3)],
                    span: Span::test_data(),
                }),
            },
            Example {
                description: "Return the first two rows of a table",
                example: "echo [[editions]; [2015] [2018] [2021]] | take 2",
                result: Some(Value::List {
                    vals: vec![
                        Value::test_record(vec!["editions"], vec![Value::test_int(2015)]),
                        Value::test_record(vec!["editions"], vec![Value::test_int(2018)]),
                    ],
                    span: Span::test_data(),
                }),
            },
            Example {
                description: "Return the first 2 bytes of a binary value",
                example: "0x[01 23 45] | take 2",
                result: Some(Value::Binary {
                    val: vec![0x01, 0x23],
                    span: Span::test_data(),
                }),
            },
            Example {
                description: "Return the last 2 bytes of a binary value",
                example: "0x[01 23 45] | take -2",
                result: Some(Value::Binary {
                    val: vec![0x23, 0x45],
                    span: Span::test_data(),
                }),
            },
            Example {
                description: "Return the first 3 elements of a range",
                example: "1..10 | take 3",
                result: Some(Value::List {
                    vals: vec![Value::test_int(1), Value::test_int(2), Value::test_int(3)],
                    span: Span::test_data(),
                }),
            },
            Example {
                description: "Return the last 3 elements of a range",
                example: "1..10 | take -3",
                result: Some(Value::List {
                    vals: vec![Value::test_int(8), Value::test_int(9), Value::test_int(10)],
                    span: Span::test_data(),
                }),
            },
        ]
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_examples() {
        use crate::test_examples;

        test_examples(Take {})
    }
}
