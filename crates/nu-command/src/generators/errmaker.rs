use nu_engine::CallExt;
use nu_protocol::{
    ast::Call,
    engine::{Command, EngineState, Stack},
    Category, ListStream, PipelineData, ShellError, Signature, SyntaxShape, Type, Value,
};
use std::time::Duration;

#[derive(Clone)]
pub struct ErrMaker;

impl Command for ErrMaker {
    fn name(&self) -> &str {
        "errmaker"
    }

    fn signature(&self) -> Signature {
        Signature::build("errmaker")
            .input_output_types(vec![(Type::Nothing, Type::ListStream)])
            .required("max", SyntaxShape::Int, "max num to return before error")
            .category(Category::Generators)
    }

    fn usage(&self) -> &str {
        "Output sequences of numbers then error"
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let max: i64 = call.req(engine_state, stack, 0)?;
        let iter = IntSeq { max, count: 0 };
        let stream = ListStream {
            stream: Box::new(iter),
            ctrlc: None,
        };
        Ok(PipelineData::ListStream(stream, None))
    }
}

struct IntSeq {
    max: i64,
    count: i64,
}

impl Iterator for IntSeq {
    type Item = Value;
    fn next(&mut self) -> Option<Value> {
        self.count += 1;

        if self.count == self.max {
            let se = ShellError::IOError("max reached".into());
            return Some(Value::Error { error: se });
        }

        if self.count > self.max {
            return None;
        }

        let ret = Some(Value::test_int(self.count));

        std::thread::sleep(Duration::from_millis(100));
        ret
    }
}
