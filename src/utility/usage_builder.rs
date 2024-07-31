
use nonempty::NonEmpty;

use crate::utility::*;


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
    pub triggers: NonEmpty<String>,
    usage: Vec<Usage>,
    example: Option<String>,
}

impl UsageBuilder {

    pub fn new(triggers: NonEmpty<String>) -> UsageBuilder {
        UsageBuilder {
            triggers: triggers,
            usage: Vec::new(),
            example: None,
        }
    }

    pub fn new_usage(mut self) -> Self {
        self.usage.push(Vec::new());
        self
    }

    fn add_parameter(mut self, parameter: Parameter) -> Self {
        let mut current_usage = self.usage.last_mut();
        if current_usage.is_none() {
            self = self.new_usage();
            current_usage = self.usage.last_mut();
        }
        current_usage.unwrap().push(parameter);
        self
    }

    pub fn add_constant<'a>(mut self, parameter: impl ToList<&'a str>) -> Self {
        for name in parameter.to_list().into_iter() {
            self = self.add_parameter(Parameter {
                name: name.to_string(),
                param_type: ParameterType::Constant,
            });
        }
        self
    }

    pub fn add_required<'a>(mut self, parameter: impl ToList<&'a str>) -> Self {
        for name in parameter.to_list().into_iter() {
            self = self.add_parameter(Parameter {
                name: name.to_string(),
                param_type: ParameterType::Required,
            });
        }
        self
    }

    pub fn add_optional<'a>(mut self, parameter: impl ToList<&'a str>) -> Self {
        for name in parameter.to_list().iter() {
            self = self.add_parameter(Parameter {
                name: name.to_string(),
                param_type: ParameterType::Optional,
            });
        }
        self
    }

    pub fn example(mut self, example: &str) -> Self {
        self.example = Some(example.to_string());
        self
    }

    fn build_usage(&self, usage: Option<&Usage>, prefix: &str, trigger: &str) -> String {

        let mut usage_string = format!("{}{}", prefix, trigger);

        match usage {
            Some(usage) => {
                usage.into_iter()
                    .for_each(|param| {
                        usage_string.push_str(
                            &match param.param_type {
                                ParameterType::Constant => format!(" {}", param.name.to_string()),
                                ParameterType::Required => format!(" <{}>", param.name.to_string()),
                                ParameterType::Optional => format!(" [{}]", param.name.to_string())
                            })
                    })
            },
            None => {},
        };

        usage_string
    }

    pub fn build(&self, prefix: &String) -> String {

        let mut usage_strings = Vec::new();

        self.triggers.iter()
            .for_each(|trigger| {
                usage_strings.push(
                    format!("`{}`",
                        match self.usage.is_empty() {
                            true  => self.build_usage(None, prefix, &trigger),
                            false => self.usage.iter()
                                .map(|usage| self.build_usage(Some(&usage), prefix, &trigger))
                                .collect::<Vec<String>>()
                                .join("\n"),
                        }))
            });

        if let Some(example) = &self.example {
            usage_strings.push(format!(
                    "\nExample Usage:\n`{}{}`",
                    prefix,
                    example));
        }

        usage_strings.join("\n")
    }

}
