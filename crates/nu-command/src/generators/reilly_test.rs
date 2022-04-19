use nu_engine::CallExt;
use nu_protocol::{
    ast::Call,
    engine::{Command, EngineState, Stack},
    Category, Example, IntoPipelineData, PipelineData, ShellError, Signature, Span, Spanned,
    SyntaxShape, Value,
};
use std::cmp;

#[derive(Clone)]
pub struct ReillyTest;

impl Command for ReillyTest {
    fn name(&self) -> &str {
        "reilly-test"
    }

    fn signature(&self) -> Signature {
        Signature::build("reilly-test").category(Category::Generators)
    }

    fn usage(&self) -> &str {
        "Reilly's test command"
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        Ok(Value::string("asdf".to_string(), call.head).into_pipeline_data())
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            description: "blah blah",
            example: "foo bar",
            result: None,
        }]
    }
}
