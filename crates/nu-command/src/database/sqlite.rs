use nu_protocol::{CustomValue, Span, Value, ShellError};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SQLiteConnection {
    pub path: String // TODO: should this be a str instead of a string?
}

impl CustomValue for SQLiteConnection {
    fn clone_value(&self,span:Span) -> Value {
        let cloned = SQLiteConnection { path: self.path.clone() };

        Value::CustomValue {
            val: Box::new(cloned),
            span,
        }
    }

    fn value_string(&self) -> String {
        self.typetag_name().to_string()
    }

    fn to_base_value(&self,span:Span) -> Result<Value, ShellError>  {
        let p = &self.path;
        Ok(Value::string(format!("a connection for {p}"), span))
    }

    fn as_any(&self) ->  &dyn std::any::Any {
        self
    }

    fn typetag_name(&self) ->  & 'static str {
        // TODO: I don't really understand how this is used
        "SQLiteConnection"
    }

    fn typetag_deserialize(&self) {
        // TODO: this is what the other custom_value implementations do but is it right?
        unimplemented!("typetag_deserialize")
    }
}