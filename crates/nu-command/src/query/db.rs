use chrono::Local;
use nu_engine::CallExt;
use nu_protocol::{
    ast::Call,
    engine::{Command, EngineState, Stack},
    Category, Example, IntoPipelineData, PipelineData, ShellError, Signature, Span, Spanned,
    SyntaxShape, Value,
};

use crate::database::SQLiteConnection;

#[derive(Clone)]
pub struct SubCommand;

impl Command for SubCommand {
    fn name(&self) -> &str {
        "query db"
    }

    fn signature(&self) -> Signature {
        Signature::build("query db")
            .required(
                "query",
                SyntaxShape::String,
                "SQL to execute against the database",
            )
            .category(Category::Date) // TODO: change category
    }

    fn usage(&self) -> &str {
        "Query a database using SQL."
    }

    // TODO: add search terms

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<nu_protocol::PipelineData, nu_protocol::ShellError> {
        let sql: Spanned<String> = call.req(engine_state, stack, 0)?;
        let head = call.head;

        input.map(
            move |value| query_input(value, head, &sql),
            engine_state.ctrlc.clone(),
        )
        // TODO: check input type
        // Ok(Value::string("mockup".to_string(), call.head).into_pipeline_data())
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            description: "blah blah blah",
            example: r#""2021-10-22 20:00:12 +01:00" | date format "%Y-%m-%d""#,
            result: None,
        }]
    }
}

fn query_input(input: Value, head: Span, sql: &Spanned<String>) -> Value {
    match input {
        Value::CustomValue { val, span } => {
            let sqlite = val.as_any().downcast_ref::<SQLiteConnection>();

            if let Some(db) = sqlite {
                return Value::string("OMG it's a SQLite database!!!!".to_string(), head);
            }

            Value::Error {
                error: ShellError::PipelineMismatch("a SQLite database".to_string(), head, span),
            }
        }
        _ => {
            let input_span = match input.span() {
                Ok(sp) => sp,
                Err(_) => head, // FIXME: is there a better span to use here?
            };

            Value::Error {
                error: ShellError::PipelineMismatch("a SQLite database".to_string(), head, input_span),
            }
        }
    }
}
