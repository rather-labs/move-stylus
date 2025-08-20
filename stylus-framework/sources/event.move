module stylus::event;

public native fun emit<T: copy + drop>(event: T);
