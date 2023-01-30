use std::collections::HashMap;
use crate::model::{BuildDoc, Document, EnvironmentStep, FunctionDoc, Import, InvokeFunctionStep, Parameter, Step, Value, ValueType};

#[test]
fn parse_valid_import_1() {
  let yaml = "
import:
-
  require: example/plugin
  version: 0.7.27
-
  require: example/other_plugin
  version: 0.1
  ";
  let parsed_obj: Document = serde_yaml::from_str(yaml).unwrap();
  let expected_obj = Document::Import(vec![
    Import {
      require: "example/plugin".into(),
      version: "0.7.27".into()
    },
    Import {
      require: "example/other_plugin".into(),
      version: "0.1".into()
    }
  ]);
  assert_eq!(parsed_obj, expected_obj);
}

#[test]
fn parse_valid_function_1() {
  let yaml = "
function:
  name: my_function
  parameter_spec:
    param1:
      type: integer
    param2:
      type: bool
      default: true
    param3:
      type: string
      default: foo
    param4:
      type: float
  steps:
    -
      action: copy_file
      source: src:assets/script.js
      dest: vm:/root/script.js
    -
      invoke_fn: install_docker
      version: 20.10.23
";
  let parsed_obj = serde_yaml::from_str(yaml).unwrap();
  let expected_obj = Document::Function(FunctionDoc {
    name: "my_function".into(),
    parameter_spec: HashMap::from([
        ("param1".into(), Parameter { type_: ValueType::Integer, default: None }),
        ("param2".into(), Parameter { type_: ValueType::Bool, default: Some(Value::Bool(true)) }),
        ("param3".into(), Parameter { type_: ValueType::String, default: Some(Value::String("foo".into())) }),
        ("param4".into(), Parameter { type_: ValueType::Float, default: None })
    ]),
    steps: vec![
      Step::EnvironmentStep(EnvironmentStep {
        action: "copy_file".into(),
        parameters: HashMap::from([
          ("source".into(), "src:assets/script.js".into()),
          ("dest".into(), "vm:/root/script.js".into())
        ])
      }),
      Step::InvokeFunctionStep(InvokeFunctionStep {
        invoke_fn: "install_docker".into(),
        parameters: HashMap::from([
          ("version".into(), "20.10.23".into())
        ])
      })
    ]
  });
  assert_eq!(parsed_obj, expected_obj);
}

#[test]
fn parse_valid_build_1() {
  let yaml = "
build:
  artifact_name: my_image
  from: alpine_317_virt
  steps:
  -
    action: copy_file
    source: src:assets/image.png
    dest: vm:/root/script.js
  -
    invoke_fn: install_docker
    version: 20.10.23
";
  let parsed_obj = serde_yaml::from_str(yaml).unwrap();
  let expected_obj = Document::Build(BuildDoc {
    artifact_name: "my_image".into(),
    from: "alpine_317_virt".to_string(),
    steps: vec![
      Step::EnvironmentStep(EnvironmentStep {
        action: "copy_file".into(),
        parameters: HashMap::from([
          ("source".into(), "src:assets/script.js".into()),
          ("dest".into(), "vm:/root/script.js".into())
        ])
      }),
      Step::InvokeFunctionStep(InvokeFunctionStep {
        invoke_fn: "install_docker".into(),
        parameters: HashMap::from([
          ("version".into(), "20.10.23".into())
        ])
      })
    ]
  });
  assert_eq!(parsed_obj, expected_obj);
}