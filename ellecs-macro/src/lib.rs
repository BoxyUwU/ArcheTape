#[macro_export]
macro_rules! spawn {
    (&mut $world:ident, $($c:expr),* $(,)?) => {
        $world.spawn()
            $(.with($c))*
            .build()
    };
}
