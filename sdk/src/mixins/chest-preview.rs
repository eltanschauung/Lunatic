fn chest(
    value: Option<crate::mobs::Part>,
    character: &crate::characters::Character,
    model: &crate::mobs::Part,
    slim: bool,
) -> Option<crate::mobs::Part> {
    crate::mobs::render::lunatic_chest::presentation(character.chest, slim, model).or(value)
}
