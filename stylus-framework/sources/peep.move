module stylus::peep;

use stylus::object::ID;

public fun peep<T: key>(owner_address: address, id: ID): &T;