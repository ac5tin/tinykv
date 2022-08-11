@0xaa051e8fd0019e92;

interface TinyKV {
    struct Record {
        key @0: Text;
        value @1: Data;
    }

    set @0(key: Text, value: Data) -> Record;
    get @1(key: Text) -> Record;
}
