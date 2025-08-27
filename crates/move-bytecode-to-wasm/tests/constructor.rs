mod common;

use common::{runtime_sandbox::RuntimeSandbox, translate_test_package_with_framework};
use rstest::{fixture, rstest};

mod constructor {
    use alloy_primitives::FixedBytes;
    use alloy_sol_types::{SolCall, sol};

    use super::*;

    #[fixture]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "constructor";
        const SOURCE_PATH: &str = "tests/constructor/constructor.move";

        let mut translated_package =
            translate_test_package_with_framework(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function constructor() public view;
        function readValue(bytes32 id) public view returns (uint64);
        function setValue(bytes32 id, uint64 value) public view;
    );

    #[rstest]
    fn test_constructor(runtime: RuntimeSandbox) {
        // Create a new counter
        let call_data = constructorCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the object id emmited from the contract's events
        let object_id = runtime.log_events.lock().unwrap().recv().unwrap();
        let object_id = FixedBytes::<32>::from_slice(&object_id);

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
    use alloy_primitives::FixedBytes;
    use alloy_sol_types::{SolCall, sol};

    use super::*;

    #[fixture]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "constructor_with_otw";
        const SOURCE_PATH: &str = "tests/constructor/constructor_with_otw.move";

        let mut translated_package =
            translate_test_package_with_framework(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function constructor() public view;
        function readValue(bytes32 id) public view returns (uint64);
        function setValue(bytes32 id, uint64 value) public view;
    );

    #[rstest]
    fn test_constructor_with_otw(runtime: RuntimeSandbox) {
        // Create a new counter
        let call_data = constructorCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the object id emmited from the contract's events
        let object_id = runtime.log_events.lock().unwrap().recv().unwrap();
        let object_id = FixedBytes::<32>::from_slice(&object_id);

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
    use alloy_sol_types::{SolCall, sol};

    use super::*;

    #[fixture]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "constructor_with_return";
        const SOURCE_PATH: &str = "tests/constructor/constructor_with_return.move";

        let mut translated_package =
            translate_test_package_with_framework(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function constructor() public view;
    );

    #[rstest]
    #[should_panic(expected = "expected no return values")]
    fn test_constructor_with_return(runtime: RuntimeSandbox) {
        let call_data = constructorCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
    }
}

mod constructor_bad_args_1 {
    use alloy_sol_types::{SolCall, sol};

    use super::*;

    #[fixture]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "constructor_bad_args_1";
        const SOURCE_PATH: &str = "tests/constructor/constructor_bad_args_1.move";

        let mut translated_package =
            translate_test_package_with_framework(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function constructor() public view;
    );

    #[rstest]
    #[should_panic(expected = "invalid arguments")]
    fn test_constructor_bad_args_1(runtime: RuntimeSandbox) {
        let call_data = constructorCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
    }
}

mod constructor_bad_args_2 {
    use alloy_sol_types::{SolCall, sol};

    use super::*;

    #[fixture]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "constructor_bad_args_2";
        const SOURCE_PATH: &str = "tests/constructor/constructor_bad_args_2.move";

        let mut translated_package =
            translate_test_package_with_framework(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function constructor() public view;
    );

    #[rstest]
    #[should_panic(expected = "invalid arguments")]
    fn test_constructor_bad_args_2(runtime: RuntimeSandbox) {
        let call_data = constructorCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
    }
}

mod constructor_bad_args_3 {
    use alloy_sol_types::{SolCall, sol};

    use super::*;

    #[fixture]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "constructor_bad_args_3";
        const SOURCE_PATH: &str = "tests/constructor/constructor_bad_args_3.move";

        let mut translated_package =
            translate_test_package_with_framework(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function constructor() public view;
    );

    #[rstest]
    #[should_panic(expected = "invalid arguments")]
    fn test_constructor_bad_args_3(runtime: RuntimeSandbox) {
        let call_data = constructorCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
    }
}

mod constructor_bad_args_4 {
    use alloy_sol_types::{SolCall, sol};

    use super::*;

    #[fixture]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "constructor_bad_args_4";
        const SOURCE_PATH: &str = "tests/constructor/constructor_bad_args_4.move";

        let mut translated_package =
            translate_test_package_with_framework(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function constructor() public view;
    );

    #[rstest]
    #[should_panic(expected = "invalid arguments")]
    fn test_constructor_bad_args_4(runtime: RuntimeSandbox) {
        let call_data = constructorCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
    }
}

mod constructor_bad_args_5 {
    use alloy_sol_types::{SolCall, sol};

    use super::*;

    #[fixture]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "constructor_bad_args_5";
        const SOURCE_PATH: &str = "tests/constructor/constructor_bad_args_5.move";

        let mut translated_package =
            translate_test_package_with_framework(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function constructor() public view;
    );

    #[rstest]
    #[should_panic(expected = "invalid arguments")]
    fn test_constructor_bad_args_5(runtime: RuntimeSandbox) {
        let call_data = constructorCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
    }
}
