
pub trait Stringable: Sized {
    type Err;
    fn parse_str(s: &str) -> Result<Self, Self::Err>;
}
macro_rules! ugh {
    ($($ty: ty),*$(,)?) => {$(
        impl Stringable for $ty {
            type Err = <$ty as std::str::FromStr>::Err;
            fn parse_str(s: &str) -> Result<Self, Self::Err> {
                s.parse()
            }
        }

    )*}
}
ugh!(
    u8, u16, u32, u64, u128, usize,
    i8, i16, i32, i64, i128, isize,
    String,
);
impl Stringable for std::sync::Arc<str> {
    type Err = ();
    fn parse_str(s: &str) -> Result<Self, Self::Err> {
        Ok(<std::sync::Arc<str> as From<&str>>::from(s))
    }
}

impl Stringable for Box<str> {
    type Err = ();
    fn parse_str(s: &str) -> Result<Self, Self::Err> {
        Ok(<Box<str> as From<&str>>::from(s))
    }
}
