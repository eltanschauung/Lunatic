// Regression probe: compile against the native horizon_world/horizon_core libraries.
// Pass a prepared resource root containing facts.json and both cached shape tables.
use horizon_core::geometry::{Direction, VoxelShape};
use horizon_world::{
    block_collision::{BlockCollisionRegistry, DynamicCollisionWorld, ShulkerCollisionState},
    block_support::{BlockSupportRegistry, SupportType},
    collision::CachedCollisionRegistry,
    dynamic_support::{DynamicSupport, DynamicSupportWorld},
    level::data::load_registry,
    registry::{BlockStateId, BlockTableFile},
};
struct World;
impl DynamicSupportWorld for World {
    type Error = std::io::Error;
    fn shulker_is_closed(&mut self, _: [i32; 3]) -> Result<Option<bool>, Self::Error> {
        panic!("unexpected lookup")
    }
    fn moving_piston_shape(&mut self, _: [i32; 3]) -> Result<Option<VoxelShape>, Self::Error> {
        panic!("unexpected lookup")
    }
}
impl DynamicCollisionWorld for World {
    fn shulker_collision_state(
        &mut self,
        _: [i32; 3],
    ) -> Result<Option<ShulkerCollisionState>, Self::Error> {
        panic!("unexpected lookup")
    }
}
fn main() {
    let root = std::path::PathBuf::from(std::env::args().nth(1).unwrap());
    let r = load_registry(&root).unwrap();
    let f = BlockTableFile::open(&root.join("cached-collision.nbt"), &r).unwrap();
    let c = CachedCollisionRegistry::from_table(f.table("collision").unwrap()).unwrap();
    let binding = BlockCollisionRegistry::new(std::sync::Arc::new(c)).unwrap();
    let f = BlockTableFile::open(&root.join("cached-support.nbt"), &r).unwrap();
    let support = BlockSupportRegistry::from_table(f.table("support").unwrap()).unwrap();
    let mut checked = 0;
    for i in 0..r.len() {
        let id = BlockStateId(i as u32);
        let s = r.state(id).unwrap();
        if s.name != "lunatic:bushy_bamboo" {
            continue;
        }
        let mut vanilla = s.clone();
        vanilla.name = "minecraft:bamboo".into();
        let reference = DynamicSupport::from_state(&vanilla).unwrap().unwrap();
        for p in [[0, 64, 0], [100, 70, -13], [-200, -60, 91]] {
            let actual = binding.collision_shape(&mut World, id, p).unwrap();
            let expected = reference.collision_shape(&mut World, p).unwrap();
            assert_eq!(actual.boxes(), expected.boxes());
            for face in Direction::ALL {
                let a = support
                    .is_face_sturdy(&mut World, id, p, face, SupportType::Full)
                    .unwrap();
                let b = reference
                    .is_face_sturdy(&mut World, p, face, SupportType::Full)
                    .unwrap();
                assert_eq!(a, b);
            }
        }
        checked += 1;
    }
    assert_eq!(checked, 12);
    println!(
        "PASS all 12 Bushy Bamboo states: collision and support match vanilla at three positive/negative positions, including crash state 48287"
    );
}
