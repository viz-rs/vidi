macro_rules! repeat {
    ($macro:ident $($name:ident $verb:tt )+) => {
        $(
            $macro!($name $verb);
        )+
    };
}
