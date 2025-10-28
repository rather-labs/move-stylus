use move_compiler::parser::ast::{Ability_, NameAccessChain_, PathEntry, Type_};

pub fn contains_abilities(expected_abilities: &[Ability_], actual_abilities: &[Ability_]) -> bool {
    for ability in expected_abilities {
        if !actual_abilities.contains(ability) {
            return false;
        }
    }
    true
}

pub fn get_single_type_name(type_: &Type_) -> Option<String> {
    match type_ {
        Type_::Apply(path_entry) => match path_entry.value {
            NameAccessChain_::Single(PathEntry { name, .. }) => {
                Some(name.value.as_str().to_string())
            }
            _ => None,
        },
        Type_::Ref(_, t) => get_single_type_name(&t.as_ref().value),
        _ => None,
    }
}
