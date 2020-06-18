use drawing::{Brush, DrawList, Extent, ShapeKind};
use euclid::Point2D;
use euclid::Rect;
use euclid::Size2D;

pub struct Pixels;
pub type Position = Point2D<i64, Pixels>;
pub type Dimensions = Size2D<i64, Pixels>;
pub type Rectangle = Rect<i64, Pixels>;

#[derive(Copy, Clone)]
pub struct LayoutSpace {
    offset: Position,
    available_width: Option<u32>,
    available_height: Option<u32>,
}

pub struct LayoutOutcome {
    consumed_space: Dimensions,
    remaining_space: LayoutSpace,
    pub draw_list: DrawList,
}

impl LayoutSpace {
    pub fn new(available_width: Option<u32>, available_height: Option<u32>) -> LayoutSpace {
        LayoutSpace {
            offset: Position::default(),
            available_width,
            available_height,
        }
    }

    fn with_offset(&self, offset: Position) -> LayoutSpace {
        LayoutSpace {
            offset: self.offset + offset.to_vector(),
            ..*self
        }
    }

    fn vertical_subsect(&self, consumed_height: u32) -> LayoutSpace {
        let remaining_height = match self.available_height {
            Some(available_height) => {
                Some(available_height.checked_sub(consumed_height).unwrap_or(0))
            }
            None => None,
        };

        LayoutSpace {
            offset: Position::new(self.offset.x, self.offset.y + consumed_height as i64),
            available_width: self.available_width,
            available_height: remaining_height,
        }
    }

    fn horizontal_subsect(&self, consumed_width: u32) -> LayoutSpace {
        let remaining_width = match self.available_width {
            Some(available_width) => Some(available_width.checked_sub(consumed_width).unwrap_or(0)),
            None => None,
        };

        LayoutSpace {
            offset: Position::new(self.offset.x + consumed_width as i64, self.offset.y),
            available_width: remaining_width,
            available_height: self.available_height,
        }
    }
}

pub enum LayoutItem {
    Leaf {
        leaf: Box<dyn LayoutLeaf>,
    },
    Container {
        container: Box<dyn LayoutContainer>,
        children: Vec<LayoutItem>,
    },
}

pub trait LayoutContainer {
    fn layout(&self, space: &LayoutSpace, children: &[LayoutItem]) -> LayoutOutcome;
}

pub trait LayoutLeaf {
    fn layout(&self, space: &LayoutSpace) -> LayoutOutcome;
}

pub struct WindowContainer {
    width: u32,
    height: u32,
}
pub struct VerticalStackContainer {}
pub struct ShapeLeaf {
    kind: ShapeKind,
    brush: Brush,
    dimensions: Dimensions,
}

impl WindowContainer {
    pub fn new(width: u32, height: u32) -> WindowContainer {
        WindowContainer { width, height }
    }
}

impl VerticalStackContainer {
    pub fn new() -> VerticalStackContainer {
        VerticalStackContainer {}
    }
}

impl ShapeLeaf {
    pub fn new(kind: ShapeKind, brush: Brush, dimensions: Dimensions) -> ShapeLeaf {
        ShapeLeaf {
            kind,
            brush,
            dimensions,
        }
    }
}

impl LayoutContainer for WindowContainer {
    fn layout(&self, _: &LayoutSpace, children: &[LayoutItem]) -> LayoutOutcome {
        assert_eq!(children.len(), 1);

        let space = LayoutSpace::new(Some(self.width), Some(self.height));

        match &children[0] {
            LayoutItem::Leaf { leaf } => leaf.layout(&space),
            LayoutItem::Container {
                container,
                children,
            } => container.layout(&space, children),
        }
    }
}

impl LayoutContainer for VerticalStackContainer {
    fn layout(&self, space: &LayoutSpace, children: &[LayoutItem]) -> LayoutOutcome {
        let mut consumed_space = Size2D::new(space.available_width.unwrap_or(0) as i64, 0);
        let mut remaining_space =
            LayoutSpace::new(space.available_width, None).with_offset(space.offset);

        let mut draw_list = DrawList::empty();

        for child in children {
            let outcome = match child {
                LayoutItem::Leaf { leaf } => leaf.layout(&remaining_space),
                LayoutItem::Container {
                    container,
                    children,
                } => container.layout(&remaining_space, &children),
            };

            remaining_space =
                remaining_space.vertical_subsect(outcome.consumed_space.height as u32);
            consumed_space = Size2D::new(
                consumed_space.width,
                consumed_space.height + outcome.consumed_space.height,
            );
            draw_list.push_list(outcome.draw_list);
        }

        LayoutOutcome {
            consumed_space,
            remaining_space,
            draw_list,
        }
    }
}

impl LayoutLeaf for ShapeLeaf {
    fn layout(&self, space: &LayoutSpace) -> LayoutOutcome {
        let mut draw_list = DrawList::empty();
        draw_list.push_shape(
            self.kind,
            self.brush,
            Extent::new(
                space.offset.x,
                space.offset.y,
                self.dimensions.width as u32,
                self.dimensions.height as u32,
            ),
        );

        LayoutOutcome {
            consumed_space: self.dimensions,
            remaining_space: *space,
            draw_list,
        }
    }
}

impl LayoutItem {
    pub fn container<C: LayoutContainer + 'static>(container: C) -> LayoutItem {
        LayoutItem::Container {
            container: Box::new(container),
            children: vec![],
        }
    }

    pub fn leaf<L: LayoutLeaf + 'static>(leaf: L) -> LayoutItem {
        LayoutItem::Leaf {
            leaf: Box::new(leaf),
        }
    }

    pub fn attach(&mut self, child: LayoutItem) -> &mut Self {
        match self {
            LayoutItem::Leaf { leaf } => (),
            LayoutItem::Container { children, .. } => children.push(child),
        };

        self
    }

    pub fn layout(&self, space: &LayoutSpace) -> LayoutOutcome {
        match self {
            LayoutItem::Leaf { leaf } => leaf.layout(space),
            LayoutItem::Container {
                container,
                children,
            } => container.layout(space, children),
        }
    }
}

mod tests {
    use crate::layout::{
        Dimensions, LayoutItem, LayoutSpace, Position, ShapeLeaf, VerticalStackContainer,
        WindowContainer,
    };
    use drawing::{Brush, ShapeKind};

    #[test]
    fn foo() {
        let mut root = LayoutItem::container(WindowContainer {
            width: 100,
            height: 100,
        });
        let mut stack = LayoutItem::container(VerticalStackContainer {});
        let brush = Brush {
            foreground: 0xFF0000FF,
            background: 0x00FF00FF,
        };
        let dimensions = Dimensions::new(10, 10);
        let shape1 = LayoutItem::leaf(ShapeLeaf::new(ShapeKind::Rectangle, brush, dimensions));
        let shape2 = LayoutItem::leaf(ShapeLeaf::new(ShapeKind::Ellipse, brush, dimensions));

        stack.attach(shape1);
        stack.attach(shape2);
        root.attach(stack);

        let outcome = root.layout(&LayoutSpace::new(Some(100), Some(100)));

        assert_eq!(outcome.consumed_space.width, 100);
        assert_eq!(outcome.consumed_space.height, 20);
        assert_eq!(outcome.remaining_space.offset, Position::new(0, 20));
        assert_eq!(outcome.remaining_space.available_height, None);
        assert_eq!(outcome.remaining_space.available_width, Some(100));
    }
}
