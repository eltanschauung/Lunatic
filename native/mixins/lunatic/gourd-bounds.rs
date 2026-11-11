// SPDX-License-Identifier: GPL-3.0-or-later
fn gourd_bounds(value: Aabb, p: &Projectile) -> Aabb {
    if !gourd_motion::ours(p) {
        return value;
    }
    let axis = gourd_physics::forward(p.yaw, p.pitch);
    let extent = axis.map(|v| gourd_physics::RADIUS + v.abs() * gourd_physics::HALF_SEGMENT);
    let at = [p.position.x, p.position.y, p.position.z];
    Aabb::new(
        gourd_motion::vec(gourd_physics::sub(at, extent)),
        gourd_motion::vec(gourd_physics::add(at, extent)),
    )
}
