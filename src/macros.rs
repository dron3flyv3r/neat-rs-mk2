macro_rules! game_manager {
    (
        $width:literal x $height:literal,
        game_objects: [
            $($game_object:expr),* $(,)?
        ]
    ) => {{
        let mut game_manager = GameManager::new(($width, $height));

        $(
            game_manager.add_game_object(Box::new($game_object));
        )*

        game_manager
    }};
}
