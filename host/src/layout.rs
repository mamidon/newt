use drawing::typesetting::{GlyphRun, TypeSet};
use drawing::{Brush, DrawList, Extent, ShapeKind};
use euclid::default::Vector2D;
use euclid::Point2D;
use euclid::Rect;
use euclid::Size2D;
use std::borrow::Borrow;
use std::cmp::{max, min};
use std::ops::Add;
use std::rc::Rc;

pub struct Pixels;
pub type Position = Point2D<i64, Pixels>;
pub type Dimensions = Size2D<i64, Pixels>;
pub type Rectangle = Rect<i64, Pixels>;

#[derive(Copy, Clone, Debug)]
pub struct RenderSpace {
    position: Position,
    dimensions: Dimensions,
}

impl RenderSpace {
    pub fn new(position: Position, dimensions: Dimensions) -> RenderSpace {
        RenderSpace {
            position,
            dimensions,
        }
    }

    fn from_dimensions(dimensions: Dimensions) -> RenderSpace {
        RenderSpace::new(Position::default(), dimensions)
    }

    pub fn subsect_vertically(&self, height: i64) -> (RenderSpace, RenderSpace) {
        let inner_height = min(self.dimensions.height, height);

        let head_position = self.position;
        let tail_position = Position::new(self.position.x, self.position.y + inner_height);

        let head = RenderSpace {
            position: head_position,
            dimensions: Dimensions::new(self.dimensions.width, inner_height),
        };

        let tail = RenderSpace {
            position: tail_position,
            dimensions: Dimensions::new(
                self.dimensions.width,
                self.dimensions.height - inner_height,
            ),
        };

        (head, tail)
    }

    pub fn center_horizontally(&self, width: i64) -> RenderSpace {
        let offset = if self.dimensions.width >= width {
            Position::new(
                (self.dimensions.width - width) / 2 + self.position.x,
                self.position.y,
            )
        } else {
            self.position
        };

        RenderSpace {
            dimensions: self.dimensions,
            position: offset,
        }
    }
}

pub struct LayoutNode {
    pub item: Rc<LayoutItem>,
    pub children: Vec<LayoutNode>,
}

pub struct RenderNode {
    item: RenderItem,
    children: Vec<RenderNode>,
    render_space: RenderSpace,
}

#[derive(Clone)]
pub enum RenderItemKind {
    Box {
        width: i64,
        height: i64,
    },
    Stack,
    Shape {
        kind: ShapeKind,
        brush: Brush,
        dimensions: Dimensions,
    },
    Text {
        lines: Vec<GlyphRun>,
        type_set: TypeSet,
    },
}

pub struct RenderItem {
    kind: RenderItemKind,
    position: Position,
    dimensions: Dimensions,
}

pub struct RenderItemIterator<'a> {
    frontier: Vec<(&'a RenderNode, Position)>,
}

pub enum LayoutItem {
    Box {
        width: i64,
        height: i64,
    },
    Stack,
    Shape {
        kind: ShapeKind,
        brush: Brush,
        dimensions: Dimensions,
    },
    Text {
        text: String,
        type_set: TypeSet,
    },
}

impl LayoutNode {
    pub fn new_box(width: i64, height: i64, children: Vec<LayoutNode>) -> LayoutNode {
        LayoutNode {
            item: Rc::new(LayoutItem::Box { width, height }),
            children,
        }
    }

    pub fn new_stack(children: Vec<LayoutNode>) -> LayoutNode {
        LayoutNode {
            item: Rc::new(LayoutItem::Stack),
            children,
        }
    }

    pub fn new_shape(kind: ShapeKind, brush: Brush, dimensions: Dimensions) -> LayoutNode {
        LayoutNode {
            item: Rc::new(LayoutItem::Shape {
                kind,
                brush,
                dimensions,
            }),
            children: Vec::new(),
        }
    }

    pub fn new_text(text: &str, type_set: TypeSet) -> LayoutNode {
        LayoutNode {
            item: Rc::new(LayoutItem::Text {
                text: text.to_string(),
                type_set,
            }),
            children: Vec::new(),
        }
    }

    pub fn layout(&self, width: Option<i64>, height: Option<i64>) -> RenderNode {
        match self.item.borrow() {
            LayoutItem::Box { width, height } => {
                LayoutNode::layout_box(*width, *height, &self.children)
            }
            LayoutItem::Shape {
                kind,
                brush,
                dimensions,
            } => LayoutNode::layout_shape(*kind, *brush, *dimensions),
            LayoutItem::Stack {} => LayoutNode::layout_stack(width, height, &self.children),
            LayoutItem::Text { text, type_set } => {
                LayoutNode::layout_text(text, type_set, width, height)
            }
        }
    }

    fn layout_box(width: i64, height: i64, children: &Vec<LayoutNode>) -> RenderNode {
        unimplemented!()
    }

    fn layout_shape(kind: ShapeKind, brush: Brush, dimensions: Dimensions) -> RenderNode {
        let render_item = RenderItem {
            kind: RenderItemKind::Shape {
                kind,
                brush,
                dimensions,
            },
            position: Position::zero(),
            dimensions,
        };

        RenderNode::leaf(render_item, RenderSpace::from_dimensions(dimensions))
    }

    fn layout_stack(
        width: Option<i64>,
        height: Option<i64>,
        children: &Vec<LayoutNode>,
    ) -> RenderNode {
        let mut dimensions = Dimensions::zero();

        let mut render_children: Vec<RenderNode> = children
            .iter()
            .scan(dimensions, |required_dimensions, child| {
                let mut render_child = child.layout(width, height);

                dimensions.width =
                    max(render_child.render_space.dimensions.width, dimensions.width);
                dimensions.height += render_child.render_space.dimensions.height;

                Some(render_child)
            })
            .collect();

        let mut remaining_space = RenderSpace {
            position: Position::zero(),
            dimensions,
        };
        let container_space = remaining_space;

        for child in render_children.iter_mut() {
            let child_width = child.render_space.dimensions.width;
            let child_height = child.render_space.dimensions.height;

            let (head, tail) = remaining_space.subsect_vertically(child_height);

            child.render_space = head;
            remaining_space = tail;
        }

        let render_item = RenderItem {
            kind: RenderItemKind::Stack,
            position: Position::zero(),
            dimensions,
        };

        RenderNode::container(render_item, container_space, render_children)
    }

    fn layout_text(
        text: &str,
        type_set: &TypeSet,
        width: Option<i64>,
        height: Option<i64>,
    ) -> RenderNode {
        let mut lines: Vec<GlyphRun> = Vec::new();
        let mut used_width: i64 = 0;
        let mut used_height: i64 = 0;

        for (index, line) in text.lines().enumerate() {
            let mut glyph_run = GlyphRun::new(type_set.clone());
            glyph_run.append_text(line);
            glyph_run.set_line_offset(index as i32);

            used_width = max(glyph_run.width() as i64, used_width);
            used_height = used_height + glyph_run.height() as i64;

            lines.push(glyph_run);
        }

        let dimensions = Dimensions::new(used_width, used_height);
        let render_item = RenderItem {
            kind: RenderItemKind::Text {
                lines,
                type_set: type_set.clone(),
            },
            position: Position::zero(),
            dimensions,
        };

        RenderNode::leaf(
            render_item,
            RenderSpace::new(Position::origin(), dimensions),
        )
    }
}

impl RenderNode {
    pub fn leaf(item: RenderItem, render_space: RenderSpace) -> RenderNode {
        RenderNode {
            item,
            render_space,
            children: Vec::new(),
        }
    }

    pub fn container(
        item: RenderItem,
        render_space: RenderSpace,
        children: Vec<RenderNode>,
    ) -> RenderNode {
        RenderNode {
            item,
            render_space,
            children,
        }
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = RenderItem> + 'a {
        RenderItemIterator::new(self)
    }
}

impl RenderItem {
    pub fn position(&self) -> Position {
        self.position
    }

    pub fn dimensions(&self) -> Dimensions {
        self.dimensions
    }

    pub fn kind(&self) -> &RenderItemKind {
        &self.kind
    }
}

impl<'a> Iterator for RenderItemIterator<'a> {
    type Item = RenderItem;

    fn next(&mut self) -> Option<Self::Item> {
        let (next_node, offset) = match self.frontier.pop() {
            Some((next_node, offset)) => (next_node, offset),
            _ => return None,
        };

        let next_absolute_position = next_node.render_space.position + offset.to_vector();
        for child in next_node.children.iter().rev() {
            self.frontier.push((child, next_absolute_position));
        }

        Some(RenderItem {
            kind: next_node.item.kind.clone(),
            dimensions: next_node.render_space.dimensions,
            position: next_absolute_position,
        })
    }
}

impl<'a> RenderItemIterator<'a> {
    fn new(node: &'a RenderNode) -> RenderItemIterator {
        RenderItemIterator {
            frontier: vec![(node, Position::zero())],
        }
    }
}

mod tests {
    use crate::layout::{Dimensions, LayoutNode, Position, RenderItem, RenderNode, RenderSpace};
    use drawing::{Brush, ShapeKind};

    #[test]
    fn layoutnode_can_traverse_children_after_layout() {
        let brush = Brush {
            foreground: 0xFF0000FF,
            background: 0x00FF00FF,
        };
        let dimensions = Dimensions::new(150, 150);

        let layout_root = LayoutNode::new_stack(vec![
            LayoutNode::new_shape(ShapeKind::Rectangle, brush, dimensions),
            LayoutNode::new_shape(ShapeKind::Rectangle, brush, dimensions),
            LayoutNode::new_shape(ShapeKind::Ellipse, brush, dimensions),
        ]);
        let render_root = layout_root.layout(Some(1024), Some(1024));
        let render_items: Vec<RenderItem> = render_root.iter().collect();
        let positions: Vec<Position> = render_items.iter().map(|n| n.position).collect();
        let dimensions: Vec<Dimensions> = render_items.iter().map(|n| n.dimensions).collect();

        assert_eq!(4, render_items.len());

        assert_eq!(Some(&Position::new(0, 0)), positions.get(0));
        assert_eq!(Some(&(150, 450).into()), dimensions.get(0));

        assert_eq!(Some(&Position::new(0, 0)), positions.get(1));
        assert_eq!(Some(&Dimensions::new(150, 150)), dimensions.get(1));

        assert_eq!(Some(&Position::new(0, 150)), positions.get(2));
        assert_eq!(Some(&Dimensions::new(150, 150)), dimensions.get(2));

        assert_eq!(Some(&Position::new(0, 300)), positions.get(3));
        assert_eq!(Some(&Dimensions::new(150, 150)), dimensions.get(3));
    }

    #[test]
    fn layoutnode_can_position_one_layer_of_children() {
        let brush = Brush {
            foreground: 0xFF0000FF,
            background: 0x00FF00FF,
        };
        let dimensions = Dimensions::new(150, 150);

        let layout_root = LayoutNode::new_stack(vec![
            LayoutNode::new_shape(ShapeKind::Rectangle, brush, dimensions),
            LayoutNode::new_shape(ShapeKind::Rectangle, brush, dimensions),
            LayoutNode::new_shape(ShapeKind::Ellipse, brush, dimensions),
        ]);
        let render_root = layout_root.layout(Some(1024), Some(1024));
        let render_items: Vec<RenderItem> = render_root.iter().collect();
        let positions: Vec<Position> = render_items.iter().map(|n| n.position).collect();
        let dimensions: Vec<Dimensions> = render_items.iter().map(|n| n.dimensions).collect();

        assert_eq!(4, render_items.len());
        assert_eq!(Some(&Position::new(0, 0)), positions.get(0));
        assert_eq!(Some(&Position::new(0, 0)), positions.get(1));
        assert_eq!(Some(&Position::new(0, 150)), positions.get(2));
        assert_eq!(Some(&Position::new(0, 300)), positions.get(3));
    }

    #[test]
    fn layoutnode_can_position_nested_children() {
        let brush = Brush {
            foreground: 0xFF0000FF,
            background: 0x00FF00FF,
        };
        let dimensions = Dimensions::new(150, 150);

        let layout_root = LayoutNode::new_stack(vec![
            LayoutNode::new_shape(ShapeKind::Rectangle, brush, dimensions),
            LayoutNode::new_stack(vec![LayoutNode::new_shape(
                ShapeKind::Ellipse,
                brush,
                dimensions,
            )]),
            LayoutNode::new_shape(ShapeKind::Rectangle, brush, dimensions),
        ]);
        let render_root = layout_root.layout(Some(1024), Some(1024));
        let render_items: Vec<RenderItem> = render_root.iter().collect();
        let positions: Vec<Position> = render_items.iter().map(|n| n.position).collect();
        let dimensions: Vec<Dimensions> = render_items.iter().map(|n| n.dimensions).collect();

        assert_eq!(5, render_items.len());
        assert_eq!(Some(&Position::new(0, 0)), positions.get(0));
        assert_eq!(Some(&Position::new(0, 0)), positions.get(1));
        assert_eq!(Some(&Position::new(0, 150)), positions.get(2));
        assert_eq!(Some(&Position::new(0, 150)), positions.get(3));
        assert_eq!(Some(&Position::new(0, 300)), positions.get(4));
    }
}
