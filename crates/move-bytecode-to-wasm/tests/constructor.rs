mod common;
use crate::common::runtime_with_framework as runtime;
use alloy_sol_types::{SolCall, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::rstest;

mod constructor {
    use super::*;

    sol!(
        #[allow(missing_docs)]
        function constructor() public view;
        function readValue(bytes32 id) public view returns (uint64);
        function setValue(bytes32 id, uint64 value) public view;
    );

    #[rstest]
    fn test_constructor(
        #[with("constructor", "tests/constructor/constructor.move")] runtime: RuntimeSandbox,
    ) {
        // Create a new counter
        let call_data = constructorCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the object id emmited from the contract's events
        let object_id = runtime.obtain_uid();

        // Read initial value (should be 101)
        let call_data = readValueCall::new((object_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(101, return_data);
        assert_eq!(0, result);

        // Set value to 102
        let call_data = setValueCall::new((object_id, 102)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Call the constructor again. It should do nothing.
        let call_data = constructorCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the value again. If the constructor was ran twice, the value should be 101 instead of 102
        let call_data = readValueCall::new((object_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(102, return_data);
        assert_eq!(0, result);
    }
}

mod constructor_with_otw {
    use super::*;

    sol!(
        #[allow(missing_docs)]
        function constructor() public view;
        function readValue(bytes32 id) public view returns (uint64);
        function setValue(bytes32 id, uint64 value) public view;
    );

    #[rstest]
    fn test_constructor_with_otw(
        #[with("constructor_with_otw", "tests/constructor/constructor_with_otw.move")]
        runtime: RuntimeSandbox,
    ) {
        // Create a new counter
        let call_data = constructorCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the object id emmited from the contract's events
        let object_id = runtime.obtain_uid();

        // Read initial value (should be 101)
        let call_data = readValueCall::new((object_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(101, return_data);
        assert_eq!(0, result);

        // Set value to 102
        let call_data = setValueCall::new((object_id, 102)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Call the constructor again. It should do nothing.
        let call_data = constructorCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the value again. If the constructor was ran twice, the value should be 101 instead of 102
        let call_data = readValueCall::new((object_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(102, return_data);
        assert_eq!(0, result);
    }
}

mod constructor_with_return {
    use super::*;

    sol!(
        #[allow(missing_docs)]
        function constructor() public view;
    );

    #[rstest]
    #[should_panic]
    fn test_constructor_with_return(
        #[with(
            "constructor_with_return",
            "tests/constructor/constructor_with_return.move"
        )]
        runtime: RuntimeSandbox,
    ) {
        let call_data = constructorCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
    }
}

mod constructor_bad_args {
    use super::*;

    sol!(
        #[allow(missing_docs)]
        function constructor() public view;
    );

    #[rstest]
    #[should_panic]
    fn test_constructor_bad_args_1(
        #[with(
            "constructor_bad_args_1",
            "tests/constructor/constructor_bad_args_1.move"
        )]
        runtime: RuntimeSandbox,
    ) {
        let call_data = constructorCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
    }

    #[rstest]
    #[should_panic]
    fn test_constructor_bad_args_2(
        #[with(
            "constructor_bad_args_2",
            "tests/constructor/constructor_bad_args_2.move"
        )]
        runtime: RuntimeSandbox,
    ) {
        let call_data = constructorCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
    }

    #[rstest]
    #[should_panic]
    fn test_constructor_bad_args_3(
        #[with(
            "constructor_bad_args_3",
            "tests/constructor/constructor_bad_args_3.move"
        )]
        runtime: RuntimeSandbox,
    ) {
        let call_data = constructorCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
    }

    #[rstest]
    #[should_panic]
    fn test_constructor_bad_args_4(
        #[with(
            "constructor_bad_args_4",
            "tests/constructor/constructor_bad_args_4.move"
        )]
        runtime: RuntimeSandbox,
    ) {
        let call_data = constructorCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
    }

    #[rstest]
    #[should_panic]
    fn test_constructor_bad_args_5(
        #[with(
            "constructor_bad_args_5",
            "tests/constructor/constructor_bad_args_5.move"
        )]
        runtime: RuntimeSandbox,
    ) {
        let call_data = constructorCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
    }
}
