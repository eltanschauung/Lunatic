// SPDX-License-Identifier: GPL-3.0-or-later
fn crow_glider(game: &crate::game::Game<'_>) -> core::ops::ControlFlow<bool> {
    if !game.lunatic_racial_glider() {
        return core::ops::ControlFlow::Continue(());
    }
    core::ops::ControlFlow::Break(game.player.as_ref().is_some_and(|p| {
        if p.__horizon_mixins_lunatic_fairy().racial_glide_armor_blocked {
            return false;
        }
        // This hook also maintains active glides: altitude gates takeoff only.
        if p.fall_flying {
            return true;
        }
        if p.ground.on_ground {
            return false;
        }
        use horizon_core::geometry::Axis;
        const MIN_CLEARANCE: f64 = 3.0;
        const EPS: f64 = 1.0e-7;
        let Some(player) = game.player.as_ref() else { return false; };
        let bounds = player.bounds();
        let feet = bounds.min(Axis::Y);
        let floor = feet - MIN_CLEARANCE;
        let mut scene = horizon_client::collisions::SceneCollisions {
            world: &game.world,
            binding: game.collision,
            subject: None,
        };
        for x in (bounds.min(Axis::X) + EPS).floor() as i32..=(bounds.max(Axis::X) - EPS).floor() as i32 {
            for z in (bounds.min(Axis::Z) + EPS).floor() as i32..=(bounds.max(Axis::Z) - EPS).floor() as i32 {
                // Include the row below the interval for tall collision shapes (fences).
                for y in floor.floor() as i32 - 1..=feet.floor() as i32 {
                    let state = game.world.block(x, y, z);
                    let Ok(shape) = game.collision.collision_shape(&mut scene, state, [x, y, z]) else {
                        return false;
                    };
                    for b in shape.boxes() {
                        if b.max(Axis::X) + f64::from(x) > bounds.min(Axis::X) + EPS
                            && b.min(Axis::X) + f64::from(x) < bounds.max(Axis::X) - EPS
                            && b.max(Axis::Z) + f64::from(z) > bounds.min(Axis::Z) + EPS
                            && b.min(Axis::Z) + f64::from(z) < bounds.max(Axis::Z) - EPS
                            && b.max(Axis::Y) + f64::from(y) > floor + EPS
                            && b.min(Axis::Y) + f64::from(y) < feet + EPS
                        {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }))
}
