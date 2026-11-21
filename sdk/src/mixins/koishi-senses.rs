fn senses(value:bool,game:&crate::game::Game<'_>,uuid:u128)->bool {
    use crate::game::koishi::GameLunaticKoishiExt as _;
value&&!game.koishi_concealed(uuid)}
