use nu_engine::CallExt;
use nu_protocol::ast::{Call, CellPath};
use nu_protocol::engine::{Command, EngineState, Stack};
use nu_protocol::{
    Category, Example, IntoInterruptiblePipelineData, ListStream, PipelineData, Signature, Span,
    SyntaxShape, Type, Value,
};

#[derive(Clone)]
pub struct Get;

impl Command for Get {
    fn name(&self) -> &str {
        "get"
    }

    fn usage(&self) -> &str {
        "Extract data using a cell path."
    }

    fn signature(&self) -> nu_protocol::Signature {
        Signature::build("get")
            .input_output_types(vec![
                (
                    // TODO: This is too permissive; if we could express this
                    // using a type parameter it would be List<T> -> T.
                    Type::List(Box::new(Type::Any)),
                    Type::Any,
                ),
                (Type::Table(vec![]), Type::Any),
            ])
            .required(
                "cell_path",
                SyntaxShape::CellPath,
                "the cell path to the data",
            )
            .rest("rest", SyntaxShape::CellPath, "additional cell paths")
            .switch(
                "ignore-errors",
                "when data can't be accessed, return nothing instead of erroring out",
                Some('i'),
            )
            .switch(
                "sensitive",
                "get path in a case sensitive manner",
                Some('s'),
            )
            .category(Category::Filters)
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<nu_protocol::PipelineData, nu_protocol::ShellError> {
        let span = call.head;
        let cell_path: CellPath = call.req(engine_state, stack, 0)?;
        let rest: Vec<CellPath> = call.rest(engine_state, stack, 1)?;
        let sensitive = call.has_flag("sensitive");
        let ignore_errors = call.has_flag("ignore-errors");
        let ctrlc = engine_state.ctrlc.clone();
        let metadata = input.metadata();

        if rest.is_empty() {
            if ignore_errors {
                // replace errors with Value::Nothing
                match input.follow_cell_path(cell_path.members, !sensitive)? {
                    PipelineData::Value(value, _) => {
                        let mapped = match value {
                            Value::List { vals, span } => Value::List {
                                vals: vals
                                    .into_iter()
                                    .map(|val| match val {
                                        Value::Error { .. } => Value::nothing(span),
                                        v => v,
                                    })
                                    .collect(),
                                span,
                            },
                            Value::Error { .. } => Value::nothing(span),
                            v => v,
                        };
                        Ok(PipelineData::Value(mapped, None))
                    }
                    PipelineData::ListStream(stream, _) => {
                        let iter = stream.map(move |val| match val {
                            Value::Error { .. } => Value::nothing(span),
                            v => v,
                        });
                        Ok(PipelineData::ListStream(
                            ListStream {
                                stream: Box::new(iter),
                                ctrlc,
                            },
                            None,
                        ))
                    }
                    pd => Ok(pd),
                }
            } else {
                input.follow_cell_path(cell_path.members, !sensitive)
            }
        } else {
            let mut output = vec![];

            let paths = vec![cell_path].into_iter().chain(rest);

            // FIXME: can we do this without collecting the pipeline into a value?
            // Bit tricky to handle all edge cases (ex: `get foo 0` with mixed int and string paths)
            // but maybe we could special-case for when all paths are ints (`get 0 2`)
            let input = input.into_value(span);

            for path in paths {
                let val = input.clone().follow_cell_path(&path.members, !sensitive);

                if ignore_errors {
                    if let Ok(val) = val {
                        output.push(val);
                    } else {
                        output.push(Value::nothing(span));
                    }
                } else {
                    output.push(val?);
                }
            }

            Ok(output.into_iter().into_pipeline_data(ctrlc))
        }
        .map(|x| x.set_metadata(metadata))
    }
    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "Get an item from a list",
                example: "[0 1 2] | get 1",
                result: Some(Value::test_int(1)),
            },
            Example {
                description: "Get a column from a table",
                example: "[{A: A0}] | get A",
                result: Some(Value::List {
                    vals: vec![Value::test_string("A0")],
                    span: Span::test_data(),
                }),
            },
            Example {
                description: "Get a cell from a table",
                example: "[{A: A0}] | get 0.A",
                result: Some(Value::test_string("A0")),
            },
            Example {
                description: "Extract the name of files as a list",
                example: "ls | get name",
                result: None,
            },
            Example {
                description: "Extract the name of the 3rd entry of a file list",
                example: "ls | get name.2",
                result: None,
            },
            Example {
                description: "Extract the name of the 3rd entry of a file list (alternative)",
                example: "ls | get 2.name",
                result: None,
            },
            Example {
                description: "Extract the cpu list from the sys information record",
                example: "sys | get cpu",
                result: None,
            },
            Example {
                description: "Getting Path/PATH in a case insensitive way",
                example: "$env | get paTH",
                result: None,
            },
            Example {
                description: "Getting Path in a case sensitive way, won't work for 'PATH'",
                example: "$env | get -s Path",
                result: None,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_examples() {
        use crate::test_examples;

        test_examples(Get)
    }
}
