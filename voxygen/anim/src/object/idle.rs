use super::{
    super::{vek::*, Animation},
    ObjectSkeleton, SkeletonAttr,
};
use common::comp::item::ToolKind;

pub struct IdleAnimation;

impl Animation for IdleAnimation {
    type Dependency = (Option<ToolKind>, Option<ToolKind>, f64);
    type Skeleton = ObjectSkeleton;

    #[cfg(feature = "use-dyn-lib")]
    const UPDATE_FN: &'static [u8] = b"object_idle\0";

    #[cfg_attr(feature = "be-dyn-lib", export_name = "object_idle")]
    #[allow(clippy::approx_constant)] // TODO: Pending review in #587
    fn update_skeleton_inner(
        skeleton: &Self::Skeleton,
        (_active_tool_kind, _second_tool_kind, _global_time): Self::Dependency,
        _anim_time: f64,
        _rate: &mut f32,
        s_a: &SkeletonAttr,
    ) -> Self::Skeleton {
        let mut next = (*skeleton).clone();

        next.bone0.position = Vec3::new(s_a.bone0.0, s_a.bone0.1, s_a.bone0.2) / 11.0;

        next.bone1.position = Vec3::new(s_a.bone1.0, s_a.bone1.1, s_a.bone1.2) / 11.0;

        next
    }
}
