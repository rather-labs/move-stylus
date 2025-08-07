module stylus::storage;

use stylus::object::UID;

/// This is the EVM slot used to save the mappings that save the objects
const OBJECTS_SLOT: address = @0x1;

public fun storage_read<T: key>(owner_id: address, id: UID): T {
    retrieve_from_storage(owner_id, id)
}

native fun retrieve_from_storage<T: key>(owner_id: address, id: UID): T;

// TODO: remove public
public native fun save_in_slot<T>(obj: T);
