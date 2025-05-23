pub trait Physics {
    fn is_in_any_hole(
        &self,
        x: isize,
        y: isize,
        holes: &[(isize, isize, isize, isize)],
    ) -> bool;
    fn is_reqtuctangle_visible_on_screen(
        &self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        camera: &Camera,
        buffer_width: &isize,
        buffer_height: &isize,
    ) -> bool;
    fn is_point_in_requctangle(
        &self,
        x: f32,
        y: f32,
        origin_x: f32,
        origin_y: f32,
        width: f32,
        height: f32,
    ) -> bool {
        if x < origin_x {
            return false;
        }
        if x > origin_x + width {
            return false;
        }
        if y < origin_y {
            return false;
        }
        if y > origin_y + height {
            return false;
        }
        return true;
    }
    fn get_block_in_distance(
        &self,
        blocks: &Vec<Block>,
        pos_x: f32,
        pos_y: f32,
        max_distance: f32,
        blacklisted: Option<usize>,
        top: bool,
    ) -> Option<usize>;
    fn get_distance_between_positions(
        &self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
    ) -> f32;
    fn is_block_visible_on_screen(
        &self,
        block: &Block,
        camera: &Camera,
        width: &isize,
        height: &isize,
    ) -> bool {
        return self.is_reqtuctangle_visible_on_screen(
            block.x.get() as f32,
            block.y.get() as f32,
            block.width.get() as f32,
            block.height.get() as f32,
            camera,
            width,
            height,
        );
    }
}

mod fast;
pub use fast::LogicFast;

mod accurate;
pub use accurate::LogicAccurate;

use crate::custom::{Block, Camera};

const _: fn() = || {
    fn assert_impl<T: Physics>() {}
    assert_impl::<LogicFast>();
    assert_impl::<LogicAccurate>();
};
