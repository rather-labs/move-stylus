mod common;

use common::{compare_human_readable_abi, compare_json_abi};
use rstest::rstest;

#[rstest]
#[case(
    "abi_error_1",
    "modules/abi_errors/abi_error_1.move",
    "json_format/abi_errors/abi_error_1.json"
)]
#[case(
    "abi_error_2",
    "modules/abi_errors/abi_error_2.move",
    "json_format/abi_errors/abi_error_2.json"
)]
#[case(
    "abi_error_3",
    "modules/abi_errors/abi_error_3.move",
    "json_format/abi_errors/abi_error_3.json"
)]
#[case(
    "events_1",
    "modules/events/events_1.move",
    "json_format/events/events_1.json"
)]
#[case(
    "events_2",
    "modules/events/events_2.move",
    "json_format/events/events_2.json"
)]
#[case(
    "events_anon_1",
    "modules/events/events_anon_1.move",
    "json_format/events/events_anon_1.json"
)]
#[case(
    "events_anon_2",
    "modules/events/events_anon_2.move",
    "json_format/events/events_anon_2.json"
)]
#[case(
    "generic_events",
    "modules/events/generic_events.move",
    "json_format/events/generic_events.json"
)]
#[case(
    "generics_1",
    "modules/structs/generics_1.move",
    "json_format/structs/generics_1.json"
)]
#[case(
    "generics_2",
    "modules/structs/generics_2.move",
    "json_format/structs/generics_2.json"
)]
#[case(
    "misc_structs",
    "modules/structs/misc_structs.move",
    "json_format/structs/misc_structs.json"
)]
fn test_json_abi(#[case] module_name: &str, #[case] module_path: &str, #[case] json_path: &str) {
    let module_path = format!("tests/{module_path}");
    let json_path = format!("tests/{json_path}");

    compare_json_abi(&json_path, &module_path, module_name).unwrap();
}

#[rstest]
#[case(
    "abi_error_1",
    "modules/abi_errors/abi_error_1.move",
    "human_readable/abi_errors/abi_error_1.sol"
)]
#[case(
    "abi_error_2",
    "modules/abi_errors/abi_error_2.move",
    "human_readable/abi_errors/abi_error_2.sol"
)]
#[case(
    "abi_error_3",
    "modules/abi_errors/abi_error_3.move",
    "human_readable/abi_errors/abi_error_3.sol"
)]
#[case(
    "events_1",
    "modules/events/events_1.move",
    "human_readable/events/events_1.sol"
)]
#[case(
    "events_2",
    "modules/events/events_2.move",
    "human_readable/events/events_2.sol"
)]
#[case(
    "events_anon_1",
    "modules/events/events_anon_1.move",
    "human_readable/events/events_anon_1.sol"
)]
#[case(
    "events_anon_2",
    "modules/events/events_anon_2.move",
    "human_readable/events/events_anon_2.sol"
)]
#[case(
    "generic_events",
    "modules/events/generic_events.move",
    "human_readable/events/generic_events.sol"
)]
#[case(
    "generics_1",
    "modules/structs/generics_1.move",
    "human_readable/structs/generics_1.sol"
)]
#[case(
    "generics_2",
    "modules/structs/generics_2.move",
    "human_readable/structs/generics_2.sol"
)]
#[case(
    "misc_structs",
    "modules/structs/misc_structs.move",
    "human_readable/structs/misc_structs.sol"
)]
fn test_human_readable_abi(
    #[case] module_name: &str,
    #[case] module_path: &str,
    #[case] human_readable_path: &str,
) {
    let module_path = format!("tests/{module_path}");
    let human_readable_path = format!("tests/{human_readable_path}");

    compare_human_readable_abi(&human_readable_path, &module_path, module_name).unwrap();
}
