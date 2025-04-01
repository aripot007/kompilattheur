pub enum InternalFuctions {
    Main,
    GenericPrint,
    Puts,
}

macro_rules! internal_prefix {
    ($name: expr) => {
        concat!("__smolpp_", $name)
    };
}

impl Into<&'static str> for InternalFuctions {
    fn into(self) -> &'static str {
        match self {
            InternalFuctions::Main => internal_prefix!("main"),
            InternalFuctions::GenericPrint => internal_prefix!("generic_print"),
            InternalFuctions::Puts => "puts",
        }
    }
}