use serde_json::{Value, json};

pub fn run(root: &Value) -> Vec<(String, Value)> {
    let suite = root.get("suite").and_then(|x| x.as_str()).unwrap_or("");
    if suite == "EP-OUTCOME-BINDING-v1-real-crypto" {
        return crate::suites::outcome_binding_exec::run_exec(root);
    } else {
        return crate::suites::outcome_binding_basic::run(root);
    }
}
