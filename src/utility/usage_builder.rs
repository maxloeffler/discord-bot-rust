
use nonempty::NonEmpty;

use crate::utility::traits::ToList;


enum ParameterType {
    Constant,
    Required,
    Optional,
}

struct Parameter {
    name: String,
    param_type: ParameterType,
}

type Usage = Vec<Parameter>;

pub struct UsageBuilder {
    command_names: NonEmpty<String>,
    usage: Vec<Usage>,
}

impl UsageBuilder {

    pub fn new(command_names: NonEmpty<String>) -> UsageBuilder {
        UsageBuilder {
            command_names,
            usage: Vec::new(),
        }
    }

    pub fn new_usage(&mut self) {
        self.usage.push(Vec::new());
    }

    fn add_parameter(&mut self, parameter: Parameter) {
        let mut current_usage = self.usage.last_mut();
        if current_usage.is_none() {
            self.new_usage();
            current_usage = self.usage.last_mut();
        }
        current_usage.unwrap().push(parameter);
    }

    pub fn add_constant<'a>(&mut self, parameter: impl ToList<&'a str>) {
        for name in parameter.to_list().into_iter() {
            self.add_parameter(Parameter {
                name: name.to_string(),
                param_type: ParameterType::Constant,
            });
        }
    }

    pub fn add_required<'a>(&mut self, parameter: impl ToList<&'a str>) {
        for name in parameter.to_list().into_iter() {
            self.add_parameter(Parameter {
                name: name.to_string(),
                param_type: ParameterType::Required,
            });
        }
    }

    pub fn add_optional<'a>(&mut self, parameter: impl ToList<&'a str>) {
        for name in parameter.to_list().iter() {
            self.add_parameter(Parameter {
                name: name.to_string(),
                param_type: ParameterType::Optional,
            });
        }
    }

    fn build_usage(&self, usage: &Usage, prefix: &str) -> String {

        let mut usage_string = format!("{}{}", prefix, self.command_names.head);
        for parameter in usage.iter() {
            match parameter.param_type {
                ParameterType::Constant => {
                    usage_string.push_str(&format!(" {}", parameter.name));
                },
                ParameterType::Required => {
                    usage_string.push_str(&format!(" <{}>", parameter.name));
                },
                ParameterType::Optional => {
                    usage_string.push_str(&format!(" [{}]", parameter.name));
                },
            }
        }
        usage_string

    }

    pub fn build(&self, prefix: &str) -> Option<String> {

        // no usage defined
        if self.usage.is_empty() {
            return None;
        }

        // build usage string
        let mut usage_string: String = self.usage
            .iter()
            .map(|usage| self.build_usage(usage, prefix))
            .collect::<Vec<String>>()
            .join("\n");

        // add alternative command names
        if !self.command_names.tail.is_empty() {
            usage_string.push_str(&format!("\nAlternative names: {:?}", self.command_names.tail));
        }

        Some(usage_string)

    }

}
