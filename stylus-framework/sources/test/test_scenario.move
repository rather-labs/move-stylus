module stylus::test_scenario;
use stylus::tx_context::TxContext;

public native fun new_tx_context(): &mut TxContext;

public native fun drop_storage_object<T: key>(storage_object: T);
