use crate::geometry::{Overflow, Point};
use crate::state::ResolvedElement;

pub(crate) fn hit_path(frame: &ResolvedElement, point: Point) -> Option<Vec<&ResolvedElement>> {
    let mut children: Vec<_> = frame.children.iter().collect();
    children.sort_by_key(|child| child.style.z_index);

    let clips_overflow =
        frame.style.overflow_x == Overflow::Scroll || frame.style.overflow_y == Overflow::Scroll;
    let may_hit_children = !clips_overflow || frame.rect.contains(point);
    if may_hit_children
        && let Some(mut child_path) = children
            .into_iter()
            .rev()
            .find_map(|child| hit_path(child, point))
    {
        let mut path = vec![frame];
        path.append(&mut child_path);
        return Some(path);
    }

    if frame.rect.contains(point) {
        return Some(vec![frame]);
    }

    None
}
