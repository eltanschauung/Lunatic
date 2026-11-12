fn senses(value:bool,game:&crate::game::Game<'_>,uuid:u128)->bool {value&&!game.koishi_concealed(uuid)}
