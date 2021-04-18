use trait_set::trait_set;

mod simple {
    use super::*;

    trait_set!{
        pub(crate) trait TraitSet = Send + Sync;
    }    

    fn test_set<T: TraitSet>(_arg: T) {}

    #[test]
    fn it_compiles() {
        test_set(10u8);
        test_set("hello");
    }
}

mod complex {
    use super::*;

    trait_set!{
        pub(crate) trait BytesIterator = Iterator<Item = u8>;
    }    

    fn test_set<T: BytesIterator>(_arg: T) {}

    #[test]
    fn it_compiles() {
        test_set([10u8, 20, 30].as_ref().iter().copied());
        test_set(b"abcde".iter().copied());
    }
}

mod generic {
    use super::*;

    trait_set!{
        pub(crate) trait GenericIterator<T> = Iterator<Item = T>;
    }    

    fn test_set<T: GenericIterator<u8>>(_arg: T) {}

    #[test]
    fn it_compiles() {
        test_set([10u8, 20, 30].as_ref().iter().copied());
        test_set(b"abcde".iter().copied());
    }
}

mod lifetimes {
    use super::*;

    trait_set!{
        pub(crate) trait Set = 'static + Send + Sync;
    }    

    fn test_set<T: Set>(_arg: T) {}

    #[test]
    fn it_compiles() {
        test_set([10u8, 20, 30].as_ref().iter().copied());
        test_set(b"abcde".iter().copied());
    }
}

mod multiple {
    use super::*;

    trait_set!{
        pub(crate) trait TraitSet = Send + Sync;
        pub trait BytesIterator = Iterator<Item = u8>;
        trait GenericIterator<T> = Iterator<Item = T>;
    }    

    fn test_set<T: TraitSet>(_arg: T) {}
    fn test_iter<T: BytesIterator>(_arg: T) {}
    fn test_generic_iter<T: GenericIterator<u8>>(_arg: T) {}

    #[test]
    fn it_compiles() {
        test_set(10u8);
        test_iter([10u8, 20, 30].as_ref().iter().copied());
        test_generic_iter([10u8, 20, 30].as_ref().iter().copied());
    }
}

mod combination {
    use super::*;

    trait_set!{
        pub trait ThreadSafe = Send + Sync;
        pub trait BytesIterator = Iterator<Item = u8>;
        pub trait ThreadSafeBytesIterator = ThreadSafe + BytesIterator;
    }    

    fn test_set<T: ThreadSafeBytesIterator>(_arg: T) {}

    #[test]
    fn it_compiles() {
        test_set([10u8, 20, 30].as_ref().iter().copied());
    }
}