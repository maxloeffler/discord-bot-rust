
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
    command_names: Vec<String>,
    usage: Vec<Usage>,
}

impl UsageBuilder {

    pub fn new(command_names: Vec<String>) -> UsageBuilder {
        UsageBuilder {
            command_names,
            usage: Vec::new(),
        }
    }

    pub fn new_usage(&mut self) {
        self.usage.push(Vec::new());
    }

    pub fn add_parameter(&mut self, parameter: Parameter) {
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

    fn build_usage(&self, usage: &Usage) -> String {

        let mut usage_string = String::new();
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
        let usage_string: String = self.usage
            .iter()
            .map(|usage| self.build_usage(usage))
            .collect::<Vec<String>>()
            .join("\n");
        Some(usage_string)

    }

}
