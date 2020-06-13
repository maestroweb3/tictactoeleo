use crate::{FunctionInput, InputValue};
use leo_inputs::{common::visibility::Visibility, files::File, InputParserError};

use leo_inputs::common::Private;
use snarkos_models::curves::PairingEngine;

#[derive(Clone)]
pub struct Inputs {
    program_inputs: Vec<Option<InputValue>>,
    public: Vec<InputValue>,
}

impl Inputs {
    pub fn new() -> Self {
        Self {
            program_inputs: vec![],
            public: vec![],
        }
    }

    pub fn get_inputs(&self) -> Vec<Option<InputValue>> {
        self.program_inputs.clone()
    }

    pub fn set_inputs(&mut self, inputs: Vec<Option<InputValue>>) {
        self.program_inputs = inputs;
    }

    pub fn set_inputs_size(&mut self, size: usize) {
        self.program_inputs = vec![None; size];
    }

    pub fn from_inputs_file(file: File, expected_inputs: Vec<FunctionInput>) -> Result<Self, InputParserError> {
        let mut private = vec![];
        let mut public = vec![];

        for section in file.sections.into_iter() {
            if section.header.name.value.eq("main") {
                for input in &expected_inputs {
                    // find input with matching name
                    let matched_input = section.assignments.clone().into_iter().find(|assignment| {
                        let visibility = assignment
                            .parameter
                            .visibility
                            .as_ref()
                            .map_or(true, |visibility| visibility.eq(&Visibility::Private(Private {})));

                        // name match
                        assignment.parameter.variable.value.eq(&input.identifier.name)
                                // visibility match
                                && visibility.eq(&input.private)
                                // type match
                                && assignment.parameter.type_.to_string().eq(&input._type.to_string())
                    });

                    match matched_input {
                        Some(assignment) => {
                            let value = InputValue::from_expression(assignment.parameter.type_, assignment.expression)?;
                            if let Some(Visibility::Public(_)) = assignment.parameter.visibility {
                                // Collect public inputs here
                                public.push(value.clone());
                            }

                            // push value to vector
                            private.push(Some(value));
                        }
                        None => return Err(InputParserError::InputNotFound(input.to_string())),
                    }
                }
            }
        }

        Ok(Self {
            program_inputs: private,
            public,
        })
    }

    pub fn get_public_inputs<E: PairingEngine>(&self) -> Result<Vec<E::Fr>, InputParserError> {
        let mut input_vec = vec![];

        for input in self.public.iter() {
            // get fields
            let mut input_fields = input.to_input_fields::<E>()?;

            // push fields to input_vec
            input_vec.append(&mut input_fields.0)
        }

        Ok(input_vec)
    }
}