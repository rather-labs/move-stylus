contract UnpackStruct {
    struct Bar {
        uint32 n;
        uint128 o;
    }


    function echoBar(Bar memory bar) external returns (uint32, uint128) {
        return (bar.n, bar.o);
    }
}
