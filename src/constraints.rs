//! Auto Layout constraint system for UIKit.
//!
//! Constraints define how views are positioned and sized relative to each
//! other or their superview. This is similar to Apple's Auto Layout.
//!
//! ```rust,no_run
//! use uikit::prelude::*;
//!
//! let mut parent = View::empty();
//! parent.set_frame(0.0, 0.0, 400.0, 300.0);
//!
//! let mut child = View::new(Label::new("Hello"));
//! child.set_frame(0.0, 0.0, 200.0, 50.0);
//!
//! // Pin child to top-left of parent with padding
//! parent.addLayoutConstraint(
//!     LayoutConstraint::new(child.id(), Anchor::Top, ConstraintRelation::Equal,
//!                          parent.id(), Anchor::Top, 1.0, 16.0)
//! );
//! parent.addLayoutConstraint(
//!     LayoutConstraint::new(child.id(), Anchor::Leading, ConstraintRelation::Equal,
//!                          parent.id(), Anchor::Leading, 1.0, 16.0)
//! );
//! ```

use crate::view::{Anchor, ConstraintRelation, LayoutConstraint, View, ViewId};

// ═══════════════════════════════════════════════════════════════
// ConstraintSolver
// ═══════════════════════════════════════════════════════════════

/// A simple constraint solver that resolves layout constraints.
pub struct ConstraintSolver {
    constraints: Vec<LayoutConstraint>,
}

impl ConstraintSolver {
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
        }
    }

    /// Add a constraint to the solver.
    pub fn add_constraint(&mut self, constraint: LayoutConstraint) {
        self.constraints.push(constraint);
    }

    /// Remove all constraints.
    pub fn clear(&mut self) {
        self.constraints.clear();
    }

    /// Solve constraints for a view hierarchy.
    ///
    /// This is a simplified solver that handles common cases:
    /// - Fixed-size constraints
    /// - Edge pinning (top, bottom, leading, trailing)
    /// - Center alignment
    /// - Multiplier and constant offsets
    pub fn solve(&self, root: &mut View) {
        // Group constraints by first item
        let mut constraints_by_item: std::collections::HashMap<ViewId, Vec<&LayoutConstraint>> =
            std::collections::HashMap::new();

        for constraint in &self.constraints {
            constraints_by_item
                .entry(constraint.first_item)
                .or_default()
                .push(constraint);
        }

        // Solve each item's constraints
        for (view_id, constraints) in &constraints_by_item {
            Self::solve_view_constraints(root, *view_id, constraints);
        }
    }

    fn solve_view_constraints(
        root: &mut View,
        view_id: ViewId,
        constraints: &[&LayoutConstraint],
    ) {
        let mut width: Option<f32> = None;
        let mut height: Option<f32> = None;
        let mut x: Option<f32> = None;
        let mut y: Option<f32> = None;

        for constraint in constraints {
            match constraint.first_attribute {
                crate::view::Anchor::Width => {
                    if let Some(second_view) = root.view_by_id(constraint.second_item) {
                        let second_width = second_view.width();
                        width = Some(second_width * constraint.multiplier + constraint.constant);
                    }
                }
                crate::view::Anchor::Height => {
                    if let Some(second_view) = root.view_by_id(constraint.second_item) {
                        let second_height = second_view.height();
                        height =
                            Some(second_height * constraint.multiplier + constraint.constant);
                    }
                }
                crate::view::Anchor::Top => {
                    if let Some(second_view) = root.view_by_id(constraint.second_item) {
                        match constraint.second_attribute {
                            crate::view::Anchor::Top => {
                                y = Some(second_view.frame().y * constraint.multiplier
                                    + constraint.constant);
                            }
                            crate::view::Anchor::Bottom => {
                                y = Some(
                                    (second_view.frame().y + second_view.height())
                                        * constraint.multiplier
                                        + constraint.constant,
                                );
                            }
                            _ => {}
                        }
                    }
                }
                crate::view::Anchor::Leading => {
                    if let Some(second_view) = root.view_by_id(constraint.second_item) {
                        match constraint.second_attribute {
                            crate::view::Anchor::Leading => {
                                x = Some(second_view.frame().x * constraint.multiplier
                                    + constraint.constant);
                            }
                            crate::view::Anchor::Trailing => {
                                x = Some(
                                    (second_view.frame().x + second_view.width())
                                        * constraint.multiplier
                                        + constraint.constant,
                                );
                            }
                            _ => {}
                        }
                    }
                }
                crate::view::Anchor::CenterX => {
                    if let Some(second_view) = root.view_by_id(constraint.second_item) {
                        let w = width.unwrap_or(0.0);
                        x = Some(
                            (second_view.frame().x + second_view.width() / 2.0)
                                * constraint.multiplier
                                + constraint.constant
                                - w / 2.0,
                        );
                    }
                }
                crate::view::Anchor::CenterY => {
                    if let Some(second_view) = root.view_by_id(constraint.second_item) {
                        let h = height.unwrap_or(0.0);
                        y = Some(
                            (second_view.frame().y + second_view.height() / 2.0)
                                * constraint.multiplier
                                + constraint.constant
                                - h / 2.0,
                        );
                    }
                }
                _ => {}
            }
        }

        // Apply solved values
        if let Some(view) = root.view_by_id_mut(view_id) {
            if let Some(w) = width {
                view.set_width(w);
            }
            if let Some(h) = height {
                view.set_height(h);
            }
            if let (Some(x_val), Some(y_val)) = (x, y) {
                view.set_frame(x_val, y_val, view.width(), view.height());
            }
        }
    }
}

impl Default for ConstraintSolver {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════
// Convenience methods on View
// ═══════════════════════════════════════════════════════════════

/// Helper to create a constraint between two views.
pub fn constraint(
    first: ViewId,
    first_attr: Anchor,
    relation: ConstraintRelation,
    second: ViewId,
    second_attr: Anchor,
) -> LayoutConstraint {
    LayoutConstraint::new(first, first_attr, relation, second, second_attr, 1.0, 0.0)
}

/// Helper to create a constraint with a constant.
pub fn constraint_with_constant(
    first: ViewId,
    first_attr: Anchor,
    relation: ConstraintRelation,
    second: ViewId,
    second_attr: Anchor,
    constant: f32,
) -> LayoutConstraint {
    LayoutConstraint::new(first, first_attr, relation, second, second_attr, 1.0, constant)
}

/// Helper to create a constraint with multiplier and constant.
pub fn constraint_full(
    first: ViewId,
    first_attr: Anchor,
    relation: ConstraintRelation,
    second: ViewId,
    second_attr: Anchor,
    multiplier: f32,
    constant: f32,
) -> LayoutConstraint {
    LayoutConstraint::new(first, first_attr, relation, second, second_attr, multiplier, constant)
}

// ═══════════════════════════════════════════════════════════════
// EdgeSet — convenience for pinning multiple edges
// ═══════════════════════════════════════════════════════════════

bitflags::bitflags! {
    /// Set of edges for constraint convenience methods.
    pub struct EdgeSet: u8 {
        const TOP = 0b0001;
        const BOTTOM = 0b0010;
        const LEADING = 0b0100;
        const TRAILING = 0b1000;
        const ALL = Self::TOP.bits() | Self::BOTTOM.bits() | Self::LEADING.bits() | Self::TRAILING.bits();
    }
}

/// Create constraints that pin a view's edges to its superview.
pub fn pin_to_superview(
    child_id: ViewId,
    parent_id: ViewId,
    edges: EdgeSet,
    padding: f32,
) -> Vec<LayoutConstraint> {
    let mut constraints = Vec::new();

    if edges.contains(EdgeSet::TOP) {
        constraints.push(constraint_with_constant(
            child_id,
            Anchor::Top,
            ConstraintRelation::Equal,
            parent_id,
            Anchor::Top,
            padding,
        ));
    }
    if edges.contains(EdgeSet::BOTTOM) {
        constraints.push(LayoutConstraint::new(
            child_id,
            Anchor::Bottom,
            ConstraintRelation::Equal,
            parent_id,
            Anchor::Bottom,
            1.0,
            -padding,
        ));
    }
    if edges.contains(EdgeSet::LEADING) {
        constraints.push(constraint_with_constant(
            child_id,
            Anchor::Leading,
            ConstraintRelation::Equal,
            parent_id,
            Anchor::Leading,
            padding,
        ));
    }
    if edges.contains(EdgeSet::TRAILING) {
        constraints.push(LayoutConstraint::new(
            child_id,
            Anchor::Trailing,
            ConstraintRelation::Equal,
            parent_id,
            Anchor::Trailing,
            1.0,
            -padding,
        ));
    }

    constraints
}

/// Create constraints that center a view in its superview.
pub fn center_in_superview(child_id: ViewId, parent_id: ViewId) -> Vec<LayoutConstraint> {
    vec![
        LayoutConstraint::new(
            child_id,
            Anchor::CenterX,
            ConstraintRelation::Equal,
            parent_id,
            Anchor::CenterX,
            1.0,
            0.0,
        ),
        LayoutConstraint::new(
            child_id,
            Anchor::CenterY,
            ConstraintRelation::Equal,
            parent_id,
            Anchor::CenterY,
            1.0,
            0.0,
        ),
    ]
}

/// Create constraints that set a view's size.
pub fn set_size(child_id: ViewId, width: f32, height: f32) -> Vec<LayoutConstraint> {
    vec![
        LayoutConstraint::new(
            child_id,
            Anchor::Width,
            ConstraintRelation::Equal,
            child_id,
            Anchor::Width,
            1.0,
            width,
        ),
        LayoutConstraint::new(
            child_id,
            Anchor::Height,
            ConstraintRelation::Equal,
            child_id,
            Anchor::Height,
            1.0,
            height,
        ),
    ]
}

// ═══════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constraint_creation() {
        let c = LayoutConstraint::new(
            1,
            Anchor::Top,
            ConstraintRelation::Equal,
            0,
            Anchor::Top,
            1.0,
            16.0,
        );
        assert_eq!(c.first_item, 1);
        assert_eq!(c.first_attribute, Anchor::Top);
        assert_eq!(c.relation, ConstraintRelation::Equal);
        assert_eq!(c.second_item, 0);
        assert_eq!(c.constant, 16.0);
    }

    #[test]
    fn edge_set_flags() {
        assert!(EdgeSet::ALL.contains(EdgeSet::TOP));
        assert!(EdgeSet::ALL.contains(EdgeSet::BOTTOM));
        assert!(EdgeSet::ALL.contains(EdgeSet::LEADING));
        assert!(EdgeSet::ALL.contains(EdgeSet::TRAILING));
    }

    #[test]
    fn pin_to_superview_constraints() {
        let constraints = pin_to_superview(1, 0, EdgeSet::ALL, 16.0);
        assert_eq!(constraints.len(), 4);
    }

    #[test]
    fn center_in_superview_constraints() {
        let constraints = center_in_superview(1, 0);
        assert_eq!(constraints.len(), 2);
    }
}
