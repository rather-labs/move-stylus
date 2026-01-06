module stylus::peep;

use stylus::object::UID;

public native fun peep<T: key>(owner_address: address, id: &UID): &T;