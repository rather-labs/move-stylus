use move_compiler::parser::ast::{Attribute_, AttributeValue_, StructDefinition, Value_};
#[derive(Debug)]
/// This struct represents the properties of a event struct.
pub struct Event {
    /// Event name
    pub name: String,

    /// `true` if the  event is anonymous, otherwise `false`
    pub is_anonymous: bool,

    /// Indexed parameters. Indexed parameters are parsed in order and there can be max of 4
    /// anonymous events and 3 for non-anonymous parameters.
    pub indexes: u8,
}

impl Event {
    pub fn try_from(struct_definition: &StructDefinition) -> Result<Self, ()> {
        // Find the attribute we neekd
        for attribute in &struct_definition.attributes {
            for att in &attribute.value {
                let parametrized = match &att.value {
                    Attribute_::Parameterized(n, spanned) if n.value.as_str() == "ext" => {
                        &spanned.value
                    }
                    _ => continue,
                };

                // To be an event, the first named parameter must be "event"
                let mut event = match parametrized.first() {
                    Some(p) if p.value.attribute_name().value.as_str() == "event" => Event {
                        name: struct_definition.name.to_string(),
                        is_anonymous: false,
                        indexes: 0,
                    },
                    _ => return Err(()),
                };

                for attribute in parametrized.iter().skip(1) {
                    match &attribute.value {
                        Attribute_::Name(n) if n.value.as_str() == "anonymous" => {
                            event.is_anonymous = true
                        }
                        Attribute_::Assigned(n, spanned1) if n.value.as_str() == "indexes" => {
                            match &spanned1.value {
                                AttributeValue_::Value(v) if matches!(v.value, Value_::Num(_)) => {
                                    match v.value {
                                        Value_::Num(n) => {
                                            let indexes = n.parse::<u8>().unwrap();
                                            if indexes <= 4 {
                                                event.indexes = indexes
                                            } else {
                                                panic!(
                                                    "an event can't have more than {indexes} indexes"
                                                )
                                            }
                                        }
                                        _ => todo!(),
                                    }
                                }
                                _ => panic!("invalid attribute {:?} in event", spanned1.value,),
                            }
                        }
                        _ => panic!(
                            "invalid attribute {} in event",
                            attribute.value.attribute_name()
                        ),
                    }
                }

                if event.is_anonymous && event.indexes == 4 {
                    panic!("an anonymous event can't have 4 indexes, maximum is 3");
                }

                return Ok(event);
            }
        }
        Err(())
    }
}
