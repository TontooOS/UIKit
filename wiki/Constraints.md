# Constraints

Auto Layout constraint system for positioning and sizing views relative to each other or their superview.

## LayoutConstraint

```rust
pub struct LayoutConstraint {
    pub first_item: ViewId,
    pub first_attribute: Anchor,
    pub relation: ConstraintRelation,
    pub second_item: ViewId,
    pub second_attribute: Anchor,
    pub multiplier: f32,
    pub constant: f32,
    pub priority: f32,
}
```

### Anchor

```rust
pub enum Anchor {
    Top, Bottom, Leading, Trailing,
    CenterX, CenterY, Width, Height,
}
```

### ConstraintRelation

```rust
pub enum ConstraintRelation {
    Equal,
    LessThanOrEqual,
    GreaterThanOrEqual,
}
```

## Creating Constraints

### `constraint()`

```rust
pub fn constraint(
    first: ViewId, first_attr: Anchor,
    relation: ConstraintRelation,
    second: ViewId, second_attr: Anchor,
) -> LayoutConstraint
```

### `constraint_with_constant()`

```rust
pub fn constraint_with_constant(
    first: ViewId, first_attr: Anchor,
    relation: ConstraintRelation,
    second: ViewId, second_attr: Anchor,
    constant: f32,
) -> LayoutConstraint
```

### `constraint_full()`

```rust
pub fn constraint_full(
    first: ViewId, first_attr: Anchor,
    relation: ConstraintRelation,
    second: ViewId, second_attr: Anchor,
    multiplier: f32, constant: f32,
) -> LayoutConstraint
```

## Convenience Functions

### `pin_to_superview()`

```rust
pub fn pin_to_superview(
    child_id: ViewId,
    parent_id: ViewId,
    edges: EdgeSet,
    padding: f32,
) -> Vec<LayoutConstraint>
```

Pins child edges to parent with padding.

### `center_in_superview()`

```rust
pub fn center_in_superview(child_id: ViewId, parent_id: ViewId) -> Vec<LayoutConstraint>
```

Centers child in parent.

### `set_size()`

```rust
pub fn set_size(child_id: ViewId, width: f32, height: f32) -> Vec<LayoutConstraint>
```

Sets fixed size.

## EdgeSet

```rust
pub struct EdgeSet: u8 {
    const TOP = 0b0001;
    const BOTTOM = 0b0010;
    const LEADING = 0b0100;
    const TRAILING = 0b1000;
    const ALL = TOP | BOTTOM | LEADING | TRAILING;
}
```

## ConstraintSolver

Simple constraint resolver for view hierarchies:

```rust
let mut solver = ConstraintSolver::new();
solver.add_constraint(constraint1);
solver.add_constraint(constraint2);
solver.solve(&mut root_view);
```

## Example

```rust
let mut parent = View::empty();
parent.set_frame(0.0, 0.0, 400.0, 300.0);

let mut child = View::new(Label::new("Hello"));
child.set_frame(0.0, 0.0, 200.0, 50.0);

// Pin child to top-left with 16px padding
let constraints = pin_to_superview(
    child.id(), parent.id(),
    EdgeSet::TOP | EdgeSet::LEADING,
    16.0,
);

// Center child in parent
let center = center_in_superview(child.id(), parent.id());

// Set fixed size
let size = set_size(child.id(), 200.0, 50.0);
```

## Cross References

- [View.md](View.md) -- View base class
- [ViewController.md](ViewController.md) -- view controllers
- [MAIN.md](MAIN.md) -- overview
