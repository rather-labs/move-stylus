use move_compiler::parser::ast::Ability_;

pub fn contains_abilities(expected_abilities: &[Ability_], actual_abilities: &[Ability_]) -> bool {
    for ability in expected_abilities {
        if !actual_abilities.contains(ability) {
            return false;
        }
    }
    true
}
